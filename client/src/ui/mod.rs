pub mod auth_screen;
pub mod main_screen;

use common::math::*;

pub use ui::auth_screen::AuthScreen;
pub use ui::main_screen::MainScreen;

pub trait InputTarget {
	fn on_drag_start(&mut self, pos: Vec2);
	fn on_drag_end(&mut self, pos: Vec2);
	fn on_drag(&mut self, pos: Vec2);

	fn on_click(&mut self, pos: Vec2);
}