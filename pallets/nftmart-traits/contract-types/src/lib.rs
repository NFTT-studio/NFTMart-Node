#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct ContractTokenInfo<NFTMetadata, Quantity, Balance, BlockNumber, AccountId> {
	pub metadata: NFTMetadata,
	pub data: ContractTokenData<Balance, BlockNumber, AccountId>,
	pub quantity: Quantity,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct ContractTokenData<Balance, BlockNumber, AccountId> {
	pub deposit: Balance,
	pub create_block: BlockNumber,
	pub royalty_rate: u16,
	pub creator: AccountId,
	pub royalty_beneficiary: AccountId,
}
