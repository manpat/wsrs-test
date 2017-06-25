mod http;

extern crate sha1;
extern crate base64;

use std::net::{TcpStream, TcpListener};
use std::io::{Write, Read};
use std::thread;
use std::time;
use std::sync::mpsc;

enum ServerMessage {
	Connect(TcpStream),
	Kill,
}

fn main() {
	println!("Is Hosted:      {}", cfg!(hosted));
	println!("Public address: {}", env!("PUBLIC_ADDRESS"));
	
	let listener = TcpListener::bind("0.0.0.0:9001").unwrap();

	let (tx, rx) = mpsc::channel::<ServerMessage>();
	let thd = thread::spawn(move || server_loop(rx));

	for stream in listener.incoming() {
		match stream {
			Ok(mut stream) => {
				let mut buf = [0u8; 1024];
				let size = stream.read(&mut buf).unwrap();

				let data = std::str::from_utf8(&buf[0..size]);
				if let Ok(data) = data {
					if let Some(header) = http::Request::parse(data) {
						if header.get("Upgrade") == Some("websocket") {
							init_websocket_connection(&mut stream, &header);
							tx.send(ServerMessage::Connect(stream)).unwrap();
						}
					}
				}
			},

			Err(e) => { println!("Connection failed: {}", e); }
		}
	}

	thd.join().unwrap();
}

struct Connection {
	stream: TcpStream,
	delete_flag: bool,

	id: u64,
}

impl Connection {
	fn new(s: TcpStream, id: u64) -> Connection {
		Connection {
			stream: s,
			delete_flag: false,
			id
		}
	}
}

fn server_loop(rx: mpsc::Receiver<ServerMessage>) {
	let mut connections = Vec::new();
	let mut packet_buffer = [0u8; 8092];
	let mut id_counter = 0u64;

	let mut click_queue = Vec::new();

	'main: loop {
		while let Some(msg) = rx.try_recv().ok() {
			match msg {
				ServerMessage::Connect(stream) => {
					stream.set_nonblocking(true).expect("Set nonblock failed");
					id_counter += 1;
					connections.push(Connection::new(stream, id_counter));
					println!("Connection ({})", id_counter);
				},

				ServerMessage::Kill => break 'main,
			}
		}

		for c in &mut connections {
			let res = c.stream.read(&mut packet_buffer);
			let length = match res {
				Ok(length) => length,
				Err(_) => continue,
			};

			if length == 0 {
				println!("Zero length packet ({})", c.id);
				c.delete_flag = true;
				continue;				
			}

			let payload = decode_ws_packet(&mut packet_buffer[..length]);
			if payload.len() == 0 {
				println!("Disconnection ({})", c.id);
				c.delete_flag = true;
				continue;
			}

			let string = std::str::from_utf8(&payload);
			if !string.is_ok() {
				c.delete_flag = true;
				println!("Invalid payload ({})", c.id);
				continue;
			}

			let string = string.unwrap();
			println!("MSG({}): {}", c.id, string);

			if string == "click" {
				click_queue.push(c.id);
			}
		}

		connections.retain(|x| !x.delete_flag);

		for cl in &click_queue {
			let payload: [u8; 8] = unsafe{ std::mem::transmute(*cl) };
			let packet = encode_ws_packet(&mut packet_buffer, &payload);

			for c in &mut connections {
				if c.id != *cl {
					let _ = c.stream.write_all(&packet);
				}
			}
		}

		click_queue.clear();

		thread::sleep(time::Duration::from_millis(100));
	}
}

fn init_websocket_connection(mut stream: &mut TcpStream, header: &http::Request) {
	if !header.get("Sec-WebSocket-Protocol").unwrap_or("").contains("binary") {
		http::Response::new("HTTP/1.1 400 Bad Request")
			.write_to_stream(&mut stream);

		return;
	}

	let magic = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
	let key = header.get("Sec-WebSocket-Key").unwrap();
	let accept_magic = format!("{}{}", key, magic);

	let mut m = sha1::Sha1::new();
	m.update(accept_magic.as_bytes());
	let accept_key = base64::encode(&m.digest().bytes());

	let mut res = http::Response::new("HTTP/1.1 101 Switching Protocols");
	res.set("Upgrade", "websocket");
	res.set("Connection", "Upgrade");
	res.set("Sec-WebSocket-Version", "13");
	res.set("Sec-WebSocket-Protocol", "binary");
	res.set("Sec-WebSocket-Accept", accept_key.as_str());

	res.set("Cache-Control", "no-cache");
	res.set("Pragma", "no-cache");

	res.write_to_stream(&mut stream);

	let mut buf = [0u8; 1024];

	{
		let packet = encode_ws_packet(&mut buf, b"hello from serverland");
		let _ = stream.write(&packet);
	}
}

 //  0                   1                   2                   3
 //  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
 // +-+-+-+-+-------+-+-------------+-------------------------------+
 // |F|R|R|R| opcode|M| Payload len |    Extended payload length    |
 // |I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
 // |N|V|V|V|       |S|             |   (if payload len==126/127)   |
 // | |1|2|3|       |K|             |                               |
 // +-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
 // |     Extended payload length continued, if payload len == 127  |
 // + - - - - - - - - - - - - - - - +-------------------------------+
 // |                               |Masking-key, if MASK set to 1  |
 // +-------------------------------+-------------------------------+
 // | Masking-key (continued)       |          Payload Data         |
 // +-------------------------------- - - - - - - - - - - - - - - - +
 // https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers#Format

fn extract_bits(v: u16, bit: u8, width: u8) -> u16 {
	let shift = 16 - bit - width;
	let mask = (1u16<<width) - 1;
	(v >> shift) & mask
}

fn test_bit(v: u16, bit: u8) -> bool {
	let bit = 15 - bit;
	(v & 1<<bit) != 0
}

fn decode_ws_packet<'a>(buf: &'a mut [u8]) -> &'a [u8] {
	let header = (buf[0] as u16) << 8 | buf[1] as u16;

	let final_packet = test_bit(header, 0);
	let opcode = extract_bits(header, 4, 4);
	let masked = test_bit(header, 8); // Client packets should always be masked
	let len = extract_bits(header, 9, 7) as usize;

	// For now, must be binary (TODO: handle continuation)
	assert!(final_packet);

	match opcode {
		0x0 => panic!("Continuation frames not implemented"),
		0x1 => panic!("Text frames not implemented"), // Emscripten doesn't do text frames so this is fine
		0x2 => {},
		0x3...0x7 => panic!("Reserved opcode {}", opcode),
		0x8 => {
			return &buf[0..0];
		},
		0x9 => panic!("Ping frame not handled"),
		0xA => panic!("Pong frame not handled"),
		0xB...0xF => panic!("Reserved control frame {}", opcode),
		_ => unreachable!()
	}

	assert!(buf.len() > 2);

	let extlen = match len {
		127 => unimplemented!(),
		126 => (buf[2] as usize) << 8 | buf[3] as usize,
		_ => len
	};

	let mut payload = match len {
		127 => &mut buf[10..],
		126 => &mut buf[4..],
		_ => &mut buf[2..],
	};

	let expected_len = if masked { extlen+4 } else { extlen };
	if payload.len() < expected_len {
		println!("Payload length doesn't match packet ({} != {})", payload.len(), expected_len);
		return &payload[0..0];
	}

	if masked {
		let mut mask = [0u8; 4];
		mask.clone_from_slice(&payload[..4]);

		for (i, val) in payload[4..expected_len].iter_mut().enumerate() {
			*val ^= mask[i % mask.len()];
		}

		&payload[4..expected_len]
	} else {
		&payload[..expected_len]
	}
}

fn encode_ws_packet<'a>(buf: &'a mut [u8], payload: &[u8]) -> &'a [u8] {
	let short_len = match payload.len() {
		l @ 0...125 => l,
		126...65535 => 126,
		_ => 127,
	};

	// Compile header
	let mut header = 0u16;
	header |= 1 << 15; // FIN
	header |= 0x2 << 8; // opcode
	header |= short_len as u16 & ((1<<7) - 1); // len field

	buf[0] = (header >> 8) as u8;
	buf[1] = (header & 0xFF) as u8;

	let len = payload.len();

	// Write payload length
	match short_len {
		127 => unimplemented!(), // 64b length
		126 => {
			buf[2] = (len >> 8) as u8;
			buf[3] = (len & 0xFF) as u8;
		},
		_ => {},
	}

	// Copy payload
	{
		let mut payload_dst = match short_len {
			127 => &mut buf[10..],
			126 => &mut buf[4..],
			_ => &mut buf[2..],
		};

		payload_dst[..len].copy_from_slice(&payload[..]);
	}

	// Return slice containing the entire packet
	match short_len {
		127 => &buf[.. 10 + len],
		126 => &buf[.. 4 + len],
		_ => &buf[.. 2 + len]
	}
}