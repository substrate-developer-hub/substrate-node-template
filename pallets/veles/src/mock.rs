use crate as pallet_veles;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU16, ConstU64},
};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, BuildStorage,
};

pub use crate::*;
#[allow(unused_imports)]
pub use common::*;
pub use pallet_veles::PenaltyLevelConfig;
pub use sp_core::H256;
pub use sp_std::collections::btree_set::BTreeSet;

// Types
type Block = frame_system::mocking::MockBlock<Test>;
type Moment = u64;
pub type AccountId = AccountId32;

// Helper functions
pub fn alice() -> AccountId {
	AccountId::from([1u8; 32])
}

pub fn bob() -> AccountId {
	AccountId::from([2u8; 32])
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Veles: pallet_veles,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const IPFSLength: u32 = 64;
	pub const CarboCreditDecimal: u8 = 4;
	pub const PenaltyLevelsConfiguration: [PenaltyLevelConfig; 5] = [
		PenaltyLevelConfig { level: 0, base: 1 },
		PenaltyLevelConfig { level: 1, base: 2 },
		PenaltyLevelConfig { level: 2, base: 3 },
		PenaltyLevelConfig { level: 3, base: 4 },
		PenaltyLevelConfig { level: 4, base: 5 }, // TODO base into Balance
	];
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

/// Configure the Veles pallet
impl pallet_veles::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type IPFSLength = IPFSLength;
	type CarboCreditDecimal = CarboCreditDecimal;
	type Time = Timestamp;
	type PenaltyLevelsConfiguration = PenaltyLevelsConfiguration;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
