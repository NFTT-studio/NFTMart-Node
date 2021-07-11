#![cfg_attr(not(feature = "std"), no_std)]

pub use nftmart_traits::*;

sp_api::decl_runtime_apis! {
	/// The helper API to calculate deposit.
	pub trait NFTMartApi {
		/// mint_token_deposit
		fn mint_token_deposit(metadata_len: u32) -> Balance;
		/// add_class_admin_deposit
		fn add_class_admin_deposit(admin_count: u32) -> Balance;
		/// create_class_deposit
		fn create_class_deposit(metadata_len: u32, name_len: u32, description_len: u32) -> (Balance, Balance);
		/// get the current price of a Dutch auction.
		fn get_dutch_auction_current_price(
			max_price: Balance, min_price: Balance,
			created_block: BlockNumber,
			deadline: BlockNumber,
			current_block: BlockNumber,
		) -> Balance;
		/// get the deadline of an auction.
		fn get_auction_deadline(
			allow_delay: bool, deadline: BlockNumber, last_bid_block: BlockNumber
		) -> BlockNumber;
	}
}
