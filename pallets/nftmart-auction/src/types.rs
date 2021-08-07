use crate::*;

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
	/// If true, the real deadline will be max(deadline, last_bid_block + delay).
	pub allow_delay: bool,
	/// Category of this auction.
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
pub struct BritishAuctionBid<AccountId, BlockNumber> {
	/// last bid price
	#[codec(compact)]
	pub last_bid_price: Balance,
	/// the last account offering.
	pub last_bid_account: Option<AccountId>,
	/// last bid block number.
	#[codec(compact)]
	pub last_bid_block: BlockNumber,
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
	/// commission rate
	#[codec(compact)]
	pub commission_rate: PerU16,
}

pub type DutchAuctionBid<AccountId, BlockNumber> = BritishAuctionBid<AccountId, BlockNumber>;

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
pub enum Releases {
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
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BritishAuctionOf<T> =
	BritishAuction<CurrencyIdOf<T>, BlockNumberFor<T>, GlobalId, ClassIdOf<T>, TokenIdOf<T>>;
pub type BritishAuctionBidOf<T> = BritishAuctionBid<AccountIdOf<T>, BlockNumberFor<T>>;
pub type DutchAuctionOf<T> =
	DutchAuction<CurrencyIdOf<T>, BlockNumberFor<T>, GlobalId, ClassIdOf<T>, TokenIdOf<T>>;
pub type DutchAuctionBidOf<T> = DutchAuctionBid<AccountIdOf<T>, BlockNumberFor<T>>;
pub const DESC_INTERVAL: BlockNumber = time::MINUTES * 30;
