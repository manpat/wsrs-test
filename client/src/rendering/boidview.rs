use common::math::*;
use boids;

use rendering::gl;
use rendering::types::*;
use rendering::shader::Shader;

use std::mem::size_of;

static BOID_VERT_SRC: &'static str = include_str!("../../assets/boid.vert");
static BOID_FRAG_SRC: &'static str = include_str!("../../assets/boid.frag");

pub struct BoidView {
	vbo: u32,
	shader: Shader,
	num_points: u32,
}

impl BoidView {
	pub fn new() -> BoidView {
		BoidView {
			vbo: unsafe {
				let mut vbo = 0u32;
				gl::GenBuffers(1, &mut vbo);
				vbo
			},

			shader: Shader::new(&BOID_VERT_SRC, &BOID_FRAG_SRC),

			num_points: 0,
		}
	}

	pub fn update(&mut self, system: &boids::BoidSystem) {
		let bs: Vec<Vec3> = system.get_boids()
			.iter()
			.map(|b| Vec3::new(b.pos.x, 1.0 + b.phase.cos() * 0.1, b.pos.y))
			.collect();

		self.num_points = bs.len() as u32;

		unsafe {
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BufferData(gl::ARRAY_BUFFER,
				(bs.len() * size_of::<Vec3>()) as isize,
				bs.as_ptr() as *const _,
				gl::STREAM_DRAW);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
	}

	pub fn render(&self, view: &Mat4) {
		if self.num_points < 1 { return }

		unsafe {
			self.shader.use_program();
			self.shader.set_view(view);
			self.shader.set_uniform_vec3("color", &Color::grey(0.2).to_vec3());

			gl::EnableVertexAttribArray(0);

			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size_of::<Vec3>() as i32, 0 as *const _);

			gl::DrawArrays(gl::POINTS, 0, self.num_points as i32);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
	}
}