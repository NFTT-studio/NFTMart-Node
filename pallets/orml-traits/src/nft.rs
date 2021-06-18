use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Zero},
	DispatchResult, RuntimeDebug,
};
use sp_std::{fmt::Debug, vec::Vec};
use codec::{Decode, Encode, FullCodec};

/// Abstraction over a non-fungible token system.
#[allow(clippy::upper_case_acronyms)]
pub trait NFT<AccountId> {
	/// The NFT class identifier.
	type ClassId: Default + Copy;

	/// The NFT token identifier.
	type TokenId: Default + Copy;

	/// The balance of account.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	/// The number of NFTs assigned to `who`.
	fn balance(who: &AccountId) -> Self::Balance;

	/// The owner of the given token ID. Returns `None` if the token does not
	/// exist.
	fn owner(token: (Self::ClassId, Self::TokenId)) -> Option<AccountId>;

	/// Transfer the given token ID from one account to another.
	fn transfer(from: &AccountId, to: &AccountId, token: (Self::ClassId, Self::TokenId)) -> DispatchResult;
}

/// Class info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ClassInfo<TokenId, AccountId, Data> {
	/// Class metadata
	pub metadata: Vec<u8>,
	/// Total issuance for the class
	#[codec(compact)]
	pub total_issuance: TokenId,
	/// Class owner
	pub owner: AccountId,
	/// Class Properties
	pub data: Data,
}

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct TokenInfo<TokenId, Data> {
	/// Token metadata
	pub metadata: Vec<u8>,
	/// Token Properties
	pub data: Data,
	/// Token's number.
	#[codec(compact)]
	pub quantity: TokenId,
}

/// Account Token
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct AccountToken<TokenId> {
	/// account token number.
	#[codec(compact)]
	pub quantity: TokenId,
	/// account reserved token number.
	#[codec(compact)]
	pub reserved: TokenId,
}

impl<TokenId> Default for AccountToken<TokenId> where TokenId: AtLeast32BitUnsigned {
	fn default() -> Self {
		Self {
			quantity: Zero::zero(),
			reserved: Zero::zero(),
		}
	}
}

impl<TokenId> AccountToken<TokenId> where TokenId: AtLeast32BitUnsigned + Copy {
	pub fn is_zero(&self) -> bool {
		self.quantity.is_zero() && self.reserved.is_zero()
	}

	pub fn new(quantity: TokenId) -> Self {
		Self{
			quantity,
			..Self::default()
		}
	}

	pub fn total(&self) -> TokenId {
		self.quantity.saturating_add(self.reserved)
	}
}
