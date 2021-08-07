#![cfg(test)]

use super::NATIVE_CURRENCY_ID;
use crate::{mock::*, utils::test_helper::*, DutchAuctionBidOf, Error};
use frame_support::{assert_noop, assert_ok, assert_storage_noop};
use nftmart_traits::{time::*, *};
use orml_nft::AccountToken;
use sp_runtime::PerU16;

fn create_auction(allow_british: bool, max_price: Balance) -> GlobalId {
	add_class::<Runtime>(ALICE);
	add_token::<Runtime>(ALICE, BOB, CLASS_ID0, 20, None);
	add_token::<Runtime>(ALICE, BOB, CLASS_ID0, 40, Some(PerU16::zero()));

	let cate_id = current_gid::<Runtime>();
	add_category::<Runtime>();

	let auction_id = current_gid::<Runtime>();
	assert_ok!(NftmartAuction::submit_dutch_auction(
		Origin::signed(BOB),
		NATIVE_CURRENCY_ID,
		cate_id,
		50,                         // deposit
		200,                        // min_price
		max_price,                  // max_price
		(MINUTES as u64) * 120 + 1, // deadline
		vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
		allow_british,
		PerU16::from_percent(50),
		PerU16::zero(),
	));
	let event = Event::NftmartAuction(crate::Event::CreatedDutchAuction(BOB, auction_id));
	assert_eq!(last_event::<Runtime>(), event);

	auction_id
}

#[test]
fn submit_dutch_auction_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_auction(false, 500);

		assert_eq!(
			vec![
				(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 10 }),
				(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 20 })
			],
			all_tokens_by(BOB)
		);
	});
}

#[test]
fn submit_dutch_auction_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		add_class::<Runtime>(ALICE);
		add_token::<Runtime>(ALICE, BOB, CLASS_ID0, 20, Some(PerU16::from_percent(5)));
		add_token::<Runtime>(ALICE, BOB, CLASS_ID0, 40, Some(PerU16::from_percent(6)));

		assert_storage_noop!(current_gid::<Runtime>());

		let cate_id = current_gid::<Runtime>();
		add_category::<Runtime>();

		assert_noop!(
			NftmartAuction::submit_dutch_auction(
				Origin::signed(BOB),
				NATIVE_CURRENCY_ID,
				cate_id,
				50,  // deposit
				200, // min_price
				500, // max_price
				10,  // deadline
				vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
				false,
				PerU16::from_percent(50),
				PerU16::zero(),
			),
			Error::<Runtime>::TooManyTokenChargedRoyalty
		);
	});
}

#[test]
fn bid_dutch_auction_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let auction_id = create_auction(false, 500);
		assert_ok!(NftmartAuction::bid_dutch_auction(Origin::signed(CHARLIE), 0, BOB, auction_id, None, None));
		let event = Event::NftmartAuction(crate::Event::RedeemedDutchAuction(CHARLIE, auction_id, None, None));
		assert_eq!(last_event::<Runtime>(), event);

		assert_eq!(
			vec![
				(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 0 }),
				(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 0 }),
			],
			all_tokens_by(BOB)
		);

		assert_eq!(free_balance(&BOB), BOB_INIT + 500 - 1);

		assert_eq!(
			vec![
				(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 0 }),
				(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 0 }),
			],
			all_tokens_by(CHARLIE)
		);

		assert_eq!(free_balance(&CHARLIE), CHARLIE_INIT - 500);

		assert_eq!(NftmartAuction::dutch_auctions(BOB, auction_id), None);
		assert_eq!(NftmartAuction::dutch_auction_bids(auction_id), None);
	});
	ExtBuilder::default().build().execute_with(|| {
		let max_price = 500;
		let bid_price = max_price + max_price / 2 + 1;

		let auction_id = create_auction(true, max_price);
		assert_ok!(NftmartAuction::bid_dutch_auction(Origin::signed(CHARLIE), 0, BOB, auction_id, None, None));
		let bid: DutchAuctionBidOf<Runtime> =
			NftmartAuction::dutch_auction_bids(auction_id).unwrap();
		assert_eq!(bid.last_bid_price, max_price);
		let event = Event::NftmartAuction(crate::Event::BidDutchAuction(CHARLIE, auction_id));
		assert_eq!(last_event::<Runtime>(), event);

		assert_noop!(
			NftmartAuction::bid_dutch_auction(Origin::signed(CHARLIE), max_price, BOB, auction_id, None, None),
			Error::<Runtime>::PriceTooLow,
		);

		assert_noop!(
			NftmartAuction::bid_dutch_auction(Origin::signed(CHARLIE), bid_price, BOB, auction_id, None, None),
			Error::<Runtime>::DuplicatedBid,
		);

		System::set_block_number(10 + 1);

		assert_eq!(reserved_balance(&CHARLIE), 500);
		assert_ok!(NftmartAuction::bid_dutch_auction(
			Origin::signed(DAVE),
			bid_price,
			BOB,
			auction_id, None, None
		));
		assert_eq!(reserved_balance(&CHARLIE), 0);
		assert_eq!(reserved_balance(&DAVE), bid_price);

		assert_noop!(
			NftmartAuction::redeem_dutch_auction(Origin::signed(CHARLIE), BOB, auction_id, None, None),
			Error::<Runtime>::CannotRedeemAuctionUntilDeadline,
		);

		System::set_block_number(10 + 1 + 10 + 1);
		assert_noop!(
			NftmartAuction::bid_dutch_auction(Origin::signed(CHARLIE), bid_price, BOB, auction_id, None, None),
			Error::<Runtime>::DutchAuctionClosed,
		);

		// DAVE redeem nfts by the help of ALICE
		assert_ok!(NftmartAuction::redeem_dutch_auction(Origin::signed(ALICE), BOB, auction_id, None, None));
		assert_eq!(reserved_balance(&DAVE), 0);
		assert_eq!(
			vec![
				(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 0 }),
				(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 0 }),
			],
			all_tokens_by(DAVE)
		);
		assert_eq!(free_balance(&BOB), BOB_INIT + bid_price - 1);
	});
}

#[test]
fn bid_dutch_auction_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NftmartAuction::bid_dutch_auction(Origin::signed(CHARLIE), 0, BOB, 120, None, None),
			Error::<Runtime>::DutchAuctionNotFound,
		);
	});
	ExtBuilder::default().build().execute_with(|| {
		let auction_id = create_auction(false, 500);
		System::set_block_number(10000);
		assert_noop!(
			NftmartAuction::bid_dutch_auction(Origin::signed(CHARLIE), 0, BOB, auction_id, None, None),
			Error::<Runtime>::DutchAuctionClosed,
		);
	});
}
