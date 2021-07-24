#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]

use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
pub use enumflags2::BitFlags;
pub use orml_traits::nft::{TokenInfo, ClassInfo, AccountToken};

pub mod constants_types;
pub use crate::constants_types::*;
pub use contract_types::*;
pub use log;

pub type ResultPost<T> = sp_std::result::Result<T, sp_runtime::DispatchErrorWithPostInfo<frame_support::weights::PostDispatchInfo>>;

pub trait NftmartConfig<AccountId, BlockNumber> {
	fn auction_close_delay() -> BlockNumber;
	fn is_in_whitelist(_who: &AccountId) -> bool;
	fn get_min_order_deposit() -> Balance;
	fn get_then_inc_id() -> Result<GlobalId, DispatchError>;
	fn inc_count_in_category (category_id: GlobalId) -> DispatchResult;
	fn dec_count_in_category (category_id: GlobalId) -> DispatchResult;
	fn do_add_whitelist(who: &AccountId);
	fn do_create_category(metadata: NFTMetadata) -> DispatchResultWithPostInfo;
	fn peek_next_gid() -> GlobalId;
}

pub trait NftmartNft<AccountId, ClassId, TokenId> {
	fn peek_next_class_id() -> ClassId;
	fn transfer(from: &AccountId, to: &AccountId, class_id: ClassId, token_id: TokenId, quantity: TokenId) -> DispatchResult;
	fn account_token(_who: &AccountId, _class_id: ClassId, _token_id: TokenId) -> AccountToken<TokenId>;
	fn reserve_tokens(who: &AccountId, class_id: ClassId, token_id: TokenId, quantity: TokenId) -> DispatchResult;
	fn unreserve_tokens(who: &AccountId, class_id: ClassId, token_id: TokenId, quantity: TokenId) -> DispatchResult;
	fn token_charged_royalty(class_id: ClassId, token_id: TokenId) -> Result<bool, DispatchError>;
	fn create_class(who: &AccountId, metadata: NFTMetadata, name: Vec<u8>, description: Vec<u8>, properties: Properties) -> ResultPost<(AccountId, ClassId)>;
	fn proxy_mint(
		delegate: &AccountId, to: &AccountId, class_id: ClassId,
		metadata: NFTMetadata, quantity: TokenId, charge_royalty: Option<bool>,
	) -> ResultPost<(AccountId, AccountId, ClassId, TokenId, TokenId)>;
}

#[repr(u8)]
#[derive(Encode, Decode, Clone, Copy, BitFlags, RuntimeDebug, PartialEq, Eq)]
pub enum ClassProperty {
	/// Token can be transferred
	Transferable = 0b00000001,
	/// Token can be burned
	Burnable = 0b00000010,
	/// Need to charge royalties when orders are completed.
	RoyaltiesChargeable = 0b00000100,
}

#[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Properties(pub BitFlags<ClassProperty>);

impl Eq for Properties {}
impl Encode for Properties {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl Decode for Properties {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u8::decode(input)?;
		Ok(Self(
			<BitFlags<ClassProperty>>::from_bits(field as u8).map_err(|_| "invalid value")?,
		))
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassData<BlockNumber> {
	/// The minimum balance to create class
	#[codec(compact)]
	pub deposit: Balance,
	/// Property of all tokens in this class.
	pub properties: Properties,
	/// Name of class.
	pub name: Vec<u8>,
	/// Description of class.
	pub description: Vec<u8>,
	#[codec(compact)]
	pub create_block: BlockNumber,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenData<AccountId, BlockNumber> {
	/// The minimum balance to create token
	#[codec(compact)]
	pub deposit: Balance,
	#[codec(compact)]
	pub create_block: BlockNumber,
	/// Charge royalty
	pub royalty: bool,
	/// The token's creator
	pub creator: AccountId,
	/// Royalty beneficiary
	pub royalty_beneficiary: AccountId,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CategoryData {
	/// The category metadata.
	pub metadata: NFTMetadata,
	/// The number of orders/auctions in this category.
	#[codec(compact)]
	pub count: Balance,
}

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
}

#[cfg(feature = "std")]
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ClassConfig<ClassId, AccountId, TokenId> {
	pub class_id: ClassId,
	pub class_metadata: String,
	pub name: String,
	pub description: String,
	pub properties: u8,
	pub admins: Vec<AccountId>,
	pub tokens: Vec<TokenConfig<AccountId, TokenId>>,
}

#[cfg(feature = "std")]
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TokenConfig<AccountId, TokenId> {
	pub token_id: TokenId,
	pub token_metadata: String,
	pub royalty: bool,
	pub token_owner: AccountId,
	pub token_creator: AccountId,
	pub royalty_beneficiary: AccountId,
	pub quantity: TokenId,
}

/// Check only one royalty constrains.
pub fn count_charged_royalty<AccountId, ClassId, TokenId, NFT>(items: &[(ClassId, TokenId, TokenId)])
	-> ResultPost<u32> where
	NFT: NftmartNft<AccountId, ClassId, TokenId>,
	ClassId: Copy, TokenId: Copy,
{
	let mut count_of_charged_royalty: u32 = 0;
	for (class_id, token_id, _quantity) in items {
		if NFT::token_charged_royalty(*class_id, *token_id)? {
			count_of_charged_royalty = count_of_charged_royalty.saturating_add(1u32);
		}
	}
	Ok(count_of_charged_royalty)
}

/// Swap assets between nfts owner and nfts purchaser.
pub fn swap_assets<MultiCurrency, NFT, AccountId, ClassId, TokenId, CurrencyId>(
	pay_currency: &AccountId,
	pay_nfts: &AccountId,
	currency_id: CurrencyId,
	price: Balance,
	items: &[(ClassId, TokenId, TokenId)],
) -> ResultPost<()> where
	MultiCurrency: orml_traits::MultiCurrency<AccountId, CurrencyId = CurrencyId, Balance = Balance>,
	NFT: NftmartNft<AccountId, ClassId, TokenId>,
	ClassId: Copy, TokenId: Copy,
{
	MultiCurrency::transfer(currency_id, pay_currency, pay_nfts, price)?;
	for (class_id, token_id, quantity) in items {
		NFT::transfer(pay_nfts, pay_currency, *class_id, *token_id, *quantity)?;
	}
	Ok(())
}

#[macro_export]
macro_rules! to_item_vec {
	($obj: ident) => {{
		let items = $obj.items.iter().map(|x|(x.class_id, x.token_id, x.quantity))
			 .collect::<Vec<(ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)>>();
		items
	}}
}

#[macro_export]
macro_rules! ensure_one_royalty {
	($items: ident) => {
		ensure!(
			count_charged_royalty::<T::AccountId, ClassIdOf<T>, TokenIdOf<T>, T::NFT>(&$items)? <= 1,
			Error::<T>::TooManyTokenChargedRoyalty,
		);
	}
}

#[macro_export]
macro_rules! nft_dbg {
	($($msg: expr),+ $(,)?) => {
		#[cfg(test)]
		println!($($msg),+);
		#[cfg(not(test))]
		log::log!(target: "nftmart", log::Level::Debug, $($msg),+);
	};
}

#[macro_export]
macro_rules! nft_info {
	($($msg: expr),+ $(,)?) => {
		#[cfg(test)]
		println!($($msg),+);
		#[cfg(not(test))]
		log::log!(target: "nftmart", log::Level::Info, $($msg),+);
	};
}

#[macro_export]
macro_rules! nft_err {
	($($msg: expr),+ $(,)?) => {
		#[cfg(test)]
		println!($($msg),+);
		#[cfg(not(test))]
		log::log!(target: "nftmart", log::Level::Error, $($msg),+);
	};
}

pub fn reserve_and_push_tokens<AccountId, ClassId, TokenId, NFT>(
	nft_owner: Option<&AccountId>,
	items: &[(ClassId, TokenId, TokenId)],
	push_to: &mut Vec<OrderItem<ClassId, TokenId>>,
) -> ResultPost<()> where
	NFT: NftmartNft<AccountId, ClassId, TokenId>,
	ClassId: Copy, TokenId: Copy,
{
	for &(class_id, token_id, quantity) in items {
		if let Some(nft_owner) = nft_owner {
			NFT::reserve_tokens(nft_owner, class_id, token_id, quantity)?;
		}
		push_to.push(OrderItem{
			class_id,
			token_id,
			quantity,
		})
	}
	Ok(())
}

