#![allow(unused_variables)]

// http://robertpenner.com/easing/
// http://easings.net/

pub trait Clamp<Bound> {
	fn clamp(&self, start: Bound, end: Bound) -> Self;
}

pub trait Ease<Param> {
	fn ease_linear(&self, start: Param, end: Param) -> Param;

	fn ease_quad_in(&self, start: Param, end: Param) -> Param;
	fn ease_quad_out(&self, start: Param, end: Param) -> Param;
	fn ease_quad_inout(&self, start: Param, end: Param) -> Param;

	fn ease_exp_in(&self, start: Param, end: Param) -> Param;
	fn ease_exp_out(&self, start: Param, end: Param) -> Param;
	fn ease_exp_inout(&self, start: Param, end: Param) -> Param;

	fn ease_elastic_in(&self, start: Param, end: Param) -> Param;
	fn ease_elastic_out(&self, start: Param, end: Param) -> Param;
	fn ease_elastic_inout(&self, start: Param, end: Param) -> Param;

	fn ease_back_in(&self, start: Param, end: Param) -> Param;
	fn ease_back_out(&self, start: Param, end: Param) -> Param;
	fn ease_back_inout(&self, start: Param, end: Param) -> Param;

	fn ease_bounce_in(&self, start: Param, end: Param) -> Param;
	fn ease_bounce_out(&self, start: Param, end: Param) -> Param;
	fn ease_bounce_inout(&self, start: Param, end: Param) -> Param;
}


impl Clamp<f32> for f32 {
	fn clamp(&self, start: f32, end: f32) -> f32 {
		let a = start.min(end);
		let b = start.max(end);
		self.max(a).min(b)
	}
}

impl Ease<f32> for f32 {
	fn ease_linear(&self, b: f32, e: f32) -> f32 {
		let t = self.clamp(0.0, 1.0);
		let c = e-b;
		c * t + b
	}

	fn ease_quad_in(&self, b: f32, e: f32) -> f32 {
		let t = self.clamp(0.0, 1.0);
		let c = e-b;
		c * t.powi(2) + b
	}

	fn ease_quad_out(&self, b: f32, e: f32) -> f32 {
		let t = self.clamp(0.0, 1.0);
		let c = e-b;
		-c * t * (t - 2.0) + b
	}

	fn ease_quad_inout(&self, b: f32, e: f32) -> f32 {
		let t = self.clamp(0.0, 1.0) * 2.0;
		let c = e-b;
		if t < 1.0 { c / 2.0 * t.powi(2) + b }
		else { -c / 2.0 * ((t - 1.0) * (t - 3.0) - 1.0) + b }
	}

	fn ease_exp_in(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		let t = self.clamp(0.0, 1.0);

		if t <= 0.0 { b }
		else { c * 2.0f32.powf(10.0 * (t - 1.0)) + b }
	}

	fn ease_exp_out(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		let t = self.clamp(0.0, 1.0);
		if t >= 1.0 { b+c }
		else { c * (-2f32.powf(-10.0 * t) + 1.0) + b }
	}

	fn ease_exp_inout(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		let t = self.clamp(0.0, 1.0) * 2.0;
		if t <= 0.0			{ b }
		else if t >= 2.0	{ b+c }
		else if t < 1.0		{ c/2.0 * 2f32.powf(10.0 * (t - 1.0)) + b }
		else 				{ c/2.0 * (-2f32.powf(-10.0 * (t - 1.0)) + 2.0) + b}
	}


	fn ease_elastic_in(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		unimplemented!();
	}

	fn ease_elastic_out(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		unimplemented!();
	}

	fn ease_elastic_inout(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		unimplemented!();
	}


	fn ease_back_in(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		let s = 1.70158;
		let t = self.clamp(0.0, 1.0);
		c*t*t*((s+1.0)*t - s) + b
	}

	fn ease_back_out(&self, b: f32, e: f32) -> f32 {	
		let s = 1.70158;
		let c = e-b;
		let t = self.clamp(0.0, 1.0) - 1.0;
		c*(t*t*((s+1.0)*t + s) + 1.0) + b
	}

	fn ease_back_inout(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		unimplemented!();
	}


	fn ease_bounce_in(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		unimplemented!();
	}

	fn ease_bounce_out(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		unimplemented!();
	}

	fn ease_bounce_inout(&self, b: f32, e: f32) -> f32 {
		let c = e-b;
		unimplemented!();
	}
}
