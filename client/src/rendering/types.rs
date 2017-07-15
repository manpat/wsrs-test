use easing::*;

use std::ops::{Add, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Vec2{pub x: f32, pub y: f32}

impl Vec2 {
	pub fn new(x: f32, y: f32) -> Vec2 { Vec2{x:x, y:y} }
	pub fn zero() -> Vec2 { Vec2::new(0.0, 0.0) }

	pub fn to_tuple(self) -> (f32,f32) { (self.x, self.y) }

	pub fn length(self) -> f32 {
		(self.x*self.x + self.y*self.y).sqrt()
	}
}

impl Add for Vec2 {
	type Output = Vec2;
	fn add(self, o: Vec2) -> Vec2 {
		Vec2::new(self.x + o.x, self.y + o.y)
	}
}

impl Sub for Vec2 {
	type Output = Vec2;
	fn sub(self, o: Vec2) -> Vec2 {
		Vec2::new(self.x - o.x, self.y - o.y)
	}
}

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
}

macro_rules! impl_ease_for_color {
    ($func: ident) => (
		fn $func(&self, start: Color, end: Color, duration: f32) -> Color {
			Color {
				r: self.$func(start.r, end.r, duration),
				g: self.$func(start.g, end.g, duration),
				b: self.$func(start.b, end.b, duration),
				a: self.$func(start.a, end.a, duration),
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
	pub size: (i32, i32),
}

impl Viewport {
	pub fn new() -> Viewport {
		Viewport{size: (0, 0)}
	}

	pub fn get_aspect(&self) -> f32 {
		let (sw, sh) = self.size;
		sw as f32 / sh as f32
	}

	pub fn client_to_gl_coords(&self, x: f32, y: f32) -> Vec2 {
		let (sw, sh) = self.size;
		let aspect = self.get_aspect();

		let (sx, sy) = (x / sw as f32, y / sh as f32);
		Vec2::new(aspect * (sx * 2.0 - 1.0), 1.0 - sy * 2.0)
	}

	pub fn get_top_left(&self) -> Vec2 {
		self.client_to_gl_coords(0.0, 0.0)
	}

	pub fn get_bottom_left(&self) -> Vec2 {
		self.client_to_gl_coords(0.0, self.size.1 as f32)
	}
}

