#![feature(link_args)]

extern crate common;
extern crate rand;
extern crate libc;
use libc::*;
use std::net::TcpStream;
use std::io::Read;

use common::*;

#[macro_use]
mod ems;
mod util;
mod context;
mod rendering;
mod connection;

use context::*;

fn main() {
	println!("Is Hosted:      {}", cfg!(hosted));
	println!("Public address: {}", env!("PUBLIC_ADDRESS"));

	let mut ctx = Box::new(MainContext::new());

	connection::start_connection(&mut (*ctx));

	unsafe {
		let ctx_ptr = Box::into_raw(ctx);
		ems::emscripten_set_socket_open_callback(ctx_ptr as *mut u8, on_open);
		ems::emscripten_set_socket_close_callback(ctx_ptr as *mut u8, on_close);
		ems::emscripten_set_socket_message_callback(ctx_ptr as *mut u8, on_message);
		ems::start(ctx_ptr);
	}
}

extern fn on_open(fd: i32, ctx: *mut u8) {
	use std::os::unix::io::FromRawFd;

	let mut ctx: &mut MainContext = unsafe{ std::mem::transmute(ctx) };
	ctx.connection = unsafe{ Some(TcpStream::from_raw_fd(fd)) };
}

extern fn on_retry(ctx: *mut u8) {
	connection::start_connection(unsafe{ std::mem::transmute(ctx) });
}

extern fn on_close(_: i32, vctx: *mut u8) {
	let ctx: &mut MainContext = unsafe{ std::mem::transmute(vctx) };

	unsafe { close(ctx.socket_fd); }

	if ctx.connection.is_some() {
		// otherwise this is the result of a reconnect attempt
		println!("ON CLOSE");
	}

	ctx.connection = None;
	ctx.socket_fd = -1;

	unsafe{ ems::emscripten_async_call(on_retry, vctx, 1500) };
}

extern fn on_message(_: i32, ctx: *mut u8) {
	println!("ON MESSAGE");

	let mut ctx: &mut MainContext = unsafe{ std::mem::transmute(ctx) };
	if ctx.connection.is_none() { return }

	let mut buf = [0u8; 1024];

	let len = match ctx.connection.as_mut().unwrap().read(&mut buf) {
		Ok(len) =>
			if len <= 0 { return }
			else { len },

		Err(e) => {
			println!("recv failed {}", e);
			return
		}
	};

	if let Some(packet) = Packet::parse(&buf[..len]) {
		handle_message(&mut ctx, &packet);
	}
}

fn handle_message(_ctx: &mut MainContext, packet: &Packet) {
	match *packet {
		Packet::Connect(_id) => {},
		Packet::Disconnect(_id) => {},

		_ => {}
	}
}