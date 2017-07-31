use rendering::types::Color;
use common::*;
use std;

#[derive(Debug)]
pub struct FaceMaterial (pub String, pub Vec<u16>);

pub struct Mesh3DS {
	pub verts: Vec<Vec3>,
	pub elements: Vec<u16>,
	pub face_materials: Vec<FaceMaterial>,
}

pub fn parse_3ds(data: &[u8]) -> Option<Mesh3DS> {
	let magic = read_u16_from_slice(&data[0..2]);
	if magic != 0x4d4d { return None }

	let main_block_len = read_u16_from_slice(&data[2..6]) as usize;

	assert!(main_block_len == data.len());

	let mut mesh = Mesh3DS {
		verts: Vec::new(),
		elements: Vec::new(),
		face_materials: Vec::new(),
	};

	parse_chunk(&mut mesh, &data[6..]);

	// println!("{:?}", mesh.face_materials);

	Some(mesh)
}

fn read_string_from_slice<'a>(data: &'a [u8]) -> &'a str {
	let index = data.iter().position(|c| *c == 0).unwrap();

	std::str::from_utf8(&data[..index]).unwrap()
}

fn read_vec3_from_slice(data: &[u8]) -> Vec3 {
	assert!(data.len() >= 12);

	Vec3::new(
		read_f32_from_slice(&data[0..]),
		read_f32_from_slice(&data[4..]),
		read_f32_from_slice(&data[8..])
	)
}

fn parse_color_chunk(data: &[u8]) -> Color {
	let id = read_u16_from_slice(&data[0..]);
	let len = read_u32_from_slice(&data[2..]) as usize;

	match id {
		// rgb f32
		0x0010 => {
			Color::rgb(
				read_f32_from_slice(&data[6 + 4*0..]),
				read_f32_from_slice(&data[6 + 4*1..]),
				read_f32_from_slice(&data[6 + 4*2..]),
			)
		}

		// rgb u8
		0x0011 => {
			Color::rgb(
				data[6 + 0] as f32 / 255.0,
				data[6 + 1] as f32 / 255.0,
				data[6 + 2] as f32 / 255.0,
			)
		}

		// rgb u8 gamma
		0x0012 => {
			println!("TODO deal w/ gamma?");
			Color::rgb(
				data[6 + 0] as f32 / 255.0,
				data[6 + 1] as f32 / 255.0,
				data[6 + 2] as f32 / 255.0,
			)
		}

		_ => unimplemented!()
	}
}

// http://wayback.archive.org/web/20090404091233/http://www.jalix.org/ressources/graphics/3DS/_unofficials/3ds-info.txt
// http://wayback.archive.org/web/20090404045225/http://www.whisqu.se/per/docs/graphics56.htm
fn parse_chunk(mut mesh: &mut Mesh3DS, data: &[u8]) {
	if data.len() < 6 { return }

	let mut idx = 0;
	while idx < data.len()-6 {
		let header = &data[idx..];

		let id = read_u16_from_slice(&header[0..]);
		let len = read_u32_from_slice(&header[2..]) as usize;

		// println!("chunk {:4x} (len {})", id, len);
		match id {
			// 3D editor block
			0x3d3d => {
				// println!(" . [3D root]");
				parse_chunk(&mut mesh, &header[6..len])
			},

			// Object block
			0x4000 => {
				let name = read_string_from_slice(&header[6..len]);
				// println!(" . . [object] '{}'", name);

				parse_chunk(&mut mesh, &header[6+name.len()+1..]);
			}

			// Triangular mesh
			0x4100 => {
				// println!(" . . . [triangle mesh]");
				parse_chunk(&mut mesh, &header[6..len]);
			}

			0x4110 => {
				let count = read_u16_from_slice(&header[6..len]) as usize;
				// println!(" . . . . [vertices list] {}", count);

				let mut v = Vec::with_capacity(count);

				let inc = 3 * 4;

				for i in 0..count {
					v.push(read_vec3_from_slice(&header[8 + i*inc .. len]));
				}

				// println!(" . . . . . {:?}", v);

				mesh.verts = v;
			}

			0x4120 => {
				let poly_count = read_u16_from_slice(&header[6..len]) as usize;

				// println!(" . . . . [faces description] {}", poly_count);

				let mut v = Vec::with_capacity(poly_count);
				let inc = 2;

				for i in 0..poly_count {
					v.push(read_u16_from_slice(&header[8 + (i*4 + 0)*inc .. len]));
					v.push(read_u16_from_slice(&header[8 + (i*4 + 1)*inc .. len]));
					v.push(read_u16_from_slice(&header[8 + (i*4 + 2)*inc .. len]));
					// v.push(read_u16_from_slice(&header[8 + (i*4 + 0)*inc .. len])); // Flags
				}

				// println!(" . . . . . {:?}", v);

				mesh.elements = v;

				let subchunk_offset = 8 + poly_count * inc * 4;
				parse_chunk(&mut mesh, &header[subchunk_offset .. len]);
			}

			0x4130 => {
				let mat_name = read_string_from_slice(&header[6..len]);
				let num_entries = read_u16_from_slice(&header[6+mat_name.len()+1 .. len]);
				let entries = &header[6 + mat_name.len()+1 + 2 .. len];

				// println!(" . . . . . [faces material] {} {}", mat_name, num_entries);

				let mut v = Vec::new();

				for i in 0..num_entries as usize {
					v.push(read_u16_from_slice(&entries[i*2..]));
				}

				mesh.face_materials.push(FaceMaterial(mat_name.to_string(), v));
			}

			// Material block
			0xAFFF => {
				// println!(" . . [material block]");
				// parse_chunk(&mut mesh, &header[6..len]);
			}

			// Material name
			0xA000 => {
				// let name = read_string_from_slice(&header[6..len]);
				// println!(" . . . [material name] '{}'", name);
			}

			// Material diffuse color
			0xA020 => {
				// let col = parse_color_chunk(&header[6..len]);
				// println!(" . . . [material diffuse color] {:?}", col);
			}

			_ => {}
		}

		idx += len;
	}
}