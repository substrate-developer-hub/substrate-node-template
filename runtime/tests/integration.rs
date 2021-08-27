// extern crate env as env;
extern crate pallet_template as pallet_template;

#[cfg(test)]
mod tests {
	use super::*;

	use frame_support::{
		assert_err,
		assert_ok,
		parameter_types,
		traits::{
			Contains,
			Currency,
		},
		weights::{
			IdentityFee,
			Weight,
		},
	};
	use frame_system::{
		EnsureRoot,
		RawOrigin,
	};
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{
			AccountIdLookup,
			BlakeTwo256,
			IdentityLookup,
		},
		DispatchError,
		DispatchResult,
		PalletId,
		Perbill,
		Percent,
		Permill,
	};
	use std::cell::RefCell;
	// Import Trait for each runtime module being tested
	use chrono::NaiveDate;
	use node_template_runtime::{
		AccountId,
		Aura,
		Balance,
		BlockNumber,
		Moment,
		SLOT_DURATION,
	};
	pub use pallet_transaction_payment::{
		CurrencyAdapter,
	};
	use pallet_template::{
		Pallet as TemplatePallet,
		Config as TemplateConfig,
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
			RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
			Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
			TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const SS58Prefix: u8 = 33;
	}
	impl frame_system::Config for Test {
		type BlockHashCount = BlockHashCount;
		type BlockLength = ();
		type BlockNumber = u64;
		type BlockWeights = ();
		type Call = Call;
		type DbWeight = ();
		type Event = ();
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Header = Header;
		type Index = u64;
		type Lookup = AccountIdLookup<AccountId, ()>;
		type OnKilledAccount = ();
		type OnNewAccount = ();
		type Origin = Origin;
		type PalletInfo = PalletInfo;
		type SS58Prefix = SS58Prefix;
		type SystemWeightInfo = ();
		type Version = ();
			type AccountData = pallet_balances::AccountData<u64>;
			type OnSetCode = ();
	}
	parameter_types! {
		pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
	}
	impl pallet_timestamp::Config for Test {
		type MinimumPeriod = MinimumPeriod;
		type Moment = u64;
		type OnTimestampSet = Aura;
		type WeightInfo = ();
	}
	parameter_types! {
		pub const ExistentialDeposit: u128 = 1;
		pub const MaxLocks: u32 = 50;
	}
	impl pallet_balances::Config for Test {
		type AccountStore = System;
		type Balance = u64;
		type DustRemoval = ();
		type Event = ();
		type ExistentialDeposit = ExistentialDeposit;
		type MaxLocks = ();
		type MaxReserves = ();
		type WeightInfo = ();
		type ReserveIdentifier = [u8; 8];
	}
	parameter_types! {
		pub const TransactionByteFee: u64 = 1;
	}
	impl pallet_transaction_payment::Config for Test {
		type FeeMultiplierUpdate = ();
		type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
		type TransactionByteFee = ();
		type WeightToFee = IdentityFee<u64>;
	}

	impl TemplateConfig for Test {
		type Event = ();
		type Currency = Balances;
	}

	pub type TemplateModule = TemplatePallet<Test>;
	type Randomness = pallet_randomness_collective_flip::Pallet<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	pub fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: vec![(0, 10), (1, 10), (2, 20), (3, 30)],
		}
			.assimilate_storage(&mut t)
			.unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	// Create Users
	#[test]
	fn setup_users() {
		new_test_ext().execute_with(|| {
			assert_eq!(Balances::free_balance(0), 10);
			assert_eq!(Balances::free_balance(1), 10);
			assert_eq!(Balances::free_balance(2), 20);
			assert_eq!(Balances::free_balance(3), 30);
			assert_eq!(Balances::reserved_balance(&1), 0);
		});
	}

	#[test]
	fn integration_test() {
		new_test_ext().execute_with(|| {

			System::set_block_number(1);

            // 27th August 2021 @ ~7am is 1630049371000
            // where milliseconds/day         86400000
			// 27th August 2021 @ 12am is 1630022400000 (start of day)
            Timestamp::set_timestamp(1630049371000u64);

			assert_ok!(TemplateModule::set_rewards_allowance_dhx_current(
				Origin::signed(0),
				5000
			));

			// Verify Storage
			assert_eq!(TemplateModule::rewards_allowance_dhx_current(), 5000);

			assert_ok!(TemplateModule::set_rewards_allowance_dhx_for_date(
				Origin::signed(0),
				5000
			));
		});
	}
}
