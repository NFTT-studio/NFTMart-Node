#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

// use sp_std::fmt::Display;

use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::StaticLookup,
};
use nftmart_auction::Call as AuctionCall;
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
	BidBritishAuction = "bidBritishAuction(uint256,bytes32,uint256,bytes32,string)",
	BidDutchAuction = "bidDutchAuction(uint256,bytes32,uint256,bytes32,string)",
	RemoveBritishAuction = "removeBritishAuction(uint256)",
	RemoveDutchAuction = "removeDutchAuction(uint256)",
	SubmitBritishAuction = "submitBritishAuction(uint256,uint256,uint256,uint256,uint256,uint256,bool,uint256[3][],uint256)",
	SubmitDutchAuction = "submitDutchAuction(uint256,uint256,uint256,uint256,uint256,uint256,bool,uint256[3][],uint256)",
}

pub struct NftmartAuctionPrecompile<T>(PhantomData<T>);

impl<T> Precompile for NftmartAuctionPrecompile<T>
where
	T: pallet_evm::Config + nftmart_auction::Config + orml_nft::Config,
	<T as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + From<AuctionCall<T>>,
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
			Action::BidBritishAuction =>
				Self::bid_british_auction(&mut input, &mut gasometer, context),
			Action::BidDutchAuction => Self::bid_dutch_auction(&mut input, &mut gasometer, context),
			Action::RemoveBritishAuction =>
				Self::remove_british_auction(&mut input, &mut gasometer, context),
			Action::RemoveDutchAuction =>
				Self::remove_dutch_auction(&mut input, &mut gasometer, context),
			/*
			Action::SubmitBritishAuction => Self::submit_british_auction(&mut input, &mut gasometer, context),
			Action::SubmitDutchAuction => Self::submit_dutch_auction(&mut input, &mut gasometer, context),
			*/
			_ => Self::bid_british_auction(&mut input, &mut gasometer, context),
		}
	}
}

/// Alias for the Balance type for the provided Runtime and Instance.
// pub type BalanceOf<Runtime> = <Runtime as pallet_balances::Config>::Balance;
pub type AccountIdOf<Runtime> = <Runtime as frame_system::Config>::AccountId;
pub type BalanceOf<Runtime> =
	<<Runtime as pallet_evm::Config>::Currency as Currency<AccountIdOf<Runtime>>>::Balance;

impl<T> NftmartAuctionPrecompile<T>
where
	T: pallet_evm::Config + nftmart_auction::Config + orml_nft::Config,
	<T as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + From<AuctionCall<T>>,
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
	fn bid_british_auction(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		input.expect_arguments(gasometer, 5)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let price: u32 = input.read::<u32>(gasometer)?.into();
		let auction_owner = input.read::<H256>(gasometer)?;
		let auction_owner: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(auction_owner.0);
		let auction_id: u32 = input.read::<u32>(gasometer)?.into();
		let commission_agent = input.read::<H256>(gasometer)?;
		let commission_agent: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(commission_agent.0);
		let commission_data = input.read::<Bytes>(gasometer)?;

		log::debug!(target: "nftmart-evm", "price: {:?}", &price);
		log::debug!(target: "nftmart-evm", "auctionOwner: {:?}", &auction_owner);
		log::debug!(target: "nftmart-evm", "auctionId: {:?}", &auction_id);
		log::debug!(target: "nftmart-evm", "commissionAgent: {:?}", &commission_agent);
		log::debug!(target: "nftmart-evm", "commissionData: {:?}", &commission_data);

		let call = AuctionCall::<T>::bid_british_auction {
			price: price.into(),
			auction_owner: <T as frame_system::Config>::Lookup::unlookup(auction_owner),
			auction_id: auction_id.into(),
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

	fn bid_dutch_auction(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		input.expect_arguments(gasometer, 5)?;

		let origin: <T as frame_system::pallet::Config>::AccountId =
			T::AddressMapping::into_account_id(context.caller);
		if_std! {
				println!("The caller account is: {:#?}", context.caller);
				println!("The caller origin is: {:#?}", origin);
		}

		log::debug!(target: "nftmart-evm", "from(evm): {:?}", &origin);

		let price: u32 = input.read::<u32>(gasometer)?.into();
		let auction_owner = input.read::<H256>(gasometer)?;
		let auction_owner: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(auction_owner.0);
		let auction_id: u32 = input.read::<u32>(gasometer)?.into();
		let commission_agent = input.read::<H256>(gasometer)?;
		let commission_agent: <T as frame_system::Config>::AccountId =
			<T as frame_system::Config>::AccountId::from(commission_agent.0);
		let commission_data = input.read::<Bytes>(gasometer)?;

		log::debug!(target: "nftmart-evm", "price: {:?}", &price);
		log::debug!(target: "nftmart-evm", "auctionOwner: {:?}", &auction_owner);
		log::debug!(target: "nftmart-evm", "auctionId: {:?}", &auction_id);
		log::debug!(target: "nftmart-evm", "commissionAgent: {:?}", &commission_agent);
		log::debug!(target: "nftmart-evm", "commissionData: {:?}", &commission_data);

		let call = AuctionCall::<T>::bid_dutch_auction {
			price: price.into(),
			auction_owner: <T as frame_system::Config>::Lookup::unlookup(auction_owner),
			auction_id: auction_id.into(),
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

	fn remove_british_auction(
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

		let auction_id: u32 = input.read::<u32>(gasometer)?.into();

		log::debug!(target: "nftmart-evm", "auctionId: {:?}", &auction_id);

		let call = AuctionCall::<T>::remove_british_auction { auction_id: auction_id.into() };

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn remove_dutch_auction(
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

		let auction_id: u32 = input.read::<u32>(gasometer)?.into();

		log::debug!(target: "nftmart-evm", "auctionId: {:?}", &auction_id);

		let call = AuctionCall::<T>::remove_dutch_auction { auction_id: auction_id.into() };

		RuntimeHelper::<T>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
