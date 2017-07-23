use rendering::*;
use ui::InputTarget;

pub struct MainScreen {}

impl MainScreen {
	pub fn new () -> Self {
		MainScreen {}
	}

	pub fn update(&mut self, dt: f32) {

	}

	pub fn render(&self, state: &mut RenderState) {

	}
}

impl InputTarget for MainScreen {
	fn on_drag_start(&mut self, pos: Vec2) {}
	fn on_drag_end(&mut self, pos: Vec2) {}
	fn on_drag(&mut self, pos: Vec2) {}

	fn on_click(&mut self, pos: Vec2) {
		println!("MainScreen click {:?}", pos);
	}
}