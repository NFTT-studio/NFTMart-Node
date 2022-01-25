// https://substrate.recipes/events.html

#[frame_support::pallet]
mod this {
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event]
	pub enum Event<T: Config> {
		DummyEvent { who: T::AccountId },
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
}

pub use this::*;
