#![feature(link_args)]

extern crate common;
extern crate rand;
extern crate libc;
use libc::*;
use std::time;
use std::net::TcpStream;
use std::io::{Write, Read};

use common::*;

mod dc;
use dc::*;

#[repr(C)]
struct EmscriptenMouseEvent {
	ts: f64,
	_screen: [i32; 2],
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
		draw_ctx: DrawContext::new(),
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
			close(ctx.socket_fd);
		}

		ctx.socket_fd = socket(AF_INET, SOCK_STREAM, 0);
		if ctx.socket_fd < -1 {
			panic!("socket creation failed");
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

	ctx.draw_ctx.ring_circles.push(Circle {
		phase: 0.0, color: Color::rgba(1.0, 0.7, 0.7, 0.7)
	});

	println!("ON OPEN");

	let mut msg = [0u8; 16];
	let len = Packet::Debug("Hello all".to_string()).write(&mut msg);

	if let Some(ref mut con) = ctx.connection {
		if let Err(e) = con.write_all(&msg[..len]) {
			println!("send failed {}", e);
		}
	}
}

extern fn on_retry(ctx: *mut u8) {
	start_connection(unsafe{ std::mem::transmute(ctx) });
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

	ctx.draw_ctx.num_connected_users = 0;
	ctx.draw_ctx.ring_circles.push(Circle {
		phase: 0.0, color: Color::grey_a(0.4, 0.5)
	});

	unsafe{ emscripten_async_call(on_retry, vctx, 1500) };
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

extern fn on_click(_: i32, ev: *const EmscriptenMouseEvent, ud: *mut u8) -> i32 {
	let mut ctx: &mut MainContext = unsafe{ std::mem::transmute(ud) };

	let (ww, wh) = unsafe { get_canvas_size() };
	let (x, y) = unsafe {
		let ev = &*ev;
		(ev.x, ev.y)
	};

	let (ox, oy) = (x - (ww/2.0) as i32, y - (wh/2.0) as i32);
	let dist2 = {
		let ox = ox as f32;
		let oy = oy as f32;
		ox*ox + oy*oy
	};

	if dist2 < 50.0*50.0 {
		ctx.draw_ctx.float_circles.push(Circle {
			phase: 0.0, color: Color::rgba(1.0, 0.6, 0.6, 0.5)
		});
	} else {
		ctx.draw_ctx.poof(x as f32, y as f32, Color::rgba(0.7, 0.7, 0.5, 0.3));
	}

	let mut msg = [0u8; 9];
	let len = Packet::Click(0, ox as i16, oy as i16).write(&mut msg);

	if let Some(ref mut con) = ctx.connection {
		if let Err(e) = con.write_all(&msg[..len]) {
			println!("send failed {}", e);
		}
	}

	1
}

#[derive(Copy, Clone)]
struct Circle {
	color: Color,
	phase: f32,
}

struct Poof {
	color: Color,
	center: (f32, f32),

	num_circles: u32,
	phase: f32,

	phase_offset: f32,
	spin_rate: f32,
}

struct DrawContext {
	float_circles: Vec<Circle>,
	ring_circles: Vec<Circle>,
	pooves: Vec<Poof>,
	num_connected_users: i32,
}

impl DrawContext {
	fn new() -> DrawContext {
		DrawContext {
			float_circles: Vec::new(),
			ring_circles: Vec::new(),
			pooves: Vec::new(),

			num_connected_users: 0,
		}
	}

	fn poof(&mut self, x: f32, y: f32, color: Color) {
		use rand::Rng;
		use rand::distributions::{Range, IndependentSample};

		let mut rng = rand::weak_rng();

		let p = Poof {
			color,
			center: (x, y),

			num_circles: Range::new(3, 9).ind_sample(&mut rng),

			phase_offset: rng.gen(),
			spin_rate: Range::new(-1.0, 1.0).ind_sample(&mut rng) * 0.2,

			phase: 0.0,
		};

		self.pooves.push(p);
	}
}

impl Poof {
	unsafe fn update(&mut self, dt: f32) {
		use std::f32::consts;

		self.phase += dt / 3.0;

		let r = 2.0;
		let a = (self.phase * consts::PI).sin();
		let d = (1.0 - (1.0 - self.phase).powf(2.0)) * 20.0;

		let (cx, cy) = self.center;
		let cy = cy - self.phase.powf(1.3) * 20.0;

		dc_stroke_color(Color{a: self.color.a * ((self.phase * 4.0 - 0.1) * consts::PI).max(0.0).min(consts::PI).sin() * 0.1, ..self.color});
		dc_draw_circle(cx, cy, self.phase * 100.0 + 3.0);

		dc_stroke_color(Color{a: self.color.a * a, ..self.color});

		for i in 0..self.num_circles {
			let inc = i as f32 / self.num_circles as f32;
			let phase = (inc + self.phase_offset + self.phase.powf(3.0) * self.spin_rate) * consts::PI * 2.0;
			let offset = (phase.cos() * d, phase.sin() * d);

			let (x,y) = (cx + offset.0, cy + offset.1);

			dc_draw_circle(x, y, r);
		}
	}

	fn is_visible(&self) -> bool {
		self.phase < 1.0
	}
}

#[no_mangle]
pub unsafe fn compile_draw_commands(ctx: *mut MainContext) {
	use std::f32::consts;

	let mut ctx = &mut (*ctx);
	let t = time::Instant::now();
	let dt = (t - ctx.prev_frame).subsec_nanos() as f32 / 1000_000_000.0;
	ctx.prev_frame = t;

	let mut dctx = &mut ctx.draw_ctx;

	let (ww, wh) = get_canvas_size();

	if ctx.connection.is_some() {
		dc_stroke_color(Color::rgba(1.0, 0.6, 0.6, 0.7));
	} else {
		dc_stroke_color(Color::grey_a(0.4, 0.7));
	}

	dc_draw_circle(ww/2.0, wh/2.0, 50.0);

	for c in &mut dctx.ring_circles {
		c.phase += dt;

		let a = 1.0 - c.phase;
		let r = 50.0 + 10.0 * c.phase;

		dc_stroke_color(Color{a: c.color.a * a, ..c.color});
		dc_draw_circle(ww/2.0, wh/2.0, r);
	}

	for p in &mut dctx.pooves {
		p.update(dt);
	}

	for c in &mut dctx.float_circles {
		c.phase += dt;

		let a = (c.phase * consts::PI).sin();
		let r = 10.0 + 30.0 * a;

		dc_stroke_color(Color{a: c.color.a * a, ..c.color});
		dc_draw_circle(ww/2.0, wh/2.0 - c.phase.powf(5.0) * 200.0, r);
	}

	dctx.float_circles.retain(|x| x.phase < 1.0);
	dctx.ring_circles.retain(|x| x.phase < 1.0);
	dctx.pooves.retain(|x| x.is_visible());

	let max_users = 6;
	let indicator_r = 5.0;
	let indicator_d = indicator_r * 2.0;
	for i in 0..std::cmp::min(dctx.num_connected_users, max_users) {
		dc_fill_color(Color::rgb(0.4, 1.0, 0.4));
		dc_fill_circle(indicator_d + i as f32 * (indicator_d + 3.0), wh - indicator_d, indicator_r);
	}

	if dctx.num_connected_users > max_users {
		dc_fill_color(Color::rgba(0.4, 1.0, 0.4, 0.3));
		dc_fill_circle(indicator_d + max_users as f32 * (indicator_d + 3.0), wh - indicator_d, indicator_r * 4.0 / 5.0);
	}
}

fn handle_message(ctx: &mut MainContext, packet: &Packet) {
	match *packet {
		Packet::Click(_id, ox, oy) => {
			let (ww, wh) = unsafe { get_canvas_size() };
			let (x, y) = (ox as f32 + ww/2.0, oy as f32 + wh/2.0);

			let dist2 = {
				let ox = ox as f32;
				let oy = oy as f32;
				ox*ox + oy*oy
			};

			if dist2 < 50.0*50.0 {
				ctx.draw_ctx.float_circles.push(Circle {
					phase: 0.0, color: Color::rgba(0.4, 1.0, 0.9, 0.4)
				});
			} else {
				ctx.draw_ctx.poof(x, y, Color::rgba(0.4, 1.0, 0.9, 0.3));
			}
		},

		Packet::Connect(_id) => {
			ctx.draw_ctx.ring_circles.push(Circle {
				phase: 0.0, color: Color::grey_a(0.7, 0.6)
			});	
		},

		Packet::Disconnect(_id) => {
			ctx.draw_ctx.ring_circles.push(Circle {
				phase: 0.0, color: Color::rgba(0.4, 0.4, 0.7, 0.8)
			});
		},

		Packet::Update(num_connected_users) => {
			ctx.draw_ctx.num_connected_users = num_connected_users as i32;
		},

		_ => {}
	}
}