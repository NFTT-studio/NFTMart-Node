#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, ExitSucceed, Precompile};
use precompile_utils::{EvmDataReader, EvmDataWriter, EvmResult, Gasometer, RuntimeHelper};

use fp_evm::{Context, PrecompileOutput};

use sp_std::{fmt::Debug, if_std, marker::PhantomData, prelude::*};

/// Each variant represents a method that is exposed in the public Solidity interface
/// The function selectors will be automatically generated at compile-time by the macros
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	DoSomething = "do_something(uint256)",
	GetValue = "get_value()",
}

pub struct PalletTemplatePrecompile<T>(PhantomData<T>);

impl<T> Precompile for PalletTemplatePrecompile<T>
where
	T: pallet_template::Config + pallet_evm::Config,
	T::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<T::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
	T::Call: From<pallet_template::Call<T>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		_is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;

		let (input, selector) = match EvmDataReader::new_with_selector(gasometer, input) {
			Ok((input, selector)) => (input, selector),
			Err(e) => return Err(e),
		};

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::DoSomething => Self::do_something(input, target_gas, context),
			Action::GetValue => Self::get_value(input, target_gas, context),
		}
	}
}

impl<T> PalletTemplatePrecompile<T>
where
	T: pallet_template::Config + pallet_evm::Config,
	T::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<T::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
	T::Call: From<pallet_template::Call<T>>,
{
	fn do_something(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// This gasometer is a handy utility that will help us account for gas as we go.
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check. We expect a single argument passed in.
		input.expect_arguments(&mut gasometer, 1)?;

		// Parse the u32 value that will be dispatched to the pallet.
		let value = input.read::<u32>(&mut gasometer)?.into();

		// Use pallet-evm's account mapping to determine what AccountId to dispatch from.
		let origin = T::AddressMapping::into_account_id(context.caller);
		if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		let call = pallet_template::Call::<T>::do_something { something: value };
		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, &mut gasometer)?;

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

	fn get_value(
		input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(&mut gasometer, 0)?;

		// fetch data from pallet
		let stored_value = pallet_template::Something::<T>::get().unwrap_or_default();

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		// Record one storage red worth of gas.
		// The utility internally uses pallet_evm's GasWeightMapping.
		gasometer.record_cost(RuntimeHelper::<T>::db_read_gas_cost())?;

		// Construct to Solidity-formatted output data
		let output = EvmDataWriter::new().write(stored_value).build();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output,
			logs: Default::default(),
		})
	}
}

// TODO Mock runtime
// TODO tests
// See Moonbeam for examples https://github.com/PureStake/moonbeam/tree/master/precompiles
