#![feature(link_args)]

extern crate libc;
use libc::*;
use std::time;
use std::net::TcpStream;
use std::io::{Write, Read};

mod dc;

#[repr(C)]
struct EmscriptenMouseEvent {
	ts: f64,
	x: i32, y: i32,
	// ... I don't care about the rest of these fields
}

type EmSocketCallback = extern fn(fd: i32, ud: *mut u8);
type EmMouseCallback = extern fn(etype: i32, evt: *const EmscriptenMouseEvent, ud: *mut u8) -> i32;
type EmArgCallback = extern fn(ud: *mut u8);

#[allow(dead_code, improper_ctypes)]
extern {
	fn emscripten_set_main_loop_arg(func: extern fn(arg: *mut MainContext), arg: *mut MainContext, fps: i32, simulate_infinite_loop: i32);
	fn emscripten_exit_with_live_runtime();

	fn emscripten_set_socket_open_callback(ud: *mut u8, callback: EmSocketCallback);
	fn emscripten_set_socket_close_callback(ud: *mut u8, callback: EmSocketCallback);
	fn emscripten_set_socket_message_callback(ud: *mut u8, callback: EmSocketCallback);

	fn emscripten_set_click_callback(target: *const u8, ud: *mut u8, useCapture: i32, cb: EmMouseCallback);

	fn emscripten_async_call(callback: EmArgCallback, ud: *mut u8, millis: i32);

	fn dc_set_userdata(ud: *mut MainContext);
}

fn errno() -> i32 {
	extern { fn __errno_location() -> *mut i32; }
	unsafe{ *__errno_location() }
}

pub struct MainContext {
	socket_fd: i32,
	connection: Option<TcpStream>,
	prev_frame: time::Instant,

	draw_ctx: DrawContext,
}

fn main() {
	println!("Is Hosted:      {}", cfg!(hosted));
	println!("Public address: {}", env!("PUBLIC_ADDRESS"));

	let mut ctx = Box::new(MainContext {
		socket_fd: -1, 
		connection: None,
		prev_frame: time::Instant::now(),
		draw_ctx: DrawContext::new()
	});

	start_connection(&mut (*ctx));

	unsafe {
		let ctx_ptr = Box::into_raw(ctx);
		emscripten_set_socket_open_callback(ctx_ptr as *mut u8, on_open);
		emscripten_set_socket_close_callback(ctx_ptr as *mut u8, on_close);
		emscripten_set_socket_message_callback(ctx_ptr as *mut u8, on_message);
		emscripten_set_click_callback(std::ptr::null(), ctx_ptr as *mut u8, 0, on_click);
		dc_set_userdata(ctx_ptr);

		emscripten_exit_with_live_runtime();
	}
}

fn start_connection(ctx: &mut MainContext) {
	use std::os::unix::io::FromRawFd;

	if ctx.connection.is_some() { return }

	unsafe {
		if ctx.socket_fd < 0 {
			ctx.socket_fd = socket(AF_INET, SOCK_STREAM, 0);
			if ctx.socket_fd < -1 {
				panic!("socket creation failed");
			}
		}

		let sock = ctx.socket_fd;

		fcntl(sock, F_SETFL, O_NONBLOCK);

		let mut addresses = std::ptr::null_mut();
		let hint = addrinfo {
			ai_family: AF_UNSPEC, // AF_INET
			ai_socktype: SOCK_STREAM,
			ai_protocol: 0,
			ai_flags: 0,

			ai_addrlen: 0,
			ai_addr: std::ptr::null_mut(),
			ai_canonname: std::ptr::null_mut(),
			ai_next: std::ptr::null_mut(),
		};

		let host_address = env!("PUBLIC_ADDRESS");
		let chost_address = std::ffi::CString::new(host_address).unwrap();

		let gairet = getaddrinfo(chost_address.as_bytes_with_nul().as_ptr(), "9001\0".as_ptr(), &hint, &mut addresses);
		if gairet < 0 {
			// let error = gai_strerror(gairet);
			// let error = std::str::from_utf8(&error);
			panic!("getaddrinfo failed");
		}

		// https://kripken.github.io/emscripten-site/docs/api_reference/emscripten.h.html#socket-event-registration
		if connect(sock, (*addresses).ai_addr, (*addresses).ai_addrlen) < 0 {
			match errno() {
				EINPROGRESS => {},
				EALREADY => {},
				EISCONN => {
					ctx.connection = Some(TcpStream::from_raw_fd(sock));
				},

				_ => panic!("connect failed ({})", errno())
			}
		}

		freeaddrinfo(addresses);
	}
}

extern fn on_open(fd: i32, ctx: *mut u8) {
	use std::os::unix::io::FromRawFd;

	let mut ctx: &mut MainContext = unsafe{ std::mem::transmute(ctx) };
	ctx.connection = unsafe{ Some(TcpStream::from_raw_fd(fd)) };

	ctx.draw_ctx.ring_circles.push(Circle{phase: 0.0, foreign: true});

	println!("ON OPEN");
	send_hello(&mut ctx);
}

extern fn on_retry(ctx: *mut u8) {
	start_connection(unsafe{ std::mem::transmute(ctx) });
}

extern fn on_close(_: i32, vctx: *mut u8) {
	let ctx: &mut MainContext = unsafe{ std::mem::transmute(vctx) };

	ctx.connection = None;
	ctx.socket_fd = -1;
	println!("ON CLOSE");

	ctx.draw_ctx.ring_circles.push(Circle{phase: 0.0, foreign: false});

	unsafe{ emscripten_async_call(on_retry, vctx, 1500) };
}

extern fn on_message(fd: i32, ctx: *mut u8) {
	println!("ON MESSAGE");

	let ctx: &mut MainContext = unsafe{ std::mem::transmute(ctx) };

	let mut buf = [0u8; 1024];

	if let Some(ref mut con) = ctx.connection {
		match con.read(&mut buf) {
			Ok(len) => {
				let string = std::str::from_utf8(&buf[..len as usize]);
				println!("RECV {:?}", string);

				ctx.draw_ctx.float_circles.push(Circle{phase: 0.0, foreign: true});
			},

			Err(e) => println!("recv failed {}", e)
		}
	}
}

extern fn on_click(_: i32, e: *const EmscriptenMouseEvent, ud: *mut u8) -> i32 {
	let mut ctx: &mut MainContext = unsafe{ std::mem::transmute(ud) };

	ctx.draw_ctx.float_circles.push(Circle{phase: 0.0, foreign: false});

	let msg = "click";
	if let Some(ref mut con) = ctx.connection {
		if let Err(e) = con.write_all(msg.as_bytes()) {
			println!("send failed {}", e);
		}
	}

	1
}

fn send_hello(ctx: &mut MainContext) {
	let msg = "Hello all";

	if let Some(ref mut con) = ctx.connection {
		if let Err(e) = con.write_all(msg.as_bytes()) {
			println!("send failed {}", e);
		}
	}
}

struct Circle{phase: f32, foreign: bool}

struct DrawContext {
	float_circles: Vec<Circle>,
	ring_circles: Vec<Circle>,
}

impl DrawContext {
	fn new() -> DrawContext {
		DrawContext {
			float_circles: Vec::new(),
			ring_circles: Vec::new(),
		}
	}
}

#[no_mangle]
pub unsafe fn compile_draw_commands(ctx: *mut MainContext) {
	use dc::*;
	use std::f32::consts;

	let mut ctx = &mut (*ctx);
	let t = time::Instant::now();
	let dt = (t - ctx.prev_frame).subsec_nanos() as f32 / 1000_000_000.0;
	ctx.prev_frame = t;

	let mut dctx = &mut ctx.draw_ctx;

	let (ww, wh) = get_canvas_size();

	if ctx.connection.is_some() {
		dc_stroke_color(255, 150, 150, 0.7);
	} else {
		dc_stroke_color(100, 100, 100, 0.7);
	}

	dc_draw_circle(ww/2, wh/2, 50.0);

	for c in &mut dctx.float_circles {
		c.phase += dt;

		let a = (c.phase * consts::PI).sin();
		let r = 10.0 + 30.0 * a;

		if c.foreign {
			dc_stroke_color(100, 255, 220, 0.4 * a);
		} else {
			dc_stroke_color(255, 150, 150, 0.5 * a);
		}
		dc_draw_circle(ww/2, wh/2 - (c.phase.powf(5.0) * 200.0) as i32, r);
	}

	for c in &mut dctx.ring_circles {
		c.phase += dt;

		let a = 1.0 - c.phase;
		let r = 50.0 + 10.0 * c.phase;

		if ctx.connection.is_some() {
			dc_stroke_color(255, 150, 150, 0.5 * a);
		} else {
			dc_stroke_color(100, 100, 100, 0.5 * a);
		}

		dc_draw_circle(ww/2, wh/2, r);
	}

	dctx.float_circles.retain(|x| x.phase < 1.0);

	let indicator_r = 5;
	for i in 0..3 {
		dc_fill_color(100, 255, 100, 1.0);
		dc_fill_circle(2*indicator_r + i * (indicator_r*2 + 3), wh - indicator_r*2, indicator_r as f32);
	}
}