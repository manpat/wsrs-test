use rendering::*;
use rendering::uibuilder::*;
use ui::InputTarget;

use std;

use common::world::Species;

#[derive(Copy, Clone, Debug)]
pub enum Action {
	Translate(Vec2),
	ClickWorld(Vec2),
	SetSpecies(Species),
}

pub struct MainScreen {
	pub viewport: Viewport,

	selector_bar: SelectorBar,

	actions: Vec<Action>,
	drag_pos: Vec2,
}

impl MainScreen {
	pub fn new () -> Self {
		MainScreen {
			actions: Vec::new(),
			viewport: Viewport::new(),

			selector_bar: SelectorBar::new(),

			drag_pos: Vec2::zero(),
		}
	}

	pub fn update(&mut self, dt: f32) {
		self.selector_bar.update(dt);
	}

	pub fn render(&mut self, mut builder: &mut UIBuilder) {
		self.selector_bar.render(&mut builder);
	}

	pub fn poll_actions(&mut self) -> Option<Action> {
		self.actions.pop()
	}
}

impl InputTarget for MainScreen {
	fn on_drag_start(&mut self, pos: Vec2) {
		self.drag_pos = pos;
	}

	fn on_drag_end(&mut self, pos: Vec2) {}
	fn on_drag(&mut self, pos: Vec2) {
		self.actions.push(Action::Translate(pos - self.drag_pos));
		self.drag_pos = pos;
	}

	fn on_click(&mut self, pos: Vec2) {
		if let Some(species) = self.selector_bar.click(pos) {
			self.actions.push(Action::SetSpecies(species));
			return
		}
		self.actions.push(Action::ClickWorld(pos));
	}
}

struct SelectorBar {
	phase: f32,

	selector_positions: [Vec2; 3],
	selector_phase: [f32; 3],
	selector_size: f32,
}

impl SelectorBar {
	fn new() -> Self {
		SelectorBar {
			phase: -1.0,

			selector_positions: [Vec2::zero(); 3],
			selector_phase: [std::f32::INFINITY; 3],
			selector_size: 0.0,
		}
	}

	fn click(&mut self, pos: Vec2) -> Option<Species> {
		for (idx, &selector) in self.selector_positions.iter().enumerate() {
			if (selector - pos).length() < self.selector_size {
				self.selector_phase[idx] = 0.0;
				return Species::from_byte(idx as u8)
			}
		}

		None
	}

	fn update(&mut self, dt: f32) {
		self.phase += dt;
		self.phase = self.phase.min(1.0);

		for phase in self.selector_phase.iter_mut() {
			*phase += dt;
		}
	}

	fn render(&mut self, builder: &mut UIBuilder) {
		let vp = builder.viewport;
		let aspect = vp.get_aspect();
		let aspect = if vp.size.x > vp.size.y { aspect } else { 1.0 / aspect };
		let separation = Vec2::new(0.4 * aspect, 0.0);
		let selector_size = 0.03 * aspect;

		let target_pos = Vec2::new(0.0, selector_size*1.1 - 1.0);
		let center = self.phase.ease_back_out(Vec2::new(0.0,-1.0 - selector_size), target_pos);

		self.selector_positions = [
			center - separation,
			center,
			center + separation,
		];

		self.selector_size = selector_size;

		let colors = [
			Color::rgb(0.197, 0.800, 0.202).pow(1.0/2.2),
			Color::rgb(0.400, 0.600, 1.000).pow(1.0/2.2),
			Color::rgb(1.000, 0.500, 0.500).pow(1.0/2.2),
		];

		let bg_color = Color::grey_a(0.3, 0.3);

		for ((&pos, &color), &phase) in self.selector_positions.iter().zip(colors.iter()).zip(self.selector_phase.iter()) {
			let click_size = phase.ease_linear(selector_size, selector_size*2.0);
			let click_col = Color{a: phase.ease_linear(1.0, 0.0), .. color};

			let main_col = phase.ease_linear(Color::white(), color);

			// TODO: Find a better way
			builder.build_poly(pos, bg_color, 4, selector_size * 1.5);
			builder.build_poly(pos, click_col, 4, click_size);
			builder.build_poly(pos, main_col, 4, selector_size);
		}
	}
}