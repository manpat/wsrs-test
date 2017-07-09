
use std::net::TcpStream;
use std::time;

pub struct MainContext {
	// TODO: Export to separate module
	pub socket_fd: i32,
	pub connection: Option<TcpStream>,

	pub prev_frame: time::Instant,
} 

impl MainContext {
	pub fn new() -> Self {
		MainContext {
			socket_fd: -1,
			connection: None,
			prev_frame: time::Instant::now(),
		}
	}

	#[allow(dead_code, unused_variables)]
	pub fn on_connect(&mut self) {}
	
	#[allow(dead_code, unused_variables)]
	pub fn on_disconnect(&mut self) {}
	
	#[allow(dead_code, unused_variables)]
	pub fn on_update(&mut self) {}

	#[allow(dead_code, unused_variables)]
	pub fn on_click(&mut self, x: i32, y: i32) {}
}