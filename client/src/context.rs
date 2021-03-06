use std::time;
use rendering::RenderingContext;
use rendering::uibuilder::UIBuilder;
use rendering::worldview::WorldView;
use connection::Connection;

use common::*;
use common::world::Species;
use ui::{self, InputTarget};

const DRAG_THRESHOLD: f32 = 10.0;

#[derive(Copy, Clone)]
enum ScreenState {
	AuthScreen,
	MainScreen,
}

pub struct MainContext {
	connection: Box<Connection>,
	auth_token: Option<u32>,

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

	selected_species: Species,

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
			auth_token: None,
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

			selected_species: Species::A,

			touch_id: None,
			touch_enabled: false,
		}
	}

	pub fn on_connect(&mut self) {
		println!("Connected...");
		self.auth_screen.on_connect();

		if let Some(token) = self.auth_token {
			self.connection.send(&Packet::AttemptAuthSession(token));
		} else {
			// TODO: Not this
			self.connection.send(&Packet::AttemptAuthSession(123));
		}
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

				use ui::auth_screen::Action;

				match self.auth_screen.poll_actions() {
					Some(Action::TryAuth(key)) => {
						println!("Really requesing auth {}", key);
						self.connection.send(&Packet::AttemptAuthSession(key));
					}

					Some(Action::RequestNewSession) => {
						println!("Requesting new session");
						self.connection.send(&Packet::RequestNewSession);
					}

					Some(Action::EnterGame) => {
						println!("Pls enter game");
						self.screen_state = ScreenState::MainScreen;
					}

					_ => {}
				}
			}

			ScreenState::MainScreen => {
				self.main_screen.update(dt);
				self.world_view.update(dt);

				use ui::main_screen::Action;

				while let Some(act) = self.main_screen.poll_actions() {
					match act {
						Action::Translate(v) => {
							self.world_view.try_world_translate(v);
						}

						Action::ClickWorld(p) => {
							let pos = self.world_view.convert_to_world_coords(p);
							self.connection.send(&Packet::RequestPlaceTree(pos.x, pos.z,
								self.selected_species));
						}

						Action::SetSpecies(s) => {
							self.selected_species = s;
						}
					}
				}
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
				self.main_screen.viewport = vp;
				self.main_screen.render(&mut self.ui_builder);
			}
		}

		self.ui_builder.flush_geom();

		self.render_ctx.prepare_render();

		if match_enum!(self.screen_state, ScreenState::MainScreen) {
			self.world_view.render(&vp);
		}
		
		self.ui_builder.render();
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
					
					// Hide screen
					self.auth_screen.on_auth_success();
					self.auth_token = Some(token);

					self.connection.send(&Packet::RequestDownloadWorld);
				}

				Packet::AuthFail => {
					println!("Auth fail");
					self.auth_screen.on_auth_fail();
				}

				Packet::NewSession(token) => {
					println!("New session: {}", token);
					self.auth_screen.set_key(token);
				}

				Packet::TreePlaced(id, pos_x, pos_y, species) => {
					self.world_view.place_tree(id, Vec3::new(pos_x, 0.0, pos_y), species);
				}

				Packet::TreeDied(id) => {
					self.world_view.kill_tree(id);
				}

				Packet::HealthUpdate(health_state) => {
					// println!("HealthUpdate {:?}", health_state);
					self.world_view.update_health_state(health_state);
				}

				Packet::TreeUpdate(tree_maturities) => {
					self.world_view.update_tree_maturities(tree_maturities);
				}

				_ => {}
			}
		}

		self.connection.event_queue.clear();
		self.connection.packet_queue.clear();
	}
}