#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]

pub use enumflags2::BitFlags;
use frame_support::pallet_prelude::*;
pub use orml_traits::nft::{AccountToken, ClassInfo, TokenInfo};
use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::PerU16;
use sp_std::{vec, vec::Vec};

pub mod constants_types;
pub use crate::constants_types::*;
pub use contract_types::*;
pub use log;

pub type ResultPost<T> = sp_std::result::Result<
	T,
	sp_runtime::DispatchErrorWithPostInfo<frame_support::weights::PostDispatchInfo>,
>;

pub trait NftmartConfig<AccountId, BlockNumber> {
	fn auction_close_delay() -> BlockNumber;
	fn is_in_whitelist(_who: &AccountId) -> bool;
	fn get_min_order_deposit() -> Balance;
	fn get_then_inc_id() -> Result<GlobalId, DispatchError>;
	fn inc_count_in_category(category_id: GlobalId) -> DispatchResult;
	fn dec_count_in_category(category_id: GlobalId) -> DispatchResult;
	fn do_add_whitelist(who: &AccountId);
	fn do_create_category(metadata: NFTMetadata) -> DispatchResultWithPostInfo;
	fn peek_next_gid() -> GlobalId;
	fn get_royalties_rate() -> PerU16;
	fn get_platform_fee_rate() -> PerU16;
	fn get_max_commission_reward_rate() -> PerU16;
	fn get_min_commission_agent_deposit() -> Balance;
}

pub trait NftmartNft<AccountId, ClassId, TokenId> {
	fn peek_next_class_id() -> ClassId;
	fn transfer(
		from: &AccountId,
		to: &AccountId,
		class_id: ClassId,
		token_id: TokenId,
		quantity: TokenId,
	) -> DispatchResult;
	fn account_token(
		_who: &AccountId,
		_class_id: ClassId,
		_token_id: TokenId,
	) -> AccountToken<TokenId>;
	fn reserve_tokens(
		who: &AccountId,
		class_id: ClassId,
		token_id: TokenId,
		quantity: TokenId,
	) -> DispatchResult;
	fn unreserve_tokens(
		who: &AccountId,
		class_id: ClassId,
		token_id: TokenId,
		quantity: TokenId,
	) -> DispatchResult;
	fn token_charged_royalty(
		class_id: ClassId,
		token_id: TokenId,
	) -> Result<(AccountId, PerU16), DispatchError>;
	fn create_class(
		who: &AccountId,
		metadata: NFTMetadata,
		name: Vec<u8>,
		description: Vec<u8>,
		royalty_rate: PerU16,
		properties: Properties,
		category_ids: Vec<GlobalId>,
	) -> ResultPost<(AccountId, ClassId)>;
	fn proxy_mint(
		delegate: &AccountId,
		to: &AccountId,
		class_id: ClassId,
		metadata: NFTMetadata,
		quantity: TokenId,
		charge_royalty: Option<PerU16>,
	) -> ResultPost<(AccountId, AccountId, ClassId, TokenId, TokenId)>;
}

#[repr(u8)]
#[derive(Encode, Decode, Clone, Copy, BitFlags, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub enum ClassProperty {
	/// Token can be transferred
	Transferable = 0b00000001,
	/// Token can be burned
	Burnable = 0b00000010,
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
		Ok(Self(<BitFlags<ClassProperty>>::from_bits(field as u8).map_err(|_| "invalid value")?))
	}
}
impl TypeInfo for Properties {
	type Identity = Self;
	fn type_info() -> Type {
		Type::builder()
			.path(Path::new("BitFlags", module_path!()))
			.type_params(vec![TypeParameter::new("T", Some(meta_type::<ClassProperty>()))])
			.composite(Fields::unnamed().field(|f| f.ty::<u64>().type_name("ClassProperty")))
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
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
	#[codec(compact)]
	pub royalty_rate: PerU16,
	/// Category of this class.
	pub category_ids: Vec<GlobalId>,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenData<AccountId, BlockNumber> {
	/// The minimum balance to create token
	#[codec(compact)]
	pub deposit: Balance,
	#[codec(compact)]
	pub create_block: BlockNumber,
	/// Charge royalty
	#[codec(compact)]
	pub royalty_rate: PerU16,
	/// The token's creator
	pub creator: AccountId,
	/// Royalty beneficiary
	pub royalty_beneficiary: AccountId,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CategoryData {
	/// The category metadata.
	pub metadata: NFTMetadata,
	/// The number of orders/auctions in this category.
	#[codec(compact)]
	pub count: Balance,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
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
#[derive(
	Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, Serialize, Deserialize, Default, TypeInfo,
)]
pub struct ClassConfig<ClassId, AccountId, TokenId> {
	pub class_id: ClassId,
	pub class_metadata: String,
	pub category_ids: Vec<GlobalId>,
	pub name: String,
	pub description: String,
	pub royalty_rate: PerU16,
	pub properties: u8,
	pub admins: Vec<AccountId>,
	pub tokens: Vec<TokenConfig<AccountId, TokenId>>,
}

#[cfg(feature = "std")]
#[derive(
	Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, Serialize, Deserialize, Default, TypeInfo,
)]
pub struct TokenConfig<AccountId, TokenId> {
	pub token_id: TokenId,
	pub token_metadata: String,
	pub royalty_rate: PerU16,
	pub token_owner: AccountId,
	pub token_creator: AccountId,
	pub royalty_beneficiary: AccountId,
	pub quantity: TokenId,
}

/// Check only one royalty constrains.
pub fn count_charged_royalty<AccountId, ClassId, TokenId, NFT>(
	items: &[(ClassId, TokenId, TokenId)],
) -> ResultPost<(u32, AccountId, PerU16)>
where
	NFT: NftmartNft<AccountId, ClassId, TokenId>,
	ClassId: Copy,
	TokenId: Copy,
	AccountId: Default,
{
	let mut count_of_charged_royalty: u32 = 0;
	let mut royalty_rate = PerU16::zero();
	let mut who = AccountId::default();
	for (class_id, token_id, _quantity) in items {
		let (id, rate) = NFT::token_charged_royalty(*class_id, *token_id)?;
		if !rate.is_zero() {
			count_of_charged_royalty = count_of_charged_royalty.saturating_add(1u32);
			royalty_rate = rate;
			who = id;
		}
	}
	Ok((count_of_charged_royalty, who, royalty_rate))
}

/// Swap assets between nfts owner and nfts purchaser.
#[allow(clippy::too_many_arguments)]
pub fn swap_assets<MultiCurrency, NFT, AccountId, ClassId, TokenId, CurrencyId>(
	pay_currency: &AccountId,
	pay_nfts: &AccountId,
	currency_id: CurrencyId,
	price: Balance,
	items: &[(ClassId, TokenId, TokenId)],
	treasury: &AccountId,
	platform_fee_rate: PerU16,
	beneficiary: &AccountId,
	royalty_rate: PerU16,
	commission_agent: &Option<(bool, AccountId, PerU16)>,
) -> ResultPost<()>
where
	MultiCurrency:
		orml_traits::MultiCurrency<AccountId, CurrencyId = CurrencyId, Balance = Balance>,
	NFT: NftmartNft<AccountId, ClassId, TokenId>,
	ClassId: Copy,
	TokenId: Copy,
	CurrencyId: Copy,
{
	let trading_fee = platform_fee_rate.mul_ceil(price);
	let royalty_fee = royalty_rate.mul_ceil(price);
	MultiCurrency::transfer(currency_id, pay_currency, pay_nfts, price)?;
	MultiCurrency::transfer(currency_id, pay_nfts, treasury, trading_fee)?;
	MultiCurrency::transfer(currency_id, pay_nfts, beneficiary, royalty_fee)?;
	if let Some((status, agent, rate)) = commission_agent {
		if *status {
			let r = price.saturating_sub(trading_fee).saturating_sub(royalty_fee);
			MultiCurrency::transfer(currency_id, pay_nfts, agent, rate.mul_ceil(r))?;
		}
	}

	for (class_id, token_id, quantity) in items {
		NFT::transfer(pay_nfts, pay_currency, *class_id, *token_id, *quantity)?;
	}
	Ok(())
}

#[macro_export]
macro_rules! to_item_vec {
	($obj: ident, $commission_agent: ident) => {{
		let items = $obj.items.iter().map(|x| (x.class_id, x.token_id, x.quantity)).collect::<Vec<(
			ClassIdOf<T>,
			TokenIdOf<T>,
			TokenIdOf<T>,
		)>>();

		let commission_agent: Option<(bool, T::AccountId, PerU16)> =
			$commission_agent.and_then(|ca| {
				let b: Balance = <T as Config>::Currency::total_balance(&ca).saturated_into();
				if b < T::ExtraConfig::get_min_commission_agent_deposit() ||
					$obj.commission_rate.is_zero()
				{
					Some((false, ca, $obj.commission_rate))
				} else {
					Some((true, ca, $obj.commission_rate))
				}
			});

		(items, commission_agent)
	}};
}

#[macro_export]
macro_rules! ensure_one_royalty {
	($items: ident) => {{
		let (c, id, r) =
			count_charged_royalty::<T::AccountId, ClassIdOf<T>, TokenIdOf<T>, T::NFT>(&$items)?;
		ensure!(c <= 1, Error::<T>::TooManyTokenChargedRoyalty);
		(id, r)
	}};
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
) -> ResultPost<()>
where
	NFT: NftmartNft<AccountId, ClassId, TokenId>,
	ClassId: Copy,
	TokenId: Copy,
{
	for &(class_id, token_id, quantity) in items {
		if let Some(nft_owner) = nft_owner {
			NFT::reserve_tokens(nft_owner, class_id, token_id, quantity)?;
		}
		push_to.push(OrderItem { class_id, token_id, quantity })
	}
	Ok(())
}
