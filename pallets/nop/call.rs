#[frame_support::pallet]
mod this {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn foo(_origin: OriginFor<T>) -> DispatchResult {
			Ok(())
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
}

pub use this::*;
