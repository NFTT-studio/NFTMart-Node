// Copyright 2019-2021 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

use alloc::collections::BTreeMap;
use fp_evm::Context;
use frame_system_precompiles::FrameSystemWrapper;
use nftmart_auction_precompiles::NftmartAuctionPrecompile;
use nftmart_nft_precompiles::NftmartNftPrecompile;
use nftmart_order_precompiles::NftmartOrderPrecompile;
use pallet_evm::{Precompile, PrecompileResult};
use pallet_evm_precompile_balances_erc20::{Erc20BalancesPrecompile, Erc20Metadata};
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use pallet_identity_precompiles::PalletIdentityWrapper;
use pallet_nop_precompiles::PalletNopPrecompile;
use pallet_staking_precompiles::PalletStakingWrapper;
use pallet_template_precompiles::PalletTemplatePrecompile;
use sp_core::H160;
use sp_std::marker::PhantomData;
use withdraw_balance_precompiles::WithdrawBalancePrecompile;

/// We include the nine Istanbul precompiles
/// <https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69>
/// as well as a special precompile for dispatching Substrate extrinsics
pub struct NftmartPrecompiles<R>(PhantomData<R>);

/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither NFTMart specific
/// 2048-4095 NFTMart specific precompiles
impl<R> NftmartPrecompiles<R>
where
	R: pallet_evm::Config
		+ pallet_template::Config
		+ pallet_staking::Config
		+ pallet_identity::Config,
	Dispatch<R>: Precompile,
	PalletTemplatePrecompile<R>: Precompile,
	PalletNopPrecompile<R>: Precompile,
	PalletStakingWrapper<R>: Precompile,
	PalletIdentityWrapper<R>: Precompile,
	FrameSystemWrapper<R>: Precompile,
	WithdrawBalancePrecompile<R>: Precompile,
	NftmartNftPrecompile<R>: Precompile,
	NftmartOrderPrecompile<R>: Precompile,
	NftmartAuctionPrecompile<R>: Precompile,
	Erc20BalancesPrecompile<R, NativeErc20Metadata>: Precompile,
{
	pub fn new() -> BTreeMap<H160, PrecompileFn> {
		let mut pset = BTreeMap::<H160, PrecompileFn>::new();
		// Ethereum precompiles :
		pset.insert(hash(0x0000000000000000000000000000000000000001), ECRecover::execute);
		pset.insert(hash(0x0000000000000000000000000000000000000002), Sha256::execute);
		pset.insert(hash(0x0000000000000000000000000000000000000003), Ripemd160::execute);
		pset.insert(hash(0x0000000000000000000000000000000000000004), Identity::execute);
		pset.insert(hash(0x0000000000000000000000000000000000000005), Modexp::execute);
		pset.insert(hash(0x0000000000000000000000000000000000000006), Bn128Add::execute);
		pset.insert(hash(0x0000000000000000000000000000000000000007), Bn128Mul::execute);
		pset.insert(hash(0x0000000000000000000000000000000000000008), Bn128Pairing::execute);
		pset.insert(hash(0x0000000000000000000000000000000000000009), Blake2F::execute);
		// Non-NFTMart specific nor Ethereum precompiles :
		pset.insert(hash(0x0000000000000000000000000000000000000400), Sha3FIPS256::execute);
		pset.insert(hash(0x0000000000000000000000000000000000000401), Dispatch::<R>::execute);
		pset.insert(hash(0x0000000000000000000000000000000000000402), ECRecoverPublicKey::execute);
		// NFTMart specific precompiles :
		pset.insert(
			hash(0x0000000000000000000000000000000000000800),
			PalletTemplatePrecompile::<R>::execute,
		);
		pset.insert(
			hash(0x0000000000000000000000000000000000000801),
			WithdrawBalancePrecompile::<R>::execute,
		);
		pset.insert(
			hash(0x0000000000000000000000000000000000000802),
			Erc20BalancesPrecompile::<R, NativeErc20Metadata>::execute,
		);
		pset.insert(
			hash(0x0000000000000000000000000000000000000803),
			NftmartNftPrecompile::<R>::execute,
		);
		pset.insert(
			hash(0x0000000000000000000000000000000000000804),
			NftmartOrderPrecompile::<R>::execute,
		);
		pset.insert(
			hash(0x0000000000000000000000000000000000000805),
			NftmartAuctionPrecompile::<R>::execute,
		);
		pset.insert(
			hash(0x0000000000000000000000000000000000000806),
			PalletNopPrecompile::<R>::execute,
		);
		pset.insert(
			hash(0x0000000000000000000000000000000000000807),
			PalletIdentityWrapper::<R>::execute,
		);
		pset.insert(
			hash(0x0000000000000000000000000000000000000808),
			PalletStakingWrapper::<R>::execute,
		);
		pset.insert(
			hash(0x0000000000000000000000000000000000000809),
			FrameSystemWrapper::<R>::execute,
		);
		pset
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

/// ERC20 metadata for the native token.
pub struct NativeErc20Metadata;

impl Erc20Metadata for NativeErc20Metadata {
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

pub type PrecompileFn = fn(&[u8], Option<u64>, &Context, bool) -> PrecompileResult;
