#![cfg(test)]

use crate::{Module, Config};
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use frame_support::{impl_outer_origin, parameter_types};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;

pub type TokenId = u128;
pub type CurrencyId = u128;
pub type Balance = u128;
pub type Amount = i128;
pub type BlockNumber = u64;

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Runtime;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl system::Config for Runtime {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type Event = ();
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
}
pub type Tokens = orml_tokens::Module<Runtime>;

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = system::Module<Runtime>;
	type MaxLocks = ();
	type WeightInfo = ();
}
pub type PalletBalances = pallet_balances::Module<Runtime>;
pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, PalletBalances, Amount, BlockNumber>;

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = 0;
}

impl orml_currencies::Config for Runtime {
	type Event = ();
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}
pub type Currencies = orml_currencies::Module<Runtime>;

impl Config for Runtime {
	type Event = ();
	type Balance = Balance;
	type PoolId = u32;
	type TokenId = TokenId;
	type Currency = Tokens;
	type WeightInfo = ();
}

pub type FixedSwap = Module<Runtime>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	orml_tokens::GenesisConfig::<Runtime> {
		// account_id, currency_id, initial_balance
		endowed_accounts: vec![
			(0, 1, 100000),
			(1, 2, 100000),
		],
	}.assimilate_storage(&mut t).unwrap();
	t.into()
}
