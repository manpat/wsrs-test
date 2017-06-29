extern crate std;

use std::net::TcpStream;
use std::io;
use std::option::Option;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Request<'a> {
	uri: &'a str,
	fields: HashMap<&'a str, &'a str>,
}

#[derive(Debug)]
pub struct Response<'a> {
	status_line: &'a str,
	fields: HashMap<&'a str, &'a str>,
	body: Option<&'a [u8]>,
}

impl<'a> Request<'a> {
	pub fn parse(data: &'a str) -> Result<Request<'a>, &str> {
		let header_end = data.split("\r\n\r\n").next().unwrap();
		let mut lines = header_end.split_terminator("\r\n");
		let reqline = lines.next().unwrap_or("");

		let mut reqlineels = reqline.split_whitespace();

		if reqlineels.next().unwrap_or("") != "GET" {
			if cfg!(debug_requests) {
				println!("{}", data);
			}
			return Err("Non-GET requests not supported");
		}

		let requri = reqlineels.next().unwrap_or("");
		let version = reqlineels.next().unwrap_or("");

		if version != "HTTP/1.0" && version != "HTTP/1.1" {
			if cfg!(debug_requests) {
				println!("{}", data);
			}
			return Err("Invalid HTTP version");
		}

		let mut fields = HashMap::new();

		for line in lines {
			let mut line = line.splitn(2, ":").map(|s| s.trim());
			let key = line.next().unwrap();
			let value = match line.next() {
				Some(v) => v,
				None => continue
			};

			fields.insert(key, value);
		}

		Ok(Request {
			uri: requri,
			fields: fields,
		})
	}

	pub fn uri(&self) -> &str {
		self.uri
	}

	pub fn get(&self, key: &str) -> Option<&str> {
		if let Some(&val) = self.fields.get(&key) {
			Some(val)
		} else {
			None
		}
	}
}

impl<'a> Response<'a> {
	pub fn new(status: &'a str) -> Response<'a> {
		Response {
			status_line: status,
			fields: HashMap::new(),
			body: None
		}
	}

	pub fn set(&mut self, key: &'a str, value: &'a str) {
		let _ = self.fields.insert(key, value);
	}

	pub fn set_body(&mut self, body: &'a [u8]) {
		self.body = Some(body); // once told me
	}

	pub fn write_to_stream(&self, stream: &mut TcpStream) -> io::Result<()> {
		use std::io::Write;

		let it = std::iter::once(self.status_line.to_string());
		let fieldit = self.fields.iter().map(|(k, v)| format!("{}: {}", k, v));
		let mut response_str = it.chain(fieldit)
			.fold(String::new(), |mut acc, s| {
				acc.push_str(s.as_str());
				acc.push_str("\r\n");
				acc
			});

		response_str.push_str("\r\n");

		stream.write_all(response_str.as_bytes())?;

		if let Some(ref body) = self.body {
			stream.write_all(&body)?;
		}

		Ok(())
	}
}

// GET / HTTP/1.1
// Host: 0.0.0.0:9001
// Connection: Upgrade
// Pragma: no-cache
// Cache-Control: no-cache
// Upgrade: websocket
// Origin: http://localhost:8000
// Sec-WebSocket-Version: 13
// User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.19 Safari/537.36
// Accept-Encoding: gzip, deflate, sdch
// Accept-Language: en-GB,en-US;q=0.8,en;q=0.6
// Sec-WebSocket-Key: va9b+qvhlhiwJCTfI84PVw==
// Sec-WebSocket-Extensions: permessage-deflate; client_max_window_bits
// Sec-WebSocket-Protocol: binary

// HTTP/1.1 101 Switching Protocols
// Upgrade: websocket
// Connection: Upgrade
// Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
