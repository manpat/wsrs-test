use std::net::TcpStream;
use std::time;
use rendering::{RenderingContext, RenderState};

pub struct MainContext {
	// TODO: Export to separate module
	pub socket_fd: i32,
	pub connection: Option<TcpStream>,

	pub prev_frame: time::Instant,

	pub render_ctx: RenderingContext,
	pub render_state: RenderState,
} 

impl MainContext {
	pub fn new() -> Self {
		let mut render_ctx = RenderingContext::new("canvas");
		render_ctx.make_current();

		MainContext {
			socket_fd: -1,
			connection: None,
			prev_frame: time::Instant::now(),

			render_ctx,
			render_state: RenderState::new(),
		}
	}

	#[allow(dead_code, unused_variables)]
	pub fn on_connect(&mut self) {}
	
	#[allow(dead_code, unused_variables)]
	pub fn on_disconnect(&mut self) {}
	
	#[allow(dead_code, unused_variables)]
	pub fn on_update(&mut self) {}

	#[allow(dead_code, unused_variables)]
	pub fn on_render(&mut self) {
		self.render_ctx.fit_target_to_viewport();
		self.render_ctx.render(&self.render_state);
	}

	#[allow(dead_code, unused_variables)]
	pub fn on_click(&mut self, x: i32, y: i32) {
		use util;

		let mut tmp = RenderingContext::new("downloadcanvas");
		tmp.set_target_size(400, 400);
		tmp.render(&self.render_state);

		util::save_canvas("downloadcanvas");
	}
}