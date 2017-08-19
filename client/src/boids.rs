use common::math::*;
use common::easing::*;

use rand::{random, Open01, Closed01};

#[derive(Clone)]
pub struct Boid {
	pub pos: Vec2,
	pub vel: Vec2,
	heading: f32,
	id: u32,

	pub phase: f32,
	pub rate: f32,
}

pub struct BoidSystem {
	boids: Vec<Boid>,
	world_bounds: Vec2,
}

// for ref: https://www.red3d.com/cwr/boids/

impl BoidSystem {
	pub fn new(world_bounds: Vec2) -> Self {
		let rand_f32 = |range| {
			let Closed01(f) = random::<Closed01<f32>>();
			f * range
		};

		let rand_vec2 = |range: Vec2| {
			Vec2::new(rand_f32(range.x), rand_f32(range.y))
		};

		let mut boids = Vec::new();

		for _ in 0..100 {
			let id = boids.len() as u32;

			boids.push(Boid{
				pos: rand_vec2(world_bounds),
				vel: rand_vec2(Vec2::splat(2.0)) - Vec2::splat(1.0),
				heading: rand_f32(PI),
				id,
				phase: 0.0,
				rate: rand_f32(PI/2.0) + PI*3.0/2.0,
			});
		}

		BoidSystem {
			boids,
			world_bounds,
		}
	}

	pub fn update(&mut self, dt: f32) {
		let prev_boids = self.boids.clone();
		let range = 4.0;
		let a_range = PI*2.0/4.0;

		for boid in self.boids.iter_mut() {
			let boid_dir = boid.vel.normalize();

			let in_range: Vec<&Boid> = prev_boids.iter()
				.filter(|&ob| {
					if ob.id == boid.id { return false }

					let diff = ob.pos - boid.pos;
					let diff_len = diff.length();

					diff_len < range
						&& (diff.dot(boid_dir)/diff_len).acos() < a_range
				})
				.collect();

			let flocking_acc = if in_range.len() > 0 {
				let count = in_range.len() as f32;
				let count = Vec2::splat(count);

				let centre: Vec2 = in_range.iter()
					.fold(Vec2::zero(), |a, b| a + b.pos) / count;

				let cohesion = centre - boid.pos;

				let separation = in_range.iter()
					.fold(Vec2::zero(), |a, ob| {
						let diff = ob.pos - boid.pos;
						let dist = diff.length();
						let separation_amount = (1.0 - dist).max(0.0);

						a - diff.normalize() * separation_amount
					});

				let average_heading: Vec2 = in_range.iter()
					.map(|ob| ob.vel.normalize())
					.fold(Vec2::zero(), |acc, d| acc + d) / count;

				cohesion + separation * 0.3 + average_heading * 0.4
			} else {
				Vec2::zero()
			};

			let edge_avoid_margin = 5.0;

			let diff_to_center = self.world_bounds*0.5 - boid.pos;
			let abs_dtc = Vec2::new(diff_to_center.x.abs(), diff_to_center.y.abs());
			let dist_to_edge = abs_dtc - (self.world_bounds*0.5 - Vec2::splat(edge_avoid_margin * 2.0));
			let clamped_dist = Vec2::new(dist_to_edge.x.max(0.0), dist_to_edge.y.max(0.0));
			let edge_avoid = diff_to_center.normalize() * clamped_dist * (1.0 / edge_avoid_margin);

			let Open01(random_heading_delta) = random::<Open01<f32>>();
			let random_heading_delta = random_heading_delta * 2.0 - 1.0;

			boid.heading += random_heading_delta * PI * dt * 2.0;
			let heading = Vec2::from_angle(boid.heading);

			let acc = flocking_acc + edge_avoid * 3.0 + heading * 2.0;

			if acc.length() > 0.01 {
				boid.vel = dt.ease_linear(boid.vel, acc.normalize(), 1.0);
			}

			boid.vel = boid.vel.normalize() * 0.75;
			boid.pos = boid.pos + boid.vel * dt;

			boid.phase += dt * boid.rate;
		}
	}

	pub fn get_boids(&self) -> &Vec<Boid> {
		&self.boids
	}
}