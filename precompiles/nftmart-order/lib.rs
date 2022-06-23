#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

// use sp_std::fmt::Display;

use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::StaticLookup,
};
use nftmart_order::Call as OrderCall;
use pallet_evm::{AddressMapping, ExitSucceed, Precompile};
use precompile_utils::{Bytes, EvmDataReader, EvmResult, Gasometer, RuntimeHelper};

use fp_evm::{Context, PrecompileOutput};
use frame_support::traits::Currency;

use sp_core::{H256, U256};
use sp_std::{fmt::Debug, if_std, marker::PhantomData, prelude::*};

/// Each variant represents a method that is exposed in the public Solidity interface
/// The function selectors will be automatically generated at compile-time by the macros
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	RemoveOffer = "removeOffer(uint256)",
	RemoveOrder = "removeOrder(uint256)",
	SubmitOffer = "submitOffer(uint256,uint256,uint256,uint256[3][],uint256)",
	SubmitOrder = "submitOrder(uint256,uint256,uint256,uint256,uint256[3][],uint256)",
	TakeOffer = "takeOffer(uint256,bytes32,bytes32,string)",
	TakeOrder = "takeOrder(uint256,bytes32,bytes32,string)",
}

pub struct NftmartOrderPrecompile<T>(PhantomData<T>);

impl<T> Precompile for NftmartOrderPrecompile<T>
where
	T: pallet_evm::Config + nftmart_order::Config + orml_nft::Config,
	<T as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + From<OrderCall<T>>,
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
			Action::RemoveOffer => Self::remove_offer(&mut input, &mut gasometer, context),
			Action::RemoveOrder => Self::remove_order(&mut input, &mut gasometer, context),
			Action::TakeOffer => Self::take_offer(&mut input, &mut gasometer, context),
			Action::TakeOrder => Self::take_order(&mut input, &mut gasometer, context),
			/*
			Action::SubmitOffer => Self::submit_offer(&mut input, &mut gasometer, context),
			Action::SubmitOrder => Self::submit_order(&mut input, &mut gasometer, context),
			*/
			_ => Self::remove_offer(&mut input, &mut gasometer, context),
		}
	}
}

/// Alias for the Balance type for the provided Runtime and Instance.
// pub type BalanceOf<Runtime> = <Runtime as pallet_balances::Config>::Balance;
pub type AccountIdOf<Runtime> = <Runtime as frame_system::Config>::AccountId;
pub type BalanceOf<Runtime> =
	<<Runtime as pallet_evm::Config>::Currency as Currency<AccountIdOf<Runtime>>>::Balance;
// frame_system::pallet::Config

impl<T> NftmartOrderPrecompile<T>
where
	T: pallet_evm::Config + nftmart_order::Config + orml_nft::Config,
	<T as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + From<OrderCall<T>>,
	<<T as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<<T as frame_system::Config>::AccountId>>,
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
	fn remove_order(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		input.expect_arguments(gasometer, 1)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let order_id: u32 = input.read::<u32>(gasometer)?.into();

		log::debug!(target: "nftmart-evm", "orderId: {:?}", &order_id);

		let call = OrderCall::<T>::remove_order { order_id: order_id.into() };

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn remove_offer(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		input.expect_arguments(gasometer, 1)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let offer_id: u32 = input.read::<u32>(gasometer)?.into();

		log::debug!(target: "nftmart-evm", "offerId: {:?}", &offer_id);

		let call = OrderCall::<T>::remove_offer { offer_id: offer_id.into() };

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
	/*
	 *
	function takeOffer(uint _offerId, bytes32 _offerOwner, bytes32 _commissionAgent, string memory _commissionData) external;
	function takeOrder(uint _orderId, bytes32 _orderOwner, bytes32 _commissionAgent, string memory _commissionData) external;
	*/
	fn take_order(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		input.expect_arguments(gasometer, 4)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let order_id: u32 = input.read::<u32>(gasometer)?.into();
		let order_owner = input.read::<H256>(gasometer)?;
		let order_owner: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(order_owner.0);
		let commission_agent = input.read::<H256>(gasometer)?;
		let commission_agent: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(commission_agent.0);
		let commission_data = input.read::<Bytes>(gasometer)?;

		log::debug!(target: "nftmart-evm", "orderId: {:?}", &order_id);
		log::debug!(target: "nftmart-evm", "orderOwner: {:?}", &order_owner);
		log::debug!(target: "nftmart-evm", "commissionAgent: {:?}", &commission_agent);
		log::debug!(target: "nftmart-evm", "commissionData: {:?}", &commission_data);

		let call = OrderCall::<T>::take_order {
			order_id: order_id.into(),
			order_owner: <T as frame_system::Config>::Lookup::unlookup(order_owner),
			commission_agent: Some(commission_agent),
			commission_data: Some(commission_data.into()),
		};

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn take_offer(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		input.expect_arguments(gasometer, 4)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let offer_id: u32 = input.read::<u32>(gasometer)?.into();
		let offer_owner = input.read::<H256>(gasometer)?;
		let offer_owner: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(offer_owner.0);
		let commission_agent = input.read::<H256>(gasometer)?;
		let commission_agent: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(commission_agent.0);
		let commission_data = input.read::<Bytes>(gasometer)?;

		log::debug!(target: "nftmart-evm", "offerId: {:?}", &offer_id);
		log::debug!(target: "nftmart-evm", "offerOwner: {:?}", &offer_owner);
		log::debug!(target: "nftmart-evm", "commissionAgent: {:?}", &commission_agent);
		log::debug!(target: "nftmart-evm", "commissionData: {:?}", &commission_data);

		let call = OrderCall::<T>::take_offer {
			offer_id: offer_id.into(),
			offer_owner: <T as frame_system::Config>::Lookup::unlookup(offer_owner),
			commission_agent: Some(commission_agent),
			commission_data: Some(commission_data.into()),
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
