#[frame_support::pallet]
mod this {
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// see https://docs.substrate.io/v3/runtime/storage/#declaring-storage-items
	#[pallet::storage]
	type SomePrivateValue<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn some_primitive_value)]
	pub(super) type SomePrimitiveValue<T> = StorageValue<
		_,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type SomeComplexValue<T: Config> = StorageValue<
		_,
		T::AccountId,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type SomeMap<T: Config> = StorageMap<
		_,
		Blake2_128Concat, T::AccountId,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type SomeDoubleMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, u32,
		Blake2_128Concat, T::AccountId,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn some_nmap)]
	pub(super) type SomeNMap<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, u32>,
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Twox64Concat, u32>,
		),
		u32,
		ValueQuery,
	>;
}

pub use this::*;
