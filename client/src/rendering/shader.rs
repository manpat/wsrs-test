use std;
use common::math::*;
use rendering::gl;

#[derive(Copy, Clone)]
pub struct Shader {
	pub gl_handle: u32,

	pub proj_loc: i32,
	pub view_loc: i32,
}

impl Shader {
	pub fn new(vertex_shader_src: &str, fragment_shader_src: &str) -> Shader {
		use std::ffi::{CStr, CString};
		unsafe {
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

			Shader {
				gl_handle: program,

				proj_loc: gl::GetUniformLocation(program, b"proj\0".as_ptr()),
				view_loc: gl::GetUniformLocation(program, b"view\0".as_ptr()),
			}
		}
	}

	pub const fn invalid() -> Shader {
		Shader {
			gl_handle: 0,
			proj_loc: 0,
			view_loc: 0,
		}
	}

	pub fn use_program(&self) {
		unsafe {
			gl::UseProgram(self.gl_handle);
		}
	}

	pub fn set_uniform_mat(&self, uniform: i32, mat: &Mat4) {
		unsafe {
			// TODO: Make sure we're bound
			gl::UniformMatrix4fv(uniform, 1, 0, mat.transpose().rows.as_ptr() as *const f32);
		}
	}

	pub fn set_proj(&self, mat: &Mat4) {
		self.set_uniform_mat(self.proj_loc, &mat);
	}

	pub fn set_view(&self, mat: &Mat4) {
		self.set_uniform_mat(self.view_loc, &mat);
	}
}