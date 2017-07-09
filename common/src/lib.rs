

pub enum Packet {
	Debug(String),
	Connect(u32),
	Disconnect(u32),
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

impl Packet {
	pub fn parse(src: &[u8]) -> Option<Packet> {
		if src.len() < 5 { return None } // Not strictly correct for Debug, but w/e

		let ty = src[0];

		match ty {
			0x0  => std::str::from_utf8(&src[1..]).ok().map(|s| Packet::Debug(String::from(s))),
			b'j' => Some( Packet::Connect(read_u32_from_slice(&src[1..])) ),
			b'd' => Some( Packet::Disconnect(read_u32_from_slice(&src[1..])) ),
			_ => None
		}
	}

	pub fn write(&self, dst: &mut [u8]) -> usize {
		let simple = match *self {
			Packet::Connect(v) => Some(v),
			Packet::Disconnect(v) => Some(v),
			_ => None
		};

		assert!(dst.len() > 0);

		dst[0] = self.get_type();

		if let Some(v) = simple {
			assert!(dst.len() >= 5);
			
			write_u32_to_slice(&mut dst[1..], v);
			return 5
		}

		0

		// match *self {
		// 	Packet::Click(id, x, y) => {
		// 		assert!(dst.len() >= 9);
		// 		write_u32_to_slice(&mut dst[1..], id);
		// 		write_u16_to_slice(&mut dst[5..], x as u16);
		// 		write_u16_to_slice(&mut dst[7..], y as u16);
		// 		9
		// 	},

		// 	Packet::Debug(ref s) => {
		// 		let len = s.len() + 1;

		// 		assert!(dst.len() >= len);
		// 		dst[1..len].copy_from_slice(&s.as_bytes());

		// 		len
		// 	},

		// 	_ => unreachable!()
		// }
	}

	pub fn get_type(&self) -> u8 {
		match *self {
			Packet::Debug(_) => 0x0,
			Packet::Connect(_) => b'j',
			Packet::Disconnect(_) => b'd',
			// Packet::Click(..) => b'c',
			// Packet::Update(_) => b'u',
		}
	}

	pub fn is_valid_from_client(&self) -> bool {
		match *self {
			// Packet::Click(..) => true,
			Packet::Debug(_) => true,
			_ => false
		}
	}

	pub fn is_valid_from_server(&self) -> bool {
		match *self {
			Packet::Debug(_) => false,
			_ => true
		}
	}

	pub fn should_server_send_to(&self, tid: u32) -> bool {
		match *self {
			// Packet::Click(id, ..) => id != tid,
			Packet::Connect(id) => id != tid,
			Packet::Disconnect(id) => id != tid,

			// Packet::Update(_) => true,
			Packet::Debug(_) => false,
		}
	}
}