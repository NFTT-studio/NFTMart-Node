//! # Non Fungible Token
//! The module provides implementations for non-fungible-token.
//!
//! - [`Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! This module provides basic functions to create and manager
//! NFT(non fungible token) such as `create_class`, `transfer`, `mint`, `burn`.

//! ### Module Functions
//!
//! - `create_class` - Create NFT(non fungible token) class
//! - `transfer` - Transfer NFT(non fungible token) to another account.
//! - `mint` - Mint NFT(non fungible token)
//! - `burn` - Burn NFT(non fungible token)
//! - `destroy_class` - Destroy NFT(non fungible token) class

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{ensure, pallet_prelude::*, Parameter};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, Zero},
	DispatchError, DispatchResult,
};
use sp_std::vec::Vec;
pub use orml_traits::nft::{TokenInfo, ClassInfo, AccountToken};

mod mock;
mod tests;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The class ID type
		type ClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + codec::FullCodec;
		/// The token ID type
		type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + codec::FullCodec;
		/// The class properties type
		type ClassData: Parameter + Member + MaybeSerializeDeserialize;
		/// The token properties type
		type TokenData: Parameter + Member + MaybeSerializeDeserialize;
	}

	pub type ClassInfoOf<T> =
		ClassInfo<<T as Config>::TokenId, <T as frame_system::Config>::AccountId, <T as Config>::ClassData>;
	pub type TokenInfoOf<T> = TokenInfo<<T as Config>::TokenId, <T as Config>::TokenData>;

	pub type GenesisTokenData<T> = (
		<T as frame_system::Config>::AccountId, // Token owner
		Vec<u8>,                                // Token metadata
		<T as Config>::TokenData,
	);
	pub type GenesisTokens<T> = (
		<T as frame_system::Config>::AccountId, // Token class owner
		Vec<u8>,                                // Token class metadata
		<T as Config>::ClassData,
		Vec<GenesisTokenData<T>>, // Vector of tokens belonging to this class
	);

	/// Error for non-fungible-token module.
	#[pallet::error]
	pub enum Error<T> {
		/// No available class ID
		NoAvailableClassId,
		/// No available token ID
		NoAvailableTokenId,
		/// Token(ClassId, TokenId) not found
		TokenNotFound,
		/// Class not found
		ClassNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Arithmetic calculation overflow
		NumOverflow,
		/// Can not destroy class
		/// Total issuance is not 0
		CannotDestroyClass,
	}

	/// Next available class ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextClassId<T: Config> = StorageValue<_, T::ClassId, ValueQuery>;

	/// Next available token ID.
	#[pallet::storage]
	#[pallet::getter(fn next_token_id)]
	pub type NextTokenId<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, T::TokenId, ValueQuery>;

	/// Store class info.
	///
	/// Returns `None` if class info not set or removed.
	#[pallet::storage]
	#[pallet::getter(fn classes)]
	pub type Classes<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, ClassInfoOf<T>>;

	/// Store token info.
	///
	/// Returns `None` if token info not set or removed.
	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	pub type Tokens<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::TokenId, TokenInfoOf<T>>;

	/// Token existence check by owner and class ID.
	///         k1                k2               value
	/// map: AccountId -> (classId, tokenId) -> token_count.
	#[pallet::storage]
	#[pallet::getter(fn tokens_by_owner)]
	pub type TokensByOwner<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, (T::ClassId, T::TokenId), AccountToken<<T as Config>::TokenId>>;

	/// An index to query owners by token
	#[pallet::storage]
	#[pallet::getter(fn owners_by_token)]
	pub type OwnersByToken<T: Config> =
		StorageDoubleMap<_, Twox64Concat, (T::ClassId, T::TokenId), Twox64Concat, T::AccountId, ()>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub tokens: Vec<GenesisTokens<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { tokens: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.tokens.iter().for_each(|token_class| {
				let class_id = Pallet::<T>::create_class(&token_class.0, token_class.1.to_vec(), token_class.2.clone())
					.expect("Create class cannot fail while building genesis");
				for (account_id, token_metadata, token_data) in &token_class.3 {
					Pallet::<T>::mint(account_id, class_id, token_metadata.to_vec(), token_data.clone(), One::one())
						.expect("Token mint cannot fail during genesis");
				}
			})
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	/// Create NFT(non fungible token) class
	pub fn create_class(
		owner: &T::AccountId,
		metadata: Vec<u8>,
		data: T::ClassData,
	) -> Result<T::ClassId, DispatchError> {
		let class_id = NextClassId::<T>::try_mutate(|id| -> Result<T::ClassId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableClassId)?;
			Ok(current_id)
		})?;

		let info = ClassInfo {
			metadata,
			total_issuance: Zero::zero(),
			owner: owner.clone(),
			data,
		};
		Classes::<T>::insert(class_id, info);

		Ok(class_id)
	}

	/// Transfer NFT(non fungible token) from `from` account to `to` account
	pub fn transfer(from: &T::AccountId, to: &T::AccountId, token: (T::ClassId, T::TokenId), quantity: T::TokenId) -> Result<bool, DispatchError> {
		if from == to || quantity.is_zero() {
			// no change needed
			return Ok(false)
		}
		TokensByOwner::<T>::try_mutate_exists(from, token, |maybe_from_count| -> Result<bool, DispatchError> {
			let mut from_count: AccountToken<T::TokenId> = maybe_from_count.unwrap_or_default();
			from_count.quantity = from_count.quantity.checked_sub(&quantity).ok_or(Error::<T>::NumOverflow)?;

			TokensByOwner::<T>::try_mutate_exists(to, token, |maybe_to_count| -> DispatchResult {
				match maybe_to_count {
					Some(to_count) => {
						to_count.quantity = to_count.quantity.checked_add(&quantity).ok_or(Error::<T>::NumOverflow)?;
					}
					None => {
						*maybe_to_count = Some(AccountToken::new(quantity));
						OwnersByToken::<T>::insert(token, to, ());
					}
				}
				Ok(())
			})?;

			if from_count.is_zero() {
				*maybe_from_count = None;
				OwnersByToken::<T>::remove(token, from);
			} else {
				*maybe_from_count = Some(from_count);
			}
			Ok(true)
		})
	}

	/// Mint NFT(non fungible token) to `owner`
	pub fn mint(
		owner: &T::AccountId,
		class_id: T::ClassId,
		metadata: Vec<u8>,
		data: T::TokenData,
		quantity: T::TokenId,
	) -> Result<T::TokenId, DispatchError> {
		NextTokenId::<T>::try_mutate(class_id, |id| -> Result<T::TokenId, DispatchError> {
			let token_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableTokenId)?;

			Classes::<T>::try_mutate(class_id, |class_info| -> DispatchResult {
				let info = class_info.as_mut().ok_or(Error::<T>::ClassNotFound)?;
				info.total_issuance = info.total_issuance.checked_add(&quantity).ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;

			let token_info = TokenInfo {
				metadata,
				data,
				quantity,
			};
			Tokens::<T>::insert(class_id, token_id, token_info);
			TokensByOwner::<T>::insert(owner, (class_id, token_id), AccountToken::new(quantity));
			OwnersByToken::<T>::insert((class_id, token_id), owner, ());
			Ok(token_id)
		})
	}

	/// Burn NFT(non fungible token) from `owner`
	#[frame_support::transactional]
	pub fn burn(owner: &T::AccountId, token: (T::ClassId, T::TokenId), quantity: T::TokenId) -> Result<Option<TokenInfoOf<T>>, DispatchError> {
		if quantity.is_zero() {
			// no change needed
			return Ok(None)
		}
		TokensByOwner::<T>::try_mutate_exists(owner, token, |maybe_owner_count| -> Result<Option<TokenInfoOf<T>>, DispatchError> {
			Classes::<T>::try_mutate(token.0, |class_info| -> DispatchResult {
				let class_info = class_info.as_mut().ok_or(Error::<T>::ClassNotFound)?;
				class_info.total_issuance = class_info.total_issuance.checked_sub(&quantity).ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;

			let c = Tokens::<T>::try_mutate_exists(token.0, token.1, |maybe_token_info| -> Result<Option<TokenInfoOf<T>>, DispatchError> {
				let token_info = maybe_token_info.as_mut().ok_or(Error::<T>::TokenNotFound)?;
				token_info.quantity = token_info.quantity.checked_sub(&quantity).ok_or(Error::<T>::NumOverflow)?;
				let deep_copy = token_info.clone();
				if token_info.quantity.is_zero() {
					*maybe_token_info = None;
				}
				Ok(Some(deep_copy))
			})?;

			let mut owner_count = maybe_owner_count.unwrap_or_default();
			owner_count.quantity = owner_count.quantity.checked_sub(&quantity).ok_or(Error::<T>::NumOverflow)?;
			if owner_count.is_zero() {
				*maybe_owner_count = None;
				OwnersByToken::<T>::remove(token, owner);
			} else {
				*maybe_owner_count = Some(owner_count);
			}
			Ok(c)
		})
	}

	/// Destroy NFT(non fungible token) class
	pub fn destroy_class(owner: &T::AccountId, class_id: T::ClassId) -> DispatchResult {
		Classes::<T>::try_mutate_exists(class_id, |class_info| -> DispatchResult {
			let info = class_info.take().ok_or(Error::<T>::ClassNotFound)?;
			ensure!(info.owner == *owner, Error::<T>::NoPermission);
			ensure!(info.total_issuance == Zero::zero(), Error::<T>::CannotDestroyClass);

			NextTokenId::<T>::remove(class_id);

			Ok(())
		})
	}

	pub fn is_owner(account: &T::AccountId, token: (T::ClassId, T::TokenId)) -> bool {
		Self::tokens_by_owner(account, token).unwrap_or_default().total() >= One::one()
	}

	pub fn reserve(owner: &T::AccountId, token: (T::ClassId, T::TokenId), quantity: T::TokenId) -> DispatchResult {
		TokensByOwner::<T>::try_mutate_exists(owner, token, |maybe_owner| -> DispatchResult {
			let mut owner = maybe_owner.unwrap_or_default();
			owner.quantity = owner.quantity.checked_sub(&quantity).ok_or(Error::<T>::NumOverflow)?;
			owner.reserved = owner.reserved.checked_add(&quantity).ok_or(Error::<T>::NumOverflow)?;
			*maybe_owner = Some(owner);
			Ok(())
		})
	}

	pub fn unreserve(owner: &T::AccountId, token: (T::ClassId, T::TokenId), quantity: T::TokenId) -> DispatchResult {
		TokensByOwner::<T>::try_mutate_exists(owner, token, |maybe_owner| -> DispatchResult {
			let mut owner = maybe_owner.unwrap_or_default();
			owner.reserved = owner.reserved.checked_sub(&quantity).ok_or(Error::<T>::NumOverflow)?;
			owner.quantity = owner.quantity.checked_add(&quantity).ok_or(Error::<T>::NumOverflow)?;
			*maybe_owner = Some(owner);
			Ok(())
		})
	}
}
