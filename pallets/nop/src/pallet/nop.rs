use frame_support::pallet;

#[pallet]
mod this {
	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
}

pub use this::*;
