use math::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Species {
	A, B, C
}

pub const ALL_SPECIES: [Species; 3] = {
	use self::Species::*; 
	[A, B, C]
};

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
	pub pos: Vec2,
	pub id: u32,
}

impl Tree {
	pub fn is_dead(&self) -> bool {
		match self.maturity {
			Maturity::Dead => true,
			_ => false,
		}
	}

	pub fn is_mature(&self) -> bool {
		match self.maturity {
			Maturity::Adult(_) => true,
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
			Adult(_) => -1.0,
			Dead => 0.0,
		}		
	}
}