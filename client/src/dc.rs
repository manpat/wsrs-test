#![allow(dead_code)]

use libc::c_char;
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
	fn dc_fill_color_raw(r: i32, g: i32, b: i32, a: f32);
	fn dc_stroke_color_raw(r: i32, g: i32, b: i32, a: f32);

	pub fn dc_fill_rect(x: i32, y: i32, w: i32, h: i32);
	pub fn dc_fill_circle(x: i32, y: i32, r: f32);
	pub fn dc_draw_circle(x: i32, y: i32, r: f32);

	fn dc_fill_text_raw(t: *const c_char, x: i32, y: i32);
	fn dc_set_font_raw(f: *const c_char);

	pub fn get_canvas_height() -> i32;
	pub fn get_canvas_width() -> i32;
}

pub unsafe fn dc_fill_color(col: Color) {
	let (r,g,b,_) = col.to_byte_tuple();
	dc_fill_color_raw(r as i32, g as i32, b as i32, col.a);
}

pub unsafe fn dc_stroke_color(col: Color) {
	let (r,g,b,_) = col.to_byte_tuple();
	dc_stroke_color_raw(r as i32, g as i32, b as i32, col.a);
}

pub unsafe fn dc_fill_text(t: &str, x: i32, y: i32) {
	let cstr = CString::new(t).unwrap();
	dc_fill_text_raw(cstr.as_bytes_with_nul().as_ptr(), x, y);
}

pub unsafe fn dc_set_font(t: &str) {
	let cstr = CString::new(t).unwrap();
	dc_set_font_raw(cstr.as_bytes_with_nul().as_ptr());
}

pub unsafe fn get_canvas_size() -> (i32,i32) {
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