//! Unit tests for the non-fungible-token module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn reserve() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 10));
		assert_ok!(NonFungibleTokenModule::reserve(&BOB, (CLASS_ID, TOKEN_ID), 10));
		assert_ok!(NonFungibleTokenModule::reserve(&BOB, (CLASS_ID, TOKEN_ID), 0));
		assert_ok!(NonFungibleTokenModule::unreserve(&BOB, (CLASS_ID, TOKEN_ID), 10));
		assert_noop!(
			NonFungibleTokenModule::reserve(&BOB, (CLASS_ID, TOKEN_ID), 11),
			Error::<Runtime>::NumOverflow
		);
		assert_noop!(
			NonFungibleTokenModule::unreserve(&BOB, (CLASS_ID, TOKEN_ID), 1),
			Error::<Runtime>::NumOverflow
		);
	});
}

#[test]
fn create_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_eq!(0, NonFungibleTokenModule::classes(CLASS_ID).unwrap().total_issuance);
	});
}

#[test]
fn create_class_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		NextClassId::<Runtime>::mutate(|id| *id = <Runtime as Config>::ClassId::max_value());
		assert_noop!(
			NonFungibleTokenModule::create_class(&ALICE, vec![1], ()),
			Error::<Runtime>::NoAvailableClassId
		);
	});
}

#[test]
fn mint_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let next_class_id = NonFungibleTokenModule::next_class_id();
		assert_eq!(next_class_id, CLASS_ID);
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_eq!(NonFungibleTokenModule::next_token_id(CLASS_ID), 0);

		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 10));
		assert_eq!(10, NonFungibleTokenModule::tokens(CLASS_ID, TOKEN_ID).unwrap().quantity);
		assert_eq!(
			10,
			NonFungibleTokenModule::tokens_by_owner(&BOB, (CLASS_ID, TOKEN_ID))
				.unwrap()
				.quantity
		);
		assert_eq!(10, NonFungibleTokenModule::classes(CLASS_ID).unwrap().total_issuance);

		assert_eq!(NonFungibleTokenModule::next_token_id(CLASS_ID), 1);

		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 1));
		assert_eq!(1, NonFungibleTokenModule::tokens(CLASS_ID, TOKEN_ID + 1).unwrap().quantity);
		assert_eq!(
			1,
			NonFungibleTokenModule::tokens_by_owner(&BOB, (CLASS_ID, TOKEN_ID + 1))
				.unwrap()
				.quantity
		);
		assert_eq!(11, NonFungibleTokenModule::classes(CLASS_ID).unwrap().total_issuance);

		assert_eq!(NonFungibleTokenModule::next_token_id(CLASS_ID), 2);

		let next_class_id = NonFungibleTokenModule::next_class_id();
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_eq!(NonFungibleTokenModule::next_token_id(next_class_id), 0);
		assert_ok!(NonFungibleTokenModule::mint(&BOB, next_class_id, vec![1], (), 1));
		assert_eq!(1, NonFungibleTokenModule::classes(next_class_id).unwrap().total_issuance);
		assert_eq!(NonFungibleTokenModule::next_token_id(next_class_id), 1);

		assert_eq!(NonFungibleTokenModule::next_token_id(CLASS_ID), 2);

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &BOB).is_some());
	});
}

#[test]
fn mint_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_noop!(
			NonFungibleTokenModule::mint(&BOB, CLASS_ID_NOT_EXIST, vec![1], (), 11),
			Error::<Runtime>::ClassNotFound
		);

		Classes::<Runtime>::mutate(CLASS_ID, |class_info| {
			class_info.as_mut().unwrap().total_issuance =
				<Runtime as Config>::TokenId::max_value() - 10;
		});
		assert_noop!(
			NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 11),
			Error::<Runtime>::NumOverflow
		);

		NextTokenId::<Runtime>::mutate(CLASS_ID, |id| {
			*id = <Runtime as Config>::TokenId::max_value()
		});
		assert_noop!(
			NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 1),
			Error::<Runtime>::NoAvailableTokenId
		);

		assert!(
			NonFungibleTokenModule::owners_by_token((CLASS_ID_NOT_EXIST, TOKEN_ID), &BOB).is_none()
		);
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &BOB).is_none());
	});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 10));
		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 1));

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 0), &BOB).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 1), &BOB).is_some());

		assert_eq!(
			Ok(false),
			NonFungibleTokenModule::transfer(
				&ALICE,
				&ALICE,
				(CLASS_ID_NOT_EXIST, TOKEN_ID_NOT_EXIST),
				10
			)
		);
		assert_eq!(
			Ok(false),
			NonFungibleTokenModule::transfer(
				&ALICE,
				&BOB,
				(CLASS_ID_NOT_EXIST, TOKEN_ID_NOT_EXIST),
				0
			)
		);
		assert_eq!(
			Ok(false),
			NonFungibleTokenModule::transfer(&BOB, &BOB, (CLASS_ID, TOKEN_ID), 11)
		);
		assert_eq!(
			Ok(false),
			NonFungibleTokenModule::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID), 0)
		);

		assert!(NonFungibleTokenModule::is_owner(&BOB, (CLASS_ID, TOKEN_ID)));
		assert!(!NonFungibleTokenModule::is_owner(&ALICE, (CLASS_ID, TOKEN_ID)));
		assert_eq!(
			10,
			NonFungibleTokenModule::tokens_by_owner(&BOB, (CLASS_ID, TOKEN_ID))
				.unwrap()
				.quantity
		);
		assert_eq!(None, NonFungibleTokenModule::tokens_by_owner(&ALICE, (CLASS_ID, TOKEN_ID)));
		assert_eq!(10, NonFungibleTokenModule::tokens(CLASS_ID, TOKEN_ID).unwrap().quantity);
		assert_eq!(11, NonFungibleTokenModule::classes(CLASS_ID).unwrap().total_issuance);

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 0), &BOB).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 1), &BOB).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 0), &ALICE).is_none());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 1), &ALICE).is_none());

		assert_eq!(
			Ok(true),
			NonFungibleTokenModule::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID), 5)
		);

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 0), &BOB).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 1), &BOB).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 0), &ALICE).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 1), &ALICE).is_none());

		assert!(NonFungibleTokenModule::is_owner(&BOB, (CLASS_ID, TOKEN_ID)));
		assert!(NonFungibleTokenModule::is_owner(&ALICE, (CLASS_ID, TOKEN_ID)));
		assert_eq!(
			5,
			NonFungibleTokenModule::tokens_by_owner(&BOB, (CLASS_ID, TOKEN_ID))
				.unwrap()
				.quantity
		);
		assert_eq!(
			5,
			NonFungibleTokenModule::tokens_by_owner(&ALICE, (CLASS_ID, TOKEN_ID))
				.unwrap()
				.quantity
		);
		assert_eq!(10, NonFungibleTokenModule::tokens(CLASS_ID, TOKEN_ID).unwrap().quantity);
		assert_eq!(11, NonFungibleTokenModule::classes(CLASS_ID).unwrap().total_issuance);

		assert_eq!(
			Ok(true),
			NonFungibleTokenModule::transfer(&ALICE, &BOB, (CLASS_ID, TOKEN_ID), 2)
		);

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 0), &BOB).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 1), &BOB).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 0), &ALICE).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 1), &ALICE).is_none());

		assert!(NonFungibleTokenModule::is_owner(&BOB, (CLASS_ID, TOKEN_ID)));
		assert!(NonFungibleTokenModule::is_owner(&ALICE, (CLASS_ID, TOKEN_ID)));
		assert_eq!(
			7,
			NonFungibleTokenModule::tokens_by_owner(&BOB, (CLASS_ID, TOKEN_ID))
				.unwrap()
				.quantity
		);
		assert_eq!(
			3,
			NonFungibleTokenModule::tokens_by_owner(&ALICE, (CLASS_ID, TOKEN_ID))
				.unwrap()
				.quantity
		);
		assert_eq!(10, NonFungibleTokenModule::tokens(CLASS_ID, TOKEN_ID).unwrap().quantity);
		assert_eq!(11, NonFungibleTokenModule::classes(CLASS_ID).unwrap().total_issuance);

		assert_eq!(
			Ok(true),
			NonFungibleTokenModule::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID), 7)
		);

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 0), &BOB).is_none());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 1), &BOB).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 0), &ALICE).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, 1), &ALICE).is_none());

		assert!(!NonFungibleTokenModule::is_owner(&BOB, (CLASS_ID, TOKEN_ID)));
		assert!(NonFungibleTokenModule::is_owner(&ALICE, (CLASS_ID, TOKEN_ID)));
		assert_eq!(None, NonFungibleTokenModule::tokens_by_owner(&BOB, (CLASS_ID, TOKEN_ID)));
		assert_eq!(
			10,
			NonFungibleTokenModule::tokens_by_owner(&ALICE, (CLASS_ID, TOKEN_ID))
				.unwrap()
				.quantity
		);
		assert_eq!(10, NonFungibleTokenModule::tokens(CLASS_ID, TOKEN_ID).unwrap().quantity);
		assert_eq!(11, NonFungibleTokenModule::classes(CLASS_ID).unwrap().total_issuance);
	});
}

#[test]
fn transfer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 1));
		assert_noop!(
			NonFungibleTokenModule::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID_NOT_EXIST), 1),
			Error::<Runtime>::NumOverflow
		);
		assert_noop!(
			NonFungibleTokenModule::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID), 2),
			Error::<Runtime>::NumOverflow
		);
		assert_noop!(
			NonFungibleTokenModule::transfer(&ALICE, &BOB, (CLASS_ID, TOKEN_ID), 1),
			Error::<Runtime>::NumOverflow
		);

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID_NOT_EXIST, TOKEN_ID), &ALICE)
			.is_none());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &ALICE).is_none());
		assert!(
			NonFungibleTokenModule::owners_by_token((CLASS_ID_NOT_EXIST, TOKEN_ID), &BOB).is_none()
		);
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &BOB).is_some());
	});
}

#[test]
fn burn_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 10));
		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 1));

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &BOB).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &ALICE).is_none());

		assert_eq!(Ok(None), NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID), 0));

		assert_ok!(NonFungibleTokenModule::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID), 7));
		assert_eq!(
			9,
			NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID), 1)
				.unwrap()
				.unwrap()
				.quantity
		);

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &BOB).is_some());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &ALICE).is_some());

		assert_eq!(
			Some(AccountToken::new(2)),
			NonFungibleTokenModule::tokens_by_owner(&BOB, (CLASS_ID, TOKEN_ID))
		);
		assert_eq!(
			Some(AccountToken::new(7)),
			NonFungibleTokenModule::tokens_by_owner(&ALICE, (CLASS_ID, TOKEN_ID))
		);
		assert_eq!(9, NonFungibleTokenModule::tokens(CLASS_ID, TOKEN_ID).unwrap().quantity);
		assert_eq!(10, NonFungibleTokenModule::classes(CLASS_ID).unwrap().total_issuance);

		assert_eq!(
			7,
			NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID), 2)
				.unwrap()
				.unwrap()
				.quantity
		);

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &BOB).is_none());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &ALICE).is_some());

		assert_eq!(None, NonFungibleTokenModule::tokens_by_owner(&BOB, (CLASS_ID, TOKEN_ID)));
		assert_eq!(
			Some(AccountToken::new(7)),
			NonFungibleTokenModule::tokens_by_owner(&ALICE, (CLASS_ID, TOKEN_ID))
		);
		assert_eq!(7, NonFungibleTokenModule::tokens(CLASS_ID, TOKEN_ID).unwrap().quantity);
		assert_eq!(8, NonFungibleTokenModule::classes(CLASS_ID).unwrap().total_issuance);

		assert_eq!(
			0,
			NonFungibleTokenModule::burn(&ALICE, (CLASS_ID, TOKEN_ID), 7)
				.unwrap()
				.unwrap()
				.quantity
		);

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &BOB).is_none());
		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &ALICE).is_none());

		assert_eq!(None, NonFungibleTokenModule::tokens_by_owner(&BOB, (CLASS_ID, TOKEN_ID)));
		assert_eq!(None, NonFungibleTokenModule::tokens_by_owner(&ALICE, (CLASS_ID, TOKEN_ID)));
		assert_eq!(None, NonFungibleTokenModule::tokens(CLASS_ID, TOKEN_ID));
		assert_eq!(1, NonFungibleTokenModule::classes(CLASS_ID).unwrap().total_issuance);
	});
}

#[test]
fn burn_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 10));

		assert_noop!(
			NonFungibleTokenModule::burn(&BOB, (CLASS_ID_NOT_EXIST, TOKEN_ID), 11),
			Error::<Runtime>::ClassNotFound
		);

		assert_noop!(
			NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID_NOT_EXIST), 11),
			Error::<Runtime>::NumOverflow
		);

		assert_noop!(
			NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID_NOT_EXIST), 10),
			Error::<Runtime>::TokenNotFound
		);

		assert_noop!(
			NonFungibleTokenModule::burn(&ALICE, (CLASS_ID, TOKEN_ID), 2),
			Error::<Runtime>::NumOverflow
		);

		assert_noop!(
			NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID), 11),
			Error::<Runtime>::NumOverflow
		);

		assert_ok!(NonFungibleTokenModule::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID), 1));

		assert_noop!(
			NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID), 10),
			Error::<Runtime>::NumOverflow
		);

		assert!(NonFungibleTokenModule::owners_by_token((CLASS_ID, TOKEN_ID), &BOB).is_some());
	});
}

#[test]
fn destroy_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 1));
		assert_ok!(NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID), 1));
		assert_ok!(NonFungibleTokenModule::destroy_class(&ALICE, CLASS_ID));
		assert_eq!(Classes::<Runtime>::contains_key(CLASS_ID), false);
		assert_eq!(NextTokenId::<Runtime>::contains_key(CLASS_ID), false);
	});
}

#[test]
fn destroy_class_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::create_class(&ALICE, vec![1], ()));
		assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], (), 1));
		assert_noop!(
			NonFungibleTokenModule::destroy_class(&ALICE, CLASS_ID_NOT_EXIST),
			Error::<Runtime>::ClassNotFound
		);

		assert_noop!(
			NonFungibleTokenModule::destroy_class(&BOB, CLASS_ID),
			Error::<Runtime>::NoPermission
		);

		assert_noop!(
			NonFungibleTokenModule::destroy_class(&ALICE, CLASS_ID),
			Error::<Runtime>::CannotDestroyClass
		);

		assert_ok!(NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID), 1));
		assert_ok!(NonFungibleTokenModule::destroy_class(&ALICE, CLASS_ID));
		assert_eq!(Classes::<Runtime>::contains_key(CLASS_ID), false);
	});
}
