#![cfg(test)]

use super::*;
use orml_currencies::BasicCurrencyAdapter;
use sp_core::constants_types::*;
use crate as nftmart_order;
use codec::{Decode, Encode};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Filter, InstanceFilter},
	RuntimeDebug, assert_ok, PalletId
};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, AccountIdConversion},
};
use nftmart_traits::{Properties, ClassProperty};

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

pub type AccountId = AccountId32;

impl frame_system::Config for Runtime {
	type BaseCallFilter = BaseFilter;
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}
parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}
impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = ();
	type WeightInfo = ();
}
impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}
parameter_types! {
	pub const ProxyDepositBase: u64 = 1;
	pub const ProxyDepositFactor: u64 = 1;
	pub const MaxProxies: u16 = 4;
	pub const MaxPending: u32 = 2;
	pub const AnnouncementDepositBase: u64 = 1;
	pub const AnnouncementDepositFactor: u64 = 1;
}
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
pub enum ProxyType {
	Any,
	JustTransfer,
	JustUtility,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::JustTransfer => matches!(c, Call::Balances(pallet_balances::Call::transfer(..))),
			ProxyType::JustUtility => matches!(c, Call::Utility(..)),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		self == &ProxyType::Any || self == o
	}
}
pub struct BaseFilter;
impl Filter<Call> for BaseFilter {
	fn filter(c: &Call) -> bool {
		match *c {
			// Remark is used as a no-op call in the benchmarking
			Call::System(SystemCall::remark(_)) => true,
			Call::System(_) => false,
			_ => true,
		}
	}
}
impl pallet_proxy::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = ();
	type CallHasher = BlakeTwo256;
	type MaxPending = MaxPending;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

orml_traits::parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: sp_core::constants_types::CurrencyId| -> Balance {
		if currency_id == &sp_core::constants_types::NATIVE_CURRENCY_ID {
			ExistentialDeposit::get()
		} else  {
			Default::default()
		}
	};
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
}

parameter_types! {
	pub const GetNativeCurrencyId: sp_core::constants_types::CurrencyId = sp_core::constants_types::NATIVE_CURRENCY_ID;
}

pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Runtime, Balances, sp_core::constants_types::Amount, sp_core::constants_types::Moment>;

impl orml_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}

impl orml_nft::Config for Runtime {
	type ClassId = sp_core::constants_types::ClassId;
	type TokenId = sp_core::constants_types::TokenId;
	type ClassData = nftmart_nft::ClassData<BlockNumberOf<Self>>;
	type TokenData = nftmart_nft::TokenData<<Self as frame_system::Config>::AccountId, BlockNumberOf<Self>>;
}

parameter_types! {
	pub const CreateClassDeposit: Balance = 50;
	pub const CreateTokenDeposit: Balance = 10;
	pub const MetaDataByteDeposit: Balance = 1;
	pub const NftModuleId: PalletId = PalletId(*b"nftmart*");
}

impl nftmart_nft::Config for Runtime {
	type Event = Event;
	type ExtraConfig = NftmartConf;
	type CreateClassDeposit = CreateClassDeposit;
	type MetaDataByteDeposit = MetaDataByteDeposit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type ModuleId = NftModuleId;
	type Currency = Balances;
	type MultiCurrency = Currencies;
}

impl nftmart_config::Config for Runtime {
	type Event = Event;
}

impl nftmart_order::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Currencies;
	type Currency = Balances;
	type ClassId = sp_core::constants_types::ClassId;
	type TokenId = sp_core::constants_types::TokenId;
	type NFT = Nftmart;
	type ExtraConfig = NftmartConf;
}

use frame_system::Call as SystemCall;

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, Call, u32, ()>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Event},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		Currencies: orml_currencies::{Pallet, Call, Event<T>},
		OrmlNFT: orml_nft::{Pallet, Storage, Config<T>},
		NftmartConf: nftmart_config::{Pallet, Call, Event<T>},
		Nftmart: nftmart_nft::{Pallet, Call, Event<T>},
		NftmartOrder: nftmart_order::{Pallet, Call, Event<T>},
	}
);

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CLASS_ID0: <Runtime as orml_nft::Config>::ClassId = 0;
pub const TOKEN_ID0: <Runtime as orml_nft::Config>::TokenId = 0;
pub const TOKEN_ID1: <Runtime as orml_nft::Config>::TokenId = 1;

pub struct ExtBuilder;
impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![
				(ALICE, 100000),
				(BOB, 100),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);
			NftmartConf::add_whitelist(Origin::root(), ALICE).unwrap();
			NftmartConf::add_whitelist(Origin::root(), BOB).unwrap();
		});
		ext
	}
}

#[allow(dead_code)]
pub fn last_event() -> Event {
	frame_system::Pallet::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}

pub fn add_class(who: AccountId) {
	assert_ok!(Nftmart::create_class(
		Origin::signed(who),
		vec![1], vec![1], vec![1],
		Properties(ClassProperty::Transferable |
			ClassProperty::Burnable |
			ClassProperty::RoyaltiesChargeable)
	));
}

pub fn class_id0_account() -> AccountId {
	<Runtime as nftmart_nft::Config>::ModuleId::get().into_sub_account(CLASS_ID0)
}

pub fn add_token(who: AccountId, quantity: TokenId, charge_royalty: Option<bool>) {
	let deposit = Nftmart::mint_token_deposit(1);
	assert_eq!(Balances::deposit_into_existing(&class_id0_account(), deposit).is_ok(), true);
	assert_ok!(Nftmart::mint(
		Origin::signed(class_id0_account()),
		who,
		CLASS_ID0,
		vec![1],
		quantity, charge_royalty,
	));
}

pub fn add_category() {
	assert_ok!(NftmartConf::create_category(Origin::root(), vec![1]));
}

pub fn all_tokens_by(who: AccountId) -> Vec<(ClassId, TokenId, orml_nft::AccountToken<TokenId>)> {
	let v: Vec<_> = orml_nft::TokensByOwner::<Runtime>::iter().filter(|(account, (_c, _t), _data)| {
		who == *account
	}).map(|(_account, (c, t), data)| {
		(c, t, data)
	}).collect();
	v.into_iter().rev().collect()
}

pub fn current_gid() -> GlobalId {
	nftmart_config::Pallet::<Runtime>::next_id()
}
