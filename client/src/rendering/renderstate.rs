use rendering::gl;

pub struct RenderState {}

impl RenderState {
	pub fn new() -> Self {
		RenderState {}
	}

	pub fn render(&self) {
		unsafe {
			gl::ClearColor(0.1, 0.1, 0.1, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}
	}
}