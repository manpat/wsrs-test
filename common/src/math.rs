use easing::*;

use std::ops::{Add, Sub, Mul};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vec2{pub x: f32, pub y: f32}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vec3{pub x: f32, pub y: f32, pub z: f32}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vec4{pub x: f32, pub y: f32, pub z: f32, pub w: f32}

#[derive(Copy, Clone, Debug)]
pub struct Vec2i{pub x: i32, pub y: i32}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Mat4{pub rows: [Vec4; 4]}

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

	pub fn to_tuple(&self) -> (f32,f32,f32) { (self.x, self.y, self.z) }
	pub fn extend(&self, w: f32) -> Vec4 { Vec4::new(self.x, self.y, self.z, w) }

	pub fn length(&self) -> f32 { self.dot(*self).sqrt() }
	pub fn normalize(&self) -> Vec3 { *self * (1.0/self.length()) }

	pub fn dot(&self, o: Vec3) -> f32 { self.x*o.x + self.y*o.y + self.z*o.z }
	pub fn cross(&self, o: Vec3) -> Vec3 {
		Vec3::new(
			self.y*o.z - self.z*o.y,
			self.z*o.x - self.x*o.z,
			self.x*o.y - self.y*o.x,
		)
	}
}

impl Vec4 {
	pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4 { Vec4{x, y, z, w} }
	pub fn zero() -> Vec4 { Vec4::new(0.0, 0.0, 0.0, 0.0) }
	pub fn from_slice(o: &[f32]) -> Vec4 {
		assert!(o.len() >= 4);
		Vec4::new(o[0], o[1], o[2], o[3])
	}

	pub fn to_tuple(&self) -> (f32,f32,f32,f32) { (self.x, self.y, self.z, self.w) }
	pub fn to_vec3(&self) -> Vec3 { Vec3::new(self.x, self.y, self.z) }

	pub fn length(&self) -> f32 { self.dot(*self).sqrt() }

	pub fn dot(&self, o: Vec4) -> f32 { self.x*o.x + self.y*o.y + self.z*o.z + self.w*o.w }
}

impl Vec2i {
	pub fn new(x: i32, y: i32) -> Vec2i { Vec2i{x, y} }
	pub fn zero() -> Vec2i { Vec2i::new(0, 0) }

	pub fn to_tuple(self) -> (i32,i32) { (self.x, self.y) }
	pub fn to_vec2(self) -> Vec2 { Vec2::new(self.x as f32, self.y as f32) }

	pub fn length(self) -> f32 {
		((self.x*self.x + self.y*self.y) as f32).sqrt()
	}
}

impl Mat4 {
	pub fn new(d: &[f32; 16]) -> Mat4 {
		Mat4 {
			rows: [
				Vec4::from_slice(&d[0..4]),
				Vec4::from_slice(&d[4..8]),
				Vec4::from_slice(&d[8..12]),
				Vec4::from_slice(&d[12..16]),
			]
		}
	}

	pub fn from_rows(rows: [Vec4; 4]) -> Mat4 { Mat4 { rows } }

	pub fn ident() -> Mat4 { Mat4::uniform_scale(1.0) }
	pub fn uniform_scale(s: f32) -> Mat4 { Mat4::scale(Vec3::new(s,s,s)) }

	pub fn scale(s: Vec3) -> Mat4 {
		Mat4::new(&[
			s.x, 0.0, 0.0, 0.0,
			0.0, s.y, 0.0, 0.0, 
			0.0, 0.0, s.z, 0.0,
			0.0, 0.0, 0.0, 1.0,
		])
	}

	pub fn translate(t: Vec3) -> Mat4 {
		Mat4::new(&[
			1.0, 0.0, 0.0, t.x,
			0.0, 1.0, 0.0, t.y, 
			0.0, 0.0, 1.0, t.z,
			0.0, 0.0, 0.0, 1.0,
		])
	}

	pub fn xrot(ph: f32) -> Mat4 {
		let (rx, ry) = (ph.cos(), ph.sin());

		Mat4::new(&[
			1.0, 0.0, 0.0, 0.0, 
			0.0,  rx, -ry, 0.0,
			0.0,  ry,  rx, 0.0,
			0.0, 0.0, 0.0, 1.0,
		])
	}
	pub fn yrot(ph: f32) -> Mat4 {
		let (rx, ry) = (ph.cos(), ph.sin());

		Mat4::new(&[
			 rx, 0.0, -ry, 0.0,
			0.0, 1.0, 0.0, 0.0, 
			 ry, 0.0,  rx, 0.0,
			0.0, 0.0, 0.0, 1.0,
		])
	}

	pub fn transpose(&self) -> Mat4 {
		let [a,b,c,d] = self.rows;

		Mat4::new(&[
			a.x, b.x, c.x, d.x,
			a.y, b.y, c.y, d.y,
			a.z, b.z, c.z, d.z,
			a.w, b.w, c.w, d.w,
		])
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

impl Mul<Vec3> for Vec3 {
	type Output = Vec3;
	fn mul(self, o: Vec3) -> Vec3 {
		Vec3::new(self.x * o.x, self.y * o.y, self.z * o.z)
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

impl Mul<Mat4> for Mat4 {
	type Output = Mat4;
	fn mul(self, o: Mat4) -> Mat4 {
		let mut d = [0.0f32; 16];
		let ot = o.transpose();

		for j in 0..4 {
			for i in 0..4 {
				d[j*4 + i] = self.rows[j].dot(ot.rows[i]);
			}
		}

		Mat4::new(&d)
	}
}

impl Mul<Vec4> for Mat4 {
	type Output = Vec4;
	fn mul(self, o: Vec4) -> Vec4 {
		Vec4::new(
			self.rows[0].dot(o),
			self.rows[1].dot(o),
			self.rows[2].dot(o),
			self.rows[3].dot(o),
		)
	}
}
impl Mul<Vec3> for Mat4 {
	type Output = Vec3;
	fn mul(self, o: Vec3) -> Vec3 {
		let o4 = o.extend(1.0);

		Vec3::new(
			self.rows[0].dot(o4),
			self.rows[1].dot(o4),
			self.rows[2].dot(o4),
		)
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