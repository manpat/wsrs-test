use common::math::*;
use rendering::gl;

use rendering::texture::*;

pub struct Framebuffer {
	gl_handle: u32,
	targets: Vec<Texture>,
	size: Vec2i,
}

impl Framebuffer {
	pub fn bind(&self) {
		unsafe {
			gl::BindFramebuffer(gl::FRAMEBUFFER, self.gl_handle);
		}
	}

	pub fn unbind() {
		unsafe {
			gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
		}
	}
}

pub struct FramebufferBuilder {
	fb: Framebuffer,
}

impl FramebufferBuilder {
	pub fn new(size: Vec2i) -> Self {
		let mut fb = Framebuffer { gl_handle: 0, targets: Vec::new(), size };

		unsafe {
			gl::GenFramebuffers(1, &mut fb.gl_handle);
			fb.bind();
		}

		FramebufferBuilder { fb }
	}

	pub fn finalize(self) -> Framebuffer {
		Framebuffer::unbind();

		self.fb
	}

	pub fn add_depth(&mut self) -> &mut Self {
		let mut gl_handle = 0;

		unsafe {
			gl::GenTextures(1, &mut gl_handle);
			gl::BindTexture(gl::TEXTURE_2D, gl_handle);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as i32, self.fb.size.x, self.fb.size.y, 0, 
				gl::DEPTH_COMPONENT, gl::UNSIGNED_INT, 0 as *const _);

			gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, 
				gl::TEXTURE_2D, gl_handle, 0);

			gl::BindTexture(gl::TEXTURE_2D, 0);
		}		

		self
	}

	pub fn add_target(&mut self) -> &mut Self {
		let mut gl_handle = 0;

		let next_target = self.fb.targets.len() as u32;

		unsafe {
			gl::GenTextures(1, &mut gl_handle);
			gl::BindTexture(gl::TEXTURE_2D, gl_handle);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, self.fb.size.x, self.fb.size.y, 0, 
				gl::RGBA, gl::UNSIGNED_BYTE, 0 as *const _);

			gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + next_target, gl::TEXTURE_2D, gl_handle, 0);

			gl::BindTexture(gl::TEXTURE_2D, 0);
		}

		self.fb.targets.push(Texture{gl_handle, size: self.fb.size});

		self
	}
}