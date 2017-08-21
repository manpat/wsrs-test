pub use common::math::*;
pub use common::easing::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
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

	pub fn to_vec3(&self) -> Vec3 { Vec3::new(self.r, self.g, self.b) }
	pub fn to_vec4(&self) -> Vec4 { Vec4::new(self.r, self.g, self.b, self.a) }

	pub fn pow(self, exp: f32) -> Color {
		Color::rgba(
			self.r.powf(exp),
			self.g.powf(exp),
			self.b.powf(exp),
			self.a,
		)
	}
}

macro_rules! impl_ease_for_color {
	($func: ident) => (
		fn $func(&self, start: Color, end: Color) -> Color {
			Color {
				r: self.$func(start.r, end.r),
				g: self.$func(start.g, end.g),
				b: self.$func(start.b, end.b),
				a: self.$func(start.a, end.a),
			}
		}
	)
}

impl Ease<Color> for f32 {
	impl_ease_for_color!(ease_linear);

	impl_ease_for_color!(ease_quad_in);
	impl_ease_for_color!(ease_quad_out);
	impl_ease_for_color!(ease_quad_inout);

	impl_ease_for_color!(ease_exp_in);
	impl_ease_for_color!(ease_exp_out);
	impl_ease_for_color!(ease_exp_inout);

	impl_ease_for_color!(ease_elastic_in);
	impl_ease_for_color!(ease_elastic_out);
	impl_ease_for_color!(ease_elastic_inout);

	impl_ease_for_color!(ease_back_in);
	impl_ease_for_color!(ease_back_out);
	impl_ease_for_color!(ease_back_inout);

	impl_ease_for_color!(ease_bounce_in);
	impl_ease_for_color!(ease_bounce_out);
	impl_ease_for_color!(ease_bounce_inout);
}

#[derive(Copy, Clone, Debug)]
pub struct Viewport {
	pub size: Vec2i,
}

impl Viewport {
	pub fn new() -> Viewport {
		Viewport{ size: Vec2i::zero() }
	}

	pub fn get_aspect(&self) -> f32 {
		let (sw, sh) = self.size.to_tuple();
		sw as f32 / sh as f32
	}

	pub fn client_to_gl_coords(&self, pos: Vec2i) -> Vec2 {
		let (sw, sh) = self.size.to_vec2().to_tuple();
		let pos = pos.to_vec2();
		let aspect = self.get_aspect();

		let (sx, sy) = (pos.x / sw, pos.y / sh);
		Vec2::new(aspect * (sx * 2.0 - 1.0), 1.0 - sy * 2.0)
	}

	pub fn get_top_left(&self) -> Vec2 {
		self.client_to_gl_coords(Vec2i::zero())
	}

	pub fn get_bottom_left(&self) -> Vec2 {
		self.client_to_gl_coords(Vec2i::new(0, self.size.y))
	}

	pub fn get_top_right(&self) -> Vec2 {
		self.client_to_gl_coords(Vec2i::new(self.size.x, 0))
	}

	pub fn get_bottom_right(&self) -> Vec2 {
		self.client_to_gl_coords(self.size)
	}
}

