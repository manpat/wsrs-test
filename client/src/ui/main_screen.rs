use rendering::*;
use rendering::uibuilder::*;
use ui::InputTarget;

#[derive(Copy, Clone, Debug)]
pub enum Action {
	Translate(Vec2),
	ClickWorld(Vec2),
}

pub struct MainScreen {
	actions: Vec<Action>,

	drag_pos: Vec2,
}

impl MainScreen {
	pub fn new () -> Self {
		MainScreen {
			actions: Vec::new(),

			drag_pos: Vec2::zero(),
		}
	}

	pub fn update(&mut self, dt: f32) {

	}

	pub fn render(&self, builder: &mut UIBuilder) {

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
		self.actions.push(Action::ClickWorld(pos));
	}
}