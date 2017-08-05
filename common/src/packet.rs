use std;
use ::*;

#[derive(Clone)]
pub enum Packet {
	// Client -> Server
	Debug(String),
	RequestNewSession,
	AttemptAuthSession(u32),
	RequestDownloadWorld,

	RequestPlaceTree(f32, f32),

	// Server -> Client
	AuthSuccessful(u32),
	AuthFail,
	NewSession(u32),

	TreePlaced(u32, f32, f32),
	TreeDied(u32),

	HealthUpdate(Vec<u8>),
	TreeUpdate(Vec<(u32, u8)>),
}

impl Packet {
	pub fn get_type(&self) -> u8 {
		match *self {
			// Client -> Server
			Packet::Debug(_) => 0x0,
			Packet::RequestNewSession => 0x1,
			Packet::AttemptAuthSession(_) => 0x2,
			Packet::RequestDownloadWorld => 0x3,

			Packet::RequestPlaceTree(..) => 0x10,

			// Server -> Client
			Packet::AuthSuccessful(_) => 0x80,
			Packet::AuthFail => 0x81,
			Packet::NewSession(_) => 0x82,

			Packet::TreePlaced(..) => 0x90,
			Packet::TreeDied(..) => 0x91,

			Packet::HealthUpdate(..) => 0x92,
			Packet::TreeUpdate(..) => 0x93,
		}
	}

	pub fn parse(src: &[u8]) -> Option<Packet> {
		let ty = src[0];

		match ty {
			0x0  => std::str::from_utf8(&src[1..]).ok().map(|s| Packet::Debug(String::from(s))),
			0x1  => Some(Packet::RequestNewSession),
			0x2  => Some(Packet::AttemptAuthSession(read_u32_from_slice(&src[1..]))),
			0x3  => Some(Packet::RequestDownloadWorld),

			0x10 => {
				let (x,y) = (read_f32_from_slice(&src[1..]), read_f32_from_slice(&src[5..]));
				Some(Packet::RequestPlaceTree(x, y))
			}

			0x80 => Some(Packet::AuthSuccessful(read_u32_from_slice(&src[1..]))),
			0x81 => Some(Packet::AuthFail),
			0x82 => Some(Packet::NewSession(read_u32_from_slice(&src[1..]))),

			0x90 => {
				let tree_id = read_u32_from_slice(&src[1..]);
				let (x,y) = (read_f32_from_slice(&src[5..]), read_f32_from_slice(&src[9..]));
				Some(Packet::TreePlaced(tree_id, x, y))
			}

			0x91 => Some(Packet::TreeDied(read_u32_from_slice(&src[1..]))),
			0x92 => Some(Packet::HealthUpdate(src[1..].to_vec())),
			0x93 => {
				let v = src[1..].chunks(5).map(|c| (read_u32_from_slice(c), c[4])).collect();
				Some(Packet::TreeUpdate(v))
			},

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
			}

			Packet::RequestNewSession => 1,
			Packet::AttemptAuthSession(tok) => {
				write_u32_to_slice(&mut dst[1..], tok);
				5
			}

			Packet::RequestDownloadWorld => 1,

			Packet::RequestPlaceTree(x, y) => {
				write_f32_to_slice(&mut dst[1..], x);
				write_f32_to_slice(&mut dst[5..], y);
				9
			}

			Packet::AuthSuccessful(tok) => {
				write_u32_to_slice(&mut dst[1..], tok);
				5
			}
			Packet::AuthFail => 1,
			Packet::NewSession(tok) => {
				write_u32_to_slice(&mut dst[1..], tok);
				5
			}

			Packet::TreePlaced(id, x, y) => {
				write_u32_to_slice(&mut dst[1..], id);
				write_f32_to_slice(&mut dst[5..], x);
				write_f32_to_slice(&mut dst[9..], y);
				13
			}

			Packet::TreeDied(id) => {
				write_u32_to_slice(&mut dst[1..], id);
				5
			}

			Packet::HealthUpdate(ref hs) => {
				dst[1..1+hs.len()].copy_from_slice(&hs);
				1 + hs.len()
			}

			Packet::TreeUpdate(ref ts) => {
				for (i, &(tid, stage)) in ts.iter().enumerate() {
					let base = 1 + i*5;

					write_u32_to_slice(&mut dst[base..], tid);
					dst[base + 4] = stage;
				}

				1 + ts.len() * 5
			}
		}
	}

	pub fn is_valid_from_client(&self) -> bool {
		self.get_type() < 0x80
	}

	pub fn is_valid_from_server(&self) -> bool {
		self.get_type() >= 0x80
	}
}
