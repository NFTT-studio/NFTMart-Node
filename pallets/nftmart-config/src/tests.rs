#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

#[test]
fn test_whitelist() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(None, NftmartConf::account_whitelist(ALICE));
		assert_eq!(None, NftmartConf::account_whitelist(BOB));
		assert_ok!(NftmartConf::add_whitelist(Origin::root(), ALICE));
		assert_eq!(last_event(), Event::nftmart_config(crate::Event::AddWhitelist(ALICE)));
		assert_eq!(Some(()), NftmartConf::account_whitelist(ALICE));
		assert_noop!(
			NftmartConf::add_whitelist(Origin::signed(BOB), BOB),
			DispatchError::BadOrigin,
		);

		assert_ok!(NftmartConf::remove_whitelist(Origin::root(), ALICE));
		assert_eq!(last_event(), Event::nftmart_config(crate::Event::RemoveWhitelist(ALICE)));
		assert_eq!(None, NftmartConf::account_whitelist(ALICE));
		assert_eq!(None, NftmartConf::account_whitelist(BOB));
	});
}

#[test]
fn update_category() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NftmartConf::update_category(Origin::signed(ALICE), 0, vec![0]),
			DispatchError::BadOrigin,
		);
	});
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftmartConf::create_category(Origin::root(), vec![22,2]));
		assert_eq!(Some(CategoryData{ metadata:  vec![22,2], count: 0 }), NftmartConf::categories(0));

		assert_ok!(NftmartConf::update_category(Origin::root(), 0, vec![2]));
		assert_eq!(Some(CategoryData{ metadata:  vec![2], count: 0 }), NftmartConf::categories(0));
	});
}

#[test]
fn create_category_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!({ let id_expect: GlobalId = Zero::zero(); id_expect }, NftmartConf::next_id());
		assert_eq!(None, NftmartConf::categories(0));

		assert_ok!(NftmartConf::create_category(Origin::root(), vec![233]));

		let event = Event::nftmart_config(crate::Event::CreatedCategory(0));
		assert_eq!(last_event(), event);

		assert_eq!({ let id_expect: GlobalId = One::one(); id_expect }, NftmartConf::next_id());
		assert_eq!(Some(CategoryData{ metadata: vec![233], count: 0 }), NftmartConf::categories(0));
		assert_eq!(None, NftmartConf::categories(100));
	});
}

#[test]
fn create_category_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NftmartConf::create_category(Origin::signed(ALICE), vec![1]),
			DispatchError::BadOrigin,
		);
	});
	ExtBuilder::default().build().execute_with(|| {
		NextId::<Runtime>::set(GlobalId::MAX);
		assert_noop!(
			NftmartConf::create_category(Origin::root(), vec![1]),
			Error::<Runtime>::NoAvailableId,
		);
	});
}
