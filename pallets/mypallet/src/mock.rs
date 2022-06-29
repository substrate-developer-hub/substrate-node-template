use crate as pallet_mypallet;
use frame_support::traits::{ConstU16, ConstU64, GenesisBuild};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		MyPallet: pallet_mypallet::{Pallet, Call, Storage, Config}, //my config doesnt depends on type, else Config<T>
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
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
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

impl pallet_mypallet::Config for Test {}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	// pallet_mypallet::GenesisConfig::<Test> { init_val: 8888 }
	// 	.assimilate_storage(&mut t)
	// 	.unwrap();

	// pallet_mypallet::GenesisConfig { init_val: 8888 }.assimilate_storage(&mut t).unwrap();
	let gen_config = pallet_mypallet::GenesisConfig { init_val: 8888 };
	// let yy = <xx as GenesisBuild::<Test>>.assimilate_storage(&mut t).unwrap();
	let yy = <pallet_mypallet::GenesisConfig as GenesisBuild::<Test>>::assimilate_storage(&gen_config, &mut t).unwrap();

	let ext = sp_io::TestExternalities::new(t);
	//   ext.execute_with(|| System::set_block_number(1));
	ext
}
