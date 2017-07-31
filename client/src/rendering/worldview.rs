use rendering::gl;
use rendering::types::*;
use rendering::shader::Shader;

use rendering::parser_3ds::*;
use rendering::texture::*;

use std::f32::consts::PI;
use std::mem::{transmute, size_of, size_of_val};

static TERRAIN_VERT_SRC: &'static str = include_str!("../../assets/terrain.vert");
static TERRAIN_FRAG_SRC: &'static str = include_str!("../../assets/terrain.frag");
static WORLD_VERT_SRC: &'static str = include_str!("../../assets/world.vert");
static WORLD_FRAG_SRC: &'static str = include_str!("../../assets/world.frag");

// static TEST_MODEL: &'static [u8] = include_bytes!("../../assets/test_model.3ds");
static TEST_MODEL: &'static [u8] = include_bytes!("../../assets/forestconcept.3ds");

const MAP_SIZE: u32 = 14;

#[derive(Debug)]
struct TreeVertex {
	pos: Vec3,
	norm: Vec3,
}

struct TreeMesh {
	verts: Vec<TreeVertex>,
	// elems: Vec<u16>,

	trunk_start: i32,
	trunk_count: i32,

	leafage_start: i32,
	leafage_count: i32,
}

pub struct WorldView {
	shader: Shader,
	terrain: TerrainView,

	test_model: TreeMesh,
	// tree_palette: Texture,

	vbo: u32,
	// ebo: u32,

	trees: Vec<Vec3>,
	translation: Vec3,
}

static mut TIME: f32 = 0.0;

impl WorldView {
	pub fn new() -> WorldView {
		let mut bufs = [0u32; 2];
		unsafe{ gl::GenBuffers(1, bufs.as_mut_ptr()); }

		// let pal = [
		// 	// Color::rgb(0.633, 0.825, 0.250),

		// 	Color::rgb(0.633, 0.825, 0.250),
		// 	Color::rgb(0.531, 0.831, 0.248),
		// 	Color::rgb(0.455, 0.800, 0.213),
		// 	Color::rgb(0.339, 0.800, 0.205),

		// 	// Color::rgb(0.339, 0.800, 0.205),
		// ];

		// let mut tree_palette = TextureBuilder::new()
		// 	.linear_magnify()
		// 	.finalize();

		// tree_palette.upload_1d(&pal);

		let mut trees = Vec::new();

		for i in 1..MAP_SIZE-1 {
			let model_trans = Vec3::new(i as f32*2.0, 0.0, i as f32*2.0);
			trees.push(model_trans);
		}

		WorldView {
			shader: Shader::new(&WORLD_VERT_SRC, &WORLD_FRAG_SRC),
			terrain: TerrainView::new(),

			test_model: process_tree_mesh(parse_3ds(&TEST_MODEL).unwrap()),
			// tree_palette,

			vbo: bufs[0],
			// ebo: bufs[1],

			translation: Vec3::zero(),
			trees,
		}
	}

	pub fn update(&mut self, _dt: f32) {

	}

	pub fn render(&mut self, vp: &Viewport) {
		unsafe{ TIME += 0.016; }

		let ph = unsafe{TIME};
		let xrotph = PI/6.0;
		let yrotph = PI/4.0;

		let sc = 0.2;
		let scale = Vec3::new(1.0/vp.get_aspect(), 1.0, 1.0/10.0);
		let trans = Vec3::new(0.0, 0.0, 3.0);

		let center = (MAP_SIZE as f32 - 1.0) * 2.0f32.sqrt();
		let world_trans = Vec3::new(-center, 0.0, 0.0);

		let world_mat = Mat4::scale(scale)
			* Mat4::uniform_scale(sc)
			* Mat4::translate(trans)
			* Mat4::xrot(-xrotph)
			* Mat4::translate(world_trans + self.translation * Vec3::new(1.0/sc, 1.0, 1.0/(sc*xrotph.sin())))
			* Mat4::yrot(-yrotph);

		let normal_mat = Mat4::yrot(-yrotph);

		unsafe {
			gl::ClearColor(0.4f32.powf(1.0/2.2), 0.808f32.powf(1.0/2.2), 0.58f32.powf(1.0/2.2), 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}

		self.terrain.render(&world_mat);

		self.shader.use_program();
		self.shader.set_uniform_mat("normal_xform", &normal_mat);

		unsafe {
			gl::Enable(gl::DEPTH_TEST);

			gl::EnableVertexAttribArray(0);
			gl::EnableVertexAttribArray(1);

			let vert_size = size_of::<TreeVertex>() as isize;

			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BufferData(gl::ARRAY_BUFFER, self.test_model.verts.len() as isize * vert_size,
				transmute(self.test_model.verts.as_ptr()), gl::STREAM_DRAW);

			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(0));
			gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(size_of::<Vec3>()));

			let trunk_color = Color::rgb(0.8, 0.411, 0.22);

			for &p in &self.trees {
				self.shader.set_view(&(world_mat * Mat4::translate(p)));

				let divisor = (MAP_SIZE as f32 - 1.0) * 2.0;
				let health = p.z / divisor;

				self.shader.set_uniform_vec3("color", &trunk_color.to_vec3());
				gl::DrawArrays(gl::TRIANGLES, self.test_model.trunk_start as i32 * 3, self.test_model.trunk_count as i32 * 3);

				self.shader.set_uniform_vec3("color", &WorldView::get_tree_color(health).to_vec3());
				gl::DrawArrays(gl::TRIANGLES, self.test_model.leafage_start as i32 * 3, self.test_model.leafage_count as i32 * 3);
			}
		}
	}

	pub fn try_world_translate(&mut self, v: Vec2) {
		// let rot = Mat4::yrot(-yrotph);
		let Vec2{x,y} = v;

		self.translation = self.translation + Vec3::new(x, 0.0, y);
	}

	pub fn try_place_tree(&mut self, p: Vec2) {
		let Vec2{x,y} = p;

		let xrot = PI/6.0;
		let normal_mat = Mat4::yrot(PI/4.0);
		let center = Vec3::new((MAP_SIZE as f32 - 1.0) * 2.0f32.sqrt(), 0.0, 0.0);

		self.trees.push(normal_mat * ((Vec3::new(x, 0.0, y) - self.translation) * Vec3::new(1.0/0.2, 1.0, 1.0/(0.2*xrot.sin())) + center));
	}

	fn get_tree_color(health: f32) -> Color {
		let pal = [
			Color::rgb(0.778, 0.895, 0.241),
			Color::rgb(0.593, 0.928, 0.257),
			Color::rgb(0.348, 0.800, 0.185),
			Color::rgb(0.197, 0.800, 0.202),
		];

		let real_idx = health * pal.len() as f32;
		let real_next = real_idx + 1.0;

		let pal_idx = (real_idx as usize).max(0).min(pal.len()-1);
		let pal_next = (real_next as usize).max(0).min(pal.len()-1);

		let col_a = pal[pal_idx];
		let col_b = pal[pal_next];
		return real_idx.fract().ease_linear(col_a, col_b, 1.0);
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
			Color::rgb(0.633, 0.825, 0.250),
			Color::rgb(0.531, 0.831, 0.248),
			Color::rgb(0.455, 0.800, 0.213),
			Color::rgb(0.339, 0.800, 0.205),
		];

		let mut terrain_palette = TextureBuilder::new()
			.linear_magnify()
			.finalize();

		terrain_palette.upload_1d(&pal);

		let mut view = TerrainView {
			shader: Shader::new(&TERRAIN_VERT_SRC, &TERRAIN_FRAG_SRC),
			terrain_palette,
			
			health_vbo: bufs[0],
			vbo: bufs[1],
			ebo: bufs[2],
		};

		view.build_main_buffers();
		view
	}

	fn build_main_buffers(&mut self) {
		let mut vs = Vec::new();
		let mut es: Vec<u16> = Vec::new();

		for y in 0..MAP_SIZE {
			for x in 0..MAP_SIZE {
				let vsbase = vs.len() as u16;
				vs.push(Vec3::new(-1.0, 0.0, 1.0) + Vec3::new(x as f32 * 2.0, 0.0, y as f32 * 2.0));
				vs.push(Vec3::new(-1.0, 0.0,-1.0) + Vec3::new(x as f32 * 2.0, 0.0, y as f32 * 2.0));
				vs.push(Vec3::new( 1.0, 0.0,-1.0) + Vec3::new(x as f32 * 2.0, 0.0, y as f32 * 2.0));
				vs.push(Vec3::new( 1.0, 0.0, 1.0) + Vec3::new(x as f32 * 2.0, 0.0, y as f32 * 2.0));

				es.extend(&[
					vsbase + 0, vsbase + 1, vsbase + 2,
					vsbase + 0, vsbase + 2, vsbase + 3
				]);
			}
		}

		unsafe {
			let vert_size = (3*4) as isize;

			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
			gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (es.len()*2) as isize, transmute(es.as_ptr()), gl::STATIC_DRAW);

			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BufferData(gl::ARRAY_BUFFER, vs.len() as isize * vert_size, transmute(vs.as_ptr()), gl::STATIC_DRAW);
		}
	}

	fn build_health_vbo(&mut self) {
		let mut data = Vec::new();

		let divisor = (MAP_SIZE*MAP_SIZE - 1) as f32;

		for i in 0..MAP_SIZE*MAP_SIZE {
			data.push(i as f32 / divisor);
			data.push(i as f32 / divisor);
			data.push(i as f32 / divisor);
			data.push(i as f32 / divisor);
		}

		unsafe {
			gl::BindBuffer(gl::ARRAY_BUFFER, self.health_vbo);
			gl::BufferData(gl::ARRAY_BUFFER, data.len() as isize * 4, transmute(data.as_ptr()), gl::STREAM_DRAW);
		}
	}

	fn render(&mut self, world_mat: &Mat4) {
		self.build_health_vbo();

		self.shader.use_program();
		self.shader.set_view(&world_mat);
		self.shader.set_uniform_mat("normal_xform", &Mat4::yrot(-PI/4.0));

		let vert_size = size_of::<Vec3>() as i32;

		unsafe {
			gl::EnableVertexAttribArray(0);
			gl::EnableVertexAttribArray(1);

			self.terrain_palette.bind_to_slot(0);
			self.shader.set_uniform_i32("health_lut", 0);

			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);

			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vert_size, transmute(0));

			gl::BindBuffer(gl::ARRAY_BUFFER, self.health_vbo);
			gl::VertexAttribPointer(1, 1, gl::FLOAT, gl::FALSE, 4, transmute(0));

			gl::DrawElements(gl::TRIANGLES, (MAP_SIZE * MAP_SIZE) as i32 * 6, gl::UNSIGNED_SHORT, transmute(0));

			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
		}
	}
}

fn process_tree_mesh(mesh: Mesh3DS) -> TreeMesh {
	let mut tree = TreeMesh {
		verts: Vec::new(),
		// elems: mesh.elements,

		trunk_start: 0,
		trunk_count: 0,

		leafage_start: 0,
		leafage_count: 0,
	};

	let overts = mesh.verts;
	let elems = mesh.elements;

	for &FaceMaterial(ref name, ref faces) in &mesh.face_materials {
		let start = *faces.first().unwrap();
		for (i, &f) in faces.iter().enumerate() {
			assert!(i as u16 + start == f);
		}

		for &f in faces.iter() {
			let els = &elems[f as usize*3..];
			let ps = [
				overts[els[0] as usize],
				overts[els[1] as usize],
				overts[els[2] as usize],
			];

			let norm = (ps[1] - ps[0]).normalize().cross(ps[2] - ps[0]).normalize();

			tree.verts.push(TreeVertex{pos: ps[0], norm});
			tree.verts.push(TreeVertex{pos: ps[1], norm});
			tree.verts.push(TreeVertex{pos: ps[2], norm});
		}

		match name.as_ref() {
			"Trunk" => {
				tree.trunk_start = start as i32;
				tree.trunk_count = faces.len() as i32;
			}

			"Leafage" => {
				tree.leafage_start = start as i32;
				tree.leafage_count = faces.len() as i32;
			}

			_ => {}
		}
	}

	println!("{:?}", tree.verts);

	// println!("trunk: {:?}      leafage: {:?}", trunk_start, leafage_start);

	tree
}