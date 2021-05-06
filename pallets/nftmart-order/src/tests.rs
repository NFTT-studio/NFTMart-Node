#![cfg(test)]

use super::{NATIVE_CURRENCY_ID};
use crate::mock::{add_class, ExtBuilder, ALICE, BOB,
				  Origin, add_token, all_tokens_by, add_category,
				  NftmartOrder, CLASS_ID0, TOKEN_ID1, TOKEN_ID0,
				  last_event, Event, current_gid};
use orml_nft::AccountToken;
use frame_support::{assert_ok};

#[test]
fn submit_order_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 10, None);
		add_token(BOB, 20, None);

		let cate_id = current_gid();
		add_category();

		assert_eq!(vec![
			(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 0 }),
			(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 0 })
		], all_tokens_by(BOB));

		let order_id = current_gid();
		assert_ok!(NftmartOrder::submit_order(Origin::signed(BOB),
			NATIVE_CURRENCY_ID,
			cate_id,
			1,
			2,
			vec![(CLASS_ID0, TOKEN_ID0, 10, 100), (CLASS_ID0, TOKEN_ID1, 20, 200)]
		));

		assert_eq!(
			last_event(),
			Event::nftmart_order(crate::Event::CreatedOrder(BOB, order_id)),
		);
	});
}
