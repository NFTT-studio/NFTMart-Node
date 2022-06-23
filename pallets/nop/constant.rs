#[frame_support::pallet]
mod this {
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type Constant: Get<()>;
		type ConstantNumber: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
}

pub use this::*;
