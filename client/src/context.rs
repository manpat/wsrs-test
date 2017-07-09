use std::time;
use rendering::{RenderingContext, RenderState};
use connection::Connection;

use common::*;

pub struct MainContext {
	pub connection: Box<Connection>,

	pub prev_frame: time::Instant,

	pub render_ctx: RenderingContext,
	pub render_state: RenderState,
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
			render_state: RenderState::new(),
		}
	}

	pub fn on_connect(&mut self) {
		println!("Connected...");

		self.connection.send(&Packet::RequestNewSession);
	}
	
	pub fn on_disconnect(&mut self) {
		println!("Connection lost");
	}
	
	#[allow(dead_code, unused_variables)]
	pub fn on_update(&mut self) {}

	pub fn on_render(&mut self) {
		self.render_ctx.fit_target_to_viewport();
		self.render_ctx.render(&self.render_state);
	}

	#[allow(dead_code, unused_variables)]
	pub fn on_click(&mut self, x: i32, y: i32) {
		self.connection.send(&Packet::Debug("Hello".to_string()));
		// use util;

		// let mut tmp = RenderingContext::new("downloadcanvas");
		// tmp.set_target_size(400, 400);
		// tmp.render(&self.render_state);

		// util::save_canvas("downloadcanvas");
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
					println!("New Session: {}", token);
				},

				_ => {}
			}
		}

		self.connection.event_queue.clear();
		self.connection.packet_queue.clear();
	}
}