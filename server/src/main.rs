mod fileserver;
mod http;
mod ws;

extern crate sha1;
extern crate base64;
extern crate flate2;
extern crate common;

use std::net::{TcpStream, TcpListener};
use std::io::{Write, Read};
use std::sync::mpsc;
use std::thread;
use std::time;

use common::*;

fn main() {
	println!("Is Hosted:      {}", cfg!(hosted));
	println!("Public address: {}", env!("PUBLIC_ADDRESS"));

	let listener = TcpListener::bind("0.0.0.0:9001").unwrap();
	let fs_listener = TcpListener::bind("0.0.0.0:8000").unwrap();

	thread::spawn(move || fileserver::start(fs_listener));

	let (tx, rx) = mpsc::channel::<TcpStream>();
	let thd = thread::spawn(move || server_loop(rx));

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
							Ok(_) => tx.send(stream).unwrap(),
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

	thd.join().unwrap();
}

struct Connection {
	stream: TcpStream,
	delete_flag: bool,

	id: u32,
}

impl Connection {
	fn new(s: TcpStream, id: u32) -> Connection {
		Connection {
			stream: s,
			delete_flag: false,
			id
		}
	}
}

fn server_loop(rx: mpsc::Receiver<TcpStream>) {
	let mut connections = Vec::new();
	let mut packet_buffer = [0u8; 8<<10];
	let mut id_counter = 0u32;

	let mut packet_queue = Vec::new();

	'main: loop {
		while let Some(stream) = rx.try_recv().ok() {
			stream.set_nonblocking(true).expect("Set nonblock failed");
			id_counter += 1;
			connections.push(Connection::new(stream, id_counter));
			packet_queue.push(Packet::Connect(id_counter));
			println!("Connection ({})", id_counter);
		}

		for c in &mut connections {
			let res = c.stream.read(&mut packet_buffer);
			let length = match res {
				Ok(length) => length,
				Err(_) => continue,
			};

			if length == 0 {
				println!("Zero length packet ({})", c.id);
				c.delete_flag = true;
				continue;				
			}

			let payload = ws::decode_ws_packet(&mut packet_buffer[..length]);
			if payload.len() == 0 {
				println!("Disconnection ({})", c.id);
				c.delete_flag = true;
				continue;
			}

			if let Some(packet) = Packet::parse(&payload) {
				if !packet.is_valid_from_client() { continue }

				match packet {
					// Packet::Click(_, x, y) => packet_queue.push(Packet::Click(c.id, x, y)),
					Packet::Debug(s) => {
						println!("Debug ({}): {}", c.id, s);
					},

					_ => unreachable!()
				}
			} else {
				c.delete_flag = true;
				println!("Invalid payload ({})", c.id);
				continue;
			}
		}

		for c in connections.iter().filter(|c| c.delete_flag) {
			packet_queue.push(Packet::Disconnect(c.id));
		}

		// let send_update = packet_queue.iter().any(|m| match *m {
		// 	Packet::Connect(_) => true,
		// 	Packet::Disconnect(_) => true,
		// 	_ => false,
		// });

		connections.retain(|x| !x.delete_flag);

		// if send_update {
		// 	packet_queue.push(Packet::Update(connections.len() as u32));
		// }

		let mut payload = [0u8; 256];

		for p in &packet_queue {
			if !p.is_valid_from_server() { continue }

			let len = p.write(&mut payload);
			let packet = ws::encode_ws_packet(&mut packet_buffer, &payload[..len]);

			for c in &mut connections {
				if p.should_server_send_to(c.id) {
					let _ = c.stream.write_all(&packet);
				}
			}
		}

		packet_queue.clear();

		thread::sleep(time::Duration::from_millis(50));
	}
}