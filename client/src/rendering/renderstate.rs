use rendering::gl;
use rendering::types::*;

use std::f32::consts::PI;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
	pos: (f32, f32, f32),
	color: Color,
}

impl Vertex {
	fn new(p: Vec2, color: Color) -> Vertex {
		let pos = (p.x, p.y, 0.0);
		Vertex {pos, color}
	}
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum StencilOp {
	Keep,
	Zero,
	Replace,
	Invert,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum StencilFunc {
	Never,
	Less,
	LessEqual,
	Equal,
	NotEqual,
	GreaterEqual,
	Greater,
	Always,
}

impl StencilOp {
	pub fn to_gl(&self) -> u32 {
		use self::StencilOp::*;

		match *self {
			Keep => gl::KEEP,
			Zero => gl::ZERO,
			Invert => gl::INVERT,
			Replace => gl::REPLACE,
		}
	}
}

impl StencilFunc {
	pub fn to_gl(&self) -> u32 {
		use self::StencilFunc::*;

		match *self {
			Never => gl::NEVER,
			Less => gl::LESS,
			LessEqual => gl::LEQUAL,
			Equal => gl::EQUAL,
			NotEqual => gl::NOTEQUAL,
			GreaterEqual => gl::GEQUAL,
			Greater => gl::GREATER,
			Always => gl::ALWAYS,
		}
	}
}

#[derive(Copy, Clone)]
enum Command {
	Geom{start: u32, count: u32},
	FullscreenQuad(Color),
	Stencil{
		func: StencilFunc,
		reference: u8,
		mask: u8,

		stencil_fail: StencilOp,
		depth_fail: StencilOp,
		pass: StencilOp,
	},

	StencilTest(bool),
	ColorWrite(bool),
	DepthWrite(bool),
}

pub struct RenderState {
	pub viewport: Viewport,

	commands: Vec<Command>,

	verts: Vec<Vertex>,
	indices: Vec<u16>,
	render_start_idx: u32,

	vbo: u32,
	ebo: u32,
	fs_quad: u32,
}

#[allow(dead_code)]
impl RenderState {
	pub fn new() -> Self {
		let mut vbos = [0u32; 3];
		unsafe { gl::GenBuffers(3, vbos.as_mut_ptr()); }

		RenderState {
			viewport: Viewport::new(),

			commands: Vec::new(),

			verts: Vec::new(),
			indices: Vec::new(),
			render_start_idx: 0,

			vbo: vbos[0],
			ebo: vbos[1],

			fs_quad: vbos[2],
		}
	}

	pub fn set_viewport(&mut self, vp: &Viewport) {
		self.viewport = *vp;
	}

	pub fn clear(&mut self) {
		self.commands.clear();
		self.verts.clear();
		self.indices.clear();
		self.render_start_idx = 0;
	}

	pub fn flush_geom(&mut self) {
		let num_indices = self.indices.len() as u32;

		if num_indices - self.render_start_idx > 0 {
			self.commands.push(Command::Geom{
				start: self.render_start_idx,
				count: num_indices - self.render_start_idx,
			});

			self.render_start_idx = num_indices;
		}
	}

	pub fn render(&self) {
		if self.verts.len() < 3 { return }
		if self.indices.len() < 3 { return }

		// TODO: assert that our buffers were generated in the current webgl context

		unsafe {
			use std::mem::{transmute, size_of, size_of_val};

			let vert_size = size_of::<Vertex>();
			let short_size = size_of::<u16>();
			gl::EnableVertexAttribArray(0);
			gl::EnableVertexAttribArray(1);

			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
			gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (self.indices.len()*short_size) as isize, transmute(self.indices.as_ptr()), gl::STREAM_DRAW);

			gl::BindBuffer(gl::ARRAY_BUFFER, self.fs_quad);

			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BufferData(gl::ARRAY_BUFFER, (self.verts.len()*vert_size) as isize, transmute(self.verts.as_ptr()), gl::STREAM_DRAW);
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(0));
			gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(12));

			let mut fs_quad_bound = false;

			for c in &self.commands {
				match *c {
					Command::Geom{start, count} => {
						if fs_quad_bound {
							fs_quad_bound = false;
							gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
							gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(0));
							gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(12));
						}

						gl::DrawElements(gl::TRIANGLES, count as i32, gl::UNSIGNED_SHORT, transmute(start*short_size as u32));
					},

					Command::FullscreenQuad(color) => {
						if !fs_quad_bound {
							fs_quad_bound = true;
							gl::BindBuffer(gl::ARRAY_BUFFER, self.fs_quad);
							gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(0));
							gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(12));
						}

						let fsquad = [
							Vertex::new(self.viewport.get_bottom_left(), color),
							Vertex::new(self.viewport.get_top_left(), color),
							Vertex::new(self.viewport.get_top_right(), color),
							Vertex::new(self.viewport.get_bottom_right(), color),
						];

						gl::BufferData(gl::ARRAY_BUFFER, size_of_val(&fsquad) as isize, transmute(fsquad.as_ptr()), gl::STREAM_DRAW);
						gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
					},

					Command::Stencil{func, reference, mask, stencil_fail, depth_fail, pass} => {
						gl::StencilFunc(func.to_gl(), reference as i32, mask as u32);
						gl::StencilOp(stencil_fail.to_gl(), depth_fail.to_gl(), pass.to_gl());
					},

					Command::StencilTest(enabled) => {
						if enabled {
							gl::Enable(gl::STENCIL_TEST);
						} else {
							gl::Disable(gl::STENCIL_TEST);
						}
					}

					Command::ColorWrite(enable) => {
						let v = enable as u8;
						gl::ColorMask(v, v, v, v);
					},
					Command::DepthWrite(enable) => {
						gl::DepthMask(enable as u8);
					},
				}
			}
		}
	}

	pub fn draw_fullscreen_quad(&mut self, col: Color) {
		self.commands.push(Command::FullscreenQuad(col));
	}

	pub fn start_stencil_write_if(&mut self, reference: u8, mask: u8, so: StencilOp, sf: StencilFunc) {
		use self::StencilOp as SO;

		self.flush_geom();

		self.commands.push(Command::StencilTest(true));
		self.commands.push(Command::ColorWrite(false));
		self.commands.push(Command::DepthWrite(false));

		self.commands.push(Command::Stencil {
			func: sf,
			reference, mask,

			stencil_fail: SO::Keep,
			depth_fail: SO::Keep,
			pass: so,
		});
	}

	pub fn start_stencil_write(&mut self, reference: u8, mask: u8, so: StencilOp) {
		self.start_stencil_write_if(reference, mask, so, StencilFunc::Always);
	}

	pub fn start_stencil_replace(&mut self, reference: u8) {
		self.start_stencil_write(reference, 0xff, StencilOp::Replace);
	}

	pub fn start_stencil_replace_if(&mut self, func: StencilFunc, reference: u8) {
		self.start_stencil_write_if(reference, 0xff, StencilOp::Replace, func);
	}

	pub fn start_stencil_erase(&mut self) {
		self.start_stencil_write(0, 0xff, StencilOp::Zero);
	}

	pub fn start_stencil_invert(&mut self) {
		self.start_stencil_write(0, 0xff, StencilOp::Invert);
	}

	pub fn start_stencilled_draw(&mut self, func: StencilFunc, reference: u8) {
		use self::StencilOp as SO;

		self.flush_geom();

		self.commands.push(Command::StencilTest(true));
		self.commands.push(Command::ColorWrite(true));
		self.commands.push(Command::DepthWrite(true));

		self.commands.push(Command::Stencil {
			func, reference, mask: 0xff,

			stencil_fail: SO::Keep,
			depth_fail: SO::Keep,
			pass: SO::Keep,
		});
	}

	pub fn stop_stencil_draw(&mut self) {
		self.flush_geom();
		self.commands.push(Command::StencilTest(false));
		self.commands.push(Command::ColorWrite(true));
		self.commands.push(Command::DepthWrite(true));
	}

	pub fn build_poly_rot(&mut self, pos: Vec2, col: Color, points: u32, s: f32, r: f32) {
		if points <= 2 { return }

		let start_idx = self.verts.len() as u16;

		let inc = 2.0 * PI / points as f32;
		let r = PI/2.0 + r;

		for i in 0..points {
			let th = i as f32 * inc + r;
			let p = pos + Vec2::from_angle(th) * s;
			self.verts.push(Vertex::new(p, col));
		}

		for i in 1..(points-1) as u16 {
			self.indices.push(start_idx);
			self.indices.push(start_idx + i);
			self.indices.push(start_idx + i + 1);
		}
	}

	pub fn build_ring_rot(&mut self, pos: Vec2, col: Color, points: u32, s: f32, thickness: f32, r: f32) {
		if points <= 2 { return }

		let start_idx = self.verts.len() as u16;

		let inc = 2.0 * PI / points as f32;
		let r = PI/2.0 + r;

		for i in 0..points {
			let th = i as f32 * inc + r;
			let p0 = pos + Vec2::from_angle(th) * s;
			let p1 = pos + Vec2::from_angle(th) * (s+thickness);
			self.verts.push(Vertex::new(p0, col));
			self.verts.push(Vertex::new(p1, col));
		}

		let points = points as u16;

		for i in 0..points*2 {
			self.indices.push(start_idx + i);
			self.indices.push(start_idx + (i + 1)%(points*2));
			self.indices.push(start_idx + (i + 2)%(points*2));
		}
	}

	pub fn build_poly(&mut self, pos: Vec2, col: Color, points: u32, s: f32) {
		self.build_poly_rot(pos, col, points, s, 0.0);
	}

	pub fn build_ring(&mut self, pos: Vec2, col: Color, points: u32, s: f32, thickness: f32) {
		self.build_ring_rot(pos, col, points, s, thickness, 0.0);
	}

	pub fn build_from_convex(&mut self, vs: &[Vec2], col: Color) {
		let points = vs.len();
		if points <= 2 { return }

		let start_idx = self.verts.len() as u16;

		for &p in vs.iter() {
			self.verts.push(Vertex::new(p, col));
		}

		for i in 1..(points-1) as u16 {
			self.indices.push(start_idx);
			self.indices.push(start_idx + i);
			self.indices.push(start_idx + i + 1);
		}
	}

	pub fn build_arc_segs(&mut self, pos: Vec2, col: Color, begin: f32, end: f32, radius: f32, segs: u32) {
		let (begin, end) = (begin.min(end), begin.max(end));

		let start_idx = self.verts.len() as u16;

		let diff = end-begin;
		let seg_width = diff / segs as f32;

		self.verts.push(Vertex::new(pos, col));
		for i in 0..(segs+1) {
			let offset = Vec2::from_angle(begin+seg_width*i as f32) * radius;
			self.verts.push(Vertex::new(pos + offset, col));
		}

		for i in 1..(segs+1) as u16 {
			self.indices.push(start_idx);
			self.indices.push(start_idx + i);
			self.indices.push(start_idx + i + 1);
		}
	}

	pub fn build_arc(&mut self, pos: Vec2, col: Color, begin: f32, end: f32, radius: f32) {
		let max_seg_width = PI / 16.0;
		let segs = ((end-begin).abs() / max_seg_width).ceil() as u32;

		self.build_arc_segs(pos, col, begin, end, radius, segs);
	}
}