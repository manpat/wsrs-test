#![allow(dead_code)]

use std::ffi::CString;

#[derive(Copy, Clone)]
pub struct Color {
	pub r:f32,
	pub g:f32,
	pub b:f32,
	pub a:f32,
}

#[link_args = "--js-library libdc.js"]
extern {
	pub fn get_canvas_height() -> f32;
	pub fn get_canvas_width() -> f32;
}

pub unsafe fn get_canvas_size() -> (f32,f32) {
	(get_canvas_width(), get_canvas_height())
}


impl Color {
	pub fn rgba(r:f32, g:f32, b:f32, a:f32) -> Color {
		Color {r,g,b,a}
	}

	pub fn rgb(r:f32, g:f32, b:f32) -> Color {
		Color {r,g,b, a: 1.0}
	}

	pub fn grey(v: f32) -> Color { Color::rgb(v, v, v) }
	pub fn grey_a(v: f32, a: f32) -> Color { Color::rgba(v, v, v, a) }
	pub fn white() -> Color { Color::grey(1.0) }
	pub fn black() -> Color { Color::grey(0.0) }

	pub fn to_byte_tuple(&self) -> (u8, u8, u8, u8) {
		let Color{r,g,b,a} = *self;
		((r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8, (a*255.0) as u8)
	}
}



pub trait Interop {
	fn as_int(self, _: &mut Vec<CString>) -> i32;
}

impl Interop for i32 {
	fn as_int(self, _: &mut Vec<CString>) -> i32 {
		return self;
	}
}

impl<'a> Interop for &'a str {
	fn as_int(self, arena: &mut Vec<CString>) -> i32 {
		let c = CString::new(self).unwrap();
		let ret = c.as_ptr() as i32;
		arena.push(c);
		return ret;
	}
}

impl<'a> Interop for *const u8 {
	fn as_int(self, _: &mut Vec<CString>) -> i32 {
		return self as i32;
	}
}

extern "C" {
	pub fn emscripten_asm_const_int(s: *const u8, ...) -> i32;
} 

#[macro_export]
macro_rules! js {
	( ($( $x:expr ),*) $y:expr ) => {
		{
			let mut arena: Vec<std::ffi::CString> = Vec::new();
			const LOCAL: &'static [u8] = $y;
			unsafe { ::dc::emscripten_asm_const_int(&LOCAL[0] as *const _ as *const u8, $($crate::Interop::as_int($x, &mut arena)),*) }
		}
	};
	( $y:expr ) => {
		{
			const LOCAL: &'static [u8] = $y;
			unsafe { ::dc::emscripten_asm_const_int(&LOCAL[0] as *const _ as *const u8) }
		}
	};
}
