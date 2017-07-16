use std::net::TcpStream;
use std::io::{Write, Read};
use common::Packet;
use ws;

pub type ConnectionID = u32;

#[derive(Debug)]
pub enum ConnectionState {
	NoAuth{attempts: i32},
	AttemptingAuth{attempts: i32, token: u32, waiting: bool},
	AwaitingNewSession,
	NewSessionRequested,
	Ready,
	AwaitingDeletion,
}

macro_rules! matches_enum {
    ($v:expr, $p:pat) => {
    	match $v {
    		$p => true,
    		_ => false,
    	}
    }
}

pub struct Connection {
	pub stream: TcpStream,
	pub state: ConnectionState,

	pub session_id: Option<u32>,
	pub id: ConnectionID,
}

impl Connection {
	pub fn get_auth_attempts(&self) -> Option<i32> {
		match self.state {
			ConnectionState::NoAuth{attempts} => Some(attempts),
			ConnectionState::AttemptingAuth{attempts, ..} => Some(attempts),
			 _ => None
		}
	}

	pub fn is_awaiting_auth(&self) -> bool {
		self.get_auth_attempts().is_some()
	}

	pub fn is_awaiting_new_session(&self) -> bool {
		match self.state {
			ConnectionState::AwaitingNewSession => true, _ => false
		}
	}

	pub fn is_awaiting_deletion(&self) -> bool {
		match self.state {
			ConnectionState::AwaitingDeletion => true, _ => false
		}
	}

	pub fn is_ready(&self) -> bool {
		match self.state {
			ConnectionState::Ready => true, _ => false
		}
	}

	pub fn send_payload(&mut self, mut packet_buffer: &mut [u8], payload: &[u8]) {
		let packet = ws::encode_ws_packet(&mut packet_buffer, &payload);
		let _ = self.stream.write_all(&packet);
	}
}

pub struct ConnectionManager {
	pub connections: Vec<Connection>,

	next_id: ConnectionID,
}

impl ConnectionManager {
	pub fn new() -> Self {
		ConnectionManager{
			connections: Vec::new(),

			next_id: 1,
		}
	}

	pub fn register_connection(&mut self, stream: TcpStream) {
		stream.set_nonblocking(true).expect("Set nonblock failed");

		println!("Connection ({})", self.next_id);

		self.connections.push(Connection {
			stream,
			state: ConnectionState::NoAuth{attempts: 0},

			session_id: None,
			id: self.next_id,
		});

		self.next_id += 1;
	}

	pub fn imbue_session(&mut self, id: ConnectionID, token: u32) -> bool {
		if let Some(ref mut con) = self.connections.iter_mut().find(|c| c.id == id) {
			assert!(con.session_id.is_none());

			con.session_id = Some(token);
			con.state = ConnectionState::Ready;
			true
		} else {
			false
		}
	}

	pub fn notify_new_session(&mut self, id: ConnectionID) -> bool {
		use self::ConnectionState::*;

		let mut con = self.connections.iter_mut().find(|c| c.id == id);

		if let Some(ref mut con) = con {
			if matches_enum!(con.state, NewSessionRequested) {
				// TODO: preserve attempts
				con.state = NoAuth{attempts: 0};
				con.session_id = None;

				return true;
			}
		}

		false
	}

	pub fn notify_auth_fail(&mut self, id: ConnectionID) {
		use self::ConnectionState::*;

		if let Some(ref mut con) = self.connections.iter_mut().find(|c| c.id == id) {
			con.state = match con.state {
				AttemptingAuth{attempts, waiting: true, ..} => 
					NoAuth{attempts: attempts+1},

				_ => {
					println!("notify_auth_fail called on connection not waiting for auth - closing...");
					AwaitingDeletion
				},
			};
		}
	}

	pub fn flush(&mut self) {
		self.connections.retain(|x| !x.is_awaiting_deletion());
	}

	pub fn poll_new_sessions(&mut self) -> Option<ConnectionID> {
		self.connections.iter_mut()
			.filter(|c| c.is_awaiting_new_session())
			.next().as_mut()
			.map(|con| {
				con.state = ConnectionState::NewSessionRequested;
				con.id
			})
	}

	pub fn poll_auth_attempts(&mut self) -> Option<(ConnectionID, u32)> {
		self.connections.iter_mut()
			.filter(|c| matches_enum!(c.state, ConnectionState::AttemptingAuth{waiting: false, ..}))
			.next().as_mut()
			.and_then(|con| {
				if let ConnectionState::AttemptingAuth{token, attempts, ..} = con.state {
					con.state = ConnectionState::AttemptingAuth{waiting: true, 	token, attempts};
					Some((con.id, token))
				} else {
					None
				}
			})
	}

	pub fn send_to(&mut self, id: ConnectionID, p: &Packet) -> bool {
		if let Some(ref mut con) = self.connections.iter_mut().find(|c| c.id == id) {
			if !p.is_valid_from_server() { return false }

			let mut payload = [0u8; 256];
			let mut packet_buffer = [0u8; 512];
			let len = p.write(&mut payload);

			con.send_payload(&mut packet_buffer, &payload[..len]);

			true
		} else {
			false
		}
	}

	pub fn broadcast_to_authed(&mut self, p: &Packet) {
		let mut payload = [0u8; 256];
		let mut packet_buffer = [0u8; 512];
		let len = p.write(&mut payload);

		for con in self.connections.iter_mut().filter(|c| c.is_ready()) {
			con.send_payload(&mut packet_buffer, &payload[..len]);
		}
	}

	pub fn try_read(&mut self, mut read_buffer: &mut [u8]) -> Option<(ConnectionID, Packet)> {
		for mut con in &mut self.connections {
			let res = con.stream.read(&mut read_buffer);
			let length = match res {
				Ok(length) => length,
				Err(_) => continue,
			};

			if length == 0 {
				println!("Zero length packet ({})", con.id);
				con.state = ConnectionState::AwaitingDeletion;
				continue;
			}

			let payload = ws::decode_ws_packet(&mut read_buffer[..length]);
			if payload.len() == 0 {
				println!("Disconnection ({})", con.id);
				con.state = ConnectionState::AwaitingDeletion;
				continue;
			}

			if let Some(packet) = Packet::parse(&payload) {
				if !packet.is_valid_from_client() { continue }

				if con.session_id.is_none() {
					ConnectionManager::process_unauthed_packet(&mut con, &packet);
				} else {
					return Some((con.id, packet))
				}

			} else {
				con.state = ConnectionState::AwaitingDeletion;
				println!("Invalid payload ({})", con.id);
			}
		}

		None
	}

	fn process_unauthed_packet(con: &mut Connection, p: &Packet) {
		if let ConnectionState::NoAuth{attempts} = con.state {
			match *p {
				Packet::RequestNewSession => {
					con.state = ConnectionState::AwaitingNewSession
				},

				Packet::AttemptAuthSession(token) => {
					println!("Client {} tried to auth with key '{}'", con.id, token);
					// TODO: if token doesn't exist, bump attempt count and potentially terminate connection
					// con.session_id = Some(token);
					// con.state = ConnectionState::Ready;
					con.state = ConnectionState::AttemptingAuth{attempts: attempts+1, token, waiting: false};
				},

				_ => {},
			}
		}

	}
}