use rendering::gl;
use rendering::types::*;

pub struct Texture {
	gl_handle: u32,
	pub size: Vec2i,
}

pub struct TextureBuilder { gl_handle: u32 }

impl Texture {
	pub fn bind_to_slot(&self, slot: u32) {
		unsafe {
			gl::ActiveTexture(gl::TEXTURE0 + slot);
			gl::BindTexture(gl::TEXTURE_2D, self.gl_handle);
		}
	}

	pub fn upload_1d(&mut self, data: &[Color]) {
		unsafe {
			self.size = Vec2i::new(data.len() as i32, 1);

			gl::BindTexture(gl::TEXTURE_2D, self.gl_handle);
			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, self.size.x, 1, 0, gl::RGBA, gl::FLOAT, data.as_ptr() as *const _);
		}
	}
}

impl TextureBuilder {
	pub fn new() -> TextureBuilder {
		let mut gl_handle = 0;

		unsafe {
			gl::GenTextures(1, &mut gl_handle);
			gl::BindTexture(gl::TEXTURE_2D, gl_handle);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
		}

		TextureBuilder { gl_handle }
	}

	pub fn finalize(&self) -> Texture {
		Texture {
			gl_handle: self.gl_handle,
			size: Vec2i::zero(),
		}
	}

	pub fn linear_minify(&self) -> &Self {
		unsafe {
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
		}

		self
	}

	pub fn linear_magnify(&self) -> &Self {
		unsafe {
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
		}

		self
	}
}