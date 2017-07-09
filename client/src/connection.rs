use std::net::TcpStream;
use std::ffi::CString;
use std::ptr;
use libc::*;
use ems;

use std::borrow::BorrowMut;
use std::mem::transmute;

use common::Packet;

#[derive(Copy, Clone)]
pub enum ConnectionEvent {
	Connect,
	Disconnect,
}

pub struct Connection {
	socket_fd: i32,
	stream: Option<TcpStream>,

	pub packet_queue: Vec<Packet>,
	pub event_queue: Vec<ConnectionEvent>,
}

impl Connection {
	pub fn new() -> Box<Self> {
		let mut b = Box::new(Connection {
			socket_fd: -1,
			stream: None,

			packet_queue: Vec::new(),
			event_queue: Vec::new(),
		});

		unsafe {
			let ptr: *mut u8 = transmute(b.borrow_mut() as *mut Connection);
			ems::emscripten_set_socket_open_callback(ptr as *mut u8, on_open);
			ems::emscripten_set_socket_close_callback(ptr as *mut u8, on_close);
			ems::emscripten_set_socket_message_callback(ptr as *mut u8, on_message);
		}

		b
	}

	pub fn attempt_connect(&mut self) {
		use std::os::unix::io::FromRawFd;

		fn errno() -> i32 {
			extern { fn __errno_location() -> *mut i32; }
			unsafe{ *__errno_location() }
		}

		if self.stream.is_some() { return }

		unsafe {
			if self.socket_fd >= 0 {
				close(self.socket_fd);
			}

			self.socket_fd = socket(AF_INET, SOCK_STREAM, 0);
			if self.socket_fd < 0 {
				panic!("socket creation failed");
			}

			let sock = self.socket_fd;

			fcntl(sock, F_SETFL, O_NONBLOCK);

			let mut addresses = ptr::null_mut();
			let hint = addrinfo {
				ai_family: AF_UNSPEC, // AF_INET
				ai_socktype: SOCK_STREAM,
				ai_protocol: 0,
				ai_flags: 0,

				ai_addrlen: 0,
				ai_addr: ptr::null_mut(),
				ai_canonname: ptr::null_mut(),
				ai_next: ptr::null_mut(),
			};

			let host_address = env!("PUBLIC_ADDRESS");
			let chost_address = CString::new(host_address).unwrap();

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
						self.stream = Some(TcpStream::from_raw_fd(sock));
					},

					_ => panic!("connect failed ({})", errno())
				}
			}

			freeaddrinfo(addresses);
		}
	}

	pub fn send(&mut self, p: &Packet) -> bool {
		use std::io::Write;

		if self.stream.is_none() { return false }
		if !p.is_valid_from_client() { return false }

		let mut buf = [0u8; 1<<10];

		let len = p.write(&mut buf);

		if let Err(e) = self.stream.as_mut().unwrap().write_all(&mut buf[..len]) {
			println!("send failed {}", e);
			return false
		}

		true
	}
}

impl Drop for Connection {
	fn drop(&mut self) {
		use std::ptr::{null, null_mut};

		unsafe {
			if self.socket_fd >= 0 {
				close(self.socket_fd);
			}

			ems::emscripten_set_socket_open_callback(null_mut(), transmute(null() as *const u8));
			ems::emscripten_set_socket_close_callback(null_mut(), transmute(null() as *const u8));
			ems::emscripten_set_socket_message_callback(null_mut(), transmute(null() as *const u8));
		}		
	}
}

extern fn on_open(fd: i32, ctx: *mut u8) {
	use std::os::unix::io::FromRawFd;

	let mut ctx: &mut Connection = unsafe{ transmute(ctx) };
	ctx.stream = unsafe{ Some(TcpStream::from_raw_fd(fd)) };
	ctx.event_queue.push(ConnectionEvent::Connect);
}

extern fn on_retry(ctx: *mut u8) {
	let ctx: &mut Connection = unsafe{ transmute(ctx) };
	ctx.attempt_connect();
}

extern fn on_close(_: i32, vctx: *mut u8) {
	let ctx: &mut Connection = unsafe{ transmute(vctx) };

	unsafe { close(ctx.socket_fd); }

	if ctx.stream.is_some() {
		// otherwise this is the result of a reconnect attempt
		ctx.event_queue.push(ConnectionEvent::Disconnect);
	}

	ctx.stream = None;
	ctx.socket_fd = -1;

	unsafe{ ems::emscripten_async_call(on_retry, vctx, 1500) };
}

extern fn on_message(_: i32, ctx: *mut u8) {
	use std::io::Read;

	let mut ctx: &mut Connection = unsafe{ transmute(ctx) };
	if ctx.stream.is_none() { return }

	let mut buf = [0u8; 8<<10];

	let len = match ctx.stream.as_mut().unwrap().read(&mut buf) {
		Ok(len) =>
			if len <= 0 { return }
			else { len },

		Err(e) => {
			println!("recv failed {}", e);
			return
		}
	};

	if let Some(packet) = Packet::parse(&buf[..len]) {
		ctx.packet_queue.push(packet);
	}
}