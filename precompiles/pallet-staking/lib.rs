#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, ExitSucceed, Precompile};
use precompile_utils::{EvmDataReader, EvmResult, Gasometer, RuntimeHelper};

use fp_evm::{Context, PrecompileOutput};
use frame_support::sp_runtime::traits::StaticLookup;

use sp_core::H256;
use sp_std::{fmt::Debug, if_std, marker::PhantomData, prelude::*};

/// Each variant represents a method that is exposed in the public Solidity interface
/// The function selectors will be automatically generated at compile-time by the macros
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	RemarkPlaceholder = "remarkPlaceholder(bytes)",
	Chill = "chill()",
	Bond = "bond(uint256)",
	Unbond = "unbond(uint256)",
	Rebond = "rebond(uint256)",
	Nominate = "nominate(bytes32[])",
}

pub struct PalletStakingWrapper<T>(PhantomData<T>);

impl<T> Precompile for PalletStakingWrapper<T>
where
	T: pallet_staking::Config + pallet_evm::Config,
	T::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<T::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
	T::Call: From<pallet_staking::Call<T>>,
	T::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		_is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;

		let (mut input, selector) = match EvmDataReader::new_with_selector(gasometer, input) {
			Ok((input, selector)) => (input, selector),
			Err(e) => return Err(e),
		};
		let input = &mut input;

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::RemarkPlaceholder => Self::remark_with_event(input, gasometer, context),
			Action::Chill => Self::chill(input, gasometer, context),
			Action::Bond => Self::bond(input, gasometer, context),
			Action::Unbond => Self::unbond(input, gasometer, context),
			Action::Rebond => Self::rebond(input, gasometer, context),
			Action::Nominate => Self::nominate(input, gasometer, context),
		}
	}
}

impl<T> PalletStakingWrapper<T>
where
	T: pallet_staking::Config + pallet_evm::Config,
	T::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<T::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
	T::Call: From<pallet_staking::Call<T>>,
	T::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
{
	fn remark_with_event(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		_context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 1)?;

		// Use pallet-evm's account mapping to determine what AccountId to dispatch from.
		/*
		let origin = T::AddressMapping::into_account_id(context.caller);
		let remark: Vec<u8> = input.read::<Bytes>(gasometer)?.into();
		let call = frame_system::Call::<T>::remark_with_event { remark: remark.clone() };

		if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
				println!("The remark is: {:#?}", remark);
				println!("The call is: {:#?}", call);
		}

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		let used_gas = gasometer.used_gas();
		// Record the gas used in the gasometer
		gasometer.record_cost(used_gas)?;

		*/
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn bond(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 1)?;

		// Use pallet-evm's account mapping to determine what AccountId to dispatch from.
		let origin = T::AddressMapping::into_account_id(context.caller);
		let amount: u32 = input.read::<u32>(gasometer)?.into();
		let call = pallet_staking::Call::<T>::bond {
			controller: <T as frame_system::Config>::Lookup::unlookup(origin.clone()),
			value: amount.into(),
			payee: pallet_staking::RewardDestination::Account(origin.clone()),
		};

		if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
				println!("The amount is: {:#?}", amount);
				println!("The call is: {:#?}", call);
		}

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		let used_gas = gasometer.used_gas();
		// Record the gas used in the gasometer
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn rebond(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 1)?;

		// Use pallet-evm's account mapping to determine what AccountId to dispatch from.
		let origin = T::AddressMapping::into_account_id(context.caller);
		let amount: u32 = input.read::<u32>(gasometer)?.into();
		let call = pallet_staking::Call::<T>::rebond { value: amount.into() };

		if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
				println!("The amount is: {:#?}", amount);
				println!("The call is: {:#?}", call);
		}

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		let used_gas = gasometer.used_gas();
		// Record the gas used in the gasometer
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn unbond(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 1)?;

		// Use pallet-evm's account mapping to determine what AccountId to dispatch from.
		let origin = T::AddressMapping::into_account_id(context.caller);
		let amount: u32 = input.read::<u32>(gasometer)?.into();
		let call = pallet_staking::Call::<T>::unbond { value: amount.into() };

		if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
				println!("The amount is: {:#?}", amount);
				println!("The call is: {:#?}", call);
		}

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		let used_gas = gasometer.used_gas();
		// Record the gas used in the gasometer
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn nominate(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 1)?;

		// Use pallet-evm's account mapping to determine what AccountId to dispatch from.
		let origin = T::AddressMapping::into_account_id(context.caller);
		let targets: Vec<H256> = input.read::<Vec<H256>>(gasometer)?.into();
		let targets1: Vec<<T as frame_system::Config>::AccountId> = targets
			.iter()
			.map(|x| <T as frame_system::Config>::AccountId::from(x.0))
			.collect();
		let targets2: Vec<<<T as frame_system::Config>::Lookup as StaticLookup>::Source> = targets1
			.iter()
			.map(|x| <T as frame_system::Config>::Lookup::unlookup(x.clone()))
			.collect();

		let call = pallet_staking::Call::<T>::nominate { targets: targets2 };

		if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
				println!("The targetsis: {:#?}", targets);
				println!("The call is: {:#?}", call);
		}

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		let used_gas = gasometer.used_gas();
		// Record the gas used in the gasometer
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn chill(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 0)?;

		// Use pallet-evm's account mapping to determine what AccountId to dispatch from.
		let origin = T::AddressMapping::into_account_id(context.caller);
		let call = pallet_staking::Call::<T>::chill {};

		if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
				println!("The call is: {:#?}", call);
		}

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		let used_gas = gasometer.used_gas();
		// Record the gas used in the gasometer
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
