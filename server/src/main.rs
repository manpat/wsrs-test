#![feature(ord_max_min)]

mod connections;
mod fileserver;
mod http;
mod ws;

mod world;

#[macro_use]
extern crate common;

extern crate rand;
extern crate sha1;
extern crate base64;
extern crate flate2;

use std::net::{TcpStream, TcpListener};
use std::io::Read;
use std::sync::mpsc;
use std::thread;
use std::time;

use common::*;
use connections::ConnectionID;

// main thread, sim -> network thread
enum NetworkMessage {
	NewConnection(TcpStream),
	NewSession(ConnectionID, u32),
	AuthSuccess(ConnectionID, u32),
	AuthFail(ConnectionID),

	WorldStateReady(ConnectionID, Vec<(u32, Vec2)>, Vec<u8>),
	PlaceTree(u32, Vec2),
	KillTree(u32),

	WorldTick(Vec<u8>),
	TreeTick(Vec<(u32, u8)>),
}

// network thread -> sim thread
enum SimulationMessage {
	RequestNewSession(ConnectionID),
	AttemptAuthSession(ConnectionID, u32),

	RequestWorldState(ConnectionID),
	RequestPlaceTree(ConnectionID, Vec2),
}

fn main() {
	println!("Is Hosted:      {}", cfg!(hosted));
	println!("Public address: {}", env!("PUBLIC_ADDRESS"));

	let listener = TcpListener::bind("0.0.0.0:9001").unwrap();
	let fs_listener = TcpListener::bind("0.0.0.0:8000").unwrap();

	thread::spawn(move || fileserver::start(fs_listener));

	let (main_tx, net_rx) = mpsc::channel::<NetworkMessage>();
	let (net_tx, sim_rx) = mpsc::channel::<SimulationMessage>();
	let sim_tx = main_tx.clone();

	let connection_thd = thread::spawn(move || network_loop(net_rx, net_tx));
	let simulation_thd = thread::spawn(move || sim_loop(sim_tx, sim_rx));

	for stream in listener.incoming() {
		match stream {
			Ok(mut stream) => {
				let mut buf = [0u8; 1024];

				// TODO: poll or async instead of block until timeout
				stream.set_read_timeout(Some(time::Duration::from_millis(500))).expect("set_read_timeout failed");

				let size = match stream.read(&mut buf) {
					Ok(s) => s, Err(_) => continue
				};

				if size == 0 { continue }

				let data = std::str::from_utf8(&buf[0..size]);
				if !data.is_ok() {
					println!("Error parsing request: Non utf8 data encountered");
					continue;
				}

				match http::Request::parse(data.unwrap()) {
					Ok(header) => {
						if header.get("Upgrade") != Some("websocket") {
							continue;
						}

						stream.set_read_timeout(None).expect("set_read_timeout failed");

						match ws::init_websocket_connection(&mut stream, &header) {
							Ok(_) => main_tx.send(NetworkMessage::NewConnection(stream)).unwrap(),
							Err(e) => println!("Error initialising connection: {}", e)
						}
					},

					Err(e) => {
						println!("Error parsing request: {}", e);
					},
				}
			},

			Err(e) => { println!("Connection failed: {}", e); }
		}
	}

	connection_thd.join().unwrap();
	simulation_thd.join().unwrap();
}

fn network_loop(rx: mpsc::Receiver<NetworkMessage>, tx: mpsc::Sender<SimulationMessage>) {
	let mut connections = connections::ConnectionManager::new();
	let mut packet_buffer = [0u8; 8<<10];

	let mut packet_queue: Vec<(Option<ConnectionID>, Packet)> = Vec::new();

	'main: loop {
		while let Some(msg) = rx.try_recv().ok() {
			use NetworkMessage as NM;

			match msg {
				NM::NewConnection(stream) => connections.register_connection(stream),

				NM::NewSession(id, token) => {
					if connections.notify_new_session(id) {
						packet_queue.push((Some(id), Packet::NewSession(token)));
					}
				}

				NM::AuthSuccess(id, token) => {
					if connections.imbue_session(id, token) {
						packet_queue.push((Some(id), Packet::AuthSuccessful(token)));
					}
				}

				NM::AuthFail(id) => {
					connections.notify_auth_fail(id);
					packet_queue.push((Some(id), Packet::AuthFail));
				}

				NM::WorldStateReady(id, state, health_state) => {
					packet_queue.push((Some(id), Packet::HealthUpdate(health_state)));

					for (t_id, pos) in state {
						// This is p' heavy - best not send hundreds of packets at once probably
						packet_queue.push((Some(id), Packet::TreePlaced(t_id, pos.x, pos.y)));
					}
				}

				NM::PlaceTree(tree_id, pos) => packet_queue.push((None, Packet::TreePlaced(tree_id, pos.x, pos.y))),
				NM::KillTree(tree_id) => packet_queue.push((None, Packet::TreeDied(tree_id))),
				NM::WorldTick(health_state) => packet_queue.push((None, Packet::HealthUpdate(health_state))),
				NM::TreeTick(tree_changes) => packet_queue.push((None, Packet::TreeUpdate(tree_changes))),
			}
		}

		use SimulationMessage as SM;

		while let Some((id, packet)) = connections.try_read(&mut packet_buffer) {
			match packet {
				Packet::Debug(s) => {
					println!("Debug ({}): {}", id, s);
				}

				Packet::RequestDownloadWorld => {
					println!("Request world state ({})", id);
					tx.send(SM::RequestWorldState(id)).unwrap();
				}

				Packet::RequestPlaceTree(x, y) => {
					println!("place tree ({}): {}, {}", id, x, y);
					tx.send(SM::RequestPlaceTree(id, Vec2::new(x, y))).unwrap();
				}

				_ => {}
			}
		}

		connections.flush();

		while let Some(id) = connections.poll_new_sessions() {
			tx.send(SM::RequestNewSession(id)).unwrap();
		}

		while let Some((id, token)) = connections.poll_auth_attempts() {
			tx.send(SM::AttemptAuthSession(id, token)).unwrap();
		}

		for &(id, ref p) in &packet_queue {
			if !p.is_valid_from_server() { continue }

			if let Some(id) = id {
				connections.send_to(id, &p);
			} else {
				connections.broadcast_to_authed(&p);
			}
		}

		packet_queue.clear();

		thread::sleep(time::Duration::from_millis(50));
	}
}

//////////////////////////////

fn sim_loop(tx: mpsc::Sender<NetworkMessage>, rx: mpsc::Receiver<SimulationMessage>) {
	use world::World;
	use world::Species;

	use NetworkMessage as NM;
	use SimulationMessage as SM;

	let mut world = World::new_random();
	let mut health_state = Vec::new();
	let mut tree_maturities = Vec::new();

	'main: loop {
		while let Some(msg) = rx.try_recv().ok() {

			// TODO: Remove all of these unwraps, save world state and exit gracefully
			match msg {
				SM::RequestNewSession(con_id) => {
					// Create new session
					println!("New Session requested for {}", con_id);

					let max_key = 3u32.pow(9);
					let random_key = rand::random::<u32>() % max_key;
					// TODO: not this
					
					tx.send(NM::NewSession(con_id, random_key)).unwrap();

					// TODO: Test if con_id already associated with pending session and delete
					// 	associate new session with connection and flag as pending
					// 	remove flag and persist once authed
				}

				SM::AttemptAuthSession(con_id, token) => {
					// Just accept everything for now
					tx.send(NM::AuthSuccess(con_id, token)).unwrap();
				}

				SM::RequestWorldState(con_id) => {
					let trees = world.trees.iter()
						.filter(|t| !t.is_dead())
						.map(|t| (t.id, t.pos))
						.collect::<Vec<_>>();

					tx.send(NM::WorldStateReady(con_id, trees, health_state.clone())).unwrap();
					tx.send(NM::TreeTick(tree_maturities.clone())).unwrap();
				}

				SM::RequestPlaceTree(con_id, pos) => {
					// TODO: Check con_id has a session and hasn't already
					//	placed too many trees
					if let Some(t_id) = world.place_tree(Species::A, pos) {
						tx.send(NM::PlaceTree(t_id, pos)).unwrap();
					}
				}
			}
		}

		if world.update() {
			health_state = world.land_health.iter()
				.map(|h| (h * 255.0) as u8)
				.collect::<Vec<_>>();

			tree_maturities = world.trees.iter()
				.filter(|&t| !t.is_dead())
				.map(|t| (t.id, t.get_maturity_stage()))
				.collect::<Vec<_>>();

			tx.send(NM::WorldTick(health_state.clone())).unwrap();
			tx.send(NM::TreeTick(tree_maturities.clone())).unwrap();
		}

		for &t_id in &world.dead_trees {
			tx.send(NM::KillTree(t_id)).unwrap();
		}

		world.dead_trees.clear();

		thread::sleep(time::Duration::from_millis(50));
	}
}