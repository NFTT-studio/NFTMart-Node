#![cfg(test)]

use super::{NATIVE_CURRENCY_ID};
use crate::mock::*;
use sp_runtime::{PerU16, SaturatedConversion};
use orml_nft::AccountToken;
use nftmart_traits::*;
use frame_support::{assert_ok};
use paste::paste;

macro_rules! submit_british_auction_should_work {
    ( $(#[$attr: meta])* $test_name: ident, $hammer_price: expr) => {
		paste! {
			#[test]
			$(#[$attr])*
			fn [<submit_british_auction_should_work $test_name>] () {
				ExtBuilder::default().build().execute_with(|| {
					add_class(ALICE);
					add_token(BOB, 20, None);
					add_token(BOB, 40, Some(false));
					assert_eq!(vec![
						(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 20, reserved: 0 }),
						(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 40, reserved: 0 })
					], all_tokens_by(BOB));

					let cate_id = current_gid();
					add_category();

					let bob_free = 100;
					assert_eq!(free_balance(&BOB), bob_free);

					let deposit = 50;

					let auction_id = current_gid();
					assert_ok!(NftmartAuction::submit_british_auction(
						Origin::signed(BOB),
						NATIVE_CURRENCY_ID,
						$hammer_price, // hammer_price
						PerU16::from_percent(50), // min_raise
						deposit, // deposit
						200, // init_price
						10, // deadline
						true, // allow_delay
						cate_id, // category_id
						vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
					));
					let event = Event::NftmartAuction(crate::Event::CreatedBritishAuction(BOB, auction_id));
					assert_eq!(last_event(), event);

					assert_eq!(vec![
						(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 10 }),
						(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 20 })
					], all_tokens_by(BOB));
					assert_eq!(free_balance(&BOB), bob_free - deposit);
					assert_eq!(1, categories(cate_id).count);
					assert!(get_bid(auction_id).is_some());
					assert!(get_auction(&BOB, auction_id).is_some());
				});
			}
		}
	};
}

submit_british_auction_should_work!(_1, 500);
submit_british_auction_should_work!(_2, 0);
submit_british_auction_should_work!(_3, 201);

#[test]
fn bid_british_auction_should_work_hammer_price() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(false));
		let cate_id = current_gid();
		add_category();
		let auction_id = current_gid();

		let bob_free = free_balance(&BOB);
		let hammer = 500;
		assert_ok!(NftmartAuction::submit_british_auction(
			Origin::signed(BOB),
			NATIVE_CURRENCY_ID,
			hammer, // hammer_price
			PerU16::from_percent(50), // min_raise
			50, // deposit
			200, // init_price
			10, // deadline
			true, // allow_delay
			cate_id, // category_id
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
		));

		let price = 600;
		assert_ok!(NftmartAuction::bid_british_auction(Origin::signed(CHARLIE), price, BOB, auction_id));
		let event = Event::NftmartAuction(crate::Event::HammerBritishAuction(CHARLIE, auction_id));
		assert_eq!(last_event(), event);

		assert_eq!(vec![
			(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 0 }),
			(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 0 })
		], all_tokens_by(CHARLIE));
		assert_eq!(free_balance(&CHARLIE), CHARLIE_INIT - hammer);
		assert_eq!(free_balance(&BOB), bob_free + hammer);
	});
}

macro_rules! bid_british_auction_should_work {
    ( $(#[$attr: meta])* $test_name: ident, hammer_price $hammer_price: expr, price $price: expr) => {
		paste! {
			#[test]
			$(#[$attr])*
			fn [<bid_british_auction_should_work $test_name>] () {
				ExtBuilder::default().build().execute_with(|| {
					add_class(ALICE);
					add_token(BOB, 20, None);
					add_token(BOB, 40, Some(false));
					let cate_id = current_gid();
					add_category();
					let auction_id = current_gid();

					let raise = PerU16::from_percent(10);
					assert_ok!(NftmartAuction::submit_british_auction(
						Origin::signed(BOB),
						NATIVE_CURRENCY_ID,
						$hammer_price, // hammer_price
						raise, // min_raise
						50, // deposit
						200, // init_price
						10, // deadline
						true, // allow_delay
						cate_id, // category_id
						vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
					));

					let price = $price;

					// CHARLIE bid
					assert_eq!(free_balance(&CHARLIE), CHARLIE_INIT);
					assert_ok!(NftmartAuction::bid_british_auction(Origin::signed(CHARLIE), price, BOB, auction_id));
					let event = Event::NftmartAuction(crate::Event::BidBritishAuction(CHARLIE, auction_id));
					assert_eq!(last_event(), event);
					assert_eq!(reserved_balance(&CHARLIE), price);
					assert_eq!(free_balance(&CHARLIE), CHARLIE_INIT - price);

					// DAVE bid
					let price = price + raise.mul_ceil(price) + 1;
					assert_eq!(free_balance(&DAVE), DAVE_INIT);
					assert_ok!(NftmartAuction::bid_british_auction(Origin::signed(DAVE), price, BOB, auction_id));
					let event = Event::NftmartAuction(crate::Event::BidBritishAuction(DAVE, auction_id));
					assert_eq!(last_event(), event);
					assert_eq!(reserved_balance(&DAVE), price);
					assert_eq!(free_balance(&DAVE), DAVE_INIT - price);

					// CHARLIE bid again
					let price = price + raise.mul_ceil(price) + 1;
					assert_eq!(free_balance(&CHARLIE), CHARLIE_INIT);
					assert_ok!(NftmartAuction::bid_british_auction(Origin::signed(CHARLIE), price, BOB, auction_id));
					let event = Event::NftmartAuction(crate::Event::BidBritishAuction(CHARLIE, auction_id));
					assert_eq!(last_event(), event);
					assert_eq!(reserved_balance(&CHARLIE), price);
					assert_eq!(free_balance(&CHARLIE), CHARLIE_INIT - price);
					assert_eq!(free_balance(&DAVE), DAVE_INIT);
				});
			}
		}
	};
}

bid_british_auction_should_work!(_1, hammer_price 5000, price 200);
bid_british_auction_should_work!(_2, hammer_price 3000, price 200);
bid_british_auction_should_work!(_3, hammer_price 0, price 200);
bid_british_auction_should_work!(_4, hammer_price 0, price 600);

macro_rules! redeem_british_auction_should_work {
( $(#[$attr: meta])* $test_name: ident, allow_delay $allow_delay: expr, set_block $set_block: expr) => {
		paste! {
			#[test]
			$(#[$attr])*
			fn [<redeem_british_auction_should_work $test_name>] () {
				ExtBuilder::default().build().execute_with(|| {
					add_class(ALICE);
					add_token(BOB, 20, None);
					add_token(BOB, 40, Some(false));
					let cate_id = current_gid();
					add_category();
					let auction_id = current_gid();
					assert_ok!(NftmartAuction::submit_british_auction(
						Origin::signed(BOB),
						NATIVE_CURRENCY_ID,
						500, // hammer_price
						PerU16::from_percent(50), // min_raise
						50, // deposit
						200, // init_price
						10, // deadline
						$allow_delay, // allow_delay
						cate_id, // category_id
						vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
					));
					let price = 300;
					assert_ok!(NftmartAuction::bid_british_auction(Origin::signed(CHARLIE), price, BOB, auction_id));
					System::set_block_number($set_block);
					assert_ok!(NftmartAuction::redeem_british_auction(Origin::signed(DAVE), BOB, auction_id));
					let event = Event::NftmartAuction(crate::Event::RedeemedBritishAuction(CHARLIE, auction_id));
					assert_eq!(last_event(), event);
					assert!(get_bid(auction_id).is_none());
					assert!(get_auction(&BOB, auction_id).is_none());

					assert_eq!(free_balance(&DAVE), DAVE_INIT);
					assert_eq!(free_balance(&BOB), BOB_INIT + price);
					assert_eq!(vec![
						(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 0 }),
						(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 0 })
					], all_tokens_by(BOB));
					assert_eq!(free_balance(&CHARLIE), CHARLIE_INIT - price);
					assert_eq!(vec![
						(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 0 }),
						(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 0 })
					], all_tokens_by(CHARLIE));
				});
			}
		}
	};
}

redeem_british_auction_should_work!(_1, allow_delay true, set_block 12);
redeem_british_auction_should_work!(_2, allow_delay true, set_block 20);
redeem_british_auction_should_work!(_3, allow_delay false, set_block 11);
redeem_british_auction_should_work!(_4, allow_delay false, set_block 20);


#[test]
fn remove_british_auction_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(false));
		let cate_id = current_gid();
		add_category();
		let auction_id = current_gid();

		let hammer = 500;
		assert_ok!(NftmartAuction::submit_british_auction(
			Origin::signed(BOB),
			NATIVE_CURRENCY_ID,
			hammer, // hammer_price
			PerU16::from_percent(50), // min_raise
			50, // deposit
			200, // init_price
			10, // deadline
			true, // allow_delay
			cate_id, // category_id
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
		));
		assert_ok!(NftmartAuction::remove_british_auction(Origin::signed(BOB), auction_id));
		let event = Event::NftmartAuction(crate::Event::RemovedBritishAuction(BOB, auction_id));
		assert_eq!(last_event(), event);
		assert!(get_bid(auction_id).is_none());
		assert!(get_auction(&BOB, auction_id).is_none());
	});
}

#[test]
fn calc_current_price_should_work() {
	for (x, y) in vec![
		(0, 10000), (1, 10000), (29, 10000),
		(30, 7525), (31, 7525), (59, 7525),
		(60, 5050), (61, 5050), (89, 5050),
		(90, 2575), (91, 2575), (119, 2575),
		(120, 100), (121, 100), (149, 100),
		(150, 100), (151, 100),  (1511, 100), (15111, 100),
	] {
		assert_eq!(
			crate::calc_current_price::<Runtime>(
				10000 * ACCURACY, 100 * ACCURACY, 0,
				(time::MINUTES * 120).saturated_into(),
				(time::MINUTES * x).saturated_into()
			),
			y * ACCURACY,
			"x={}, y={}", x, y,
		);
	}
	for (x, y) in vec![
		(0, 10000), (1, 10000), (2, 100), (29*time::MINUTES, 100),
		(29*time::MINUTES + 9, 100), (29*time::MINUTES + 10, 100), (30*time::MINUTES, 100),
	] {
		assert_eq!(
			crate::calc_current_price::<Runtime>(
				10000 * ACCURACY, 100 * ACCURACY, 0, 1,
				x.saturated_into(),
			),
			y * ACCURACY,
			"x={}, y={}", x, y,
		);
	}
	for (x, y) in vec![
		(0, 101), (1, 101), (2, 100), (29*time::MINUTES, 100),
		(29*time::MINUTES + 9, 100), (29*time::MINUTES + 10, 100), (30*time::MINUTES, 100),
	] {
		assert_eq!(
			crate::calc_current_price::<Runtime>(
				101, 100, 0, 1,
				x.saturated_into(),
			),
			y,
			"x={}, y={}", x, y,
		);
	}
}
