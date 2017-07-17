use ems;
use std::ops::Drop;
use std::ffi::CString;
use std::mem::uninitialized;

#[link_args = "-s FULL_ES2=1"]
extern {}

pub mod gl {
	#![allow(non_upper_case_globals)]
	include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub mod types;
pub use self::types::*;

pub mod renderstate;
pub use self::renderstate::*;

pub struct RenderingContext {
	ems_context_handle: ems::EmWebGLContext,
	canvas_id: String,

	viewport: Viewport,
	program: u32,
	view_loc: i32,
}

impl RenderingContext {
	pub fn new(canvas_id: &str) -> Self {
		let mut attribs = unsafe { uninitialized() };
		unsafe { ems::emscripten_webgl_init_context_attributes(&mut attribs) };
		attribs.alpha = 0;
		attribs.stencil = 1;
		attribs.antialias = 1;
		attribs.preserve_drawing_buffer = 0;
		attribs.enable_extensions_by_default = 0;

		let s = CString::new(canvas_id).unwrap();
		let ems_context_handle = unsafe{ ems::emscripten_webgl_create_context(s.as_ptr(), &attribs) };

		assert!(ems_context_handle > 0, "WebGL context creation failed for {} ({})", canvas_id, ems_context_handle);

		let mut ctx = RenderingContext {
			ems_context_handle,
			canvas_id: canvas_id.to_string(),
			viewport: Viewport::new(),
			program: 0,
			view_loc: 0,
		};

		assert!(ctx.make_current(), "Failed to make WebGL context current");

		let vertex_shader_src = r#"
			attribute vec3 position;
			attribute vec4 color;

			uniform mat4 view;

			varying vec4 vcolor;

			void main() {
				vec4 pos = view * vec4(position, 1.0);
				gl_Position = vec4(pos.xyz, 1.0);
				vcolor = color;
			}
		"#;

		let fragment_shader_src = r#"
			precision mediump float;

			varying vec4 vcolor;
			void main() {
				gl_FragColor = vcolor;
			}
		"#;

		unsafe {
			use std::ffi::{CStr, CString};
			use std;

			let (vs,fs) = (gl::CreateShader(gl::VERTEX_SHADER), gl::CreateShader(gl::FRAGMENT_SHADER));
			let program = gl::CreateProgram();

			for &(sh, src) in [(vs, vertex_shader_src), (fs, fragment_shader_src)].iter() {
				let src = CString::new(src).unwrap();
				gl::ShaderSource(sh, 1, &src.as_ptr(), std::ptr::null());
				gl::CompileShader(sh);

				let mut status = 0i32;
				gl::GetShaderiv(sh, gl::COMPILE_STATUS, &mut status);
				if status == 0 {
					let mut buf = [0u8; 1024];
					let mut len = 0i32;
					gl::GetShaderInfoLog(sh, buf.len() as i32, &mut len, buf.as_mut_ptr());

					println!("{}", CStr::from_bytes_with_nul_unchecked(&buf[..len as usize]).to_str().unwrap());
				}
				
				gl::AttachShader(program, sh);
			}

			gl::LinkProgram(program);
			gl::UseProgram(program);

			gl::DeleteShader(vs);
			gl::DeleteShader(fs);

			gl::UseProgram(program);
			ctx.program = program;
			ctx.view_loc = gl::GetUniformLocation(program, CString::new("view").unwrap().as_ptr());

			gl::Enable(gl::BLEND);
			gl::BlendEquationSeparate(gl::FUNC_ADD, gl::FUNC_ADD);
			gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ZERO);
		}

		ctx
	}

	pub fn make_current(&mut self) -> bool {
		unsafe { ems::emscripten_webgl_make_context_current(self.ems_context_handle) == 0 }
	}

	pub fn is_current(&self) -> bool {
		unsafe { ems::emscripten_webgl_get_current_context() == self.ems_context_handle }
	}

	#[allow(dead_code)]
	pub fn set_target_size(&mut self, w: i32, h: i32) {
		js! { (self.canvas_id.as_ptr(), self.canvas_id.len() as i32) 
			b"Module.canvas = document.getElementById(Pointer_stringify($0, $1))\0" };

		js! { (w) b"Module.canvas.width = Module.canvas.style.width = $0\0" };
		js! { (h) b"Module.canvas.height = Module.canvas.style.height = $0\0" };

		self.viewport.size = Vec2i::new(w,h);
	}

	pub fn fit_target_to_viewport(&mut self) {
		js! { (self.canvas_id.as_ptr(), self.canvas_id.len() as i32) 
			b"Module.canvas = document.getElementById(Pointer_stringify($0, $1))\0" };

		let w = js! { b"return (Module.canvas.width = Module.canvas.style.width = window.innerWidth)\0" };
		let h = js! { b"return (Module.canvas.height = Module.canvas.style.height = window.innerHeight)\0" };

		self.viewport.size = Vec2i::new(w,h);
	}

	pub fn get_viewport(&self) -> Viewport {
		self.viewport
	}

	pub fn render(&mut self, state: &RenderState) {
		if !self.is_current() {
			assert!(self.make_current());
		}

		let aspect = self.viewport.get_aspect();

		let matrix = [
			1.0/aspect,		0.0,	0.0, 0.0,
			0.0,			1.0,	0.0, 0.0,
			0.0,			0.0,	1.0, 0.0,
			0.0,			0.0,	0.0, 1.0f32,
		];

		unsafe {
			gl::ClearColor(0.1, 0.1, 0.1, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
			gl::Viewport(0, 0, self.viewport.size.x, self.viewport.size.y);

			gl::UniformMatrix4fv(self.view_loc, 1, 0, matrix.as_ptr());
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