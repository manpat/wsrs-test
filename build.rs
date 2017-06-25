#![feature(libc)]

use std::fs::File;
use std::net::TcpStream;
use std::io::{Read, Write};

use std::env;

extern crate libc;
use libc::*;

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

		let mut metadata_server = TcpStream::connect("169.254.169.254").unwrap();
		let request = b"GET /latest/meta-data/public-ipv4 HTTP/1.1\r\n\r\n";
		metadata_server.write_all(request).unwrap();

		let mut response = String::new();
		metadata_server.read_to_string(&mut response).unwrap();

		let mut f = File::create("address").unwrap();
		f.write_all(response.as_bytes()).unwrap();
		// println!("cargo:rustc-env=public-address={}", response);
	} else {		
		println!("cargo:rustc-env=PUBLIC_ADDRESS={}", "192.168.1.85");
	}
}
