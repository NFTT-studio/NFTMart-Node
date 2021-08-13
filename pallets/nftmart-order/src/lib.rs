#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
	transactional,
};
use frame_system::pallet_prelude::*;
pub use nftmart_traits::*;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, StaticLookup},
	PerU16, RuntimeDebug, SaturatedConversion,
};
use sp_std::vec::Vec;

mod mock;
mod tests;

pub use module::*;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Order<CurrencyId, BlockNumber, CategoryId, ClassId, TokenId> {
	/// currency ID.
	#[codec(compact)]
	pub currency_id: CurrencyId,
	/// The balances to create an order
	#[codec(compact)]
	pub deposit: Balance,
	/// Price of this token.
	#[codec(compact)]
	pub price: Balance,
	/// This order will be invalidated after `deadline` block number.
	#[codec(compact)]
	pub deadline: BlockNumber,
	/// Category of this order.
	#[codec(compact)]
	pub category_id: CategoryId,
	/// nft list
	pub items: Vec<OrderItem<ClassId, TokenId>>,
	/// commission rate
	#[codec(compact)]
	pub commission_rate: PerU16,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Offer<CurrencyId, BlockNumber, CategoryId, ClassId, TokenId> {
	/// currency ID.
	#[codec(compact)]
	pub currency_id: CurrencyId,
	/// Price of this token.
	#[codec(compact)]
	pub price: Balance,
	/// This order will be invalidated after `deadline` block number.
	#[codec(compact)]
	pub deadline: BlockNumber,
	/// Category of this order.
	#[codec(compact)]
	pub category_id: CategoryId,
	/// nft list
	pub items: Vec<OrderItem<ClassId, TokenId>>,
	/// commission rate
	#[codec(compact)]
	pub commission_rate: PerU16,
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
pub type BalanceOf<T> =
	<<T as module::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CurrencyIdOf<T> = <<T as module::Config>::MultiCurrency as MultiCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type OrderOf<T> =
	Order<CurrencyIdOf<T>, BlockNumberOf<T>, GlobalId, ClassIdOf<T>, TokenIdOf<T>>;
pub type OfferOf<T> =
	Offer<CurrencyIdOf<T>, BlockNumberOf<T>, GlobalId, ClassIdOf<T>, TokenIdOf<T>>;

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
		type ClassId: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ codec::FullCodec;

		/// The token ID type
		type TokenId: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ codec::FullCodec;

		/// NFTMart nft
		type NFT: NftmartNft<Self::AccountId, Self::ClassId, Self::TokenId>;

		/// Extra Configurations
		type ExtraConfig: NftmartConfig<Self::AccountId, BlockNumberFor<Self>>;

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type TreasuryPalletId: Get<frame_support::PalletId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// submit with invalid deposit
		SubmitWithInvalidDeposit,
		/// submit with invalid deadline
		SubmitWithInvalidDeadline,
		// Take Expired Order or Offer
		TakeExpiredOrderOrOffer,
		/// too many token charged royalty
		TooManyTokenChargedRoyalty,
		/// order not found
		OrderNotFound,
		OfferNotFound,
		/// cannot take one's own order
		TakeOwnOrder,
		TakeOwnOffer,
		InvalidCommissionRate,
		SenderTakeCommission,
	}

	#[pallet::event]
	#[pallet::metadata(
		T::AccountId = "AccountId",
		Option<(bool, T::AccountId, PerU16)> = "Option<(bool, AccountId, PerU16)>",
	)]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// CreatedOrder \[who, order_id\]
		CreatedOrder(T::AccountId, GlobalId),
		/// RemovedOrder \[who, order_id\]
		RemovedOrder(T::AccountId, GlobalId),
		RemovedOffer(T::AccountId, GlobalId),
		/// TakenOrder \[purchaser, order_owner, order_id\]
		TakenOrder(
			T::AccountId,
			T::AccountId,
			GlobalId,
			Option<(bool, T::AccountId, PerU16)>,
			Option<Vec<u8>>,
		),
		/// TakenOrder \[token_owner, offer_owner, order_id\]
		TakenOffer(
			T::AccountId,
			T::AccountId,
			GlobalId,
			Option<(bool, T::AccountId, PerU16)>,
			Option<Vec<u8>>,
		),
		/// CreatedOffer \[who, order_id\]
		CreatedOffer(T::AccountId, GlobalId),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			0
		}

		fn integrity_test() {}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		_phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { _phantom: Default::default() }
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
	pub type Orders<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, GlobalId, OrderOf<T>>;

	/// Index/store offers by account as primary key and order id as secondary key.
	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, GlobalId, OfferOf<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create an order.
		///
		/// - `currency_id`: currency id
		/// - `category_id`: category id
		/// - `deposit`: The balances to create an order
		/// - `price`: nfts' price.
		/// - `deadline`: deadline
		/// - `items`: a list of `(class_id, token_id, quantity, price)`
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn submit_order(
			origin: OriginFor<T>,
			#[pallet::compact] currency_id: CurrencyIdOf<T>,
			#[pallet::compact] category_id: GlobalId,
			#[pallet::compact] deposit: Balance,
			#[pallet::compact] price: Balance,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			items: Vec<(ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)>,
			#[pallet::compact] commission_rate: PerU16,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				commission_rate <= T::ExtraConfig::get_max_commission_reward_rate(),
				Error::<T>::InvalidCommissionRate
			);

			ensure!(
				deposit >= T::ExtraConfig::get_min_order_deposit(),
				Error::<T>::SubmitWithInvalidDeposit
			);
			<T as Config>::Currency::reserve(&who, deposit.saturated_into())?;

			ensure!(
				frame_system::Pallet::<T>::block_number() < deadline,
				Error::<T>::SubmitWithInvalidDeadline
			);
			let mut order = Order {
				currency_id,
				deposit,
				price,
				deadline,
				category_id,
				items: Vec::with_capacity(items.len()),
				commission_rate,
			};

			ensure_one_royalty!(items);
			reserve_and_push_tokens::<_, _, _, T::NFT>(Some(&who), &items, &mut order.items)?;

			T::ExtraConfig::inc_count_in_category(category_id)?;
			let order_id = T::ExtraConfig::get_then_inc_id()?;
			Orders::<T>::insert(&who, order_id, order);
			Self::deposit_event(Event::CreatedOrder(who, order_id));
			Ok(().into())
		}

		/// Take a NFT order.
		///
		/// - `order_id`: order id
		/// - `order_owner`: token owner
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn take_order(
			origin: OriginFor<T>,
			#[pallet::compact] order_id: GlobalId,
			order_owner: <T::Lookup as StaticLookup>::Source,
			commission_agent: Option<T::AccountId>,
			commission_data: Option<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			let purchaser = ensure_signed(origin)?;
			let order_owner = T::Lookup::lookup(order_owner)?;

			// Simplify the logic, to make life easier.
			ensure!(purchaser != order_owner, Error::<T>::TakeOwnOrder);

			if let Some(c) = &commission_agent {
				ensure!(&purchaser != c, Error::<T>::SenderTakeCommission);
			}

			let order: OrderOf<T> = Self::delete_order(&order_owner, order_id)?;

			// Check deadline of this order
			ensure!(
				frame_system::Pallet::<T>::block_number() < order.deadline,
				Error::<T>::TakeExpiredOrderOrOffer
			);

			let (items, commission_agent) = to_item_vec!(order, commission_agent);
			let (beneficiary, royalty_rate) = ensure_one_royalty!(items);
			swap_assets::<T::MultiCurrency, T::NFT, _, _, _, _>(
				&purchaser,
				&order_owner,
				order.currency_id,
				order.price,
				&items,
				&Self::treasury_account_id(),
				T::ExtraConfig::get_platform_fee_rate(),
				&beneficiary,
				royalty_rate,
				&commission_agent,
			)?;

			Self::deposit_event(Event::TakenOrder(
				purchaser,
				order_owner,
				order_id,
				commission_agent,
				commission_data,
			));
			Ok(().into())
		}

		/// remove an order by order owner.
		///
		/// - `order_id`: order id
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_order(
			origin: OriginFor<T>,
			#[pallet::compact] order_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::delete_order(&who, order_id)?;
			Self::deposit_event(Event::RemovedOrder(who, order_id));
			Ok(().into())
		}

		/// remove an offer by offer owner.
		///
		/// - `offer_id`: offer id
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_offer(
			origin: OriginFor<T>,
			#[pallet::compact] offer_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::delete_offer(&who, offer_id)?;
			Self::deposit_event(Event::RemovedOffer(who, offer_id));
			Ok(().into())
		}

		#[pallet::weight(100_000)]
		#[transactional]
		pub fn submit_offer(
			origin: OriginFor<T>,
			#[pallet::compact] currency_id: CurrencyIdOf<T>,
			#[pallet::compact] category_id: GlobalId,
			#[pallet::compact] price: Balance,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			items: Vec<(ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)>,
			#[pallet::compact] commission_rate: PerU16,
		) -> DispatchResultWithPostInfo {
			let purchaser = ensure_signed(origin)?;
			ensure!(
				frame_system::Pallet::<T>::block_number() < deadline,
				Error::<T>::SubmitWithInvalidDeadline
			);

			ensure!(
				commission_rate <= T::ExtraConfig::get_max_commission_reward_rate(),
				Error::<T>::InvalidCommissionRate
			);

			// Reserve balances of `currency_id` for tokenOwner to accept this offer.
			T::MultiCurrency::reserve(currency_id, &purchaser, price)?;

			let mut offer = Offer {
				currency_id,
				price,
				deadline,
				category_id,
				items: Vec::with_capacity(items.len()),
				commission_rate,
			};

			ensure_one_royalty!(items);
			reserve_and_push_tokens::<_, _, _, T::NFT>(None, &items, &mut offer.items)?;

			T::ExtraConfig::inc_count_in_category(category_id)?;
			let offer_id = T::ExtraConfig::get_then_inc_id()?;
			Offers::<T>::insert(&purchaser, offer_id, offer);
			Self::deposit_event(Event::CreatedOffer(purchaser, offer_id));
			Ok(().into())
		}

		/// Take a NFT offer.
		///
		/// - `offer_id`: offer id
		/// - `offer_owner`: token owner
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn take_offer(
			origin: OriginFor<T>,
			#[pallet::compact] offer_id: GlobalId,
			offer_owner: <T::Lookup as StaticLookup>::Source,
			commission_agent: Option<T::AccountId>,
			commission_data: Option<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			let token_owner = ensure_signed(origin)?;
			let offer_owner = T::Lookup::lookup(offer_owner)?;

			// Simplify the logic, to make life easier.
			ensure!(offer_owner != token_owner, Error::<T>::TakeOwnOffer);

			if let Some(c) = &commission_agent {
				ensure!(&token_owner != c, Error::<T>::SenderTakeCommission);
			}

			let offer: OfferOf<T> = Self::delete_offer(&offer_owner, offer_id)?;

			// Check deadline of this offer
			ensure!(
				frame_system::Pallet::<T>::block_number() < offer.deadline,
				Error::<T>::TakeExpiredOrderOrOffer
			);

			let (items, commission_agent) = to_item_vec!(offer, commission_agent);
			let (beneficiary, royalty_rate) = ensure_one_royalty!(items);
			swap_assets::<T::MultiCurrency, T::NFT, _, _, _, _>(
				&offer_owner,
				&token_owner,
				offer.currency_id,
				offer.price,
				&items,
				&Self::treasury_account_id(),
				T::ExtraConfig::get_platform_fee_rate(),
				&beneficiary,
				royalty_rate,
				&commission_agent,
			)?;

			Self::deposit_event(Event::TakenOffer(
				token_owner,
				offer_owner,
				offer_id,
				commission_agent,
				commission_data,
			));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn delete_order(who: &T::AccountId, order_id: GlobalId) -> Result<OrderOf<T>, DispatchError> {
		Orders::<T>::try_mutate_exists(who, order_id, |maybe_order| {
			let order: OrderOf<T> = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?.clone();

			// Can we safely ignore this remain value?
			let _remain: BalanceOf<T> =
				<T as Config>::Currency::unreserve(who, order.deposit.saturated_into());

			for item in &order.items {
				T::NFT::unreserve_tokens(who, item.class_id, item.token_id, item.quantity)?;
			}

			T::ExtraConfig::dec_count_in_category(order.category_id)?;
			*maybe_order = None;
			Ok(order)
		})
	}

	fn delete_offer(who: &T::AccountId, order_id: GlobalId) -> Result<OfferOf<T>, DispatchError> {
		Offers::<T>::try_mutate_exists(who, order_id, |maybe_offer| {
			let offer: OfferOf<T> = maybe_offer.as_mut().ok_or(Error::<T>::OfferNotFound)?.clone();

			// Can we safely ignore this remain value?
			let _remain: Balance = T::MultiCurrency::unreserve(offer.currency_id, who, offer.price);

			T::ExtraConfig::dec_count_in_category(offer.category_id)?;
			*maybe_offer = None;
			Ok(offer)
		})
	}

	pub fn treasury_account_id() -> T::AccountId {
		sp_runtime::traits::AccountIdConversion::<T::AccountId>::into_account(
			&T::TreasuryPalletId::get(),
		)
	}
}
