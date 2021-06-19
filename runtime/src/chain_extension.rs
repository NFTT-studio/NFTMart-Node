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
use sp_runtime::{DispatchError, AccountId32};
use nftmart_traits::{Properties, BitFlags, ClassProperty};

/// Contract extension for `FetchRandom`
pub struct FetchRandomExtension;
use super::Runtime;

pub fn to_account_id(account: &[u8]) -> AccountId32 {
	use core::convert::TryFrom;
	AccountId32::try_from(account).unwrap()
}

impl ChainExtension<Runtime> for FetchRandomExtension {
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
				let random_seed = super::RandomnessCollectiveFlip::random_seed().0;
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
				let caller = to_account_id(caller.as_ref());
				let (metadata, name, description, properties): (_, _, _, u8) = env.read_as()?;
				let p = Properties(<BitFlags<ClassProperty>>::from_bits(properties).map_err(|_| "invalid class properties value")?);
				let (owner, class_id) = super::Nftmart::do_create_class(&caller, metadata, name, description, p).map_err(|e| e.error)?;
				let r = (owner, class_id).encode();
				env.write(&r, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to return result from do_create_class"))?;
			}

			2003 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				let caller = to_account_id(caller.as_ref());
				let (to, class_id, metadata, quantity, charge_royalty) = env.read_as()?;
				let (class_owner, beneficiary, class_id, token_id, quantity) =
					super::Nftmart::do_proxy_mint(&caller, &to, class_id, metadata, quantity, charge_royalty).map_err(|e| e.error)?;
				let r = (class_owner, beneficiary, class_id, token_id, quantity).encode();
				env.write(&r, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to return result from do_proxy_mint"))?;
			}

			2004 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				let caller = to_account_id(caller.as_ref());
				let (to, class_id, token_id, quantity) = env.read_as()?;
				super::Nftmart::do_transfer(&caller, &to, class_id, token_id, quantity)?;
				let r = ().encode();
				env.write(&r, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to return result from do_transfer"))?;
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
