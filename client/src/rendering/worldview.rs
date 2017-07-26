use rendering::gl;
use rendering::types::*;
use rendering::shader::Shader;

use rendering::parser_3ds::*;
use rendering::texture::*;

use std::f32::consts::PI;
use std::mem::{transmute, size_of, size_of_val};

static TERRAIN_VERT_SRC: &'static str = include_str!("../../assets/terrain.vert");
static WORLD_VERT_SRC: &'static str = include_str!("../../assets/world.vert");
static FRAG_SRC: &'static str = include_str!("../../assets/world.frag");

static TEST_MODEL: &'static [u8] = include_bytes!("../../assets/test_model.3ds");

pub struct WorldView {
	shader: Shader,
	terrain: TerrainView,

	vbo: u32,
	ebo: u32,
}

static mut TIME: f32 = 0.0;

impl WorldView {
	pub fn new() -> WorldView {
		let mut bufs = [0u32; 2];
		unsafe{ gl::GenBuffers(2, bufs.as_mut_ptr()); }

		parse_3ds(&TEST_MODEL);

		WorldView {
			shader: Shader::new(&WORLD_VERT_SRC, &FRAG_SRC),
			terrain: TerrainView::new(),

			vbo: bufs[0],
			ebo: bufs[1],
		}
	}

	pub fn render(&mut self, vp: &Viewport) {
		unsafe{ TIME += 0.016; }

		let ph = unsafe{TIME};
		let xrotph = PI/6.0;
		let yrotph = PI/4.0;

		let sc = 0.1;
		let scale = Vec3::new(1.0/vp.get_aspect(), 1.0, 1.0);
		let trans = Vec3::new(0.0, 0.0, 3.0);

		let world_mat = Mat4::scale(scale)
			* Mat4::uniform_scale(sc)
			* Mat4::translate(trans)
			* Mat4::xrot(-xrotph)
			* Mat4::translate(Vec3::new(ph.cos(), 0.0, ph.sin()))
			* Mat4::yrot(-yrotph);

		self.terrain.render(&world_mat);

		self.shader.use_program();
		self.shader.set_view(&world_mat);

		// let vs = [
		// 	Vec3::new(-1.0, 0.0, 1.0),
		// 	Vec3::new(-1.0, 0.0,-1.0),
		// 	Vec3::new( 1.0, 0.0,-1.0),
		// 	Vec3::new( 1.0, 0.0, 1.0),

		// 	Vec3::new(-1.0, 0.0, 1.0) + Vec3::new(0.0, 0.0,-3.0),
		// 	Vec3::new(-1.0, 0.0,-1.0) + Vec3::new(0.0, 0.0,-3.0),
		// 	Vec3::new( 1.0, 0.0,-1.0) + Vec3::new(0.0, 0.0,-3.0),
		// 	Vec3::new( 1.0, 0.0, 1.0) + Vec3::new(0.0, 0.0,-3.0),
		// ];

		// let es = [
		// 	0, 1, 2, 0, 2, 3,
		// 	4, 5, 6, 4, 6, 7u16,
		// ];

		unsafe {
			// gl::EnableVertexAttribArray(0);
			// gl::DisableVertexAttribArray(1);

			// let vert_size = (3*4) as isize;

			// gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
			// gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (es.len()*2) as isize, transmute(es.as_ptr()), gl::STREAM_DRAW);

			// gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			// gl::BufferData(gl::ARRAY_BUFFER, vs.len() as isize * vert_size, transmute(vs.as_ptr()), gl::STREAM_DRAW);
			// gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(0));
			// // gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(12));

			// gl::DrawElements(gl::TRIANGLES, 12, gl::UNSIGNED_SHORT, transmute(0));
		}
	}
}

struct TerrainView {
	shader: Shader,
	terrain_palette: Texture,
	
	health_vbo: u32,
	vbo: u32,
	ebo: u32,
}

impl TerrainView {
	fn new() -> TerrainView {
		let mut bufs = [0u32; 3];
		unsafe{ gl::GenBuffers(3, bufs.as_mut_ptr()); }

		let pal = [
			Color::rgb(1.0, 0.0, 0.0),
			Color::rgb(1.0, 1.0, 0.0),
			Color::rgb(0.0, 1.0, 0.0),
		];

		let mut terrain_palette = TextureBuilder::new()
			.linear_magnify()
			.finalize();

		terrain_palette.upload_1d(&pal);

		TerrainView {
			shader: Shader::new(&TERRAIN_VERT_SRC, &FRAG_SRC),
			terrain_palette,
			
			health_vbo: bufs[0],
			vbo: bufs[1],
			ebo: bufs[2],
		}
	}

	fn build_main_buffers(&mut self) {
		let vs = [
			Vec3::new(-1.0, 0.0, 1.0) + Vec3::new(0.0, 0.0,-2.5),
			Vec3::new(-1.0, 0.0,-1.0) + Vec3::new(0.0, 0.0,-2.5),
			Vec3::new( 1.0, 0.0,-1.0) + Vec3::new(0.0, 0.0,-2.5),
			Vec3::new( 1.0, 0.0, 1.0) + Vec3::new(0.0, 0.0,-2.5),

			Vec3::new(-1.0, 0.0, 1.0),
			Vec3::new(-1.0, 0.0,-1.0),
			Vec3::new( 1.0, 0.0,-1.0),
			Vec3::new( 1.0, 0.0, 1.0),

			Vec3::new(-1.0, 0.0, 1.0) + Vec3::new(0.0, 0.0, 2.5),
			Vec3::new(-1.0, 0.0,-1.0) + Vec3::new(0.0, 0.0, 2.5),
			Vec3::new( 1.0, 0.0,-1.0) + Vec3::new(0.0, 0.0, 2.5),
			Vec3::new( 1.0, 0.0, 1.0) + Vec3::new(0.0, 0.0, 2.5),
		];

		let es = [
			0, 1, 2, 0, 2, 3,
			4, 5, 6, 4, 6, 7,
			8, 9,10, 8,10,11u16,
		];

		unsafe {
			let vert_size = (3*4) as isize;

			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
			gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (es.len()*2) as isize, transmute(es.as_ptr()), gl::STATIC_DRAW);

			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BufferData(gl::ARRAY_BUFFER, vs.len() as isize * vert_size, transmute(vs.as_ptr()), gl::STATIC_DRAW);
		}
	}

	fn build_health_vbo(&mut self) {
		let data = [
			0.0, 0.0, 0.0, 0.0,
			0.2, 0.2, 0.5, 0.5,
			1.0, 1.0, 1.0, 1.0f32,
		];

		unsafe {
			gl::BindBuffer(gl::ARRAY_BUFFER, self.health_vbo);
			gl::BufferData(gl::ARRAY_BUFFER, data.len() as isize * 4, transmute(data.as_ptr()), gl::STREAM_DRAW);
		}
	}

	fn render(&mut self, world_mat: &Mat4) {
		self.build_main_buffers();
		self.build_health_vbo();

		self.shader.use_program();
		self.shader.set_view(&world_mat);

		let vert_size = (3*4) as isize;

		unsafe {
			gl::EnableVertexAttribArray(0);
			gl::EnableVertexAttribArray(1);

			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(0));

			gl::BindBuffer(gl::ARRAY_BUFFER, self.health_vbo);
			gl::VertexAttribPointer(1, 1, gl::FLOAT, gl::FALSE, 4, transmute(0));

			gl::DrawElements(gl::TRIANGLES, 18, gl::UNSIGNED_SHORT, transmute(0));
		}
	}
}