#[frame_support::pallet]
mod this {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn emit(origin: OriginFor<T>) -> DispatchResult {
			let who: T::AccountId = ensure_signed(origin)?;
			Self::deposit_event(Event::DummyEvent { who });
			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DummyEvent { who: T::AccountId },
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
}

pub use this::*;
