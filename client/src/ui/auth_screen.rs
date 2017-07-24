use rendering::RenderingContext;
use rendering::uibuilder::*;
use rendering::types::*;

use ui::InputTarget;

use std::f32::consts::PI;

const KEY_LENGTH: u32 = 9;
const KEY_BASE: u32 = 3;

#[derive(Copy, Clone)]
struct KeyTumbler {
	state: u8,

	pos: f32,
	anim_phase: f32,
	prev_pos: f32,
}

struct KeyRing {
	tumblers: [KeyTumbler; KEY_LENGTH as usize],
}

#[derive(Copy, Clone)]
enum StatusAnimation {
	Success,
	Fail,
	Connect,
	Disconnect,
}

struct StatusRing {
	phase: f32,

	position: Vec2,
	drag_target: Vec2,
	drag_offset: Vec2,
	is_dragging: bool,

	plus_offset: f32,
	aperture_phase: f32,

	animation: Option<StatusAnimation>,
	anim_phase: f32,
}

#[derive(Copy, Clone, Debug)]
pub enum AuthScreenAction {
	RequestNewSession,
	TryAuth(u32),
	EnterGame,
}

pub struct AuthScreen {
	key_ring: KeyRing,
	status_ring: StatusRing,

	pub viewport: Viewport,
	download_button_pos: Vec2,

	action: Option<AuthScreenAction>,

	hide_anim_phase: Option<f32>,
}

impl AuthScreen {
	pub fn new() -> Self {
		AuthScreen {
			key_ring: KeyRing::new(),
			status_ring: StatusRing::new(),

			viewport: Viewport::new(),
			download_button_pos: Vec2::zero(),

			action: None,

			hide_anim_phase: None,
		}
	}

	pub fn on_auth_success(&mut self) {
		self.status_ring.start_animation(StatusAnimation::Success);
		self.hide_anim_phase = Some(-0.5);
	}

	pub fn on_auth_fail(&mut self) {
		self.status_ring.start_animation(StatusAnimation::Fail);
	}

	pub fn on_connect(&mut self) {
		self.status_ring.start_animation(StatusAnimation::Connect);
	}

	pub fn on_disconnect(&mut self) {
		self.status_ring.start_animation(StatusAnimation::Disconnect);
	}

	pub fn poll_actions(&mut self) -> Option<AuthScreenAction> {
		let action = self.action;
		self.action = None;
		action
	}

	pub fn update(&mut self, dt: f32) {
		self.key_ring.update(dt);
		self.status_ring.update(dt);

		self.download_button_pos = self.viewport.get_bottom_left() + Vec2::new(0.14, 0.14);

		self.hide_anim_phase = match self.hide_anim_phase {
			Some(phase) if phase < 1.0 => Some(phase + dt),

			Some(_) => {
				self.action = Some(AuthScreenAction::EnterGame);
				None
			}

			None => None,
		}
	}

	pub fn render(&self, mut builder: &mut UIBuilder) {
		// TODO: some animation w/ self.hide_anim_phase

		self.key_ring.render(&mut builder);
		self.status_ring.render(&mut builder);

		builder.build_poly(self.download_button_pos, Color::grey(0.3), 15, 0.1);
	}

	pub fn download_key(&self) {
		use util;

		let mut tmp = RenderingContext::new("downloadcanvas");
		let mut tmpstate = UIBuilder::new();
		tmp.set_target_size(400, 400);
		tmpstate.set_viewport(&tmp.get_viewport());

		self.key_ring.render(&mut tmpstate);
		tmpstate.build_ring(Vec2::new(0.0, 0.0), Color::grey(0.25), 18, 0.12, 0.08);

		tmpstate.flush_geom();

		tmp.prepare_render();
		tmpstate.render();

		util::save_canvas("downloadcanvas");
	}

	pub fn calculate_key(&self) -> u32 {
		self.key_ring.calculate_key()
	}

	pub fn set_key(&mut self, key: u32) {
		self.key_ring.set_key(key);
	}
}

impl InputTarget for AuthScreen {
	fn on_drag_start(&mut self, pos: Vec2) {
		if self.status_ring.try_start_drag(pos) { return }
	}

	fn on_drag_end(&mut self, pos: Vec2) {
		if self.status_ring.on_drag_end(pos) {
			self.action = Some(AuthScreenAction::RequestNewSession);
		}
	}

	fn on_drag(&mut self, pos: Vec2) {
		self.status_ring.on_drag(pos);
	}

	fn on_click(&mut self, click_pos: Vec2) {
		let mut key_changed = false;

		if self.key_ring.on_click(click_pos) {
			key_changed = true;

		} else if self.status_ring.on_click(click_pos) {
			let key = self.calculate_key();
			println!("Requesting auth {}", key);
			self.action = Some(AuthScreenAction::TryAuth(key));
		}

		if (self.download_button_pos - click_pos).length() < 0.1 {
			self.download_key();
		}

		// if (self.viewport.get_top_left() - click_pos).length() < 0.1 {
		// 	use rand;

		// 	let max_key = KEY_BASE.pow(KEY_LENGTH);
		// 	let random_key = rand::random::<u32>() % max_key;

		// 	self.set_key(random_key);
		// 	key_changed = true;
		// }

		if key_changed {
			println!("New key: {}", self.calculate_key());
		}
	}
}

impl StatusRing {
	fn new() -> StatusRing {
		StatusRing {
			phase: 0.0,

			position: Vec2::new(0.0, 0.0),
			drag_target: Vec2::zero(),
			drag_offset: Vec2::zero(),
			is_dragging: false,

			plus_offset: 1.0,
			aperture_phase: 1.0,

			animation: None,
			anim_phase: 0.0,
		}
	}

	fn start_animation(&mut self, anim: StatusAnimation) {
		self.animation = Some(anim);
		self.anim_phase = 0.0;
	}

	fn on_click(&mut self, click_pos: Vec2) -> bool {
		(self.position - click_pos).length() < 0.2
	}

	fn try_start_drag(&mut self, click_pos: Vec2) -> bool {
		let diff = click_pos - self.position;
		let in_bounds = diff.length() < 0.2;

		if in_bounds {
			self.drag_offset = diff;
			self.drag_target = click_pos;
			self.is_dragging = true;
		}

		in_bounds
	}

	fn on_drag(&mut self, pos: Vec2) {
		if self.is_dragging {
			self.drag_target = pos;
		}
	}

	fn on_drag_end(&mut self, _pos: Vec2) -> bool {
		if self.is_dragging {
			self.is_dragging = false;

			// Is our center outside of the main ring
			self.position.length() > 0.5 + 0.12
		} else {
			false
		}
	}

	fn update(&mut self, dt: f32) {
		self.phase += dt;

		if self.animation.is_some() {
			self.anim_phase += dt;
		}

		if self.is_dragging {
			self.position = self.position + (self.drag_target - self.drag_offset - self.position) * (dt * 30.0).min(1.0);
			self.aperture_phase = (self.aperture_phase - dt * 6.0).max(0.0);
		} else {
			self.position = self.position * (1.0 - dt * 6.0).max(0.0);
			self.aperture_phase = (self.aperture_phase + dt * 6.0).min(1.0);
		}

		if self.is_dragging && self.position.length() > 0.5 + 0.12 {
			self.plus_offset = (self.plus_offset - dt*2.0).max(0.0);
		} else {
			self.plus_offset = (self.plus_offset + dt*2.0).min(1.0);
		}
	}

	fn render(&self, builder: &mut UIBuilder) {
		let main_shape_segs = 18;

		builder.start_stencil_erase();
		builder.draw_fullscreen_quad(Color::black());

		// Main circle -> stencil
		builder.start_stencil_replace(2);
		builder.build_poly(Vec2::new(0.0, 0.0), Color::black(), main_shape_segs, 0.5);

		// Status circle outside main -> stencil
		builder.start_stencil_replace_if(StencilFunc::Greater, 1);
		builder.build_poly(self.position, Color::black(), main_shape_segs, 0.12);
		
		// Mask main ring
		builder.start_stencil_erase();
		builder.build_ring(Vec2::new(0.0, 0.0), Color::black(), main_shape_segs, 0.45, 0.05);
		
		let plus_pos = self.position + self.plus_offset.ease_exp_in(Vec2::zero(), Vec2::new(0.0, -0.3), 1.0);
		builder.start_stencilled_draw(StencilFunc::Equal, 1);
		builder.build_poly(self.position, Color::white(), main_shape_segs, 0.12);
		builder.build_poly_rot(plus_pos, Color::rgb(0.3, 0.8, 0.6), 4, 0.08, PI/4.0);

		// Clear
		builder.start_stencil_erase();
		builder.draw_fullscreen_quad(Color::white());

		// Main circle -> stencil
		builder.start_stencil_replace(2);
		builder.build_poly(Vec2::new(0.0, 0.0), Color::black(), main_shape_segs, 0.45);

		let hole_mod = self.aperture_phase.ease_quad_inout(0.12, 0.0, 1.0);

		// Status ring inside main -> stencil
		builder.start_stencil_replace_if(StencilFunc::Less, 1);
		builder.build_ring(self.position, Color::black(), main_shape_segs, 0.12 - hole_mod, 0.08 + hole_mod);

		// Status ring fill
		builder.start_stencilled_draw(StencilFunc::Equal, 1);
		let ph = self.phase * PI * 2.0;
		let o = (ph/5.0).sin();
		let o2 = (ph/7.0).sin();
		let o3 = (ph/11.0).cos();
		builder.build_poly(Vec2::zero(), Color::rgb(0.6, 0.4, 0.9), 4, 0.5 * 2.0f32.sqrt());
		builder.build_poly(Vec2::new(o3*0.05, o*0.02 - 0.2), Color::rgb(0.8, 0.7, 0.4), main_shape_segs, 0.3);
		builder.build_poly(Vec2::new(o2*0.03 - 0.2, 0.1 - o3*0.05), Color::rgb(0.4, 0.6, 0.9), main_shape_segs, 0.3);
		builder.build_poly(Vec2::new(0.2 + o3 *0.1, o2 * 0.01 + o3*0.05), Color::rgb(0.4, 0.9, 0.6), main_shape_segs, 0.3);
		builder.build_poly(Vec2::new(0.2 + o*0.03, 0.3), Color::rgb(0.9, 0.4, 0.6), main_shape_segs, 0.3);

		match self.animation {
			Some(StatusAnimation::Fail) => {
				let r = self.anim_phase.ease_exp_out(0.12, 0.2, 0.75);
				let a = (self.anim_phase-0.5).ease_linear(1.0, 0.0, 1.0);
				builder.build_poly(self.position, Color::rgba(1.0, 0.4, 0.4, a), main_shape_segs, r);
			},

			Some(StatusAnimation::Success) => {
				let r = self.anim_phase.ease_exp_in(0.12, 0.2, 0.7);
				builder.build_poly(self.position, Color::rgb(0.4, 1.0, 0.4), main_shape_segs, r);
			},

			Some(StatusAnimation::Connect) => {
				let a = self.anim_phase.ease_quad_in(1.0, 0.0, 1.0);
				builder.build_poly(self.position, Color::grey_a(0.9, a), main_shape_segs, 0.2);
			},

			Some(StatusAnimation::Disconnect) => {
				let r = self.anim_phase.ease_exp_in(0.12, 0.2, 0.7);
				let a = self.anim_phase.ease_quad_in(0.3, 1.0, 1.0);
				builder.build_poly(self.position, Color::grey_a(0.4, a), main_shape_segs, r);
			},

			_ => {},
		}

		builder.stop_stencil_draw();
	}
}

impl KeyTumbler {
	fn new() -> KeyTumbler {
		KeyTumbler {
			state: 0,

			pos: 0.0,
			anim_phase: 0.0,
			prev_pos: 0.0,
		}
	}

	fn update(&mut self, dt: f32) {
		let target_pos = self.state as f32;
		self.pos = self.anim_phase.ease_back_out(self.prev_pos, target_pos, 0.8);

		self.anim_phase += dt;
	}

	fn set_state(&mut self, nstate: u8) {
		self.prev_pos = self.pos;
		self.anim_phase = 0.0;
		self.state = nstate;
	}
}

impl KeyRing {
	fn new() -> KeyRing {
		KeyRing{
			tumblers: [KeyTumbler::new(); KEY_LENGTH as usize],
		}
	}

	fn on_click(&mut self, click_pos: Vec2) -> bool {
		let increment = PI * 2.0 / KEY_LENGTH as f32;
		let th_start = increment/2.0 + PI/2.0;

		let dist_to_center = click_pos.length();
		let angle = click_pos.y.atan2(click_pos.x);

		if dist_to_center > 0.2 && dist_to_center < 0.7 {
			let segment = (angle - th_start) / increment + 0.5 + KEY_LENGTH as f32;
			let segment = segment as u32 % KEY_LENGTH;

			let thing = &mut self.tumblers[segment as usize];

			let nstate = (thing.state + 1)%KEY_BASE as u8;
			thing.set_state(nstate);

			true
		} else {
			false
		}
	}

	fn update(&mut self, dt: f32) {
		for tumbler in &mut self.tumblers {
			tumbler.update(dt);
		}
	}

	fn render(&self, builder: &mut UIBuilder) {
		let main_shape_segs = 18;
		
		// Main ring
		builder.build_ring(Vec2::new(0.0, 0.0), Color::grey(0.25), main_shape_segs, 0.45, 0.05);

		// Main circle -> stencil
		builder.start_stencil_replace(1);
		builder.build_poly(Vec2::new(0.0, 0.0), Color::white(), main_shape_segs, 0.5);

		let increment = PI * 2.0 / KEY_LENGTH as f32;
		let th_start = increment/2.0 + PI / 2.0;

		// Tumblers inside the main circle
		builder.start_stencilled_draw(StencilFunc::Equal, 1);
		for (i, thing) in self.tumblers.iter().enumerate() {
			let th = i as f32 * increment + th_start;
			let r = 0.5 - (1.0 - thing.pos) * 0.15;

			let offset = Vec2::from_angle(th) * r;

			builder.build_poly(offset, Color::grey(0.5), 17, 0.06);
		}
		
		// Tumblers outside the main circle
		builder.start_stencilled_draw(StencilFunc::NotEqual, 1);
		for (i, thing) in self.tumblers.iter().enumerate() {
			let th = i as f32 * increment;

			let prog = (thing.pos - 1.0).max(0.0);
			let r = 0.4 + prog * 0.2;
			let offset = Vec2::from_angle(th + th_start) * r;

			builder.build_ring(offset, Color::grey(0.35), 18, 0.04, 0.03);
		}

		builder.stop_stencil_draw();
	}

	fn calculate_key(&self) -> u32 {
		use std;
		assert!(KEY_BASE.pow(KEY_LENGTH) < std::u32::MAX);

		self.tumblers.iter().enumerate().fold(0, |acc, (i, th)| {
			assert!((th.state as u32) < KEY_BASE);
			
			acc + th.state as u32 * KEY_BASE.pow(i as u32)
		})
	}

	fn set_key(&mut self, mut key: u32) {
		let max_key = KEY_BASE.pow(KEY_LENGTH);
		assert!(key < max_key);

		for (i, mut th) in self.tumblers.iter_mut().enumerate().rev() {
			let factor = KEY_BASE.pow(i as u32);
			let place = key/factor;
			key -= place * factor;
			th.set_state(place as u8);
		}
	}
}