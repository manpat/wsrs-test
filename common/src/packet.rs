use std;
use ::*;

#[derive(Clone)]
pub enum Packet {
	// Client -> Server
	Debug(String),
	RequestNewSession,
	AttemptAuthSession(u32),

	// Server -> Client
	AuthSuccessful(u32),
	AuthFail,
	NewSession(u32),
}

impl Packet {
	pub fn get_type(&self) -> u8 {
		match *self {
			// Client -> Server
			Packet::Debug(_) => 0x0,
			Packet::RequestNewSession => 0x1,
			Packet::AttemptAuthSession(_) => 0x2,

			// Server -> Client
			Packet::AuthSuccessful(_) => 0x80,
			Packet::AuthFail => 0x81,
			Packet::NewSession(_) => 0x82,
		}
	}

	pub fn parse(src: &[u8]) -> Option<Packet> {
		let ty = src[0];

		match ty {
			0x0  => std::str::from_utf8(&src[1..]).ok().map(|s| Packet::Debug(String::from(s))),
			0x1  => Some(Packet::RequestNewSession),
			0x2  => Some(Packet::AttemptAuthSession(read_u32_from_slice(&src[1..]))),

			0x80 => Some(Packet::AuthSuccessful(read_u32_from_slice(&src[1..]))),
			0x81 => Some(Packet::AuthFail),
			0x82 => Some(Packet::NewSession(read_u32_from_slice(&src[1..]))),
			_ => None
		}
	}

	pub fn write(&self, dst: &mut [u8]) -> usize {
		assert!(dst.len() > 64);

		dst[0] = self.get_type();

		match *self {
			Packet::Debug(ref s) => {
				let len = s.len() + 1;

				assert!(dst.len() >= len);
				dst[1..len].copy_from_slice(&s.as_bytes());

				len
			},

			Packet::RequestNewSession => 1,
			Packet::AttemptAuthSession(tok) => {
				write_u32_to_slice(&mut dst[1..], tok);
				5
			},

			Packet::AuthSuccessful(tok) => {
				write_u32_to_slice(&mut dst[1..], tok);
				5
			},
			Packet::AuthFail => 1,
			Packet::NewSession(tok) => {
				write_u32_to_slice(&mut dst[1..], tok);
				5
			},
		}
	}

	pub fn is_valid_from_client(&self) -> bool {
		match *self {
			Packet::Debug(_) => true,
			Packet::RequestNewSession => true,
			Packet::AttemptAuthSession(..) => true,
			_ => false
		}
	}

	pub fn is_valid_from_server(&self) -> bool {
		match *self {
			Packet::Debug(_) => false,
			Packet::RequestNewSession => false,
			Packet::AttemptAuthSession(_) => false,
			_ => true
		}
	}
}
