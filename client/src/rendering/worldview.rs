use rendering::gl;
use rendering::types::*;
use rendering::shader::Shader;

use rendering::parser_3ds::*;
use rendering::texture::*;

use std::f32::consts::PI;
use std::mem::{transmute, size_of, size_of_val};

use common::world::*;

use boids::BoidSystem;
use rendering::boidview::BoidView;

static TERRAIN_VERT_SRC: &'static str = include_str!("../../assets/terrain.vert");
static TERRAIN_FRAG_SRC: &'static str = include_str!("../../assets/terrain.frag");
static WORLD_VERT_SRC: &'static str = include_str!("../../assets/world.vert");
static WORLD_FRAG_SRC: &'static str = include_str!("../../assets/world.frag");

static TREE_MODELS: [&'static [u8]; 4] = [
	include_bytes!("../../assets/tree0.3ds"),
	include_bytes!("../../assets/tree1.3ds"),
	include_bytes!("../../assets/tree2.3ds"),
	include_bytes!("../../assets/tree3.3ds"),
];

pub const MAP_SIZE: u32 = 28;
const MAP_MEM_SIZE: usize = (MAP_SIZE*MAP_SIZE) as usize;

pub const TILE_SIZE: f32 = 2.0;

#[derive(Debug, Clone, Copy)]
struct TreeVertex {
	pos: Vec3,
	norm: Vec3,
}

struct TreeMesh {
	verts: Vec<TreeVertex>,

	trunk_start: i32,
	trunk_count: i32,

	leafage_start: i32,
	leafage_count: i32,
}

struct TreeInstance {
	id: u32,
	pos: Vec3,
	stage: u8,
	species: Species,

	current_health: f32,
}

pub struct WorldView {
	shader: Shader,
	terrain: TerrainView,
	boidview: BoidView,
	boids: BoidSystem,

	world_scale: f32,

	tree_models: [TreeMesh; 4],
	tree_starts: [i32; 4],
	tree_vbo: u32,

	trees: Vec<TreeInstance>,
	translation: Vec3,
}

static mut TIME: f32 = 0.0;

impl WorldView {
	pub fn new() -> WorldView {
		let world_scale = 1.0 / 7.0;
		// let world_scale = 1.0 / (MAP_SIZE as f32 - 1.0) * 2.0f32.sqrt();

		let mut view = WorldView {
			shader: Shader::new(&WORLD_VERT_SRC, &WORLD_FRAG_SRC),
			terrain: TerrainView::new(),

			boids: BoidSystem::new(Vec2::splat(MAP_SIZE as f32 * TILE_SIZE)),
			boidview: BoidView::new(),

			tree_models: [
				process_tree_mesh(parse_3ds(&TREE_MODELS[0]).unwrap()),
				process_tree_mesh(parse_3ds(&TREE_MODELS[1]).unwrap()),
				process_tree_mesh(parse_3ds(&TREE_MODELS[2]).unwrap()),
				process_tree_mesh(parse_3ds(&TREE_MODELS[3]).unwrap())
			],

			tree_starts: [0i32; 4],

			tree_vbo: unsafe {
				let mut vbo = 0u32;
				gl::GenBuffers(1, &mut vbo);
				vbo
			},

			// Center the starting view
			translation: Vec3::new(-(MAP_SIZE as f32 - 1.0) * 2.0f32.sqrt() * TILE_SIZE * world_scale/2.0, 0.0, 0.0),
			trees: Vec::new(),

			world_scale,
		};

		view.build_tree_buffer();

		for _ in 0..30 {
			view.boids.update(1.0/2.0);
		}

		view
	}

	pub fn update(&mut self, dt: f32) {
		self.boids.update(dt);
		self.boidview.update(&self.boids);

		for tree in self.trees.iter_mut() {
			let health = self.terrain.get_health_at(Vec2::new(tree.pos.x, tree.pos.z));
			tree.current_health = (dt/16.0).ease_linear(tree.current_health, health);
		}
	}

	pub fn render(&mut self, vp: &Viewport) {
		unsafe{ TIME += 0.016; }

		let ph = unsafe{TIME};
		let xrotph = PI/6.0;
		let yrotph = PI/4.0;

		let sc = self.world_scale;
		let scale = Vec3::new(1.0/vp.get_aspect(), 1.0, 1.0/10.0);
		let trans = Vec3::new(0.0, 0.0, 3.0);

		let world_mat = Mat4::scale(scale)
			* Mat4::uniform_scale(sc)
			* Mat4::translate(trans)
			* Mat4::xrot(-xrotph)
			* Mat4::translate(self.translation * Vec3::new(1.0/sc, 1.0, 1.0/(sc*xrotph.sin())))
			* Mat4::yrot(-yrotph);

		let normal_mat = Mat4::yrot(-yrotph);

		unsafe {
			let clear_col = Color::rgb(0.4, 0.808, 0.58).pow(1.0/2.2);
			gl::ClearColor(clear_col.r, clear_col.g, clear_col.b, 1.0);
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

			gl::BindBuffer(gl::ARRAY_BUFFER, self.tree_vbo);
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(0));
			gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, vert_size as i32, transmute(size_of::<Vec3>()));

			let trunk_color = Color::rgb(0.8, 0.411, 0.22);

			for tree in &self.trees {
				self.shader.set_view(&(world_mat * Mat4::translate(tree.pos * TILE_SIZE)));

				let tree_idx = tree.stage as usize;

				let trunk_count = self.tree_models[tree_idx].trunk_count;
				let leafage_count = self.tree_models[tree_idx].leafage_count;

				let base_start = self.tree_starts[tree_idx];
				let trunk_start = self.tree_models[tree_idx].trunk_start;
				let leafage_start = self.tree_models[tree_idx].leafage_start;

				if trunk_count > 0 {
					self.shader.set_uniform_vec3("color", &trunk_color.to_vec3());
					gl::DrawArrays(gl::TRIANGLES, base_start + trunk_start as i32 * 3, trunk_count as i32 * 3);
				}

				if leafage_count > 0 {
					let color = WorldView::get_tree_color(tree.current_health, tree.species);
					self.shader.set_uniform_vec3("color", &color.to_vec3());
					gl::DrawArrays(gl::TRIANGLES, base_start + leafage_start as i32 * 3, leafage_count as i32 * 3);
				}
			}

			gl::DisableVertexAttribArray(1);

			self.boidview.render(&world_mat, vp.size.y as f32 / 967.0);
		}
	}

	fn build_tree_buffer(&mut self) {
		unsafe {
			let vert_size = size_of::<TreeVertex>() as isize;

			let mut verts = Vec::new();

			for (tree, start) in self.tree_models.iter().zip(self.tree_starts.iter_mut()) {
				*start = verts.len() as i32;
				verts.extend_from_slice(&tree.verts);
			}

			gl::BindBuffer(gl::ARRAY_BUFFER, self.tree_vbo);
			gl::BufferData(gl::ARRAY_BUFFER, verts.len() as isize * vert_size,
				transmute(verts.as_ptr()), gl::STATIC_DRAW);

			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
	}

	pub fn try_world_translate(&mut self, v: Vec2) {
		// let rot = Mat4::yrot(-yrotph);
		let Vec2{x,y} = v;

		self.translation = self.translation + Vec3::new(x, 0.0, y);
	}

	pub fn convert_to_world_coords(&self, p: Vec2) -> Vec3 {
		let Vec2{x,y} = p;

		let sc = self.world_scale*TILE_SIZE;
		let xrot = PI/6.0;
		let normal_mat = Mat4::yrot(PI/4.0);

		normal_mat * ((Vec3::new(x, 0.0, y) - self.translation) * Vec3::new(1.0/sc, 1.0, 1.0/(sc*xrot.sin())))
	}

	pub fn place_tree(&mut self, id: u32, pos: Vec3, species: Species) {
		self.trees.push(TreeInstance {
			id, pos,
			stage: 0, species,
			current_health: self.terrain.get_health_at(Vec2::new(pos.x, pos.z)),
		});
	}

	pub fn set_tree_stage(&mut self, id: u32, stage: u8) {
		for tree in self.trees.iter_mut() {
			if tree.id == id { tree.stage = stage; } 
		}
	}

	pub fn kill_tree(&mut self, id: u32) {
		self.trees.retain(|tree| tree.id != id);
	}

	fn get_tree_color(health: f32, species: Species) -> Color {
		let pal = match species {
			Species::A => [
				Color::rgb(0.778, 0.895, 0.241),
				Color::rgb(0.593, 0.928, 0.257),
				Color::rgb(0.348, 0.800, 0.185),
				Color::rgb(0.197, 0.800, 0.202),
			],

			Species::B => [
				Color::rgb(0.6, 0.7, 0.8),
				Color::rgb(0.5, 0.7, 0.9),
				Color::rgb(0.4, 0.6, 0.9),
				Color::rgb(0.4, 0.6, 1.0),
			],

			Species::C => [
				Color::rgb(0.8, 0.8, 0.7),
				Color::rgb(0.9, 0.8, 0.6),
				Color::rgb(1.0, 0.7, 0.5),
				Color::rgb(1.0, 0.5, 0.5),
			],
		};

		let real_idx = health * pal.len() as f32;
		let real_next = real_idx + 1.0;

		let pal_idx = (real_idx as usize).max(0).min(pal.len()-1);
		let pal_next = (real_next as usize).max(0).min(pal.len()-1);

		let col_a = pal[pal_idx];
		let col_b = pal[pal_next];
		return real_idx.fract().ease_linear(col_a, col_b);
	}

	pub fn update_health_state(&mut self, hs: Vec<u8>) {
		self.boids.update_health_state(&hs);
		self.terrain.update_health_state(hs);
	}

	pub fn update_tree_maturities(&mut self, ts: Vec<(u32, u8)>) {
		for (id, stage) in ts {
			self.set_tree_stage(id, stage);
		}
	}
}

struct TerrainView {
	shader: Shader,
	terrain_palette: Texture,

	health_state: Vec<u8>,
	
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

			health_state: vec![0; MAP_MEM_SIZE],
			
			health_vbo: bufs[0],
			vbo: bufs[1],
			ebo: bufs[2],
		};

		view.build_main_buffers();
		view.build_health_vbo();
		view
	}

	fn get_health_at(&self, p: Vec2) -> f32 {
		let y = (p.y as usize).min(MAP_SIZE as usize - 1);
		let x = (p.x as usize).min(MAP_SIZE as usize - 1);

		self.health_state[x + y * MAP_SIZE as usize] as f32 / 255.0
	}

	fn build_main_buffers(&mut self) {
		let mut vs = Vec::new();
		let mut es: Vec<u16> = Vec::new();

		for y in 0..MAP_SIZE {
			for x in 0..MAP_SIZE {
				let vsbase = vs.len() as u16;
				vs.push((Vec3::new(-0.5, 0.0, 0.5) + Vec3::new(x as f32, 0.0, y as f32)) * TILE_SIZE);
				vs.push((Vec3::new(-0.5, 0.0,-0.5) + Vec3::new(x as f32, 0.0, y as f32)) * TILE_SIZE);
				vs.push((Vec3::new( 0.5, 0.0,-0.5) + Vec3::new(x as f32, 0.0, y as f32)) * TILE_SIZE);
				vs.push((Vec3::new( 0.5, 0.0, 0.5) + Vec3::new(x as f32, 0.0, y as f32)) * TILE_SIZE);

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
		assert!(self.health_state.len() == MAP_MEM_SIZE);

		let mut data = Vec::with_capacity(MAP_MEM_SIZE*4);

		let s = 1.0/2.0;

		for y in 0..MAP_SIZE {
			for x in 0..MAP_SIZE {
				let (x,y) = (x as f32, y as f32);
				data.push(self.get_health_at(Vec2::new(x-s, y+s)));
				data.push(self.get_health_at(Vec2::new(x-s, y-s)));
				data.push(self.get_health_at(Vec2::new(x+s, y-s)));
				data.push(self.get_health_at(Vec2::new(x+s, y+s)));
			}
		}

		unsafe {
			gl::BindBuffer(gl::ARRAY_BUFFER, self.health_vbo);
			gl::BufferData(gl::ARRAY_BUFFER, data.len() as isize * 4, transmute(data.as_ptr()), gl::STREAM_DRAW);
		}
	}

	fn render(&mut self, world_mat: &Mat4) {
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

	fn update_health_state(&mut self, hs: Vec<u8>) {
		if hs.len() != MAP_MEM_SIZE { return }
		self.health_state = hs;
		self.build_health_vbo();
	}
}

fn process_tree_mesh(mesh: Mesh3DS) -> TreeMesh {
	let mut tree = TreeMesh {
		verts: Vec::new(),

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

	// println!("{:?}", tree.verts);

	// println!("trunk: {:?}      leafage: {:?}", trunk_start, leafage_start);

	tree
}