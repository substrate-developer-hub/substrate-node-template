use crate as pallet_dex;
use frame_support::{parameter_types, PalletId};

use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::traits::{ConstU128, ConstU32};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Assets: pallet_assets::{Pallet, Call, Storage, Config<T>, Event<T>},
		DEX: pallet_dex::{Pallet, Call, Storage, Event<T>},
	}
);

pub type AssetId = u32;
pub type AccountId = u64;
pub type Balance = u128;

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = ();
	type AssetAccountDeposit = ();
	type MetadataDepositBase = ();
	type MetadataDepositPerByte = ();
	type ApprovalDeposit = ();
	type StringLimit = ConstU32<15>;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<5>;
	type WeightInfo = ();
}

parameter_types! {
	pub const DEXPallet: PalletId = PalletId(*b"py/de-ex");
	pub const LiquidityPoolTokenMinimumBalance: u32 = 1;
}

impl pallet_dex::Config for Test {
	type Event = Event;
	type AssetId = AssetId;
	type Assets = Assets;
	type LiquidityPoolTokenMinimumBalance = LiquidityPoolTokenMinimumBalance;
	type LiquidityPoolTokenDecimals = ();
	type MinimumLiquidity = ();
	type SwapFeeUnits = ConstU128<1000>;
	type SwapFeeValue = ConstU128<997>;
	type NativeCurrency = Balances;
	type NativeAssetId = ();
	type PalletId = DEXPallet;
	type Time = Timestamp;

	fn exists(id: Self::AssetId) -> bool {
		Assets::maybe_total_supply(id).is_some()
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
