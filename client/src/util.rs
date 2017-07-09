#![allow(dead_code)]

#[link_args = "--js-library libutil.js"]
extern {
	fn save_canvas_raw(target: *const u8, targetLen: usize);
}

pub fn save_canvas(target: &str) {
	unsafe{ save_canvas_raw(target.as_ptr(), target.len()) };
}