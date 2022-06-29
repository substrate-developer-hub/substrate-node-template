use frame_support::parameter_types;
use frame_support::sp_runtime::{
	app_crypto::sp_core::{H160, H256, U256},
	testing::Header,
	traits::BlakeTwo256,
	traits::IdentityLookup,
};
use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot, IdentityAddressMapping};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage},
        EVM: pallet_evm::{Pallet, Call, Storage, Config, Event<T>}
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = H160;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 0;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 1000;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<Self::AccountId>;
	type WithdrawOrigin = EnsureAddressNever<Self::AccountId>;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
}
