#![cfg_attr(not(feature = "std"), no_std)]

mod primitives;

pub mod prelude {
	pub use super::primitives::*;
}

pub use primitives::*;
