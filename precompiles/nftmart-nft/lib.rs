#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

// use sp_std::fmt::Display;
use codec::Decode;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::StaticLookup,
};
use pallet_evm::{AddressMapping, ExitSucceed, Precompile};
use precompile_utils::{
	Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, RuntimeHelper,
};

use fp_evm::{Context, PrecompileOutput};
use frame_support::traits::{Currency, ExistenceRequirement};

use sp_core::{crypto::UncheckedFrom, H256, U256};
use sp_std::{fmt::Debug, if_std, marker::PhantomData, prelude::*};

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
	T: pallet_evm::Config,
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
	T: pallet_evm::Config,
	// T: pallet_evm::Config + pallet_balances::Config,
	// T::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	// <T::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
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

		// let call = pallet_balances::Call::<T>::transfer { dest: T::Lookup::unlookup(to), value: amount };

		// RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, &mut gasometer)?;
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
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<[u8; 32]>(&mut gasometer)?.into();
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<H256>(&mut gasometer)?.into();
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<T::AccountId>(gasometer)?.into();
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<Vec<u8>>(&mut gasometer)?.into();
		let to = sp_core::sr25519::Public::unchecked_from(input.read::<H256>(gasometer)?);
		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		// let metadata: &[u8] = input.read::<Bytes>(gasometer)?.as_bytes();
		let metadata = input.read::<Bytes>(gasometer)?;
		let metadata = metadata.as_str();
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
		let dest = sp_core::sr25519::Public::unchecked_from(input.read::<H256>(gasometer)?);
		// how do I convert sp_core::sr25519::Public to AccountId?
		//
		// AccountId is defined in ../../pallets/nftmart-traits/src/constants_types.rs
		// pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
		//
		// let desta: <T as frame_system::pallet::Config>::AccountId = dest.into();
		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "dest: {:?}", &dest);

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
		let metadata = metadata.as_str();

		let name = input.read::<Bytes>(gasometer)?;
		let name = name.as_str();

		let description = input.read::<Bytes>(gasometer)?;
		let description = description.as_str();

		let royalty_rate: u32 = input.read::<u32>(gasometer)?.into();

		let properties: u8 = input.read::<u8>(gasometer)?.into();

		let category_ids: Vec<u32> = input.read::<Vec<u32>>(gasometer)?.into();
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

		let to = sp_core::sr25519::Public::unchecked_from(input.read::<H256>(gasometer)?);
		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		let metadata = input.read::<Bytes>(gasometer)?;
		let metadata = metadata.as_str();
		let quantity: u32 = input.read::<u32>(gasometer)?.into();
		let charge_royalty: u32 = input.read::<u32>(gasometer)?.into();

		log::debug!(target: "nftmart-evm", "to: {:?}", &to);
		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "metadata: {:?}", &metadata);
		log::debug!(target: "nftmart-evm", "quantity: {:?}", &quantity);
		log::debug!(target: "nftmart-evm", "charge_royalty: {:?}", &charge_royalty);

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
		let metadata = metadata.as_str();

		let name = input.read::<Bytes>(gasometer)?;
		let name = name.as_str();

		let description = input.read::<Bytes>(gasometer)?;
		let description = description.as_str();

		let royalty_rate: u32 = input.read::<u32>(gasometer)?.into();

		let properties: u8 = input.read::<u8>(gasometer)?.into();

		let category_ids: Vec<u32> = input.read::<Vec<u32>>(gasometer)?.into();
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

		let to = sp_core::sr25519::Public::unchecked_from(input.read::<H256>(gasometer)?);
		let class_id: u32 = input.read::<u32>(gasometer)?.into();
		let token_id: u32 = input.read::<u32>(gasometer)?.into();
		let quantity: u32 = input.read::<u32>(gasometer)?.into();
		let metadata = input.read::<Bytes>(gasometer)?;
		let metadata = metadata.as_str();
		let charge_royalty: u32 = input.read::<u32>(gasometer)?.into();

		log::debug!(target: "nftmart-evm", "to: {:?}", &to);
		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "tokenId: {:?}", &token_id);
		log::debug!(target: "nftmart-evm", "quantity: {:?}", &quantity);
		log::debug!(target: "nftmart-evm", "metadata: {:?}", &metadata);
		log::debug!(target: "nftmart-evm", "charge_royalty: {:?}", &charge_royalty);

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
		let metadata = metadata.as_str();

		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "tokenId: {:?}", &token_id);
		log::debug!(target: "nftmart-evm", "metadata: {:?}", &metadata);

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
		let royalty_beneficiary =
			sp_core::sr25519::Public::unchecked_from(input.read::<H256>(gasometer)?);

		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "tokenId: {:?}", &token_id);
		log::debug!(target: "nftmart-evm", "royalty_beneficiary: {:?}", &royalty_beneficiary);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
