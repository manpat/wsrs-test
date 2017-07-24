use rendering::gl;
use rendering::types::*;
use rendering::shader::Shader;

use rendering::parser_3ds::*;

use std::f32::consts::PI;

static VERT_SRC: &'static str = include_str!("../../assets/world.vert");
static FRAG_SRC: &'static str = include_str!("../../assets/world.frag");

static TEST_MODEL: &'static [u8] = include_bytes!("../../assets/test_model.3ds");

pub struct WorldView {
	shader: Shader,
	terrain: TerrainView,

	vbo: u32,
	ebo: u32,
}

struct TerrainView {
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
			shader: Shader::new(&VERT_SRC, &FRAG_SRC),
			terrain: TerrainView::new(),

			vbo: bufs[0],
			ebo: bufs[1],
		}
	}

	pub fn render(&mut self, vp: &Viewport) {
		use std::mem::{transmute, size_of, size_of_val};

		unsafe{ TIME += 0.016; }

		self.shader.use_program();

		let ph = unsafe{TIME};
		let xrotph = PI/6.0;
		let yrotph = PI/4.0;

		let sc = 0.1;
		let scale = Vec3::new(1.0/vp.get_aspect(), 1.0, 1.0);
		let trans = Vec3::new(0.0, 0.0, 3.0);

		let projmat = Mat4::ident()
			* Mat4::scale(scale)
			* Mat4::uniform_scale(sc)
			* Mat4::translate(trans)
			* Mat4::xrot(-xrotph)
			* Mat4::translate(Vec3::new(ph.cos(), 0.0, ph.sin()))
			* Mat4::yrot(-yrotph)
			;

		self.shader.set_view(&projmat);

		let vs = [
			Vec3::new(-1.0, 0.0, 1.0),
			Vec3::new(-1.0, 0.0,-1.0),
			Vec3::new( 1.0, 0.0,-1.0),
			Vec3::new( 1.0, 0.0, 1.0),

			Vec3::new(-1.0, 0.0, 1.0) + Vec3::new(0.0, 0.0,-3.0),
			Vec3::new(-1.0, 0.0,-1.0) + Vec3::new(0.0, 0.0,-3.0),
			Vec3::new( 1.0, 0.0,-1.0) + Vec3::new(0.0, 0.0,-3.0),
			Vec3::new( 1.0, 0.0, 1.0) + Vec3::new(0.0, 0.0,-3.0),
		];

		let es = [
			0, 1, 2, 0, 2, 3,
			4, 5, 6, 4, 6, 7u16,
		];

		unsafe {
			gl::EnableVertexAttribArray(0);
			gl::DisableVertexAttribArray(1);

			let vert_size = (3*4) as isize;

			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
			gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (es.len()*2) as isize, transmute(es.as_ptr()), gl::STREAM_DRAW);

			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BufferData(gl::ARRAY_BUFFER, vs.len() as isize * vert_size, transmute(vs.as_ptr()), gl::STREAM_DRAW);
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(0));
			// gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(12));

			gl::DrawElements(gl::TRIANGLES, 12, gl::UNSIGNED_SHORT, transmute(0));
		}
	}
}

impl TerrainView {
	pub fn new() -> TerrainView {
		let mut bufs = [0u32; 2];
		unsafe{ gl::GenBuffers(2, bufs.as_mut_ptr()); }

		TerrainView {
			vbo: bufs[0],
			ebo: bufs[1],
		}
	}
}