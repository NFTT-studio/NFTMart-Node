#![cfg(test)]

use super::*;
use crate::mock::{Event, *};
use frame_support::{
	assert_noop, assert_ok, generate_storage_alias, traits::PalletInfo, Twox64Concat,
};
use orml_nft::ClassInfoOf;

#[test]
fn migrate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		{
			use crate::migrations::{OldClass, OldClassData};
			generate_storage_alias!(OrmlNFT, Classes => Map<(ClassId, Twox64Concat), OldClass<Runtime>>);
			for i in 0..1000 {
				Classes::insert(
					i,
					OldClass::<Runtime> {
						metadata: b"class metadata".to_vec(),
						total_issuance: 44,
						owner: ALICE,
						data: OldClassData {
							deposit: 33,
							properties: Default::default(),
							name: b"class name".to_vec(),
							description: b"class description".to_vec(),
						},
					},
				);
			}
			assert_eq!(44, Classes::get(1).unwrap().total_issuance);
		}
		assert_eq!(None, OrmlNFT::classes(1));
		use std::time::Instant;
		let now = Instant::now();
		crate::migrations::do_migrate::<Runtime>();
		let elapsed = now.elapsed();
		println!("Elapsed: {:.2?}", elapsed);
		assert_eq!(44, OrmlNFT::classes(1).unwrap().total_issuance);
	});
}

#[test]
fn update_token_royalty() {
	// royalty
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			<Runtime as frame_system::Config>::PalletInfo::name::<Nftmart>(),
			Some("Nftmart")
		);
		add_category();
		ensure_bob_balances(ACCURACY * 4);
		add_class(ALICE);
		add_token(BOB, 1, None);
		add_token(BOB, 2, Some(PerU16::zero())); // erc1155
		assert_noop!(
			Nftmart::update_token_royalty(
				Origin::signed(ALICE),
				CLASS_ID,
				TOKEN_ID_NOT_EXIST,
				Some(PerU16::from_percent(5))
			),
			Error::<Runtime>::TokenIdNotFound,
		);
		assert_noop!(
			Nftmart::update_token_royalty(
				Origin::signed(ALICE),
				CLASS_ID,
				TOKEN_ID,
				Some(PerU16::from_percent(5))
			),
			Error::<Runtime>::NoPermission,
		);
		assert_eq!(
			orml_nft::Tokens::<Runtime>::get(CLASS_ID, TOKEN_ID).unwrap().data.royalty_rate,
			PerU16::from_percent(5)
		);

		assert_ok!(Nftmart::update_token_royalty(
			Origin::signed(BOB),
			CLASS_ID,
			TOKEN_ID,
			Some(PerU16::from_percent(5))
		));
		assert_eq!(
			orml_nft::Tokens::<Runtime>::get(CLASS_ID, TOKEN_ID).unwrap().data.royalty_rate,
			PerU16::from_percent(5)
		);

		assert_ok!(Nftmart::update_token_royalty(
			Origin::signed(BOB),
			CLASS_ID,
			TOKEN_ID,
			Some(PerU16::from_percent(5))
		));
		assert_eq!(
			orml_nft::Tokens::<Runtime>::get(CLASS_ID, TOKEN_ID).unwrap().data.royalty_rate,
			PerU16::from_percent(5)
		);

		assert_ok!(Nftmart::update_token_royalty(
			Origin::signed(BOB),
			CLASS_ID,
			TOKEN_ID,
			Some(PerU16::from_percent(5))
		));
		assert_eq!(
			orml_nft::Tokens::<Runtime>::get(CLASS_ID, TOKEN_ID).unwrap().data.royalty_rate,
			PerU16::from_percent(5)
		);

		assert_ok!(Nftmart::update_token_royalty(Origin::signed(BOB), CLASS_ID, TOKEN_ID, None));
		assert_eq!(
			orml_nft::Tokens::<Runtime>::get(CLASS_ID, TOKEN_ID).unwrap().data.royalty_rate,
			PerU16::from_percent(5)
		);

		assert_ok!(Nftmart::update_token_royalty_beneficiary(
			Origin::signed(BOB),
			CLASS_ID,
			TOKEN_ID,
			ALICE
		));
		assert_noop!(
			Nftmart::update_token_royalty(
				Origin::signed(BOB),
				CLASS_ID,
				TOKEN_ID,
				Some(PerU16::from_percent(5))
			),
			Error::<Runtime>::NoPermission,
		);

		// erc1155
		assert_ok!(Nftmart::update_token_royalty(
			Origin::signed(BOB),
			CLASS_ID,
			TOKEN_ID2,
			Some(PerU16::from_percent(5))
		),);
		// erc1155
		assert_eq!(
			orml_nft::Tokens::<Runtime>::get(CLASS_ID, TOKEN_ID2).unwrap().data.royalty_rate,
			PerU16::from_percent(5)
		);
	});
	// royalty beneficiary erc1155
	ExtBuilder::default().build().execute_with(|| {
		add_category();
		ensure_bob_balances(ACCURACY * 4);
		add_class(ALICE);
		add_token(BOB, 2, None);
		assert_noop!(
			Nftmart::update_token_royalty_beneficiary(
				Origin::signed(BOB),
				CLASS_ID,
				TOKEN_ID_NOT_EXIST,
				ALICE
			),
			Error::<Runtime>::TokenIdNotFound,
		);
		assert_noop!(
			Nftmart::update_token_royalty_beneficiary(
				Origin::signed(ALICE),
				CLASS_ID,
				TOKEN_ID,
				ALICE
			),
			Error::<Runtime>::NoPermission,
		);
		assert_ok!(Nftmart::update_token_royalty_beneficiary(
			Origin::signed(BOB),
			CLASS_ID,
			TOKEN_ID,
			ALICE
		));
		assert_ok!(Nftmart::update_token_royalty_beneficiary(
			Origin::signed(ALICE),
			CLASS_ID,
			TOKEN_ID,
			BOB
		));
	});
}

#[test]
fn create_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let cate_id1 = get_cid();
		add_category();
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			METADATA.to_vec(),
			METADATA.to_vec(),
			METADATA.to_vec(),
			PerU16::from_percent(5),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			vec![cate_id1]
		));

		let event = Event::Nftmart(crate::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

		let reserved = Nftmart::create_class_deposit(
			METADATA.len() as u32,
			METADATA.len() as u32,
			METADATA.len() as u32,
		)
		.1;
		assert_eq!(reserved_balance(&class_id_account()), reserved);

		let class: ClassInfoOf<Runtime> = OrmlNFT::classes(CLASS_ID).unwrap();
		assert!(class.data.properties.0.contains(ClassProperty::Transferable));
		assert!(class.data.properties.0.contains(ClassProperty::Burnable));
	});
}

#[test]
fn create_class_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let cate_id1 = get_cid();
		add_category();
		assert_noop!(
			Nftmart::create_class(
				Origin::signed(BOB),
				METADATA.to_vec(),
				METADATA.to_vec(),
				METADATA.to_vec(),
				PerU16::from_percent(5),
				Properties(ClassProperty::Transferable | ClassProperty::Burnable),
				vec![cate_id1]
			),
			pallet_balances::Error::<Runtime, _>::InsufficientBalance
		);
	});
}

#[test]
fn mint_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let (metadata, reserved) = {
			let cate_id1 = get_cid();
			add_category();
			assert_ok!(Nftmart::create_class(
				Origin::signed(ALICE),
				METADATA.to_vec(),
				METADATA.to_vec(),
				METADATA.to_vec(),
				PerU16::from_percent(5),
				Properties(ClassProperty::Transferable | ClassProperty::Burnable),
				vec![cate_id1]
			));
			let event = Event::Nftmart(crate::Event::CreatedClass(class_id_account(), CLASS_ID));
			assert_eq!(last_event(), event);
			let deposit = Nftmart::create_class_deposit(
				METADATA.len() as u32,
				METADATA.len() as u32,
				METADATA.len() as u32,
			)
			.1;
			(METADATA.to_vec(), deposit)
		};

		let reserved = {
			let deposit = Nftmart::mint_token_deposit(metadata.len() as u32);
			assert_eq!(
				Balances::deposit_into_existing(&class_id_account(), deposit as Balance).is_ok(),
				true
			);
			deposit.saturating_add(reserved)
		};

		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			2u64,
			None,
		));
		let event = Event::Nftmart(crate::Event::MintedToken(
			class_id_account(),
			BOB,
			CLASS_ID,
			TOKEN_ID,
			2u64,
		));
		assert_eq!(last_event(), event);

		assert_eq!(reserved_balance(&class_id_account()), reserved);
	});
}

#[test]
fn mint_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		{
			let metadata = vec![1];
			let deposit = Nftmart::mint_token_deposit(metadata.len() as u32);
			assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit).is_ok(), true);
		}

		assert_noop!(
			Nftmart::mint(Origin::signed(ALICE), BOB, CLASS_ID_NOT_EXIST, vec![1], 2, None),
			Error::<Runtime>::ClassIdNotFound
		);

		assert_noop!(
			Nftmart::mint(Origin::signed(BOB), BOB, CLASS_ID, vec![1], 0, None),
			Error::<Runtime>::InvalidQuantity
		);

		assert_noop!(
			Nftmart::mint(Origin::signed(BOB), BOB, CLASS_ID, vec![1], 2, None),
			Error::<Runtime>::NoPermission
		);

		orml_nft::Classes::<Runtime>::mutate(CLASS_ID, |cls| {
			cls.as_mut().and_then(|x| -> Option<()> {
				x.total_issuance = <Runtime as orml_nft::Config>::TokenId::max_value();
				Some(())
			});
		});

		assert_noop!(
			Nftmart::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 2, None),
			orml_nft::Error::<Runtime>::NumOverflow
		);

		orml_nft::NextTokenId::<Runtime>::mutate(CLASS_ID, |id| {
			*id = <Runtime as orml_nft::Config>::TokenId::max_value()
		});

		assert_noop!(
			Nftmart::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 2, None),
			orml_nft::Error::<Runtime>::NoAvailableTokenId
		);
	});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 2, None);

		assert_ok!(Nftmart::transfer(Origin::signed(BOB), ALICE, vec![(CLASS_ID, TOKEN_ID, 2)]));
		let event =
			Event::Nftmart(crate::Event::TransferredToken(BOB, ALICE, CLASS_ID, TOKEN_ID, 2));
		assert_eq!(last_event(), event);

		assert_ok!(Nftmart::transfer(Origin::signed(ALICE), BOB, vec![(CLASS_ID, TOKEN_ID, 2)]));
		let event =
			Event::Nftmart(crate::Event::TransferredToken(ALICE, BOB, CLASS_ID, TOKEN_ID, 2));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn transfer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 1, None);
		assert_noop!(
			Nftmart::transfer(Origin::signed(BOB), ALICE, vec![(CLASS_ID_NOT_EXIST, TOKEN_ID, 1)]),
			Error::<Runtime>::ClassIdNotFound
		);
		assert_noop!(
			Nftmart::transfer(Origin::signed(BOB), ALICE, vec![(CLASS_ID, TOKEN_ID_NOT_EXIST, 1)]),
			orml_nft::Error::<Runtime>::NumOverflow
		);
		assert_noop!(
			Nftmart::transfer(Origin::signed(ALICE), BOB, vec![(CLASS_ID, TOKEN_ID, 1)]),
			orml_nft::Error::<Runtime>::NumOverflow
		);
	});

	ExtBuilder::default().build().execute_with(|| {
		let cate_id1 = get_cid();
		add_category();
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			METADATA.to_vec(),
			METADATA.to_vec(),
			METADATA.to_vec(),
			PerU16::from_percent(5),
			Default::default(),
			vec![cate_id1]
		));
		let deposit = Nftmart::mint_token_deposit(METADATA.len() as u32);
		assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit).is_ok(), true);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			METADATA.to_vec(),
			1,
			None
		));
		assert_noop!(
			Nftmart::transfer(Origin::signed(BOB), ALICE, vec![(CLASS_ID, TOKEN_ID, 1)]),
			Error::<Runtime>::NonTransferable
		);
	});
}

#[test]
fn burn_should_work() {
	let metadata = vec![1];
	let name = vec![1];
	let description = vec![1];
	let deposit_token = Nftmart::mint_token_deposit(metadata.len() as u32);
	let deposit_class = Nftmart::create_class_deposit(
		metadata.len() as u32,
		name.len() as u32,
		description.len() as u32,
	)
	.1;
	ExtBuilder::default().build().execute_with(|| {
		let cate_id1 = get_cid();
		add_category();
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata,
			name,
			description,
			PerU16::from_percent(5),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			vec![cate_id1]
		));
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), deposit_token).is_ok(),
			true
		);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			2,
			None
		));
		assert_eq!(
			reserved_balance(&class_id_account()),
			deposit_class.saturating_add(deposit_token)
		);
		assert_ok!(Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID, 1));
		let event = Event::Nftmart(crate::Event::BurnedToken(BOB, CLASS_ID, TOKEN_ID, 1, 0));
		assert_eq!(last_event(), event);
		assert_eq!(
			reserved_balance(&class_id_account()),
			deposit_class.saturating_add(deposit_token)
		);

		assert_ok!(Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID, 1));
		let event =
			Event::Nftmart(crate::Event::BurnedToken(BOB, CLASS_ID, TOKEN_ID, 1, deposit_token));
		assert_eq!(last_event(), event);
		assert_eq!(reserved_balance(&class_id_account()), deposit_class);
	});
}

#[test]
fn burn_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 1, None);
		assert_noop!(
			Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID_NOT_EXIST, 0),
			Error::<Runtime>::InvalidQuantity
		);
		assert_noop!(
			Nftmart::burn(Origin::signed(BOB), CLASS_ID_NOT_EXIST, TOKEN_ID, 1),
			Error::<Runtime>::ClassIdNotFound,
		);
		assert_noop!(
			Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID_NOT_EXIST, 1),
			orml_nft::Error::<Runtime>::TokenNotFound
		);
		assert_noop!(
			Nftmart::burn(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, 1),
			orml_nft::Error::<Runtime>::NumOverflow
		);
		assert_noop!(
			Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID, 2),
			orml_nft::Error::<Runtime>::NumOverflow
		);
		orml_nft::Classes::<Runtime>::mutate(CLASS_ID, |class_info| {
			class_info.as_mut().unwrap().total_issuance = 0;
		});
		assert_noop!(
			Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID, 1),
			orml_nft::Error::<Runtime>::NumOverflow
		);
	});

	ExtBuilder::default().build().execute_with(|| {
		let cate_id1 = get_cid();
		add_category();
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			METADATA.to_vec(),
			METADATA.to_vec(),
			METADATA.to_vec(),
			PerU16::from_percent(5),
			Default::default(),
			vec![cate_id1]
		));
		add_token(BOB, 1, None);
		assert_noop!(
			Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID, 1),
			Error::<Runtime>::NonBurnable
		);
	});
}

#[test]
fn destroy_class_should_work() {
	let metadata = vec![1];
	let name = vec![1];
	let description = vec![1];
	let deposit_token = Nftmart::mint_token_deposit(metadata.len() as u32);
	let deposit_class = Nftmart::create_class_deposit(
		metadata.len() as u32,
		name.len() as u32,
		description.len() as u32,
	)
	.1;
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(reserved_balance(&class_id_account()), 0);
		assert_eq!(free_balance(&class_id_account()), 0);
		assert_eq!(free_balance(&ALICE), 100000);
		let cate_id1 = get_cid();
		add_category();
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata,
			name,
			description,
			PerU16::from_percent(5),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			vec![cate_id1]
		));
		assert_eq!(free_balance(&ALICE), 100000 - deposit_class);
		assert_eq!(free_balance(&class_id_account()), 0);
		assert_eq!(reserved_balance(&class_id_account()), deposit_class);
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), deposit_token).is_ok(),
			true
		);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1,
			None
		));
		assert_eq!(free_balance(&class_id_account()), 0);
		assert_eq!(
			reserved_balance(&class_id_account()),
			deposit_class.saturating_add(deposit_token)
		);
		assert_ok!(Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID, 1));
		assert_eq!(reserved_balance(&class_id_account()), deposit_class);
		assert_eq!(free_balance(&class_id_account()), 0);
		assert_ok!(Nftmart::destroy_class(Origin::signed(class_id_account()), CLASS_ID, BOB));
		let event = Event::Nftmart(crate::Event::DestroyedClass(class_id_account(), CLASS_ID, BOB));
		assert_eq!(last_event(), event);
		assert_eq!(free_balance(&class_id_account()), 0);

		assert_eq!(reserved_balance(&class_id_account()), Proxy::deposit(1));

		let free_bob =
			deposit_class.saturating_add(deposit_token).saturating_sub(Proxy::deposit(1));
		assert_eq!(free_balance(&ALICE), 100000 - deposit_class);
		assert_eq!(free_balance(&BOB), free_bob);
	});
}

#[test]
fn destroy_class_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 1, None);
		assert_noop!(
			Nftmart::destroy_class(Origin::signed(class_id_account()), CLASS_ID_NOT_EXIST, BOB),
			Error::<Runtime>::ClassIdNotFound
		);
		assert_noop!(
			Nftmart::destroy_class(Origin::signed(BOB), CLASS_ID, BOB),
			Error::<Runtime>::NoPermission
		);
		assert_noop!(
			Nftmart::destroy_class(Origin::signed(class_id_account()), CLASS_ID, BOB),
			Error::<Runtime>::CannotDestroyClass
		);
		assert_ok!(Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID, 1));
		assert_ok!(Nftmart::destroy_class(Origin::signed(class_id_account()), CLASS_ID, BOB));
	});
}
