// ref: https://docs.substrate.io/v3/runtime/debugging/

#[frame_support::pallet]
mod this {
	use frame_support::{debug, pallet_prelude::*, sp_runtime::print, sp_std::if_std};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn foo(origin: OriginFor<T>) -> DispatchResult {
			// Ensure that the caller is a regular keypair account
			let caller = ensure_signed(origin)?;

			print("print: foo");

			debug(&caller);

			log::debug!("log::debug: called by {:?}", caller);

			log::info!("log::info: called by {:?}", caller);

			if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", caller);
			}

			Ok(())
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
}

pub use this::*;
