#![feature(slice_patterns)]
extern crate rand;

pub mod easing;
pub mod packet;
pub mod world;
pub mod math;

pub use packet::*;
pub use easing::*;
pub use math::*;

#[macro_export]
macro_rules! match_enum {
	($v:expr, $p:pat) => {
		match $v {
			$p => true,
			_ => false,
		}
	}
}

pub fn write_f32_to_slice(dst: &mut [u8], value: f32) {
	assert!(dst.len() >= 4);

	use std::mem::transmute;

	let a: [u8; 4] = unsafe {transmute(value)};
	dst[..4].copy_from_slice(&a);
}

pub fn write_u32_to_slice(dst: &mut [u8], value: u32) {
	assert!(dst.len() >= 4);

	use std::mem::transmute;

	let a: [u8; 4] = unsafe {transmute(value)};
	dst[..4].copy_from_slice(&a);
}

pub fn write_u16_to_slice(dst: &mut [u8], value: u16) {
	assert!(dst.len() >= 2);

	use std::mem::transmute;

	let a: [u8; 2] = unsafe {transmute(value)};
	dst[..2].copy_from_slice(&a);
}

pub fn read_f32_from_slice(src: &[u8]) -> f32 {
	assert!(src.len() >= 4);

	let mut a = [0u8; 4];
	a.copy_from_slice(&src[..4]);

	unsafe { std::mem::transmute(a) }
}

pub fn read_u32_from_slice(src: &[u8]) -> u32 {
	assert!(src.len() >= 4);

	let mut a = [0u8; 4];
	a.copy_from_slice(&src[..4]);

	unsafe { std::mem::transmute(a) }
}

pub fn read_u16_from_slice(src: &[u8]) -> u16 {
	assert!(src.len() >= 2);

	let mut a = [0u8; 2];
	a.copy_from_slice(&src[..2]);

	unsafe { std::mem::transmute(a) }
}

pub use rand::{thread_rng, Rng};
use rand::{random, Closed01};

pub fn rand_f32 (range: f32) -> f32 {
	let Closed01(f) = random::<Closed01<f32>>();
	f * range
}

pub fn rand_vec2 (range: Vec2) -> Vec2 {
	Vec2::new(rand_f32(range.x), rand_f32(range.y))
}