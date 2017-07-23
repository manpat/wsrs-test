use easing::*;

use std::ops::{Add, Sub, Mul};

#[derive(Copy, Clone, Debug)]
pub struct Vec2{pub x: f32, pub y: f32}

#[derive(Copy, Clone, Debug)]
pub struct Vec3{pub x: f32, pub y: f32, pub z: f32}

#[derive(Copy, Clone, Debug)]
pub struct Vec2i{pub x: i32, pub y: i32}

impl Vec2 {
	pub fn new(x: f32, y: f32) -> Vec2 { Vec2{x, y} }
	pub fn zero() -> Vec2 { Vec2::new(0.0, 0.0) }
	pub fn from_angle(th: f32) -> Vec2 { Vec2::new(th.cos(), th.sin()) }

	pub fn to_tuple(self) -> (f32,f32) { (self.x, self.y) }

	pub fn length(self) -> f32 {
		(self.x*self.x + self.y*self.y).sqrt()
	}
}

impl Vec3 {
	pub fn new(x: f32, y: f32, z: f32) -> Vec3 { Vec3{x, y, z} }
	pub fn zero() -> Vec3 { Vec3::new(0.0, 0.0, 0.0) }

	pub fn to_tuple(self) -> (f32,f32,f32) { (self.x, self.y, self.z) }

	pub fn length(self) -> f32 {
		(self.x*self.x + self.y*self.y + self.z*self.z).sqrt()
	}
}

impl Vec2i {
	pub fn new(x: i32, y: i32) -> Vec2i { Vec2i{x:x, y:y} }
	pub fn zero() -> Vec2i { Vec2i::new(0, 0) }

	pub fn to_tuple(self) -> (i32,i32) { (self.x, self.y) }
	pub fn to_vec2(self) -> Vec2 { Vec2::new(self.x as f32, self.y as f32) }

	pub fn length(self) -> f32 {
		((self.x*self.x + self.y*self.y) as f32).sqrt()
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

impl Mul<f32> for Vec2 {
	type Output = Vec2;
	fn mul(self, o: f32) -> Vec2 {
		Vec2::new(self.x * o, self.y * o)
	}
}

impl Add for Vec3 {
	type Output = Vec3;
	fn add(self, o: Vec3) -> Vec3 {
		Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
	}
}

impl Sub for Vec3 {
	type Output = Vec3;
	fn sub(self, o: Vec3) -> Vec3 {
		Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
	}
}

impl Mul<f32> for Vec3 {
	type Output = Vec3;
	fn mul(self, o: f32) -> Vec3 {
		Vec3::new(self.x * o, self.y * o, self.z * o)
	}
}

impl Add for Vec2i {
	type Output = Vec2i;
	fn add(self, o: Vec2i) -> Vec2i {
		Vec2i::new(self.x + o.x, self.y + o.y)
	}
}

impl Sub for Vec2i {
	type Output = Vec2i;
	fn sub(self, o: Vec2i) -> Vec2i {
		Vec2i::new(self.x - o.x, self.y - o.y)
	}
}


macro_rules! impl_ease_for_vec2 {
	($func: ident) => (
		fn $func(&self, start: Vec2, end: Vec2, duration: f32) -> Vec2 {
			Vec2 {
				x: self.$func(start.x, end.x, duration),
				y: self.$func(start.y, end.y, duration),
			}
		}
	)
}

macro_rules! impl_ease_for_vec3 {
	($func: ident) => (
		fn $func(&self, start: Vec3, end: Vec3, duration: f32) -> Vec3 {
			Vec3 {
				x: self.$func(start.x, end.x, duration),
				y: self.$func(start.y, end.y, duration),
				z: self.$func(start.z, end.z, duration),
			}
		}
	)
}

impl Ease<Vec2> for f32 {
	impl_ease_for_vec2!(ease_linear);

	impl_ease_for_vec2!(ease_quad_in);
	impl_ease_for_vec2!(ease_quad_out);
	impl_ease_for_vec2!(ease_quad_inout);

	impl_ease_for_vec2!(ease_exp_in);
	impl_ease_for_vec2!(ease_exp_out);
	impl_ease_for_vec2!(ease_exp_inout);

	impl_ease_for_vec2!(ease_elastic_in);
	impl_ease_for_vec2!(ease_elastic_out);
	impl_ease_for_vec2!(ease_elastic_inout);

	impl_ease_for_vec2!(ease_back_in);
	impl_ease_for_vec2!(ease_back_out);
	impl_ease_for_vec2!(ease_back_inout);

	impl_ease_for_vec2!(ease_bounce_in);
	impl_ease_for_vec2!(ease_bounce_out);
	impl_ease_for_vec2!(ease_bounce_inout);
}

impl Ease<Vec3> for f32 {
	impl_ease_for_vec3!(ease_linear);

	impl_ease_for_vec3!(ease_quad_in);
	impl_ease_for_vec3!(ease_quad_out);
	impl_ease_for_vec3!(ease_quad_inout);

	impl_ease_for_vec3!(ease_exp_in);
	impl_ease_for_vec3!(ease_exp_out);
	impl_ease_for_vec3!(ease_exp_inout);

	impl_ease_for_vec3!(ease_elastic_in);
	impl_ease_for_vec3!(ease_elastic_out);
	impl_ease_for_vec3!(ease_elastic_inout);

	impl_ease_for_vec3!(ease_back_in);
	impl_ease_for_vec3!(ease_back_out);
	impl_ease_for_vec3!(ease_back_inout);

	impl_ease_for_vec3!(ease_bounce_in);
	impl_ease_for_vec3!(ease_bounce_out);
	impl_ease_for_vec3!(ease_bounce_inout);
}