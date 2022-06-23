#[frame_support::pallet]
mod this {
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Parameter: Get<u64>;
		type OtherParameter: Get<u64>;
		type StorageParameter: Get<u64>;
		type StaticParameter: Get<u64>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
}

pub use this::*;
