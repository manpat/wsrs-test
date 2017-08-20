use math::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Species {
	A, B, C
}

pub const ALL_SPECIES: [Species; 3] = {
	use self::Species::*; 
	[A, B, C]
};

impl Species {
	pub fn to_byte(self) -> u8 {
		match self {
			Species::A => 0,
			Species::B => 1,
			Species::C => 2,
		}
	}

	pub fn from_byte(b: u8) -> Option<Self> {
		match b {
			0 => Some(Species::A),
			1 => Some(Species::B),
			2 => Some(Species::C),
			_ => None
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum Maturity {
	// [0, 1000) - affected by tick rate
	Seed(i32),
	// [0, 1000) - affected by tick rate
	Child(i32),
	// [0, 50)
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
			Seed(_) => 0.25,
			Child(_) => 0.5,
			Adult(x) if x < 25 => 0.6,
			Adult(_) => 1.0,
			Dead => 0.0,
		}
	}

	pub fn get_consumption_rate(&self) -> f32 {
		use self::Maturity::*;

		match self.maturity {
			Seed(_) => 1.0,
			Child(_) => 0.8,
			Adult(x) if x < 25 => 0.4,
			Adult(_) => -1.0,
			Dead => 0.0,
		}		
	}

	pub fn get_maturity_stage(&self) -> u8 {
		use self::Maturity::*;

		match self.maturity {
			Seed(_) => 0,
			Child(_) => 1,
			Adult(x) if x < 25 => 2,
			Adult(_) => 3,
			Dead => 4,
		}		
	}
}