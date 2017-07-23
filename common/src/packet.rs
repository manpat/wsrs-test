use std;

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



pub fn write_u32_to_slice(dst: &mut [u8], value: u32) {
	assert!(dst.len() >= 4);

	use std::mem::transmute;

	let a: [u8; 4] = unsafe {transmute(value)};
	dst[..4].copy_from_slice(&a);
}

pub fn write_u16_to_slice(dst: &mut [u8], value: u16) {
	assert!(dst.len() >= 2);

	use std::mem::transmute;

	let a: [u8; 2] = unsafe {transmute(value)};
	dst[..2].copy_from_slice(&a);
}

pub fn read_u32_from_slice(src: &[u8]) -> u32 {
	assert!(src.len() >= 4);

	let mut a = [0u8; 4];
	a.copy_from_slice(&src[..4]);

	unsafe { std::mem::transmute(a) }
}
pub fn read_u16_from_slice(src: &[u8]) -> u16 {
	assert!(src.len() >= 2);

	let mut a = [0u8; 2];
	a.copy_from_slice(&src[..2]);

	unsafe { std::mem::transmute(a) }
}