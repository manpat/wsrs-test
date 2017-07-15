use rendering::{RenderingContext, RenderState, StencilFunc};
use rendering::types::*;
use easing::*;

use std::f32::consts::PI;

const KEY_LENGTH: u32 = 9;
const KEY_BASE: u32 = 3;

struct ClickyThing {
	state: u8,

	pos: f32,
	anim_phase: f32,
	prev_pos: f32,
}

impl ClickyThing {
	fn new() -> ClickyThing {
		ClickyThing {
			state: 1,

			pos: 0.0,
			anim_phase: 0.0,
			prev_pos: 0.0,
		}
	}

	fn update(&mut self, dt: f32) {
		let target_pos = self.state as f32;
		self.pos = self.anim_phase.ease_back_out(self.prev_pos, target_pos, 0.4);

		self.anim_phase += dt;
	}

	fn set_state(&mut self, nstate: u8) {
		self.prev_pos = self.pos;
		self.anim_phase = 0.0;
		self.state = nstate;
	}
}

pub struct AuthScreen {
	clicky_things: Vec<ClickyThing>,
	phase: f32,

	pub viewport: Viewport,
	download_button_pos: Vec2,
}

impl AuthScreen {
	pub fn new() -> Self {
		let mut clicky_things = Vec::with_capacity(KEY_LENGTH as usize);
		for i in 0..KEY_LENGTH {
			clicky_things.push(ClickyThing::new());
		}

		AuthScreen {
			clicky_things,
			phase: 0.0,

			viewport: Viewport::new(),
			download_button_pos: Vec2::zero(),
		}
	}

	pub fn on_click(&mut self, x: f32, y: f32) {
		let click_pos = Vec2::new(x, y);

		let increment = PI * 2.0 / KEY_LENGTH as f32;
		let th_start = increment/2.0 + PI / 2.0;
		let r = 0.5;
		let click_zone = 0.13;

		let mut has_changed = false;

		for (i, mut thing) in self.clicky_things.iter_mut().enumerate() {
			let th = i as f32 * increment + th_start;

			let offset = Vec2::new(r * th.cos(), r * th.sin());
			if (click_pos - offset).length() < click_zone {
				let nstate = (thing.state + 1)%KEY_BASE as u8;
				thing.set_state(nstate);

				has_changed = true;
			}
		}

		if (self.download_button_pos - click_pos).length() < 0.1 {
			self.download_key();
		}

		if has_changed {
			println!("New key: {}", self.calculate_key());
		}
	}

	fn calculate_key(&self) -> u32 {
		// TODO ???
		// assert!(KEY_LENGTH.pow(KEY_BASE) < u32::MAX);

		self.clicky_things.iter().enumerate().fold(0, |acc, (i, th)| {
			assert!((th.state as u32) < KEY_BASE);
			
			acc + th.state as u32 * KEY_BASE.pow(i as u32)
		})
	}

	pub fn update(&mut self, dt: f32) {
		self.phase += dt;

		for thing in &mut self.clicky_things {
			thing.update(dt);
		}

		self.download_button_pos = self.viewport.get_bottom_left() + Vec2::new(0.14, 0.14);
	}

	fn render_key(&self, state: &mut RenderState) {
		let main_shape_segs = 30;

		state.start_stencil_write(1, 0x1);
		state.build_poly(Vec2::new(0.0, 0.0), Color::white(), main_shape_segs, 0.45);

		state.start_stencilled_draw(StencilFunc::NotEqual, 1, 0x1);
		state.build_poly(Vec2::new(0.0, 0.0), Color::grey(0.25), main_shape_segs, 0.5);

		state.start_stencil_write(2, 0x2);
		state.build_poly(Vec2::new(0.0, 0.0), Color::white(), main_shape_segs, 0.14);

		state.start_stencilled_draw(StencilFunc::NotEqual, 2, 0x2);
		state.build_poly(Vec2::new(0.0, 0.0), Color::rgb(0.4, 0.9, 0.6), main_shape_segs, 0.2);

		state.start_stencil_write(1, 0xff);
		state.build_poly(Vec2::new(0.0, 0.0), Color::white(), main_shape_segs, 0.5);

		let increment = PI * 2.0 / KEY_LENGTH as f32;
		let th_start = increment/2.0 + PI / 2.0;

		state.start_stencilled_draw(StencilFunc::Equal, 1, 0x1);
		for (i, thing) in self.clicky_things.iter().enumerate() {
			let th = i as f32 * increment + th_start;
			let r = 0.5 - (1.0 - thing.pos) * 0.15;

			let offset = Vec2::new(r * th.cos(), r * th.sin());

			state.build_poly(offset, Color::grey(0.5), 17, 0.06);
		}
		
		state.start_stencilled_draw(StencilFunc::NotEqual, 1, 0x1);
		for (i, thing) in self.clicky_things.iter().enumerate() {
			let th = i as f32 * increment + th_start;
			let r = 0.5 + (thing.pos - 2.3) * 0.1;

			let offset = Vec2::new(r * th.cos(), r * th.sin());

			state.build_poly(offset, Color::grey(0.35), 19, 0.13);
		}

		state.stop_stencil_draw();		
	}

	pub fn render(&self, mut state: &mut RenderState) {
		self.render_key(&mut state);
		state.build_poly(self.download_button_pos, Color::grey(0.3), 15, 0.1);
	}

	pub fn download_key(&self) {
		use util;

		let mut tmp = RenderingContext::new("downloadcanvas");
		let mut tmpstate = RenderState::new();
		tmp.set_target_size(400, 400);
		tmpstate.set_viewport(&tmp.get_viewport());
		self.render_key(&mut tmpstate);
		tmp.render(&tmpstate);

		util::save_canvas("downloadcanvas");
	}
}