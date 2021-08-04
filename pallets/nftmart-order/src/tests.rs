#![cfg(test)]

use super::NATIVE_CURRENCY_ID;
use crate::mock::{
	add_category, add_class, add_token, all_offers, all_orders, all_tokens_by, current_gid,
	ensure_account, free_balance, last_event, Event, ExtBuilder, Nftmart, NftmartOrder, Origin,
	ALICE, BOB, CHARLIE, CLASS_ID0, TOKEN_ID0, TOKEN_ID1,
};
use frame_support::assert_ok;
use orml_nft::AccountToken;
use sp_runtime::PerU16;

#[test]
fn submit_order_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(PerU16::zero()));
		assert_eq!(
			vec![
				(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 20, reserved: 0 }),
				(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 40, reserved: 0 })
			],
			all_tokens_by(BOB)
		);

		let cate_id = current_gid();
		add_category();

		let deposit = 10;
		let price = 100;
		let deadline = 2;

		let order_id = current_gid();
		assert_ok!(NftmartOrder::submit_order(
			Origin::signed(BOB),
			NATIVE_CURRENCY_ID,
			cate_id,
			deposit,
			price,
			deadline,
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)]
		));

		assert_eq!(last_event(), Event::NftmartOrder(crate::Event::CreatedOrder(BOB, order_id)),);

		// Some tokens should be reserved.
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID0, 10, 10);
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID1, 20, 20);
	});
}

#[test]
fn take_order_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		assert_eq!(198, free_balance(&ALICE));

		add_token(BOB, 20, Some(PerU16::from_percent(20)));
		add_token(BOB, 40, Some(PerU16::zero()));

		assert_ok!(Nftmart::update_token_royalty_beneficiary(
			Origin::signed(BOB),
			CLASS_ID0,
			TOKEN_ID0,
			CHARLIE
		));

		let cate_id = current_gid();
		add_category();

		let deposit = 10;
		let price = 100;
		let deadline = 2;

		let order_id = current_gid();
		assert_ok!(NftmartOrder::submit_order(
			Origin::signed(BOB),
			NATIVE_CURRENCY_ID,
			cate_id,
			deposit,
			price,
			deadline,
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)]
		));
		assert_eq!(1, all_orders().len());
		assert_ok!(NftmartOrder::take_order(Origin::signed(ALICE), order_id, BOB));
		assert_eq!(0, all_orders().len());

		assert_eq!(98, free_balance(&ALICE));
		assert_eq!(100 + 20, free_balance(&CHARLIE));
		assert_eq!(200 - 1 - 20, free_balance(&BOB));
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID0, 0, 10);
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID1, 0, 20);
		ensure_account(&ALICE, CLASS_ID0, TOKEN_ID0, 0, 10);
		ensure_account(&ALICE, CLASS_ID0, TOKEN_ID1, 0, 20);

		assert_eq!(
			last_event(),
			Event::NftmartOrder(crate::Event::TakenOrder(ALICE, BOB, order_id)),
		);
	});
}

#[test]
fn submit_offer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(PerU16::zero()));
		assert_eq!(100, free_balance(&CHARLIE));

		let cate_id = current_gid();
		add_category();

		let price = 100;
		let deadline = 2;

		let order_id = current_gid();
		assert_ok!(NftmartOrder::submit_offer(
			Origin::signed(CHARLIE),
			NATIVE_CURRENCY_ID,
			cate_id,
			price,
			deadline,
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)]
		));

		assert_eq!(
			last_event(),
			Event::NftmartOrder(crate::Event::CreatedOffer(CHARLIE, order_id)),
		);

		assert_eq!(0, free_balance(&CHARLIE));
	});
}

#[test]
fn take_offer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(PerU16::zero()));

		let cate_id = current_gid();
		add_category();

		let price = 100;
		let deadline = 2;

		let order_id = current_gid();
		assert_ok!(NftmartOrder::submit_offer(
			Origin::signed(CHARLIE),
			NATIVE_CURRENCY_ID,
			cate_id,
			price,
			deadline,
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)]
		));
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID0, 0, 20);
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID1, 0, 40);
		ensure_account(&CHARLIE, CLASS_ID0, TOKEN_ID0, 0, 0);
		ensure_account(&CHARLIE, CLASS_ID0, TOKEN_ID1, 0, 0);

		assert_eq!(1, all_offers().len());
		assert_ok!(NftmartOrder::take_offer(Origin::signed(BOB), order_id, CHARLIE));
		assert_eq!(0, all_offers().len());

		assert_eq!(
			last_event(),
			Event::NftmartOrder(crate::Event::TakenOffer(BOB, CHARLIE, order_id)),
		);

		assert_eq!(0, free_balance(&CHARLIE));
		assert_eq!(199, free_balance(&BOB));

		ensure_account(&BOB, CLASS_ID0, TOKEN_ID0, 0, 10);
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID1, 0, 20);
		ensure_account(&CHARLIE, CLASS_ID0, TOKEN_ID0, 0, 10);
		ensure_account(&CHARLIE, CLASS_ID0, TOKEN_ID1, 0, 20);
	});
}
