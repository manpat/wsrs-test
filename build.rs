use std::fs::File;
use std::net::TcpStream;
use std::io::{Read, Write};

fn main() {
	let mut hosted = false;
	let mut fres = File::open("/sys/hypervisor/uuid");
	if let Ok(ref mut f) = fres {
		let mut data = String::new();
		f.read_to_string(&mut data).unwrap();

		if &data[..3] == "ec2" {
			hosted = true;
		}
	}

	if hosted {
		println!("cargo:rustc-cfg=hosted");
		// http://169.254.169.254/latest/meta-data/public-ipv4

		let mut metadata_server = TcpStream::connect("169.254.169.254:80").unwrap();
		let request = b"GET /latest/meta-data/public-ipv4 HTTP/1.1\r\n\r\n";
		metadata_server.write_all(request).unwrap();

		let mut response = String::new();
		metadata_server.read_to_string(&mut response).unwrap();

		if response.contains("200 OK") {
			if let Some(address) = response.split("\r\n\r\n").skip(1).next() {
				println!("cargo:rustc-env=PUBLIC_ADDRESS={}", address);
			} else {
				println!("cargo:warning=Couldn't determine public address! Falling back to constant");
				println!("cargo:rustc-env=PUBLIC_ADDRESS={}", "18.220.1.127");
			}
		}
	} else {
		// TODO: look into gethostname gethostbyname, or getifaddrs
		// can be used to determine local ip
		println!("cargo:rustc-env=PUBLIC_ADDRESS={}", "192.168.1.85");
	}

	// println!("cargo:rustc-cfg=debug_requests");

	match env!("CARGO_PKG_NAME") {
		"wsserver" => {},
		"wsclient" => {},
		_ => println!("cargo:warning=Compiling unknown package")
	}
}
