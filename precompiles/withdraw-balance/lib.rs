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
use sp_std::{fmt::Debug, if_std, marker::PhantomData, prelude::*, vec};

/// Each variant represents a method that is exposed in the public Solidity interface
/// The function selectors will be automatically generated at compile-time by the macros
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	WithdrawBalance = "withdrawBalance(bytes32,uint256)",
	TotalSupply = "totalSupply()",
	FreeBalance = "freeBalance()",
	BalanceOf = "balanceOf(bytes32)",
	Name = "name()",
	Symbol = "symbol()",
	Decimals = "decimals()",
	Whoami = "whoami()",
}

pub struct WithdrawBalancePrecompile<T>(PhantomData<T>);

impl<T> Precompile for WithdrawBalancePrecompile<T>
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
			Action::WithdrawBalance => Self::withdraw_balance(input, target_gas, context),
			Action::TotalSupply => Self::total_supply(input, target_gas, context),
			Action::FreeBalance => Self::free_balance(input, target_gas, context),
			Action::BalanceOf => Self::balance_of(input, target_gas, context),
			Action::Name => Self::name(input, target_gas, context),
			Action::Symbol => Self::symbol(input, target_gas, context),
			Action::Decimals => Self::decimals(input, target_gas, context),
			Action::Whoami => Self::whoami(input, target_gas, context),
		}

		/*
		let input = EvmDataReader::new(input);
		Self::withdraw_balance(input, target_gas, context)
		*/
	}
}

/// Alias for the Balance type for the provided Runtime and Instance.
// pub type BalanceOf<Runtime> = <Runtime as pallet_balances::Config>::Balance;
pub type AccountIdOf<Runtime> = <Runtime as frame_system::Config>::AccountId;
pub type BalanceOf<Runtime> =
	<<Runtime as pallet_evm::Config>::Currency as Currency<AccountIdOf<Runtime>>>::Balance;
// frame_system::pallet::Config

impl<T> WithdrawBalancePrecompile<T>
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
	fn withdraw_balance(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// This gasometer is a handy utility that will help us account for gas as we go.
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check. We expect a single argument passed in.
		input.expect_arguments(&mut gasometer, 2)?;

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

		let to: [u8; 32] = input.read::<H256>(&mut gasometer)?.into();
		let to: T::AccountId = to.into();
		// log::debug!(target: "nftmart-evm", "to(sub): {:?} {}", &to, &to);
		log::debug!(target: "nftmart-evm", "to(sub): {:?}", &to);

		let mut amount: U256 = input.read::<U256>(&mut gasometer)?;
		// amount = 1000000000000000000u128.into();
		log::debug!(target: "nftmart-evm", "amount(sub): {:?}", &amount);
		// let amount = pallet_evm::Pallet::<T>::convert_decimals_from_evm(amount.low_u256()).unwrap();

		let amount = Self::u256_to_amount(&mut gasometer, amount)?;
		log::debug!(target: "nftmart-evm", "amount(sub): {:?}", &amount);

		// let call = pallet_balances::Call::<T>::transfer { dest: T::Lookup::unlookup(to), value: amount };

		// RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, &mut gasometer)?;
		let _ = T::Currency::transfer(&origin, &to, amount, ExistenceRequirement::AllowDeath);

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

	fn u256_to_amount(gasometer: &mut Gasometer, value: U256) -> EvmResult<BalanceOf<T>> {
		value
			.try_into()
			.map_err(|_| gasometer.revert("amount is too large for provided balance type"))
	}

	fn total_supply(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;
		gasometer.record_cost(RuntimeHelper::<T>::db_read_gas_cost())?;

		// Parse input.
		input.expect_arguments(gasometer, 0)?;

		// Fetch info.
		let amount: U256 = T::Currency::total_issuance().into();
		log::debug!(target: "nftmart-evm", "amount(total): {:?}", &amount);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(amount).build(),
			logs: vec![],
		})
	}

	fn balance_of(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;
		gasometer.record_cost(RuntimeHelper::<T>::db_read_gas_cost())?;

		// Parse input.
		input.expect_arguments(gasometer, 1)?;

		// Fetch info.
		let who: [u8; 32] = input.read::<H256>(gasometer)?.into();
		let who: T::AccountId = who.into();
		log::debug!(target: "nftmart-evm", "who(sub): {:?}", &who);

		let amount: U256 = T::Currency::free_balance(&who).into();
		log::debug!(target: "nftmart-evm", "amount(total): {:?}", &amount);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(amount).build(),
			logs: vec![],
		})
	}

	fn free_balance(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;
		gasometer.record_cost(RuntimeHelper::<T>::db_read_gas_cost())?;

		// Parse input.
		input.expect_arguments(gasometer, 0)?;

		// Fetch info.
		// let amount: U256 = T::Currency::total_issuance().into();
		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		// log::debug!(target: "nftmart-evm", "origin: {:?} {}", &origin, &origin);
		log::debug!(target: "nftmart-evm", "origin: {:?}", &origin);
		// pallet_balances::Pallet::<Runtime, Instance>::usable_balance(&owner).into()
		let amount: U256 = T::Currency::free_balance(&origin).into();
		log::debug!(target: "nftmart-evm", "amount(total): {:?}", &amount);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(amount).build(),
			logs: vec![],
		})
	}

	fn whoami(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Build output.
		let mut gasometer = Gasometer::new(target_gas);
		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		// log::debug!(target: "nftmart-evm", "origin: {:?} {}", &origin, &origin);
		log::debug!(target: "nftmart-evm", "origin: {:?}", &origin);
		let origin: [u8; 32] = origin.into();
		log::debug!(target: "nftmart-evm", "origin: {:?}", &origin);
		let origin = &origin[0..32];
		log::debug!(target: "nftmart-evm", "origin: {:?}", &origin);
		// let mut target: [u8] = [0];
		// target[0..32].copy_from_slice(&origin[0..32]);
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Bytes>(origin.into()).build(),
			logs: Default::default(),
		})
	}

	fn name(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Build output.
		let mut gasometer = Gasometer::new(target_gas);
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Bytes>(Metadata::name().into()).build(),
			logs: Default::default(),
		})
	}

	fn symbol(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Build output.
		let mut gasometer = Gasometer::new(target_gas);
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Bytes>(Metadata::symbol().into()).build(),
			logs: Default::default(),
		})
	}

	fn decimals(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Build output.
		let mut gasometer = Gasometer::new(target_gas);
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(Metadata::decimals()).build(),
			logs: Default::default(),
		})
	}
}

struct Metadata;

impl Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str {
		"NFTMart Token"
	}

	/// Returns the symbol of the token.
	fn symbol() -> &'static str {
		"NMT"
	}

	/// Returns the decimals places of the token.
	fn decimals() -> u8 {
		12
	}
}
