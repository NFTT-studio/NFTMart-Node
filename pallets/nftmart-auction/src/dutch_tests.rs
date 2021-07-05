#![cfg(test)]

use super::{NATIVE_CURRENCY_ID};
use crate::mock::*;
use sp_runtime::{PerU16};
use orml_nft::AccountToken;
use frame_support::{assert_ok};

#[test]
fn submit_dutch_auction_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(false));
		let cate_id = current_gid();
		add_category();
		let auction_id = current_gid();

		assert_ok!(NftmartAuction::submit_dutch_auction(
			Origin::signed(BOB),
			NATIVE_CURRENCY_ID,
			cate_id,
			50, // deposit
			200, // min_price
			500, // max_price
			10, // deadline
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
			false,
			PerU16::from_percent(50),
		));
		let event = Event::NftmartAuction(crate::Event::CreatedDutchAuction(BOB, auction_id));
		assert_eq!(last_event(), event);

		assert_eq!(vec![
			(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 10 }),
			(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 20 })
		], all_tokens_by(BOB));
	});
}
