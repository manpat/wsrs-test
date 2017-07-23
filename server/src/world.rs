use std::time::{Instant, Duration};
use common::world::*;

const WORLD_DIMS: (usize,usize) = (20, 20);
const DIVERSITY_RANGE: f32 = 1.3;
const DEATH_AFFECT_RANGE: f32 = 1.0;
const GROWTH_AFFECT_RANGE: f32 = 2.0;
const TREE_RADIUS: f32 = 0.5;

pub struct World {
	pub trees: Vec<Tree>,
	pub land: [f32; WORLD_DIMS.0 * WORLD_DIMS.1],
	pub land_health: [f32; WORLD_DIMS.0 * WORLD_DIMS.1],

	next_tree_id: u32,
	last_tick: Instant,
}

impl World {
	pub fn new() -> Self {
		World {
			trees: Vec::new(),
			land: [0.0f32; WORLD_DIMS.0 * WORLD_DIMS.1],
			land_health: [0.0f32; WORLD_DIMS.0 * WORLD_DIMS.1],
			next_tree_id: 0,

			last_tick: Instant::now(),
		}
	}

	pub fn place_tree(&mut self, s: Species, p: Pos) -> bool {
		let pos_available = self.trees.iter()
			.all(|x| x.pos.dist_to(&p) > TREE_RADIUS);

		if pos_available {
			self.trees.push(Tree{
				species: s,
				maturity: Maturity::Seed(0),
				pos: p,
				id: self.next_tree_id,
			});

			self.next_tree_id += 1;
		}

		pos_available
	}

	pub fn update(&mut self) {
		use self::Maturity::*;

		let now = Instant::now();

		let diff = now - self.last_tick;
		if diff < Duration::from_millis(1500) { return }
		self.last_tick = now;

		for t in &mut self.trees {
			let p = t.pos;
			let (x,y) = (p.0 as usize, p.1 as usize);
			let health = self.land_health[x + y*WORLD_DIMS.0];

			let tick_rate = 100 + (200.0*health) as i32;

			t.maturity = match t.maturity {
				Dead => Dead,
				Adult(t) if t > 50 => Dead,
				Child(t) if t > 1000 => Adult(0),
				Seed(t) if t > 1000 => Child(0),

				Adult(i) => Adult(i + 1),
				Child(t) => Child(t + tick_rate),
				Seed(t) => Seed(t + tick_rate),
			};
		}

		let mut blur_buf = [0.0f32; WORLD_DIMS.0 * WORLD_DIMS.1];

		let ww = WORLD_DIMS.0 as i32;
		let wh = WORLD_DIMS.1 as i32;

		for y in 0..wh {
			for x in 0..ww {
				let sample = |x, y| {
					if x < 0 || y < 0 { return None }
					if x >= ww || y >= wh { return None }

					let idx = x + y*ww;
					Some(self.land[idx as usize])
				};

				let c: f32 = sample(x, y).unwrap();
				let o: f32 = [
					sample(x+1, y),
					sample(x, y+1),
					sample(x-1, y),
					sample(x, y-1),
				].iter().map(|x| x.unwrap_or(0.0)).sum();

				let d: f32 = [
					sample(x+1, y+1),
					sample(x+1, y-1),
					sample(x-1, y+1),
					sample(x-1, y-1),
				].iter().map(|x| x.unwrap_or(0.0)).sum();

				blur_buf[(x + y*ww) as usize] = c * 0.3 + o * 0.15 + d * 0.025;
			}
		}

		self.land.copy_from_slice(&blur_buf);

		for y in 0..WORLD_DIMS.1 {
			for x in 0..WORLD_DIMS.0 {
				let idx = x + y*WORLD_DIMS.0;

				let pos = Pos(x as f32 + 0.5, y as f32 + 0.5);

				let nearby_dead: f32 = self.trees.iter()
					.filter(|&t| t.is_dead())
					.map(|t| 1.0 - t.pos.dist_to(&pos) / DEATH_AFFECT_RANGE)
					.filter(|&d| d > 0.0)
					.sum();

				let nearby_growing: f32 = self.trees.iter()
					.filter(|&t| t.is_growing())
					.map(|t| t.get_consumption_rate() * (1.0 - t.pos.dist_to(&pos) / GROWTH_AFFECT_RANGE))
					.filter(|&d| d > 0.0)
					.sum();

				let local_diversity = self.get_diversity_at(pos, DIVERSITY_RANGE);

				let mut c = self.land[idx];
				c -= 0.5; // steady decay
				c += local_diversity / 0.4;
				c += nearby_dead * 50.0;
				c -= nearby_growing * 0.75;
				c = c.max(0.0);

				self.land[idx] = c;
				self.land_health[idx] = 1.0 - (c + 1.0).powf(1.0/3.0) / (c + 1.0);
			}
		}

		self.trees.retain(|x| !x.is_dead());
	}

	pub fn get_diversity_at(&self, p: Pos, r: f32) -> f32 {
		let q = 2.0;

		let trees_in_range = self.trees.iter()
			.map(|t| (t, t.pos.dist_to(&p)))
			.filter(|&(_, d)| d < r)
			.collect::<Vec<_>>();

		let total_potential_diversity = trees_in_range.iter()
			.fold(0.0, |a, &(t, d)| a + t.get_diversity_contribution() * 4.0 / d);

		let abundances: Vec<_> = ALL_SPECIES.iter()
			.map(|x| trees_in_range.iter().filter(|&tp| tp.0.species == *x)
				.fold(0.0, |a, &(t, d)| a + t.get_diversity_contribution() * 4.0 / d))
			.filter(|&x| x > 0.0)
			.map(|x| x / total_potential_diversity)
			.collect();

		let diversity = abundances.iter()
			.map(|p| p * p.powf(q - 1.0))
			.sum::<f32>();

		if diversity > 0.0 {
			let diversity = diversity.powf(-1.0 / (q - 1.0));
			(diversity - 1.0) / (ALL_SPECIES.len() - 1) as f32
		} else {
			0.0
		}
	}
}
