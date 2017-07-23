pub mod easing;
pub mod packet;
pub mod world;
pub mod math;

pub use packet::*;
pub use easing::*;
pub use math::*;

#[macro_export]
macro_rules! match_enum {
	($v:expr, $p:pat) => {
		match $v {
			$p => true,
			_ => false,
		}
	}
}