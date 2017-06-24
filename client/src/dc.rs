#![allow(dead_code)]

use libc::c_char;
use std::ffi::CString;

#[link_args = "--js-library libdc.js"]
extern {
	pub fn dc_fill_color(r: i32, g: i32, b: i32, a: f32);
	pub fn dc_stroke_color(r: i32, g: i32, b: i32, a: f32);

	pub fn dc_fill_rect(x: i32, y: i32, w: i32, h: i32);
	pub fn dc_fill_circle(x: i32, y: i32, r: f32);
	pub fn dc_draw_circle(x: i32, y: i32, r: f32);

	fn dc_fill_text_raw(t: *const c_char, x: i32, y: i32);
	fn dc_set_font_raw(f: *const c_char);

	pub fn get_canvas_height() -> i32;
	pub fn get_canvas_width() -> i32;
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