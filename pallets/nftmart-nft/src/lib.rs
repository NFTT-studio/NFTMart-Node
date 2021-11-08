#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::KeepAlive, ReservableCurrency},
	transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::{
	traits::{
		AccountIdConversion, AtLeast32BitUnsigned, Bounded, CheckedAdd, One, StaticLookup, Zero,
	},
	PerU16, RuntimeDebug, SaturatedConversion,
};
use sp_std::vec::Vec;

mod mock;
mod tests;

pub use module::*;
use nftmart_traits::*;
use orml_nft::{ClassInfoOf, TokenInfoOf};

pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
pub type AccountTokenOf<T> = AccountToken<TokenIdOf<T>>;
pub type BalanceOf<T> =
	<<T as module::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CurrencyIdOf<T> = <<T as module::Config>::MultiCurrency as MultiCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
enum Releases {
	V1_0_0,
	V2_0_0,
}

impl Default for Releases {
	fn default() -> Self {
		Releases::V2_0_0
	}
}

pub mod migrations {
	use super::*;
	pub type OldClass<T> =
		orml_nft::ClassInfo<TokenIdOf<T>, <T as frame_system::Config>::AccountId, OldClassData>;
	pub type NewClass<T> = orml_nft::ClassInfo<
		TokenIdOf<T>,
		<T as frame_system::Config>::AccountId,
		ClassData<BlockNumberOf<T>>,
	>;
	pub type OldToken<T> = orml_nft::TokenInfo<TokenIdOf<T>, OldTokenData>;
	pub type NewToken<T> = orml_nft::TokenInfo<
		TokenIdOf<T>,
		TokenData<<T as frame_system::Config>::AccountId, BlockNumberOf<T>>,
	>;

	#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
	pub struct OldClassData {
		#[codec(compact)]
		pub deposit: Balance,
		pub properties: Properties,
		pub name: Vec<u8>,
		pub description: Vec<u8>,
	}

	#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
	pub struct OldTokenData {
		#[codec(compact)]
		pub deposit: Balance,
	}

	impl OldClassData {
		#[allow(dead_code)]
		fn upgraded<T>(self) -> ClassData<T>
		where
			T: AtLeast32BitUnsigned + Bounded + Copy + From<u32>,
		{
			let create_block: T = One::one();
			ClassData {
				create_block: create_block * 2u32.into(),
				deposit: self.deposit,
				properties: self.properties,
				name: self.name,
				description: self.description,
				royalty_rate: PerU16::zero(),
				category_ids: Default::default(),
			}
		}
	}

	impl OldTokenData {
		#[allow(dead_code)]
		fn upgraded<AccountId: Clone, T>(self, who: AccountId) -> TokenData<AccountId, T>
		where
			T: AtLeast32BitUnsigned + Bounded + Copy + From<u32>,
		{
			let create_block: T = One::one();
			TokenData {
				create_block: create_block * 3u32.into(),
				deposit: self.deposit,
				royalty_rate: PerU16::from_percent(0),
				creator: who.clone(),
				royalty_beneficiary: who,
			}
		}
	}

	pub fn do_migrate<T: Config>() -> Weight {
		// migrate classes
		orml_nft::Classes::<T>::translate::<OldClass<T>, _>(|_, p: OldClass<T>| {
			let new_data: NewClass<T> = NewClass::<T> {
				metadata: p.metadata,
				total_issuance: p.total_issuance,
				owner: p.owner,
				data: p.data.upgraded::<BlockNumberOf<T>>(),
			};
			Some(new_data)
		});
		// migrate tokens
		orml_nft::Tokens::<T>::translate::<OldToken<T>, _>(|_, _, p: OldToken<T>| {
			let new_data: NewToken<T> = NewToken::<T> {
				metadata: p.metadata,
				data: p.data.upgraded::<<T as frame_system::Config>::AccountId, BlockNumberOf<T>>(
					Default::default(),
				),
				quantity: p.quantity,
			};
			Some(new_data)
		});
		// migrate account_data.
		// ...
		// ...
		T::BlockWeights::get().max_block
	}
}

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_nft::Config<
			ClassData = ClassData<BlockNumberOf<Self>>,
			TokenData = TokenData<<Self as frame_system::Config>::AccountId, BlockNumberOf<Self>>,
		> + pallet_proxy::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Extra Configurations
		type ExtraConfig: NftmartConfig<Self::AccountId, BlockNumberFor<Self>>;

		/// The minimum balance to create class
		#[pallet::constant]
		type CreateClassDeposit: Get<Balance>;

		/// The amount of balance that must be deposited per byte of metadata.
		#[pallet::constant]
		type MetaDataByteDeposit: Get<Balance>;

		/// The minimum balance to create token
		#[pallet::constant]
		type CreateTokenDeposit: Get<Balance>;

		/// The NFT's module id
		#[pallet::constant]
		type ModuleId: Get<PalletId>;

		/// MultiCurrency type for trading
		type MultiCurrency: MultiReservableCurrency<Self::AccountId, Balance = Balance>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// ClassId not found
		ClassIdNotFound,
		/// TokenId not found
		TokenIdNotFound,
		/// Category not found
		CategoryNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Quantity is invalid. need >= 1
		InvalidQuantity,
		/// Invalid deadline
		InvalidDeadline,
		/// Invalid deposit
		InvalidDeposit,
		/// Property of class don't support transfer
		NonTransferable,
		/// Property of class don't support burn
		NonBurnable,
		/// Can not destroy class Total issuance is not 0
		CannotDestroyClass,
		/// No available category ID
		NoAvailableCategoryId,
		/// NameTooLong
		NameTooLong,
		/// DescriptionTooLong
		DescriptionTooLong,
		/// account not in whitelist
		AccountNotInWhitelist,
		CategoryOutOfBound,
		DuplicatedCategories,
		RoyaltyRateTooHigh,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Created NFT class. \[owner, class_id\]
		CreatedClass(T::AccountId, ClassIdOf<T>),
		/// Updated NFT class. \[owner, class_id\]
		UpdatedClass(T::AccountId, ClassIdOf<T>),
		/// Minted NFT token. \[from, to, class_id, token_id, quantity\]
		MintedToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>),
		/// Transferred NFT token. \[from, to, class_id, token_id, quantity\]
		TransferredToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>),
		/// Burned NFT token. \[owner, class_id, token_id, quantity, unreserved\]
		BurnedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>, Balance),
		/// Destroyed NFT class. \[owner, class_id, dest\]
		DestroyedClass(T::AccountId, ClassIdOf<T>, T::AccountId),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			if StorageVersion::<T>::get() == Releases::V1_0_0 {
				StorageVersion::<T>::put(Releases::V2_0_0);
				migrations::do_migrate::<T>()
			} else {
				0
			}
		}

		fn integrity_test() {}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub classes: Vec<ClassConfig<ClassIdOf<T>, T::AccountId, TokenIdOf<T>>>,
		pub _phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { classes: Default::default(), _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<StorageVersion<T>>::put(Releases::default());
			let mut max_class_id = Zero::zero();
			// Initialize classes.
			for ClassConfig {
				class_id,
				class_metadata,
				category_ids,
				name,
				description,
				royalty_rate,
				properties,
				admins,
				tokens,
			} in &self.classes
			{
				let class_metadata: NFTMetadata = class_metadata.as_bytes().to_vec();
				let name: NFTMetadata = name.as_bytes().to_vec();
				let description: NFTMetadata = description.as_bytes().to_vec();
				let properties =
					Properties(<BitFlags<ClassProperty>>::from_bits(*properties).unwrap());
				assert!(orml_nft::Classes::<T>::get(*class_id).is_none(), "Dup class id");
				orml_nft::NextClassId::<T>::set(*class_id);
				let owner: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
				let (deposit, all_deposit) = Pallet::<T>::create_class_deposit_num_proxies(
					class_metadata.len().saturated_into(),
					name.len().saturated_into(),
					description.len().saturated_into(),
					admins.len() as u32,
				);
				<T as Config>::Currency::deposit_creating(&owner, all_deposit.saturated_into());
				<T as Config>::Currency::reserve(&owner, deposit.saturated_into()).unwrap();
				for admin in admins {
					<pallet_proxy::Pallet<T>>::add_proxy_delegate(
						&owner,
						admin.clone(),
						Default::default(),
						Zero::zero(),
					)
					.unwrap();
				}
				let data: ClassData<BlockNumberOf<T>> = ClassData {
					deposit,
					properties,
					name: name.clone(),
					description: description.clone(),
					create_block: <frame_system::Pallet<T>>::block_number(),
					royalty_rate: *royalty_rate,
					category_ids: category_ids.clone(),
				};
				for category_id in category_ids {
					T::ExtraConfig::inc_count_in_category(*category_id).unwrap();
				}
				orml_nft::Pallet::<T>::create_class(&owner, class_metadata.clone(), data).unwrap();

				if max_class_id < *class_id {
					max_class_id = *class_id;
				}

				let mut max_token_id = Zero::zero();
				for TokenConfig {
					token_id,
					token_metadata,
					royalty_rate,
					token_owner,
					token_creator,
					royalty_beneficiary,
					quantity,
				} in tokens
				{
					assert!(
						orml_nft::Tokens::<T>::get(*class_id, *token_id).is_none(),
						"Dup token id"
					);
					let token_metadata: NFTMetadata = token_metadata.as_bytes().to_vec();
					let deposit =
						Pallet::<T>::mint_token_deposit(token_metadata.len().saturated_into());
					<T as Config>::Currency::deposit_creating(&owner, deposit.saturated_into());
					<T as Config>::Currency::reserve(&owner, deposit.saturated_into()).unwrap();
					let data: TokenData<T::AccountId, BlockNumberOf<T>> = TokenData {
						deposit,
						create_block: <frame_system::Pallet<T>>::block_number(),
						royalty_rate: *royalty_rate,
						creator: token_creator.clone(),
						royalty_beneficiary: royalty_beneficiary.clone(),
					};
					orml_nft::NextTokenId::<T>::insert(*class_id, *token_id);
					orml_nft::Pallet::<T>::mint(
						token_owner,
						*class_id,
						token_metadata.clone(),
						data,
						*quantity,
					)
					.unwrap();

					if max_token_id < *token_id {
						max_token_id = *token_id;
					}
				}
				orml_nft::NextTokenId::<T>::insert(
					*class_id,
					max_token_id.checked_add(&One::one()).unwrap(),
				);
			}
			orml_nft::NextClassId::<T>::set(max_class_id.checked_add(&One::one()).unwrap());
		}
	}

	/// Storage version of the pallet.
	#[pallet::storage]
	pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create NFT class, tokens belong to the class.
		///
		/// - `metadata`: external metadata
		/// - `properties`: class property, include `Transferable` `Burnable`
		/// - `name`: class name, with len limitation.
		/// - `description`: class description, with len limitation.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn create_class(
			origin: OriginFor<T>,
			metadata: NFTMetadata,
			name: Vec<u8>,
			description: Vec<u8>,
			#[pallet::compact] royalty_rate: PerU16,
			properties: Properties,
			category_ids: Vec<GlobalId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_create_class(
				&who,
				metadata,
				name,
				description,
				royalty_rate,
				properties,
				category_ids,
			)?;
			Ok(().into())
		}

		/// Update NFT class.
		///
		/// - `class_id`: class id
		/// - `metadata`: external metadata
		/// - `properties`: class property, include `Transferable` `Burnable`
		/// - `name`: class name, with len limitation.
		/// - `description`: class description, with len limitation.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn update_class(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			metadata: NFTMetadata,
			name: Vec<u8>,
			description: Vec<u8>,
			#[pallet::compact] royalty_rate: PerU16,
			properties: Properties,
			category_ids: Vec<GlobalId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_update_class(
				&who,
				class_id,
				metadata,
				name,
				description,
				royalty_rate,
				properties,
				category_ids,
			)?;
			Ok(().into())
		}

		/// Update token royalty.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn update_token_royalty(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
			charge_royalty: Option<PerU16>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			orml_nft::Tokens::<T>::try_mutate(
				class_id,
				token_id,
				|maybe_token| -> DispatchResultWithPostInfo {
					let token_info: &mut TokenInfoOf<T> =
						maybe_token.as_mut().ok_or(Error::<T>::TokenIdNotFound)?;
					ensure!(who == token_info.data.royalty_beneficiary, Error::<T>::NoPermission);
					ensure!(
						orml_nft::Pallet::<T>::total_count(&who, (class_id, token_id)) ==
							token_info.quantity,
						Error::<T>::NoPermission
					);

					token_info.data.royalty_rate = charge_royalty
						.ok_or_else(|| -> Result<PerU16, DispatchError> {
							let class_info: ClassInfoOf<T> =
								orml_nft::Pallet::<T>::classes(class_id)
									.ok_or(Error::<T>::ClassIdNotFound)?;
							Ok(class_info.data.royalty_rate)
						})
						.or_else(core::convert::identity)?;
					Ok(().into())
				},
			)
		}

		/// Update token royalty beneficiary.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn update_token_royalty_beneficiary(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
			to: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			orml_nft::Tokens::<T>::try_mutate(
				class_id,
				token_id,
				|maybe_token| -> DispatchResultWithPostInfo {
					let token_info: &mut TokenInfoOf<T> =
						maybe_token.as_mut().ok_or(Error::<T>::TokenIdNotFound)?;
					ensure!(who == token_info.data.royalty_beneficiary, Error::<T>::NoPermission);
					let to = T::Lookup::lookup(to)?;
					token_info.data.royalty_beneficiary = to;
					Ok(().into())
				},
			)
		}

		/// Mint NFT token
		///
		/// - `to`: the token owner's account
		/// - `class_id`: token belong to the class id
		/// - `metadata`: external metadata
		/// - `quantity`: token quantity
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] class_id: ClassIdOf<T>,
			metadata: NFTMetadata,
			#[pallet::compact] quantity: TokenIdOf<T>,
			charge_royalty: Option<PerU16>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			let class_info: ClassInfoOf<T> =
				orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			let _ = Self::do_mint(
				&who,
				&to,
				&class_info,
				class_id,
				metadata,
				quantity,
				charge_royalty,
			)?;
			Ok(().into())
		}

		/// Mint NFT token by a proxy account.
		///
		/// - `origin`: a proxy account
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn proxy_mint(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] class_id: ClassIdOf<T>,
			metadata: NFTMetadata,
			#[pallet::compact] quantity: TokenIdOf<T>,
			charge_royalty: Option<PerU16>,
		) -> DispatchResultWithPostInfo {
			let delegate = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			let _ =
				Self::do_proxy_mint(&delegate, &to, class_id, metadata, quantity, charge_royalty)?;
			Ok(().into())
		}

		/// Transfer NFT tokens to another account
		///
		/// - `to`: the token owner's account
		/// - `class_id`: class id
		/// - `token_id`: token id
		/// - `quantity`: quantity
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			items: Vec<(ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			for (class_id, token_id, quantity) in items {
				if quantity > Zero::zero() {
					Self::do_transfer(&who, &to, class_id, token_id, quantity)?;
				}
			}
			Ok(().into())
		}

		/// Burn NFT token
		///
		/// - `class_id`: class id
		/// - `token_id`: token id
		/// - `quantity`: quantity
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn burn(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
			#[pallet::compact] quantity: TokenIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_burnable(class_id)?, Error::<T>::NonBurnable);
			ensure!(quantity >= One::one(), Error::<T>::InvalidQuantity);

			if let Some(token_info) =
				orml_nft::Pallet::<T>::burn(&who, (class_id, token_id), quantity)?
			{
				if token_info.quantity.is_zero() {
					let class_owner: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
					let data: TokenData<T::AccountId, T::BlockNumber> = token_info.data;
					// `repatriate_reserved` will check `to` account exist and return `DeadAccount`.
					// `transfer` not do this check.
					<T as Config>::Currency::unreserve(&class_owner, data.deposit.saturated_into());
					<T as Config>::Currency::transfer(
						&class_owner,
						&who,
						data.deposit.saturated_into(),
						KeepAlive,
					)?;
					Self::deposit_event(Event::BurnedToken(
						who,
						class_id,
						token_id,
						quantity,
						data.deposit,
					));
				} else {
					Self::deposit_event(Event::BurnedToken(who, class_id, token_id, quantity, 0));
				}
			}
			Ok(().into())
		}

		/// Destroy NFT class
		///
		/// - `class_id`: destroy class id
		/// - `dest`: transfer reserve balance from sub_account to dest
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn destroy_class(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			let class_info =
				orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
            let existing_nft_number = orml_nft::Tokens::<T>::iter_prefix_values(class_id).count();
			ensure!(who == class_info.owner, Error::<T>::NoPermission);
			ensure!(class_info.total_issuance == Zero::zero() || existing_nft_number == 0, Error::<T>::CannotDestroyClass);

			let owner: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
			let data = class_info.data;
			// `repatriate_reserved` will check `to` account exist and return `DeadAccount`.
			// `transfer` not do this check.
			<T as Config>::Currency::unreserve(&owner, data.deposit.saturated_into());
			// At least there is one admin at this point.
			<T as Config>::Currency::transfer(
				&owner,
				&dest,
				data.deposit.saturated_into(),
				KeepAlive,
			)?;

			for category_id in data.category_ids {
				T::ExtraConfig::dec_count_in_category(category_id)?;
			}

			// transfer all free from origin to dest
			orml_nft::Pallet::<T>::destroy_class(&who, class_id)?;

			Self::deposit_event(Event::DestroyedClass(who, class_id, dest));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	#[transactional]
	#[allow(clippy::type_complexity)]
	pub fn do_proxy_mint(
		delegate: &T::AccountId,
		to: &T::AccountId,
		class_id: ClassIdOf<T>,
		metadata: NFTMetadata,
		quantity: TokenIdOf<T>,
		charge_royalty: Option<PerU16>,
	) -> ResultPost<(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)> {
		let class_info: ClassInfoOf<T> =
			orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;

		let _ = pallet_proxy::Pallet::<T>::find_proxy(&class_info.owner, delegate, None)?;
		let deposit = Self::mint_token_deposit(metadata.len().saturated_into());
		<T as Config>::Currency::transfer(
			delegate,
			&class_info.owner,
			deposit.saturated_into(),
			KeepAlive,
		)?;

		Self::do_mint(
			&class_info.owner,
			to,
			&class_info,
			class_id,
			metadata,
			quantity,
			charge_royalty,
		)
	}

	#[allow(clippy::type_complexity)]
	fn do_mint(
		who: &T::AccountId,
		to: &T::AccountId,
		class_info: &ClassInfoOf<T>,
		class_id: ClassIdOf<T>,
		metadata: NFTMetadata,
		quantity: TokenIdOf<T>,
		charge_royalty: Option<PerU16>,
	) -> ResultPost<(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)> {
		ensure!(T::ExtraConfig::is_in_whitelist(to), Error::<T>::AccountNotInWhitelist);

		ensure!(quantity >= One::one(), Error::<T>::InvalidQuantity);
		ensure!(who == &class_info.owner, Error::<T>::NoPermission);
		let deposit = Self::mint_token_deposit(metadata.len().saturated_into());

		<T as Config>::Currency::reserve(&class_info.owner, deposit.saturated_into())?;
		let data: TokenData<T::AccountId, BlockNumberOf<T>> = TokenData {
			deposit,
			create_block: <frame_system::Pallet<T>>::block_number(),
			royalty_rate: charge_royalty.unwrap_or(class_info.data.royalty_rate),
			creator: to.clone(),
			royalty_beneficiary: to.clone(),
		};

		ensure!(
			T::ExtraConfig::get_royalties_rate() >= data.royalty_rate,
			Error::<T>::RoyaltyRateTooHigh
		);

		let token_id: TokenIdOf<T> =
			orml_nft::Pallet::<T>::mint(to, class_id, metadata, data, quantity)?;

		Self::deposit_event(Event::MintedToken(
			who.clone(),
			to.clone(),
			class_id,
			token_id,
			quantity,
		));
		Ok((who.clone(), to.clone(), class_id, token_id, quantity))
	}

	#[transactional]
	pub fn do_create_class(
		who: &T::AccountId,
		metadata: NFTMetadata,
		name: Vec<u8>,
		description: Vec<u8>,
		royalty_rate: PerU16,
		properties: Properties,
		category_ids: Vec<GlobalId>,
	) -> ResultPost<(T::AccountId, ClassIdOf<T>)> {
		ensure!(T::ExtraConfig::is_in_whitelist(who), Error::<T>::AccountNotInWhitelist);
		ensure!(category_ids.len() <= MAX_CATEGORY_PER_CLASS, Error::<T>::CategoryOutOfBound);
		ensure!(category_ids.len() >= 1, Error::<T>::CategoryOutOfBound);
		if category_ids.len() == 2 {
			ensure!(category_ids[0] != category_ids[1], Error::<T>::DuplicatedCategories);
		}

		ensure!(name.len() <= 20, Error::<T>::NameTooLong); // TODO: pass configurations from runtime configuration.
		ensure!(description.len() <= 256, Error::<T>::DescriptionTooLong); // TODO: pass configurations from runtime configuration.

		ensure!(
			T::ExtraConfig::get_royalties_rate() >= royalty_rate,
			Error::<T>::RoyaltyRateTooHigh
		);

		let next_id = orml_nft::Pallet::<T>::next_class_id();
		let owner: T::AccountId = T::ModuleId::get().into_sub_account(next_id);
		let (deposit, all_deposit) = Self::create_class_deposit(
			metadata.len().saturated_into(),
			name.len().saturated_into(),
			description.len().saturated_into(),
		);

		<T as Config>::Currency::transfer(who, &owner, all_deposit.saturated_into(), KeepAlive)?;
		<T as Config>::Currency::reserve(&owner, deposit.saturated_into())?;
		// owner add proxy delegate to origin
		<pallet_proxy::Pallet<T>>::add_proxy_delegate(
			&owner,
			who.clone(),
			Default::default(),
			Zero::zero(),
		)?;

		for category_id in &category_ids {
			T::ExtraConfig::inc_count_in_category(*category_id)?;
		}
		let data: ClassData<BlockNumberOf<T>> = ClassData {
			deposit,
			properties,
			name,
			description,
			create_block: <frame_system::Pallet<T>>::block_number(),
			royalty_rate,
			category_ids,
		};

		orml_nft::Pallet::<T>::create_class(&owner, metadata, data)?;

		Self::deposit_event(Event::CreatedClass(owner.clone(), next_id));
		Ok((owner, next_id))
	}

	#[transactional]
	pub fn do_update_class(
		who: &T::AccountId,
		class_id: ClassIdOf<T>,
		metadata: NFTMetadata,
		name: Vec<u8>,
		description: Vec<u8>,
		royalty_rate: PerU16,
		properties: Properties,
		category_ids: Vec<GlobalId>,
	) -> ResultPost<(T::AccountId, ClassIdOf<T>)> {
		let old_class_info =
			orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
		let owner = old_class_info.owner;
		ensure!(who == &owner, Error::<T>::NoPermission);

		ensure!(category_ids.len() <= MAX_CATEGORY_PER_CLASS, Error::<T>::CategoryOutOfBound);
		ensure!(category_ids.len() >= 1, Error::<T>::CategoryOutOfBound);
		if category_ids.len() == 2 {
			ensure!(category_ids[0] != category_ids[1], Error::<T>::DuplicatedCategories);
		}

		ensure!(name.len() <= 20, Error::<T>::NameTooLong); // TODO: pass configurations from runtime configuration.
		ensure!(description.len() <= 256, Error::<T>::DescriptionTooLong); // TODO: pass configurations from runtime configuration.

		let old_data = old_class_info.data;
		let data: ClassData<BlockNumberOf<T>> = ClassData {
			deposit: old_data.deposit,
			properties,
			name,
			description,
			create_block: old_data.create_block,
			royalty_rate,
			category_ids,
		};

		orml_nft::Pallet::<T>::create_class(&owner, metadata, data)?;

		Self::deposit_event(Event::UpdatedClass(owner.clone(), class_id));

		Ok((owner, class_id))
	}

	#[transactional]
	pub fn do_transfer(
		from: &T::AccountId,
		to: &T::AccountId,
		class_id: ClassIdOf<T>,
		token_id: TokenIdOf<T>,
		quantity: TokenIdOf<T>,
	) -> DispatchResult {
		ensure!(Self::is_transferable(class_id)?, Error::<T>::NonTransferable);

		orml_nft::Pallet::<T>::transfer(from, to, (class_id, token_id), quantity)?;

		Self::deposit_event(Event::TransferredToken(
			from.clone(),
			to.clone(),
			class_id,
			token_id,
			quantity,
		));
		Ok(())
	}

	// ################## read only ##################

	pub fn contract_tokens(
		class_id: ClassIdOf<T>,
		token_id: TokenIdOf<T>,
	) -> Option<
		nftmart_traits::ContractTokenInfo<
			NFTMetadata,
			Quantity,
			Balance,
			BlockNumber,
			T::AccountId,
		>,
	> {
		orml_nft::Pallet::<T>::tokens(class_id, token_id).map(|t: TokenInfoOf<T>| {
			nftmart_traits::ContractTokenInfo {
				metadata: t.metadata,
				quantity: t.quantity.saturated_into(),
				data: nftmart_traits::ContractTokenData {
					deposit: t.data.deposit.saturated_into(),
					create_block: t.data.create_block.saturated_into(),
					royalty_rate: t.data.royalty_rate.deconstruct(),
					creator: t.data.creator,
					royalty_beneficiary: t.data.royalty_beneficiary,
				},
			}
		})
	}

	pub fn classes(class_id: ClassIdOf<T>) -> Option<ClassInfoOf<T>> {
		orml_nft::Pallet::<T>::classes(class_id)
	}

	pub fn tokens_by_owner(
		account_id: T::AccountId,
		page: u32,
		page_size: u32,
	) -> Vec<(ClassIdOf<T>, TokenIdOf<T>, AccountTokenOf<T>)> {
		orml_nft::TokensByOwner::<T>::iter_prefix(account_id)
			.map(|((c, t), a)| (c, t, a))
			.skip(page.saturating_mul(page_size) as usize)
			.take(page_size as usize)
			.collect()
	}

	pub fn owners_by_token(
		class_id: ClassIdOf<T>,
		token_id: TokenIdOf<T>,
		page: u32,
		page_size: u32,
	) -> Vec<(T::AccountId, AccountTokenOf<T>)> {
		orml_nft::OwnersByToken::<T>::iter_prefix((class_id, token_id))
			.map(|(account_id, _): (T::AccountId, ())| account_id)
			.skip(page.saturating_mul(page_size) as usize)
			.take(page_size as usize)
			.map(|account_id| {
				let at = orml_nft::TokensByOwner::<T>::get(&account_id, (class_id, token_id))
					.unwrap_or_default();
				(account_id, at)
			})
			.collect()
	}

	fn is_transferable(class_id: ClassIdOf<T>) -> Result<bool, DispatchError> {
		let class_info =
			orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		Ok(data.properties.0.contains(ClassProperty::Transferable))
	}

	fn is_burnable(class_id: ClassIdOf<T>) -> Result<bool, DispatchError> {
		let class_info =
			orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		Ok(data.properties.0.contains(ClassProperty::Burnable))
	}

	pub fn add_class_admin_deposit(admin_count: u32) -> Balance {
		let proxy_deposit_before: Balance = <pallet_proxy::Pallet<T>>::deposit(1).saturated_into();
		let proxy_deposit_after: Balance =
			<pallet_proxy::Pallet<T>>::deposit(admin_count.saturating_add(1)).saturated_into();
		proxy_deposit_after.saturating_sub(proxy_deposit_before)
	}

	pub fn mint_token_deposit(metadata_len: u32) -> Balance {
		T::CreateTokenDeposit::get()
			.saturating_add((metadata_len as Balance).saturating_mul(T::MetaDataByteDeposit::get()))
	}

	pub fn create_class_deposit(
		metadata_len: u32,
		name_len: u32,
		description_len: u32,
	) -> (Balance, Balance) {
		Self::create_class_deposit_num_proxies(metadata_len, name_len, description_len, 1)
	}

	fn create_class_deposit_num_proxies(
		metadata_len: u32,
		name_len: u32,
		description_len: u32,
		num_proxies: u32,
	) -> (Balance, Balance) {
		let deposit: Balance = {
			let total_bytes = metadata_len.saturating_add(name_len).saturating_add(description_len);
			T::CreateClassDeposit::get().saturating_add(
				(total_bytes as Balance).saturating_mul(T::MetaDataByteDeposit::get()),
			)
		};
		let proxy_deposit: Balance =
			<pallet_proxy::Pallet<T>>::deposit(num_proxies).saturated_into();
		(deposit, deposit.saturating_add(proxy_deposit))
	}
}

impl<T: Config> nftmart_traits::NftmartNft<T::AccountId, ClassIdOf<T>, TokenIdOf<T>> for Pallet<T> {
	fn peek_next_class_id() -> ClassIdOf<T> {
		orml_nft::NextClassId::<T>::get()
	}

	fn transfer(
		from: &T::AccountId,
		to: &T::AccountId,
		class_id: ClassIdOf<T>,
		token_id: TokenIdOf<T>,
		quantity: TokenIdOf<T>,
	) -> DispatchResult {
		Self::do_transfer(from, to, class_id, token_id, quantity)
	}

	fn account_token(
		who: &T::AccountId,
		class_id: ClassIdOf<T>,
		token_id: TokenIdOf<T>,
	) -> AccountToken<TokenIdOf<T>> {
		orml_nft::Pallet::<T>::tokens_by_owner(who, (class_id, token_id)).unwrap_or_default()
	}

	fn reserve_tokens(
		who: &T::AccountId,
		class_id: ClassIdOf<T>,
		token_id: TokenIdOf<T>,
		quantity: TokenIdOf<T>,
	) -> DispatchResult {
		orml_nft::Pallet::<T>::reserve(who, (class_id, token_id), quantity)
	}

	fn unreserve_tokens(
		who: &T::AccountId,
		class_id: ClassIdOf<T>,
		token_id: TokenIdOf<T>,
		quantity: TokenIdOf<T>,
	) -> DispatchResult {
		orml_nft::Pallet::<T>::unreserve(who, (class_id, token_id), quantity)
	}

	fn token_charged_royalty(
		class_id: ClassIdOf<T>,
		token_id: TokenIdOf<T>,
	) -> Result<(T::AccountId, PerU16), DispatchError> {
		let token: TokenInfoOf<T> =
			orml_nft::Tokens::<T>::get(class_id, token_id).ok_or(Error::<T>::TokenIdNotFound)?;
		let data: TokenData<T::AccountId, T::BlockNumber> = token.data;
		Ok((data.royalty_beneficiary, data.royalty_rate))
	}

	fn create_class(
		who: &T::AccountId,
		metadata: NFTMetadata,
		name: Vec<u8>,
		description: Vec<u8>,
		royalty_rate: PerU16,
		properties: Properties,
		category_ids: Vec<GlobalId>,
	) -> ResultPost<(T::AccountId, ClassIdOf<T>)> {
		Self::do_create_class(
			who,
			metadata,
			name,
			description,
			royalty_rate,
			properties,
			category_ids,
		)
	}

	fn update_class(
		who: &T::AccountId,
		class_id: ClassIdOf<T>,
		metadata: NFTMetadata,
		name: Vec<u8>,
		description: Vec<u8>,
		royalty_rate: PerU16,
		properties: Properties,
		category_ids: Vec<GlobalId>,
	) -> ResultPost<(T::AccountId, ClassIdOf<T>)> {
		Self::do_update_class(
			who,
			class_id,
			metadata,
			name,
			description,
			royalty_rate,
			properties,
			category_ids,
		)
	}

	#[allow(clippy::type_complexity)]
	fn proxy_mint(
		delegate: &T::AccountId,
		to: &T::AccountId,
		class_id: ClassIdOf<T>,
		metadata: NFTMetadata,
		quantity: TokenIdOf<T>,
		charge_royalty: Option<PerU16>,
	) -> ResultPost<(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)> {
		Self::do_proxy_mint(delegate, to, class_id, metadata, quantity, charge_royalty)
	}
}
