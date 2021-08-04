use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	MultiSignature,
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

/// Money matters.
pub mod currency {
	use super::*;

	pub const MILLICENTS: Balance = CENTS / 1000;
	pub const CENTS: Balance = DOLLARS / 100; // assume this is worth about a cent.
	pub const DOLLARS: Balance = ACCURACY;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}

/// Time.
pub mod time {
	use super::*;

	/// Since BABE is probabilistic this is the average expected block time that
	/// we are targeting. Blocks will be produced at a minimum duration defined
	/// by `SLOT_DURATION`, but some slots will not be allocated to any
	/// authority and hence no block will be produced. We expect to have this
	/// block time on average following the defined slot duration and the value
	/// of `c` configured for BABE (where `1 - c` represents the probability of
	/// a slot being empty).
	/// This value is only used indirectly to define the unit constants below
	/// that are expressed in blocks. The rest of the code should use
	/// `SLOT_DURATION` instead (like the Timestamp pallet for calculating the
	/// minimum period).
	///
	/// If using BABE with secondary slots (default) then all of the slots will
	/// always be assigned, in which case `MILLISECS_PER_BLOCK` and
	/// `SLOT_DURATION` should have the same value.
	///
	/// <https://research.web3.foundation/en/latest/polkadot/block-production/Babe.html#-6.-practical-results>
	pub const MILLISECS_PER_BLOCK: Moment = 6000;
	pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

	// NOTE: Currently it is not possible to change the slot duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 4 * HOURS;
	// pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
	pub const EPOCH_DURATION_IN_SLOTS: u64 = {
		const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

		(EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
	};

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
	// pub const MINUTES: BlockNumber = 1;
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
}

pub const TARGET__BLOCK__FULLNESS: u64 = 25;

/// Fee-related.
pub mod fee {
	use super::*;
	use frame_support::weights::{
		constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	};
	use smallvec::smallvec;
	pub use sp_runtime::Perbill;

	/// The block saturation level. Fees will be updates based on this value.
	// pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(TARGET__BLOCK__FULLNESS as u32);

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	///
	/// coeff_integer * x^(degree) + coeff_frac * x^(degree)
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// Extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
			let p = super::currency::CENTS;
			let q = 10 * Balance::from(ExtrinsicBaseWeight::get());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

/// These constants are specific to FRAME, and the current implementation of its various components.
/// For example: FRAME System, FRAME Executive, our FRAME support libraries, etc...
pub mod constants {
	pub use frame_support::weights::constants::{
		WEIGHT_PER_MICROS, WEIGHT_PER_MILLIS, WEIGHT_PER_NANOS, WEIGHT_PER_SECOND,
	};
	use frame_support::{
		parameter_types,
		weights::{RuntimeDbWeight, Weight},
	};

	parameter_types! {
		/// Importing a block with 0 txs takes ~5 ms
		pub const BlockExecutionWeight: Weight = 42 * WEIGHT_PER_MILLIS;
		/// Executing 10,000 System remarks (no-op) txs takes ~1.26 seconds -> ~125 µs per tx
		pub const ExtrinsicBaseWeight: Weight = 125 * WEIGHT_PER_MICROS;
		/// By default, Substrate uses RocksDB, so this will be the weight used throughout
		/// the runtime.
		pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
			read: 60 * WEIGHT_PER_MICROS,   // ~60 µs @ 200,000 items
			write: 242 * WEIGHT_PER_MICROS, // ~242 µs @ 200,000 items
		};
	}
}
