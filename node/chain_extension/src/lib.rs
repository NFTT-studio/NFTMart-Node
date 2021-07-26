#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::log::{
	error,
	trace,
};
use frame_support::traits::Randomness;
use pallet_contracts::chain_extension::{
	ChainExtension,
	Environment,
	Ext,
	InitState,
	RetVal,
	SysConfig,
	UncheckedFrom,
};
use sp_runtime::{DispatchError, AccountId32, traits::Verify, PerU16};
use nftmart_traits::{Properties, BitFlags, ClassProperty, Signature, ClassId, TokenId};
use core::convert::TryFrom;
use sp_std::vec::Vec;

pub struct NftmartExtension<Runtime>(sp_std::marker::PhantomData<Runtime>);

pub fn to_account_id(account: &[u8]) -> Result<AccountId32, DispatchError> {
	AccountId32::try_from(account).map_err(|_|DispatchError::Other("Cannot be converted into AccountId32"))
}

fn sr25519_signature(sign: &[u8]) -> Result<Signature, DispatchError> {
	if let Ok(signature) = sp_core::sr25519::Signature::try_from(sign) {
		Ok(signature.into())
	} else {
		Err(DispatchError::Other("Not a sr25519 signature"))
	}
}

impl<Runtime> ChainExtension<Runtime> for NftmartExtension<Runtime> where
	Runtime: frame_system::Config<AccountId = AccountId32>,
	Runtime: pallet_contracts::Config,
	Runtime: pallet_randomness_collective_flip::Config,
	Runtime: nftmart_nft::Config<ClassId = ClassId, TokenId = TokenId>,
{
	fn call<E: Ext>(
		func_id: u32,
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError>
		where
			<E::T as SysConfig>::AccountId:
			UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			2001 => {
				let mut env = env.buf_in_buf_out();
				let random_seed = pallet_randomness_collective_flip::Pallet::<Runtime>::random_seed().0;
				let random_slice = random_seed.encode();
				trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);
				env.write(&random_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call random")
				})?;
			}

			2002 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				let caller: <Runtime as SysConfig>::AccountId = to_account_id(caller.as_ref())?;
				let (metadata, name, description, properties): (_, _, _, u8) = env.read_as_unbounded(env.in_len())?;
				let p = Properties(<BitFlags<ClassProperty>>::from_bits(properties).map_err(|_| "invalid class properties value")?);
				let (owner, class_id) = nftmart_nft::Pallet::<Runtime>::do_create_class(&caller, metadata, name, description, PerU16::zero(), p).map_err(|e| e.error)?;
				let r = (owner, class_id).encode();
				env.write(&r, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to return result from do_create_class"))?;
			}

			2003 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				let caller = to_account_id(caller.as_ref())?;
				let (to, class_id, metadata, quantity, charge_royalty) = env.read_as_unbounded(env.in_len())?;
				let (class_owner, beneficiary, class_id, token_id, quantity) =
					nftmart_nft::Pallet::<Runtime>::do_proxy_mint(&caller, &to, class_id, metadata, quantity, charge_royalty).map_err(|e| e.error)?;
				let r = (class_owner, beneficiary, class_id, token_id, quantity).encode();
				env.write(&r, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to return result from do_proxy_mint"))?;
			}

			2004 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				let caller: AccountId32 = to_account_id(caller.as_ref())?;
				let (to, class_id, token_id, quantity) = env.read_as()?;
				nftmart_nft::Pallet::<Runtime>::do_transfer(&caller, &to, class_id, token_id, quantity)?;
				let r = ().encode();
				env.write(&r, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to return result from do_transfer"))?;
			}

			// ################## read only ##################

			1001 => {
				let mut env = env.buf_in_buf_out();
				let (class_id, token_id) = env.read_as()?;
				let r = nftmart_nft::Pallet::<Runtime>::contract_tokens(class_id, token_id);
				env.write(&r.encode(), false, None)?;
			}

			1101 => {
				let mut env = env.buf_in_buf_out();
				let (account_id, signature, msg): (AccountId32, Vec<u8>, Vec<u8>) = env.read_as_unbounded(env.in_len())?;
				let s = sr25519_signature(&signature[..])?;
				let r = s.verify(&msg[..], &account_id);
				env.write(&r.encode(), false, None)?;
			}

			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"))
			}
		}
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
