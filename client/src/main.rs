#![feature(link_args)]
#![feature(ord_max_min)]

#[macro_use]
extern crate common;
extern crate rand;
extern crate libc;

#[macro_use]
mod ems;
mod util;
mod context;
mod rendering;
mod connection;

mod ui;

mod worldstate;

use context::*;

fn main() {
	println!("Is Hosted:      {}", cfg!(hosted));
	println!("Public address: {}", env!("PUBLIC_ADDRESS"));

	ems::start(Box::into_raw(Box::new(MainContext::new())));
}

