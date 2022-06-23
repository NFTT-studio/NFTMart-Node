#[frame_support::pallet]
mod this {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn emit(_origin: OriginFor<T>) -> DispatchResult {
			Self::deposit_event(Event::DummyEvent);
			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		DummyEvent,
		FooEvent(u8),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
}

pub use this::*;
