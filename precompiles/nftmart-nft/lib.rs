#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

// use sp_std::fmt::Display;
use codec::Decode;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::{IdentifyAccount, StaticLookup},
};
use nftmart_nft::Call as NftCall;
use pallet_evm::{AddressMapping, ExitSucceed, Precompile};
use precompile_utils::{
	Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, RuntimeHelper,
};
use sp_arithmetic::PerU16;

use fp_evm::{Context, PrecompileOutput};
use frame_support::traits::{Currency, ExistenceRequirement};

use sp_core::{crypto::UncheckedFrom, H256, U256};
use sp_std::{fmt::Debug, if_std, marker::PhantomData, prelude::*};

use nftmart_traits::{ClassProperty, Properties};

/// Each variant represents a method that is exposed in the public Solidity interface
/// The function selectors will be automatically generated at compile-time by the macros
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	Burn = "burn(uint256,uint256,uint256)",
	CreateClass = "createClass(string,string,string,uint256,uint8,uint256[])",
	DestroyClass = "destroyClass(uint256,bytes32)",
	Mint = "mint(bytes32,uint256,string,uint256,uint256)",
	ProxyMint = "proxyMint(bytes32,uint256,string,uint256,uint256)",
	UpdateClass = "updateClass(uint256,string,string,string,uint256,uint8,uint256[])",
	UpdateToken = "updateToken(bytes32,uint256,uint256,uint256,string,uint256)",
	UpdateTokenMetadata = "updateTokenMetadata(uint256,uint256,string)",
	UpdateTokenRoyalty = "updateTokenRoyalty(uint256,uint256,uint256)",
	UpdateTokenRoyaltyBeneficiary = "updateTokenRoyaltyBeneficiary(uint256,uint256,bytes32)",
}

pub struct NftmartNftPrecompile<T>(PhantomData<T>);

impl<T> Precompile for NftmartNftPrecompile<T>
where
	T: pallet_evm::Config + nftmart_nft::Config + orml_nft::Config,
	<T as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + From<NftCall<T>>,
	<<T as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<<T as frame_system::Config>::AccountId>>,
	// T: pallet_evm::Config + pallet_balances::Config,
	// T::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	// <T::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
	// T::Call: From<pallet_balances::Call<T>>,
	// T::AccountId: EvmData,
	// T::AccountId: From<H256>,
	T::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
	// T::AccountId: Display,
	BalanceOf<T>: TryFrom<U256> + Into<U256>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		_is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		let (mut input, selector) = match EvmDataReader::new_with_selector(&mut gasometer, input) {
			Ok((input, selector)) => (input, selector),
			Err(e) => return Err(e),
		};

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::Burn => Self::burn(&mut input, &mut gasometer, context),
			Action::CreateClass => Self::create_class(&mut input, &mut gasometer, context),
			Action::DestroyClass => Self::destroy_class(&mut input, &mut gasometer, context),
			Action::Mint => Self::mint(&mut input, &mut gasometer, context),
			Action::ProxyMint => Self::proxy_mint(&mut input, &mut gasometer, context),
			Action::UpdateClass => Self::update_class(&mut input, &mut gasometer, context),
			Action::UpdateToken => Self::update_token(&mut input, &mut gasometer, context),
			Action::UpdateTokenMetadata =>
				Self::update_token_metadata(&mut input, &mut gasometer, context),
			Action::UpdateTokenRoyalty =>
				Self::update_token_royalty(&mut input, &mut gasometer, context),
			Action::UpdateTokenRoyaltyBeneficiary =>
				Self::update_token_royalty_beneficiary(&mut input, &mut gasometer, context),
		}
	}
}

/// Alias for the Balance type for the provided Runtime and Instance.
// pub type BalanceOf<Runtime> = <Runtime as pallet_balances::Config>::Balance;
pub type AccountIdOf<Runtime> = <Runtime as frame_system::Config>::AccountId;
pub type BalanceOf<Runtime> =
	<<Runtime as pallet_evm::Config>::Currency as Currency<AccountIdOf<Runtime>>>::Balance;
// frame_system::pallet::Config

impl<T> NftmartNftPrecompile<T>
where
	T: pallet_evm::Config + nftmart_nft::Config + orml_nft::Config,
	// `<<T as frame_system::Config>::Call as Dispatchable>::PostInfo = PostDispatchInfo`
	// `<T as frame_system::Config>::Call: GetDispatchInfo`
	// T: pallet_evm::Config + pallet_balances::Config,
	// T::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<T as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + From<NftCall<T>>,
	<<T as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<<T as frame_system::Config>::AccountId>>,
	// T::Call: From<pallet_balances::Call<T>>,
	// T::AccountId: EvmData,
	// T::AccountId: From<H256>,
	// T::AccountId: From<[u8; 32]>,
	T::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
	// T::AccountId: Display,
	BalanceOf<T>: TryFrom<U256> + Into<U256>,
{
	fn burn(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 3)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);
		// log::debug!(target: "nftmart-evm", "from(evm): {:?} {}", &origin, &origin);
		// let a : u32 = 0; a = _origin;
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<T::AccountId>(&mut gasometer)?.into();
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<H256>(&mut gasometer)?.into();
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<[u8; 32]>(&mut gasometer)?.into();

		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		let token_id: u32 = input.read::<u32>(gasometer)?.into();
		let quantity: u32 = input.read::<u32>(gasometer)?.into();
		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "tokenId: {:?}", &token_id);
		log::debug!(target: "nftmart-evm", "quantity: {:?}", &quantity);

		let call = NftCall::<T>::burn {
			class_id: class_id.into(),
			token_id: class_id.into(),
			quantity: quantity.into(),
		};

		// RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, &mut gasometer)?;
		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;
		// let _ = T::Currency::transfer(&origin, &to, amount, ExistenceRequirement::AllowDeath);

		// let used_gas = gasometer.used_gas();
		// Record the gas used in the gasometer
		// gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn mint(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 5)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);

		if_std! {
				use sp_core::sr25519::{Public as sr25519Public};
				use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
				println!("decode nmt {:?}", sr25519Public::from_string_with_version("nmqxkVdwMn5VnbormnohxxZ4H2MpnJGTmuAxRVwZ1Y6ts5ne5"));
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<[u8; 32]>(&mut gasometer)?.into();
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<H256>(&mut gasometer)?.into();
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<T::AccountId>(gasometer)?.into();
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<Vec<u8>>(&mut gasometer)?.into();
		// let to = sp_core::sr25519::Public::unchecked_from(input.read::<H256>(gasometer)?);
		let to: H256 = input.read::<H256>(gasometer)?;

		// let to = sp_core::sr25519::Public::unchecked_from(to);

		// let to = sp_core::sr25519::Public::from_h256(to);

		// let to: <T as frame_system::Config>::AccountId = to.into_account();

		if_std! {
				let to1 = sp_core::sr25519::Public::from_h256(to);
				let to2 = to1.into_account();
				println!("to: {:?}", to2.to_ss58check_with_version(Ss58AddressFormat::custom(12191)));
		}

		let to: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(to.0);

		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		// let metadata: &[u8] = input.read::<Bytes>(gasometer)?.as_bytes();
		let metadata = input.read::<Bytes>(gasometer)?;
		// let metadata = metadata.as_str();
		let quantity: u32 = input.read::<u32>(gasometer)?.into();
		let charge_royalty: u32 = input.read::<u32>(gasometer)?.into();
		// let token_id: u32 = input.read::<u32>(gasometer)?.into();
		// let quantity: u32 = input.read::<u32>(gasometer)?.into();
		log::debug!(target: "nftmart-evm", "to: {:?}", &to);
		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "metadata: {:?}", &metadata);
		log::debug!(target: "nftmart-evm", "quantity: {:?}", &quantity);
		log::debug!(target: "nftmart-evm", "charge_royalty: {:?}", &charge_royalty);
		// log::debug!(target: "nftmart-evm", "metadata: {:?}", &metadata.clone());
		// log::debug!(target: "nftmart-evm", "tokenId: {:?}", &token_id);
		// log::debug!(target: "nftmart-evm", "quantity: {:?}", &quantity);

		let call = NftCall::<T>::mint {
			to: <T as frame_system::Config>::Lookup::unlookup(to),
			class_id: class_id.into(),
			metadata: metadata.into(),
			quantity: quantity.into(),
			charge_royalty: Some(PerU16::from_parts(charge_royalty.try_into().unwrap())),
		};

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn destroy_class(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 2)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		let dest = input.read::<H256>(gasometer)?;
		let dest: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(dest.0);

		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "dest: {:?}", &dest);

		let call = NftCall::<T>::destroy_class {
			class_id: class_id.into(),
			dest: <T as frame_system::Config>::Lookup::unlookup(dest),
		};

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn create_class(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 6)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let metadata = input.read::<Bytes>(gasometer)?;
		// let metadata = metadata.as_str();

		let name = input.read::<Bytes>(gasometer)?;
		// let name = name.as_str();

		let description = input.read::<Bytes>(gasometer)?;
		// let description = description.as_str();

		let royalty_rate: u32 = input.read::<u32>(gasometer)?.into();

		let properties: u8 = input.read::<u8>(gasometer)?.into();

		let category_ids: Vec<u64> = input.read::<Vec<u64>>(gasometer)?.into();
		// how do I convert sp_core::sr25519::Public to AccountId?
		//
		// AccountId is defined in ../../pallets/nftmart-traits/src/constants_types.rs
		// pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
		//
		// let desta: <T as frame_system::pallet::Config>::AccountId = dest.into();
		log::debug!(target: "nftmart-evm", "metadata: {:?}", &metadata);
		log::debug!(target: "nftmart-evm", "name: {:?}", &name);
		log::debug!(target: "nftmart-evm", "description: {:?}", &description);
		log::debug!(target: "nftmart-evm", "royalty_rate: {:?}", &royalty_rate);
		log::debug!(target: "nftmart-evm", "properties: {:?}", &properties);
		log::debug!(target: "nftmart-evm", "category_ids: {:?}", &category_ids);

		let call = NftCall::<T>::create_class {
			metadata: metadata.into(),
			name: name.into(),
			description: description.into(),
			royalty_rate: PerU16::from_parts(royalty_rate.try_into().unwrap()),
			properties: Properties(ClassProperty::Transferable.into()), // TODO: use real properties,
			category_ids,
		};

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn proxy_mint(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 5)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		// let to = sp_core::sr25519::Public::unchecked_from(input.read::<H256>(gasometer)?);
		// let to = to.into_account();
		let to = input.read::<H256>(gasometer)?;
		let to: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(to.0);
		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		let metadata = input.read::<Bytes>(gasometer)?;
		// let metadata = metadata.as_str();
		let quantity: u32 = input.read::<u32>(gasometer)?.into();
		let charge_royalty: u32 = input.read::<u32>(gasometer)?.into();

		log::debug!(target: "nftmart-evm", "to: {:?}", &to);
		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "metadata: {:?}", &metadata);
		log::debug!(target: "nftmart-evm", "quantity: {:?}", &quantity);
		log::debug!(target: "nftmart-evm", "charge_royalty: {:?}", &charge_royalty);

		let call = NftCall::<T>::mint {
			to: <T as frame_system::Config>::Lookup::unlookup(to),
			class_id: class_id.into(),
			metadata: metadata.into(),
			quantity: quantity.into(),
			charge_royalty: Some(PerU16::from_parts(charge_royalty.try_into().unwrap())),
		};

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn update_class(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 7)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let class_id: u32 = input.read::<u32>(gasometer)?.into();

		let metadata = input.read::<Bytes>(gasometer)?;
		// let metadata = metadata.as_str();

		let name = input.read::<Bytes>(gasometer)?;
		// let name = name.as_str();

		let description = input.read::<Bytes>(gasometer)?;
		// let description = description.as_str();

		let royalty_rate: u32 = input.read::<u32>(gasometer)?.into();

		let properties: u8 = input.read::<u8>(gasometer)?.into();

		let category_ids: Vec<u64> = input.read::<Vec<u64>>(gasometer)?.into();
		// how do I convert sp_core::sr25519::Public to AccountId?
		//
		// AccountId is defined in ../../pallets/nftmart-traits/src/constants_types.rs
		// pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
		//
		// let desta: <T as frame_system::pallet::Config>::AccountId = dest.into();
		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "metadata: {:?}", &metadata);
		log::debug!(target: "nftmart-evm", "name: {:?}", &name);
		log::debug!(target: "nftmart-evm", "description: {:?}", &description);
		log::debug!(target: "nftmart-evm", "royalty_rate: {:?}", &royalty_rate);
		log::debug!(target: "nftmart-evm", "properties: {:?}", &properties);
		log::debug!(target: "nftmart-evm", "category_ids: {:?}", &category_ids);

		let call = NftCall::<T>::update_class {
			class_id: class_id.into(),
			metadata: metadata.into(),
			name: name.into(),
			description: description.into(),
			royalty_rate: PerU16::from_parts(royalty_rate.try_into().unwrap()),
			properties: Properties(ClassProperty::Transferable.into()), // TODO: use real properties,
			category_ids,
		};

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn update_token(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 6)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		// let to = sp_core::sr25519::Public::unchecked_from(input.read::<H256>(gasometer)?);
		// let to = to.into_account();
		let to = input.read::<H256>(gasometer)?;
		let to: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(to.0);
		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		let token_id: u32 = input.read::<u32>(gasometer)?.into();
		let quantity: u32 = input.read::<u32>(gasometer)?.into();
		let metadata = input.read::<Bytes>(gasometer)?;
		// let metadata = metadata.as_str();
		let charge_royalty: u32 = input.read::<u32>(gasometer)?.into();

		log::debug!(target: "nftmart-evm", "to: {:?}", &to);
		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "tokenId: {:?}", &token_id);
		log::debug!(target: "nftmart-evm", "quantity: {:?}", &quantity);
		log::debug!(target: "nftmart-evm", "metadata: {:?}", &metadata);
		log::debug!(target: "nftmart-evm", "charge_royalty: {:?}", &charge_royalty);

		let call = NftCall::<T>::update_token {
			to: <T as frame_system::Config>::Lookup::unlookup(to),
			class_id: class_id.into(),
			token_id: token_id.into(),
			quantity: quantity.into(),
			metadata: metadata.into(),
			charge_royalty: Some(PerU16::from_parts(charge_royalty.try_into().unwrap())),
		};

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn update_token_metadata(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 3)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		let token_id: u32 = input.read::<u32>(gasometer)?.into();
		let metadata = input.read::<Bytes>(gasometer)?;
		// let metadata = metadata.as_str();

		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "tokenId: {:?}", &token_id);
		log::debug!(target: "nftmart-evm", "metadata: {:?}", &metadata);

		let call = NftCall::<T>::update_token_metadata {
			class_id: class_id.into(),
			token_id: token_id.into(),
			metadata: metadata.into(),
		};

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn update_token_royalty(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 3)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		let token_id: u32 = input.read::<u32>(gasometer)?.into();
		let charge_royalty: u32 = input.read::<u32>(gasometer)?.into();

		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "tokenId: {:?}", &token_id);
		log::debug!(target: "nftmart-evm", "charge_royalty: {:?}", &charge_royalty);

		let call = NftCall::<T>::update_token_royalty {
			class_id: class_id.into(),
			token_id: token_id.into(),
			charge_royalty: Some(PerU16::from_parts(charge_royalty.try_into().unwrap())),
		};

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn update_token_royalty_beneficiary(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 3)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		let token_id: u32 = input.read::<u32>(gasometer)?.into();
		// let royalty_beneficiary = sp_core::sr25519::Public::unchecked_from(input.read::<H256>(gasometer)?);
		// let royalty_beneficiary = royalty_beneficiary.into_account();
		let royalty_beneficiary = input.read::<H256>(gasometer)?;
		let royalty_beneficiary: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(royalty_beneficiary.0);

		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "tokenId: {:?}", &token_id);
		log::debug!(target: "nftmart-evm", "royalty_beneficiary: {:?}", &royalty_beneficiary);

		let call = NftCall::<T>::update_token_royalty_beneficiary {
			class_id: class_id.into(),
			token_id: token_id.into(),
			to: <T as frame_system::Config>::Lookup::unlookup(royalty_beneficiary),
		};

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
