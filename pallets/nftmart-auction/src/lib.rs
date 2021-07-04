#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
	transactional,
};
use sp_std::vec::Vec;
use frame_system::pallet_prelude::*;
use nftmart_traits::constants_types::{GlobalId, Balance, ACCURACY, NATIVE_CURRENCY_ID};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, StaticLookup, Zero, Saturating, CheckedDiv},
	RuntimeDebug, SaturatedConversion, PerU16, FixedU128, FixedPointNumber,
};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use nftmart_traits::*;

mod mock;
mod tests;

pub use module::*;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BritishAuction<CurrencyId, BlockNumber, CategoryId, ClassId, TokenId> {
	/// currency ID for this auction
	#[codec(compact)]
	pub currency_id: CurrencyId,
	/// If encountered this price, the auction should be finished.
	#[codec(compact)]
	pub hammer_price: Balance,
	/// The new price offered should meet `new_price>old_price*(1+min_raise)`
	/// if Some(min_raise), min_raise > 0.
	#[codec(compact)]
	pub min_raise: PerU16,
	/// The auction owner/creator should deposit some balances to create an auction.
	/// After this auction finishing or deleting, this balances
	/// will be returned to the auction owner.
	#[codec(compact)]
	pub deposit: Balance,
	/// The initialized price of `currency_id` for auction.
	#[codec(compact)]
	pub init_price: Balance,
	/// The auction should be forced to be ended if current block number higher than this value.
	#[codec(compact)]
	pub deadline: BlockNumber,
	/// If true, the real deadline will be max(deadline, last_offer_block + delay).
	pub allow_delay: bool,
	/// Category of this auction.
	#[codec(compact)]
	pub category_id: CategoryId,
	/// nft list
	pub items: Vec<OrderItem<ClassId, TokenId>>,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BritishAuctionBid<AccountId, BlockNumber> {
	/// the newest price offered by
	#[codec(compact)]
	pub last_offer_price: Balance,
	/// the last account offering.
	pub last_offer_account: Option<AccountId>,
	/// last offer block number.
	#[codec(compact)]
	pub last_offer_block: BlockNumber,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DutchAuction<CurrencyId, BlockNumber, CategoryId, ClassId, TokenId> {
	#[codec(compact)]
	pub currency_id: CurrencyId,
	#[codec(compact)]
	pub category_id: CategoryId,
	#[codec(compact)]
	pub deposit: Balance,
	#[codec(compact)]
	pub min_price: Balance,
	#[codec(compact)]
	pub max_price: Balance,
	#[codec(compact)]
	pub deadline: BlockNumber,
	#[codec(compact)]
	pub created_block: BlockNumber,
	pub items: Vec<OrderItem<ClassId, TokenId>>,
	pub allow_british_auction: bool,
	#[codec(compact)]
	pub min_raise: PerU16,
}

pub type DutchAuctionBid<AccountId, BlockNumber> = BritishAuctionBid<AccountId, BlockNumber>;

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
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BritishAuctionOf<T> = BritishAuction<CurrencyIdOf<T>,BlockNumberFor<T>,GlobalId,ClassIdOf<T>,TokenIdOf<T>>;
pub type BritishAuctionBidOf<T> = BritishAuctionBid<AccountIdOf<T>,BlockNumberFor<T>>;
pub type DutchAuctionOf<T> = DutchAuction<CurrencyIdOf<T>,BlockNumberFor<T>,GlobalId,ClassIdOf<T>,TokenIdOf<T>>;
pub type DutchAuctionBidOf<T> = DutchAuctionBid<AccountIdOf<T>,BlockNumberFor<T>>;

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
		type ExtraConfig: NftmartConfig<Self::AccountId, BlockNumberFor<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// submit with invalid deposit
		SubmitWithInvalidDeposit,
		SubmitWithInvalidDeadline,
		TooManyTokenChargedRoyalty,
		InvalidHammerPrice,
		BritishAuctionNotFound,
		DutchAuctionNotFound,
		BritishAuctionBidNotFound,
		DutchAuctionBidNotFound,
		BritishAuctionClosed,
		DutchAuctionClosed,
		PriceTooLow,
		CannotRemoveAuction,
		CannotRedeemAuction,
		CannotRedeemAuctionNoBid,
		CannotRedeemAuctionUntilDeadline,
		DuplicatedBid,
		MaxPriceShouldBeGreaterThanMinPrice,
		InvalidDutchMinPrice,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// CreatedBritishAuction \[who, auction_id\]
		CreatedBritishAuction(T::AccountId, GlobalId),
		CreatedDutchAuction(T::AccountId, GlobalId),
		/// RemovedBritishAuction \[who, auction_id\]
		RemovedBritishAuction(T::AccountId, GlobalId),
		RedeemedBritishAuction(T::AccountId, GlobalId),
		BidBritishAuction(T::AccountId, GlobalId),
		HammerBritishAuction(T::AccountId, GlobalId),
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

	/// BritishAuctions
	#[pallet::storage]
	#[pallet::getter(fn british_auctions)]
	pub type BritishAuctions<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, GlobalId, BritishAuctionOf<T>>;

	/// BritishAuctionBids
	#[pallet::storage]
	#[pallet::getter(fn british_auction_bids)]
	pub type BritishAuctionBids<T: Config> = StorageMap<_, Twox64Concat, GlobalId, BritishAuctionBidOf<T>>;

	/// DutchAuctions
	#[pallet::storage]
	#[pallet::getter(fn dutch_auctions)]
	pub type DutchAuctions<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, GlobalId, DutchAuctionOf<T>>;

	/// DutchAuctionBids
	#[pallet::storage]
	#[pallet::getter(fn dutch_auction_bids)]
	pub type DutchAuctionBids<T: Config> = StorageMap<_, Twox64Concat, GlobalId, DutchAuctionBidOf<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(100_000)]
		#[transactional]
		#[allow(clippy::too_many_arguments)]
		pub fn submit_dutch_auction(
			origin: OriginFor<T>,
			#[pallet::compact] currency_id: CurrencyIdOf<T>,
			#[pallet::compact] category_id: GlobalId,
			#[pallet::compact] deposit: Balance,
			#[pallet::compact] min_price: Balance,
			#[pallet::compact] max_price: Balance,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			items: Vec<(ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)>,
			allow_british_auction: bool,
			#[pallet::compact] min_raise: PerU16,
		) -> DispatchResultWithPostInfo {
			let who: T::AccountId = ensure_signed(origin)?;

			// check and reserve `deposit`
			ensure!(deposit >= T::ExtraConfig::get_min_order_deposit(), Error::<T>::SubmitWithInvalidDeposit);
			<T as Config>::Currency::reserve(&who, deposit.saturated_into())?;

			let created_block = frame_system::Pallet::<T>::block_number();
			// check deadline
			ensure!(created_block < deadline, Error::<T>::SubmitWithInvalidDeadline);

			// check min price and max price
			ensure!(0 < min_price , Error::<T>::InvalidDutchMinPrice);
			ensure!(min_price < max_price , Error::<T>::MaxPriceShouldBeGreaterThanMinPrice);

			let mut auction: DutchAuctionOf<T> = DutchAuction {
				currency_id,
				deposit,
				min_price,
				max_price,
				deadline,
				created_block,
				category_id,
				items: Vec::with_capacity(items.len()),
				allow_british_auction,
				min_raise,
			};

			ensure_one_royalty!(items);
			push_tokens::<_, _, _, T::NFT>(Some(&who), &items, &mut auction.items)?;

			// add the auction to a category
			T::ExtraConfig::inc_count_in_category(category_id)?;

			// generate an auction id
			let auction_id = T::ExtraConfig::get_then_inc_id()?;

			// save auction information.
			DutchAuctions::<T>::insert(&who, auction_id, auction);

			let auction_bid: DutchAuctionBidOf<T> = DutchAuctionBid {
				last_offer_price: min_price,
				last_offer_account: None,
				last_offer_block: Zero::zero(),
			};

			BritishAuctionBids::<T>::insert(auction_id, auction_bid);

			// emit event.
			Self::deposit_event(Event::CreatedDutchAuction(who, auction_id));
			Ok(().into())
		}

		// #[pallet::weight(100_000)]
		// #[transactional]
		// pub fn bid_dutch_auction(
		// 	origin: OriginFor<T>,
		// 	#[pallet::compact] price: Balance,
		// 	auction_owner: <T::Lookup as StaticLookup>::Source,
		// 	#[pallet::compact] auction_id: GlobalId,
		// ) -> DispatchResultWithPostInfo {
		// 	let purchaser: T::AccountId = ensure_signed(origin)?;
		// 	let auction_owner: T::AccountId = T::Lookup::lookup(auction_owner)?;
		// 	let auction: DutchAuctionOf<T> = Self::dutch_auctions(&auction_owner, auction_id).ok_or(Error::<T>::DutchAuctionNotFound)?;
		// 	let auction_bid: DutchAuctionBidOf<T> = Self::dutch_auction_bids(auction_id).ok_or(Error::<T>::DutchAuctionBidNotFound)?;
		//
		// 	match (auction_bid.last_offer_account, auction.allow_british_auction) {
		// 		(None, false) => {
		// 			// check deadline
		// 			let current_block: BlockNumberOf<T> = frame_system::Pallet::<T>::block_number();
		// 			ensure!(auction.deadline >= current_block, Error::<T>::DutchAuctionClosed);
		// 			let current_price: Balance = calc_current_price::<T>(&auction, current_block);
		//
		// 		},
		// 		(last_offer_account, true) => {
		// 			if let Some(last_offer_account) = last_offer_account {
		//
		// 			}
		// 		},
		// 		_ => {
		// 			return Err(Error::<T>::DutchAuctionClosed.into());
		// 		}
		// 	}
		// 	Ok(().into())
		// }

		/// Create an British auction.
		///
		/// - `currency_id`: Currency Id
		/// - `hammer_price`: If somebody offer this price, the auction will be finished. Set to zero to disable.
		/// - `min_raise`: The next price of bid should be larger than old_price * ( 1 + min_raise )
		/// - `deposit`: A higher deposit will be good for the display of the auction in the market.
		/// - `init_price`: The initial price for the auction to kick off.
		/// - `deadline`: A block number which represents the end of the auction activity.
		/// - `allow_delay`: If ture, in some cases the deadline will be extended.
		/// - `category_id`: Category Id
		/// - `items`: Nft list.
		#[pallet::weight(100_000)]
		#[transactional]
		#[allow(clippy::too_many_arguments)]
		pub fn submit_british_auction(
			origin: OriginFor<T>,
			#[pallet::compact] currency_id: CurrencyIdOf<T>,
			#[pallet::compact] hammer_price: Balance,
			#[pallet::compact] min_raise: PerU16,
			#[pallet::compact] deposit: Balance,
			#[pallet::compact] init_price: Balance,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			allow_delay: bool,
			#[pallet::compact] category_id: GlobalId,
			items: Vec<(ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// check and reserve `deposit`
			ensure!(deposit >= T::ExtraConfig::get_min_order_deposit(), Error::<T>::SubmitWithInvalidDeposit);
			<T as Config>::Currency::reserve(&who, deposit.saturated_into())?;

			// check deadline
			ensure!(frame_system::Pallet::<T>::block_number() < deadline, Error::<T>::SubmitWithInvalidDeadline);

			// check hammer price
			if hammer_price > Zero::zero() {
				ensure!(hammer_price > init_price, Error::<T>::InvalidHammerPrice);
			}

			let mut auction: BritishAuctionOf<T> = BritishAuction {
				currency_id,
				hammer_price,
				min_raise,
				deposit,
				init_price,
				deadline,
				allow_delay,
				category_id,
				items: Vec::with_capacity(items.len()),
			};

			let auction_bid: BritishAuctionBidOf<T> = BritishAuctionBid {
				last_offer_price: init_price,
				last_offer_account: None,
				last_offer_block: Zero::zero(),
			};

			ensure_one_royalty!(items);
			push_tokens::<_, _, _, T::NFT>(Some(&who), &items, &mut auction.items)?;

			// add the auction to a category
			T::ExtraConfig::inc_count_in_category(category_id)?;

			// generate an auction id
			let auction_id = T::ExtraConfig::get_then_inc_id()?;

			// save auction information.
			BritishAuctions::<T>::insert(&who, auction_id, auction);
			BritishAuctionBids::<T>::insert(auction_id, auction_bid);

			// emit event.
			Self::deposit_event(Event::CreatedBritishAuction(who, auction_id));
			Ok(().into())
		}

		/// Bid
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn bid_british_auction(
			origin: OriginFor<T>,
			#[pallet::compact] price: Balance,
			auction_owner: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] auction_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let purchaser = ensure_signed(origin)?;
			let auction_owner = T::Lookup::lookup(auction_owner)?;

			let auction: BritishAuctionOf<T> = Self::british_auctions(&auction_owner, auction_id).ok_or(Error::<T>::BritishAuctionNotFound)?;
			let auction_bid: BritishAuctionBidOf<T> = Self::british_auction_bids(auction_id).ok_or(Error::<T>::BritishAuctionBidNotFound)?;

			// check deadline
			ensure!(Self::get_deadline(&auction, &auction_bid) >= frame_system::Pallet::<T>::block_number(), Error::<T>::BritishAuctionClosed);

			// check hammer price
			if !auction.hammer_price.is_zero() && price >= auction.hammer_price {
				// delete the auction and release all assets reserved by this auction.
				Self::delete_british_auction(&auction_owner, auction_id)?;

				let items = to_item_vec!(auction);
				ensure_one_royalty!(items);
				swap_assets::<T::MultiCurrency, T::NFT, _, _, _, _>(
					&purchaser, &auction_owner, auction.currency_id, auction.hammer_price, &items,
				)?;

				Self::deposit_event(Event::HammerBritishAuction(purchaser, auction_id));
				Ok(().into())
			} else {
				if let Some(account) = &auction_bid.last_offer_account {
					// check the new bid price.
					let lowest_price: Balance = auction_bid.last_offer_price.saturating_add(auction.min_raise.mul_ceil(auction_bid.last_offer_price));
					ensure!(price > lowest_price, Error::<T>::PriceTooLow);

					ensure!(&purchaser != account, Error::<T>::DuplicatedBid);
					let _ = T::MultiCurrency::unreserve(auction.currency_id, account, auction_bid.last_offer_price);
				} else {
					// check the new bid price.
					ensure!(price >= auction.init_price, Error::<T>::PriceTooLow);
				}

				T::MultiCurrency::reserve(auction.currency_id, &purchaser, price)?;

				let mut auction_bid = auction_bid;
				auction_bid.last_offer_price = price;
				auction_bid.last_offer_account = Some(purchaser.clone());
				auction_bid.last_offer_block = frame_system::Pallet::<T>::block_number();

				BritishAuctionBids::<T>::insert(auction_id, auction_bid);
				Self::deposit_event(Event::BidBritishAuction(purchaser, auction_id));
				Ok(().into())
			}
		}

		/// redeem
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn redeem_british_auction(
			origin: OriginFor<T>,
			auction_owner: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] auction_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			let auction_owner = T::Lookup::lookup(auction_owner)?;
			let (auction,auction_bid) = Self::delete_british_auction(&auction_owner, auction_id)?;
			ensure!(Self::get_deadline(&auction, &auction_bid) < frame_system::Pallet::<T>::block_number(), Error::<T>::CannotRedeemAuctionUntilDeadline);
			ensure!(auction_bid.last_offer_account.is_some(), Error::<T>::CannotRedeemAuctionNoBid);
			let purchaser = auction_bid.last_offer_account.expect("Must be Some");

			let items = to_item_vec!(auction);
			ensure_one_royalty!(items);
			swap_assets::<T::MultiCurrency, T::NFT, _, _, _, _>(
				&purchaser, &auction_owner, auction.currency_id, auction_bid.last_offer_price, &items,
			)?;

			Self::deposit_event(Event::RedeemedBritishAuction(purchaser, auction_id));
			Ok(().into())
		}

		/// remove an auction by auction owner.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_british_auction(
			origin: OriginFor<T>,
			#[pallet::compact] auction_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let (_, bid) = Self::delete_british_auction(&who, auction_id)?;
			ensure!(bid.last_offer_account.is_none(), Error::<T>::CannotRemoveAuction);
			Self::deposit_event(Event::RemovedBritishAuction(who, auction_id));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn delete_british_auction(who: &T::AccountId, auction_id: GlobalId) -> Result<(BritishAuctionOf<T>, BritishAuctionBidOf<T>), DispatchError> {
		BritishAuctionBids::<T>::try_mutate_exists(auction_id, |maybe_british_auction_bid| {
			let auction_bid: BritishAuctionBidOf<T> = maybe_british_auction_bid.as_mut().ok_or(Error::<T>::BritishAuctionBidNotFound)?.clone();
			BritishAuctions::<T>::try_mutate_exists(who, auction_id, |maybe_british_auction| {
				let auction: BritishAuctionOf<T> = maybe_british_auction.as_mut().ok_or(Error::<T>::BritishAuctionNotFound)?.clone();

				if let Some(account) = &auction_bid.last_offer_account {
					let _ = T::MultiCurrency::unreserve(auction.currency_id, account, auction_bid.last_offer_price);
				}

				let _remain: BalanceOf<T> = <T as Config>::Currency::unreserve(&who, auction.deposit.saturated_into());

				for item in &auction.items {
					T::NFT::unreserve_tokens(who, item.class_id, item.token_id, item.quantity)?;
				}

				T::ExtraConfig::dec_count_in_category(auction.category_id)?;
				*maybe_british_auction_bid = None;
				*maybe_british_auction = None;
				Ok((auction, auction_bid))
			})
		})
	}

	fn get_deadline(auction: &BritishAuctionOf<T>, bid: &BritishAuctionBidOf<T>) -> BlockNumberFor<T> {
		if auction.allow_delay {
			let delay = bid.last_offer_block.saturating_add(T::ExtraConfig::auction_delay());
			core::cmp::max(auction.deadline,delay)
		} else {
			auction.deadline
		}
	}
}

fn calc_current_price<T: Config>(auction: &DutchAuctionOf<T>, current_block: BlockNumberOf<T>) -> Balance {
	let created_block: BlockNumber = auction.created_block.saturated_into();
	let aligned_block: BlockNumber = current_block
		.saturated_into::<BlockNumber>()
		.saturating_sub(created_block) // >= 0
		.checked_div(time::MINUTES * 30) // >= 0
		.map(|x| x.saturating_mul(time::MINUTES * 30)) // >= 0
		.map(|x| x.saturating_add(created_block)) // >= created_block
		.unwrap_or(created_block); // >= created_block

	let deadline: FixedU128 = (auction.deadline.saturated_into::<BlockNumber>(), 1).into();
	let created_block: FixedU128 = (created_block, 1).into();
	let current_block: FixedU128 = (aligned_block, 1).into();
	let max_price: FixedU128 = (auction.max_price, ACCURACY).into();
	let min_price: FixedU128 = (auction.min_price, ACCURACY).into();

	// calculate current price.
	let current_price: Balance = max_price
		.saturating_sub(min_price) // > 0
		.saturating_mul(current_block.saturating_sub(created_block)) // >= 0
		.checked_div(&deadline.saturating_sub(created_block)) // >= 0
		.map(|x| max_price.saturating_sub(x)) // >= min_price && <= max_price
		.unwrap_or(max_price) // >= min_price && <= max_price
		.saturating_mul_int(ACCURACY); // >= min_price && <= max_price

	current_price
}
