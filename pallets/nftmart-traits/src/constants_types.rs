use sp_runtime::{
	traits::{Verify, IdentifyAccount}, MultiSignature
};

/// Balance type.
pub type Balance = u128;

/// A unit balance
pub const ACCURACY: Balance = 1_000_000_000_000u128;

/// A type for ORML currency Id
pub type CurrencyId = u32;

/// Signed version of Balance
pub type Amount = i128;

/// Native currency
pub const NATIVE_CURRENCY_ID: CurrencyId = 0;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// NFT class ID type.
pub type ClassId = u32;

/// NFT token ID type.
pub type TokenId = u64;

/// For counting NFTs.
pub type Quantity = u64;

/// GlobalId ID type.
pub type GlobalId = u64;

/// Metadata for NFT.
pub type NFTMetadata = sp_std::prelude::Vec<u8>;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
