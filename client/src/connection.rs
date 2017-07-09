use context::MainContext;
use std::net::TcpStream;
use std::ffi::CString;
use std::ptr;
use libc::*;

fn errno() -> i32 {
	extern { fn __errno_location() -> *mut i32; }
	unsafe{ *__errno_location() }
}


pub fn start_connection(ctx: &mut MainContext) {
	use std::os::unix::io::FromRawFd;

	if ctx.connection.is_some() { return }

	unsafe {
		if ctx.socket_fd >= 0 {
			close(ctx.socket_fd);
		}

		ctx.socket_fd = socket(AF_INET, SOCK_STREAM, 0);
		if ctx.socket_fd < 0 {
			panic!("socket creation failed");
		}

		let sock = ctx.socket_fd;

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
					ctx.connection = Some(TcpStream::from_raw_fd(sock));
				},

				_ => panic!("connect failed ({})", errno())
			}
		}

		freeaddrinfo(addresses);
	}
}