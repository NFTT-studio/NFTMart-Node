#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, ExitSucceed, Precompile};
use precompile_utils::{Bytes, EvmDataReader, EvmResult, Gasometer, RuntimeHelper};

use fp_evm::{Context, PrecompileOutput};

use sp_std::{fmt::Debug, if_std, marker::PhantomData, prelude::*};

/// Each variant represents a method that is exposed in the public Solidity interface
/// The function selectors will be automatically generated at compile-time by the macros
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	RemarkWithEvent = "remarkWithEvent(bytes)",
}

pub struct FrameSystemWrapper<T>(PhantomData<T>);

impl<T> Precompile for FrameSystemWrapper<T>
where
	T: frame_system::Config + pallet_evm::Config,
	T::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<T::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
	T::Call: From<frame_system::Call<T>>,
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
			Action::RemarkWithEvent => Self::remark_with_event(input, gasometer, context),
		}
	}
}

impl<T> FrameSystemWrapper<T>
where
	T: frame_system::Config + pallet_evm::Config,
	T::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<T::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
	T::Call: From<frame_system::Call<T>>,
{
	fn remark_with_event(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check. We expect a single argument passed in.
		input.expect_arguments(gasometer, 1)?;

		// Use pallet-evm's account mapping to determine what AccountId to dispatch from.
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

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
