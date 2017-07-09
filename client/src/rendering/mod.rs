use ems;
use std::ops::Drop;
use std::ffi::CString;
use std::mem::uninitialized;

pub mod gl {
	#![allow(non_upper_case_globals)]
	include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod renderstate;
pub use self::renderstate::*;

pub struct RenderingContext {
	ems_context_handle: ems::EmWebGLContext,
	canvas_id: String,

	viewport_size: (i32, i32),
}

impl RenderingContext {
	pub fn new(canvas_id: &str) -> Self {
		let mut attribs = unsafe { uninitialized() };
		unsafe { ems::emscripten_webgl_init_context_attributes(&mut attribs) };
		attribs.alpha = 0;
		attribs.antialias = 1;
		attribs.preserve_drawing_buffer = 0;
		attribs.enable_extensions_by_default = 0;

		let s = CString::new(canvas_id).unwrap();
		let ems_context_handle = unsafe{ ems::emscripten_webgl_create_context(s.as_ptr(), &attribs) };

		assert!(ems_context_handle > 0, "WebGL context creation failed for {} ({})", canvas_id, ems_context_handle);

		RenderingContext {
			ems_context_handle,
			canvas_id: canvas_id.to_string(),
			viewport_size: (0,0),
		}
	}

	pub fn make_current(&mut self) -> bool {
		unsafe { ems::emscripten_webgl_make_context_current(self.ems_context_handle) == 0 }
	}

	pub fn is_current(&self) -> bool {
		unsafe { ems::emscripten_webgl_get_current_context() == self.ems_context_handle }
	}

	pub fn set_target_size(&mut self, w: i32, h: i32) {
		js! { (self.canvas_id.as_ptr(), self.canvas_id.len() as i32) 
			b"Module.canvas = document.getElementById(Pointer_stringify($0, $1))\0" };

		js! { (w) b"Module.canvas.width = Module.canvas.style.width = $0\0" };
		js! { (h) b"Module.canvas.height = Module.canvas.style.height = $0\0" };

		self.viewport_size = (w,h);
	}

	pub fn fit_target_to_viewport(&mut self) {
		js! { (self.canvas_id.as_ptr(), self.canvas_id.len() as i32) 
			b"Module.canvas = document.getElementById(Pointer_stringify($0, $1))\0" };

		let w = js! { b"return (Module.canvas.width = Module.canvas.style.width = window.innerWidth)\0" };
		let h = js! { b"return (Module.canvas.height = Module.canvas.style.height = window.innerHeight)\0" };

		self.viewport_size = (w,h);
	}

	pub fn render(&mut self, state: &RenderState) {
		if !self.is_current() {
			assert!(self.make_current());
		}

		state.render();
	}
}

impl Drop for RenderingContext {
	fn drop(&mut self) {
		unsafe {
			if self.ems_context_handle > 0 {
				ems::emscripten_webgl_destroy_context(self.ems_context_handle);
			}
		}
	}
}

#[derive(Copy, Clone)]
pub struct Color {
	pub r:f32,
	pub g:f32,
	pub b:f32,
	pub a:f32,
}

#[allow(dead_code)]
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

