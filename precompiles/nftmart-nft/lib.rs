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

use sp_core::{H256, U256};
use sp_std::{fmt::Debug, if_std, marker::PhantomData, prelude::*};

/// Each variant represents a method that is exposed in the public Solidity interface
/// The function selectors will be automatically generated at compile-time by the macros
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	Burn = "burn(uint,uint,uint)",
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
		let gasometer = &mut gasometer;

		let (input, selector) = match EvmDataReader::new_with_selector(gasometer, input) {
			Ok((input, selector)) => (input, selector),
			Err(e) => return Err(e),
		};

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::Burn => Self::burn(input, target_gas, context),
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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// This gasometer is a handy utility that will help us account for gas as we go.
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check. We expect a single argument passed in.
		input.expect_arguments(&mut gasometer, 3)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				/*
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
				*/
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);
		// log::debug!(target: "nftmart-evm", "from(evm): {:?} {}", &origin, &origin);
		// let a : u32 = 0; a = _origin;
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<T::AccountId>(&mut gasometer)?.into();
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<H256>(&mut gasometer)?.into();
		// let to: <T as frame_system::pallet::Config>::AccountId = input.read::<[u8; 32]>(&mut gasometer)?.into();

		let class_id: u32 = input.read::<u32>(&mut gasometer)?.into();
		let token_id: u32 = input.read::<u32>(&mut gasometer)?.into();
		let quantity: u32 = input.read::<u32>(&mut gasometer)?.into();
		log::debug!(target: "nftmart-evm", "classId: {:?}", &class_id);
		log::debug!(target: "nftmart-evm", "tokenId: {:?}", &token_id);
		log::debug!(target: "nftmart-evm", "quantity: {:?}", &quantity);

		// let call = pallet_balances::Call::<T>::transfer { dest: T::Lookup::unlookup(to), value: amount };

		// RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, &mut gasometer)?;
		// let _ = T::Currency::transfer(&origin, &to, amount, ExistenceRequirement::AllowDeath);

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
