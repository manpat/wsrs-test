use std::time;
use rendering::{RenderingContext, Shader};
use rendering::uibuilder::UIBuilder;
use rendering::worldview::WorldView;
use connection::Connection;

use common::*;
use ui::{self, InputTarget};

const DRAG_THRESHOLD: f32 = 5.0;

#[derive(Copy, Clone)]
enum ScreenState {
	AuthScreen,
	MainScreen,
}

pub struct MainContext {
	connection: Box<Connection>,

	prev_frame: time::Instant,

	render_ctx: RenderingContext,
	ui_builder: UIBuilder,
	world_view: WorldView,

	screen_state: ScreenState,
	auth_screen: ui::AuthScreen,
	main_screen: ui::MainScreen,

	click_start_pos: Vec2i,
	is_dragging: bool,
	is_mouse_down: bool,

	// hack hack hack
	pub touch_id: Option<i32>,
	pub touch_enabled: bool,
} 

impl MainContext {
	pub fn new() -> Self {
		let mut render_ctx = RenderingContext::new("canvas");
		render_ctx.make_current();

		let mut connection = Connection::new();
		connection.attempt_connect();

		MainContext {
			connection,
			prev_frame: time::Instant::now(),

			render_ctx,
			ui_builder: UIBuilder::new(),
			world_view: WorldView::new(),

			screen_state: ScreenState::AuthScreen,
			auth_screen: ui::AuthScreen::new(),
			main_screen: ui::MainScreen::new(),

			click_start_pos: Vec2i::zero(),
			is_dragging: false,
			is_mouse_down: false,

			touch_id: None,
			touch_enabled: false,
		}
	}

	pub fn on_connect(&mut self) {
		println!("Connected...");
		self.auth_screen.on_connect();
	}
	
	pub fn on_disconnect(&mut self) {
		println!("Connection lost");
		self.auth_screen.on_disconnect();
	}
	
	pub fn on_update(&mut self) {
		let now = time::Instant::now();
		let diff = now - self.prev_frame;
		self.prev_frame = now;

		let udt = diff.subsec_nanos() / 1000;
		let dt = udt as f32 / 1000_000.0;

		match self.screen_state {
			ScreenState::AuthScreen => {
				self.auth_screen.update(dt);

				use ui::AuthScreenAction as ASA;

				match self.auth_screen.poll_actions() {
					Some(ASA::TryAuth(key)) => {
						println!("Really requesing auth {}", key);
						self.connection.send(&Packet::AttemptAuthSession(key));
					}

					Some(ASA::RequestNewSession) => {
						println!("Requesting new session");
						self.connection.send(&Packet::RequestNewSession);
					}

					Some(ASA::EnterGame) => {
						println!("Pls enter game");
						self.screen_state = ScreenState::MainScreen;
					}

					_ => {}
				}
			}

			ScreenState::MainScreen => {
				self.main_screen.update(dt);
			}
		}
	}

	pub fn on_render(&mut self) {
		self.render_ctx.fit_target_to_viewport();
		let vp = self.render_ctx.get_viewport();

		self.ui_builder.set_viewport(&vp);
		self.ui_builder.clear();

		match self.screen_state {
			ScreenState::AuthScreen => {
				self.auth_screen.viewport = vp;
				self.auth_screen.render(&mut self.ui_builder);
			}

			ScreenState::MainScreen => {
				self.main_screen.render(&mut self.ui_builder);
			}
		}

		self.ui_builder.flush_geom();

		self.render_ctx.prepare_render();
		self.ui_builder.render();

		if match_enum!(self.screen_state, ScreenState::MainScreen) {
			self.world_view.render(&vp);
		}
	}

	fn get_input_target<'a>(&'a mut self) -> &'a mut InputTarget {
		match self.screen_state {
			ScreenState::AuthScreen => &mut self.auth_screen,
			ScreenState::MainScreen => &mut self.main_screen,
		}
	}

	pub fn on_mouse_down(&mut self, x: i32, y: i32, button: u16) {
		// Only allow left click
		if button != 0 { return }

		self.click_start_pos = Vec2i::new(x, y);
		self.is_mouse_down = true;
	}

	pub fn on_mouse_up(&mut self, x: i32, y: i32, button: u16) {
		// Only allow left click
		if button != 0 { return }

		let pos = Vec2i::new(x, y);
		let spos = self.render_ctx.get_viewport()
			.client_to_gl_coords(pos);

		if !self.is_dragging {
			self.get_input_target().on_click(spos);
		} else {
			self.get_input_target().on_drag_end(spos);
		}

		self.is_dragging = false;
		self.is_mouse_down = false;
	}

	pub fn on_mouse_move(&mut self, x: i32, y: i32) {
		let pos = Vec2i::new(x, y);
		let spos = self.render_ctx.get_viewport()
			.client_to_gl_coords(pos);

		if self.is_mouse_down && (pos - self.click_start_pos).length() > DRAG_THRESHOLD {
			if !self.is_dragging {
				self.is_dragging = true;

				self.get_input_target().on_drag_start(spos);
				// Cancel any clicks
			} else {
				self.get_input_target().on_drag(spos);
			}

		} else {
			// Send regular ol' mouse move
		}
	}

	pub fn process_packets(&mut self) {
		for e in self.connection.event_queue.clone() {
			use connection::ConnectionEvent as CE;

			match e {
				CE::Connect => self.on_connect(),
				CE::Disconnect => self.on_disconnect(),
			}
		}

		for packet in self.connection.packet_queue.clone() {
			match packet {
				Packet::AuthSuccessful(token) => {
					println!("Auth success: {}", token);
					self.auth_screen.on_auth_success();
					// Hide screen
				},

				Packet::AuthFail => {
					println!("Auth fail");
					self.auth_screen.on_auth_fail();
				},

				Packet::NewSession(token) => {
					println!("New session: {}", token);
					self.auth_screen.set_key(token);
				},

				_ => {}
			}
		}

		self.connection.event_queue.clear();
		self.connection.packet_queue.clear();
	}
}