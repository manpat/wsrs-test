#[derive(Copy, Clone, Debug)]
pub struct Pos (pub f32, pub f32);

impl Pos {
	pub fn dist_to(&self, o: &Pos) -> f32 {
		let (x,y) = (self.0-o.0, self.1-o.1);
		(x*x + y*y).sqrt()
	}
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Species {
	A, B, C
}

#[derive(Copy, Clone, Debug)]
pub enum Maturity {
	// [0, 1000) - affected by tick rate
	Seed(i32),
	// [0, 1000) - affected by tick rate
	Child(i32),
	// [0, 10)
	Adult(i32),
	Dead,
}

#[derive(Debug)]
pub struct Tree {
	pub species: Species,
	pub maturity: Maturity,
	pub pos: Pos,
	pub id: u32,
}

impl Tree {
	pub fn is_dead(&self) -> bool {
		match self.maturity {
			Maturity::Dead => true,
			_ => false,
		}
	}

	pub fn is_growing(&self) -> bool {
		match self.maturity {
			Maturity::Seed(_) => true,
			Maturity::Child(_) => true,
			_ => false
		}
	}

	pub fn get_diversity_contribution(&self) -> f32 {
		use self::Maturity::*;

		match self.maturity {
			Seed(_) => 0.5,
			Child(_) => 0.75,
			Adult(_) => 1.0,
			Dead => 0.0,
		}
	}

	pub fn get_consumption_rate(&self) -> f32 {
		use self::Maturity::*;

		match self.maturity {
			Seed(_) => 1.0,
			Child(_) => 0.5,
			Adult(_) => 0.1,
			Dead => 0.0,
		}		
	}
}