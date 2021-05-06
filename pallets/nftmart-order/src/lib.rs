#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
	transactional,
};
use sp_std::vec::Vec;
use frame_system::pallet_prelude::*;
pub use sp_core::constants_types::{GlobalId, Balance, ACCURACY, NATIVE_CURRENCY_ID};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned},
	RuntimeDebug, SaturatedConversion,
};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use nftmart_traits::{NftmartConfig, NftmartNft};


mod mock;
mod tests;

pub use module::*;

// #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
// pub enum OrderKind {
// 	Normal,
// 	Offer,
// 	British,
// 	Dutch,
// }

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrderItem<ClassId, TokenId> {
	/// class id
	#[codec(compact)]
	pub class_id: ClassId,
	/// token id
	#[codec(compact)]
	pub token_id: TokenId,
	/// quantity
	#[codec(compact)]
	pub quantity: TokenId,
	/// Price of this token.
	#[codec(compact)]
	pub price: Balance,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrderData<CurrencyId, BlockNumber, CategoryId, ClassId, TokenId> {
	/// currency ID.
	#[codec(compact)]
	pub currency_id: CurrencyId,
	/// The balances to create an order
	#[codec(compact)]
	pub deposit: Balance,
	/// This order will be invalidated after `deadline` block number.
	#[codec(compact)]
	pub deadline: BlockNumber,
	/// Category of this order.
	#[codec(compact)]
	pub category_id: CategoryId,
	/// nft list
	pub items: Vec<OrderItem<ClassId, TokenId>>
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
enum Releases {
	V1_0_0,
}

impl Default for Releases {
	fn default() -> Self {
		Releases::V1_0_0
	}
}

pub type TokenIdOf<T> = <T as module::Config>::TokenId;
pub type ClassIdOf<T> = <T as module::Config>::ClassId;
pub type BalanceOf<T> = <<T as module::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CurrencyIdOf<T> = <<T as module::Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// MultiCurrency type for trading
		type MultiCurrency: MultiReservableCurrency<Self::AccountId, Balance = Balance>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The class ID type
		type ClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + codec::FullCodec;

		/// The token ID type
		type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + codec::FullCodec;

		/// NFTMart nft
		type NFT: NftmartNft<Self::AccountId, Self::ClassId, Self::TokenId>;

		/// Extra Configurations
		type ExtraConfig: NftmartConfig<Self::AccountId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// no available order id
		NoAvailableOrderId,
		/// submit order with invalid deposit
		SubmitOrderWithInvalidDeposit,
		/// submit order with invalid deadline
		SubmitOrderWithInvalidDeadline,
		/// not token owner or not enough quantity
		NotTokenOwnerOrNotEnoughQuantity,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// CreatedOrder \[who, order_id\]
		CreatedOrder(T::AccountId, GlobalId),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			0
		}

		fn integrity_test () {}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		_phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				_phantom: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<StorageVersion<T>>::put(Releases::default());
		}
	}

	/// Storage version of the pallet.
	#[pallet::storage]
	pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

	// /// Index/store orders by token as primary key and order id as secondary key.
	// #[pallet::storage]
	// #[pallet::getter(fn order_by_token)]
	// pub type OrderByToken<T: Config> = StorageDoubleMap<_, Blake2_128Concat, (ClassIdOf<T>, TokenIdOf<T>), Twox64Concat, OrderIdOf<T>, T::AccountId>;

	/// Index/store orders by account as primary key and order id as secondary key.
	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub type Orders<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, GlobalId, OrderData<CurrencyIdOf<T>, BlockNumberOf<T>, GlobalId, ClassIdOf<T>, TokenIdOf<T>>>;

	// pub type Offers<T: Config> =

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create an order.
		///
		/// - `currency_id`: currency id
		/// - `category_id`: category id
		/// - `deposit`: The balances to create an order
		/// - `deadline`: deadline
		/// - `items`: a list of `(class_id, token_id, quantity, price)`
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn submit_order(
			origin: OriginFor<T>,
			#[pallet::compact] currency_id: CurrencyIdOf<T>,
			#[pallet::compact] category_id: GlobalId,
			#[pallet::compact] deposit: Balance,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			items: Vec<(ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>, Balance)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(deposit >= T::ExtraConfig::get_min_order_deposit(), Error::<T>::SubmitOrderWithInvalidDeposit);
			<T as Config>::Currency::reserve(&who, deposit.saturated_into())?;

			ensure!(frame_system::Pallet::<T>::block_number() < deadline, Error::<T>::SubmitOrderWithInvalidDeadline);
			let mut order = OrderData {
				currency_id,
				deposit,
				deadline,
				category_id,
				items: Vec::with_capacity(items.len()),
			};
			// check tokens' ownership.
			for item in items{
				let (class_id, token_id, quantity, price) = item;
				ensure!(T::NFT::free_quantity(&who, class_id, token_id) >= quantity, Error::<T>::NotTokenOwnerOrNotEnoughQuantity);
				order.items.push(OrderItem{
					class_id,
					token_id,
					quantity,
					price,
				})
			}

			T::ExtraConfig::inc_count_in_category(category_id)?;
			let order_id = T::ExtraConfig::get_then_inc_id()?;
			Orders::<T>::insert(&who, order_id, order);
			Self::deposit_event(Event::CreatedOrder(who, order_id));
			Ok(().into())
		}

		// /// Take an NFT order.
		// ///
		// /// - `class_id`: class id
		// /// - `token_id`: token id
		// /// - `price`: The max/min price to take an order. Usually it is set to the price of the target order.
		// #[pallet::weight(100_000)]
		// #[transactional]
		// pub fn take_order(
		// 	origin: OriginFor<T>,
		// 	#[pallet::compact] class_id: ClassIdOf<T>,
		// 	#[pallet::compact] token_id: TokenIdOf<T>,
		// 	#[pallet::compact] price: Balance,
		// 	order_owner: <T::Lookup as StaticLookup>::Source,
		// ) -> DispatchResultWithPostInfo {
		// 	let who = ensure_signed(origin)?;
		// 	let order_owner = T::Lookup::lookup(order_owner)?;
		// 	// Simplify the logic, to make life easier.
		// 	ensure!(order_owner != who, Error::<T>::TakeOwnOrder);
		// 	let token_owner = orml_nft::Pallet::<T>::tokens(class_id, token_id).ok_or(Error::<T>::TokenIdNotFound)?.owner;
		//
		// 	let order: OrderData<T> = {
		// 		let order = Self::orders((class_id, token_id), &order_owner);
		// 		ensure!(order.is_some(), Error::<T>::OrderNotFound);
		// 		let order = order.unwrap();
		// 		ensure!(<frame_system::Pallet<T>>::block_number() <= order.deadline, Error::<T>::OrderExpired);
		// 		order
		// 	};
		//
		// 	match (order_owner == token_owner, token_owner == who) {
		// 		(true, false) => {
		// 			ensure!(price >= order.price, Error::<T>::CanNotAfford);
		// 			// `who` will take the order submitting by `order_owner`/`token_owner`
		// 			Self::delete_order(class_id, token_id, &order_owner)?;
		// 			// Try to delete another order for safety.
		// 			// Because `who` may have already submitted an order to the same token.
		// 			Self::try_delete_order(class_id, token_id, &who);
		// 			// `order_owner` transfers this NFT to `who`
		// 			Self::do_transfer(&order_owner, &who, class_id, token_id)?;
		// 			T::MultiCurrency::transfer(order.currency_id, &who, &order_owner, order.price)?;
		// 			// TODO: T::MultiCurrency::transfer(order.currency_id, &order_owner, some_account,platform-fee)?;
		// 			Self::deposit_event(Event::TakenOrder(class_id, token_id, order_owner));
		// 		},
		// 		(false, true) => {
		// 			ensure!(price <= order.price, Error::<T>::PriceTooLow);
		// 			// `who`/`token_owner` will accept the order submitted by `order_owner`
		// 			Self::delete_order(class_id, token_id, &order_owner)?;
		// 			Self::try_delete_order(class_id, token_id, &who);
		// 			// `order_owner` transfers this NFT to `who`
		// 			Self::do_transfer(&who, &order_owner, class_id, token_id)?;
		// 			T::MultiCurrency::transfer(order.currency_id, &order_owner, &who, order.price)?;
		// 			// TODO: T::MultiCurrency::transfer(order.currency_id, &who, some_account,platform-fee)?;
		// 			Self::deposit_event(Event::TakenOrder(class_id, token_id, order_owner));
		// 		},
		// 		_ => {
		// 			return Err(Error::<T>::NoPermission.into());
		// 		},
		// 	}
		// 	Ok(().into())
		// }

		// /// remove an order by order owner.
		// ///
		// /// - `class_id`: class id
		// /// - `token_id`: token id
		// #[pallet::weight(100_000)]
		// #[transactional]
		// pub fn remove_order(
		// 	origin: OriginFor<T>,
		// 	#[pallet::compact] class_id: ClassIdOf<T>,
		// 	#[pallet::compact] token_id: TokenIdOf<T>,
		// ) -> DispatchResultWithPostInfo {
		// 	let who = ensure_signed(origin)?;
		// 	Self::delete_order(class_id, token_id, &who)?;
		// 	Ok(().into())
		// }
	}
}

impl<T: Config> Pallet<T> {

	// fn delete_order(class_id: ClassIdOf<T>, token_id: TokenIdOf<T>, who: &T::AccountId) -> DispatchResult {
	// 	Orders::<T>::try_mutate_exists((class_id, token_id), who, |maybe_order| {
	// 		let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
	//
	// 		let mut deposit: Balance = Zero::zero();
	// 		if !order.by_token_owner {
	// 			// todo: emit an event for `order.currency_id`.
	// 			let d = T::MultiCurrency::unreserve(order.currency_id, &who, order.price.saturated_into());
	// 			deposit = deposit.saturating_add(order.price).saturating_sub(d);
	// 		}
	//
	// 		Categories::<T>::try_mutate(order.category_id, |category| -> DispatchResult {
	// 			category.as_mut().map(|cate| cate.nft_count = cate.nft_count.saturating_sub(One::one()) );
	// 			Ok(())
	// 		})?;
	//
	// 		let deposit = {
	// 			let d = <T as Config>::Currency::unreserve(&who, order.deposit.saturated_into());
	// 			deposit.saturating_add(order.deposit).saturating_sub(d.saturated_into())
	// 		};
	// 		Self::deposit_event(Event::RemovedOrder(class_id, token_id, who.clone(), deposit.saturated_into()));
	// 		*maybe_order = None;
	// 		Ok(())
	// 	})
	// }

	// fn try_delete_order(class_id: ClassIdOf<T>, token_id: TokenIdOf<T>, who: &T::AccountId) {
	// 	let _ = Self::delete_order(class_id, token_id, who);
	// }
}
