#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]

use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;
pub use crate::constants_types::*;
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
pub use enumflags2::BitFlags;
pub use orml_traits::nft::{TokenInfo, ClassInfo, AccountToken};

pub mod constants_types;

pub trait NftmartConfig<AccountId, BlockNumber> {
	fn auction_delay() -> BlockNumber;
	fn is_in_whitelist(_who: &AccountId) -> bool;
	fn get_min_order_deposit() -> Balance;
	fn get_then_inc_id() -> Result<GlobalId, DispatchError>;
	fn inc_count_in_category (category_id: GlobalId) -> DispatchResult;
	fn dec_count_in_category (category_id: GlobalId) -> DispatchResult;
}

pub trait NftmartNft<AccountId, ClassId, TokenId> {
	fn transfer(from: &AccountId, to: &AccountId, class_id: ClassId, token_id: TokenId, quantity: TokenId) -> DispatchResult;
	fn account_token(_who: &AccountId, _class_id: ClassId, _token_id: TokenId) -> AccountToken<TokenId>;
	fn reserve_tokens(who: &AccountId, class_id: ClassId, token_id: TokenId, quantity: TokenId) -> DispatchResult;
	fn unreserve_tokens(who: &AccountId, class_id: ClassId, token_id: TokenId, quantity: TokenId) -> DispatchResult;
	fn token_charged_royalty(class_id: ClassId, token_id: TokenId) -> Result<bool, DispatchError>;
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

#[derive(Debug, PartialEq, Encode)]
pub struct ContractTokenInfo<AccountId> {
	pub metadata: NFTMetadata,
	pub data: ContractTokenData<AccountId>,
	pub quantity: Quantity,
}

#[derive(Debug, PartialEq, Encode)]
pub struct ContractTokenData<AccountId> {
	pub deposit: Balance,
	pub create_block: BlockNumber,
	pub royalty: bool,
	pub creator: AccountId,
	pub royalty_beneficiary: AccountId,
}
