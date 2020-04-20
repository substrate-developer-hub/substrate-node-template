// Creating mock runtime here

use crate::{Module, Trait};
use sp_core::{hashing::blake2_256, sr25519, Blake2Hasher, Pair, Public, H256};
use frame_support::{impl_outer_origin, impl_outer_event, parameter_types, weights::Weight};
use sp_runtime::{
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify}, 
    testing::Header, 
    Perbill,
};

impl_outer_origin! {
	pub enum Origin for TestRuntime {}
}

// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`TestRuntime`) which `impl`s each of the
// configuration traits of pallets we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct TestRuntime;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}
impl system::Trait for TestRuntime {
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
}
impl Trait for TestRuntime {
	type Event = TestEvent;
}

// Easy access alias
pub type Registry = Module<TestRuntime>;
pub type System = system::Module<TestRuntime>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap().into()
}

/// The signature type used by accounts/transactions.
pub type Signature = sr25519::Signature;
/// An identifier for an account on this system.
pub type AccountId = <Signature as Verify>::Signer;
pub type AccountPublic = <Signature as Verify>::Signer;

mod registry {
	pub use crate::Event;
}

impl_outer_event! {
	pub enum TestEvent for TestRuntime {
		registry<T>,
		system<T>,
	}
}


/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::substratee_registry;
    use externalities::set_and_run_with_externalities;
    use node_primitives::{AccountId, Signature};
    use primitives::{sr25519, Blake2Hasher, Pair, Public, H256};
    use sr_primitives::weights::Weight;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
        Perbill,
    };
    use std::{cell::RefCell, collections::HashSet};
    use support::traits::{Currency, FindAuthor, Get, LockIdentifier};
    use support::{assert_ok, impl_outer_event, impl_outer_origin, parameter_types};

    thread_local! {
        static EXISTENTIAL_DEPOSIT: RefCell<u64> = RefCell::new(0);
    }
    //pub type AccountId = u64;
    pub type BlockNumber = u64;
    pub type Balance = u64;
    pub struct ExistentialDeposit;
    impl Get<u64> for ExistentialDeposit {
        fn get() -> u64 {
            EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
        }
    }


    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct TestRuntime;
    impl Trait for TestRuntime {
        type Event = TestEvent;
    }

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: u32 = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
    }
    impl system::Trait for TestRuntime {
        type Origin = Origin;
        type Index = u64;
        type Call = ();
        type BlockNumber = BlockNumber;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = TestEvent;
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }
    pub type System = system::Module<TestRuntime>;

    parameter_types! {
        pub const TransferFee: Balance = 0;
        pub const CreationFee: Balance = 0;
        pub const TransactionBaseFee: u64 = 0;
        pub const TransactionByteFee: u64 = 0;
    }
    impl balances::Trait for TestRuntime {
        type Balance = Balance;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = TestEvent;
        type TransferPayment = ();
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
    }
    pub type Balances = balances::Module<TestRuntime>;

    type AccountPublic = <Signature as Verify>::Signer;

    // Easy access alias
    type Registry = super::Module<TestRuntime>;

    pub struct ExtBuilder;

    impl ExtBuilder {
        pub fn build() -> runtime_io::TestExternalities {
            let mut storage = system::GenesisConfig::default()
                .build_storage::<TestRuntime>()
                .unwrap();
            balances::GenesisConfig::<TestRuntime> {
                balances: vec![],
                vesting: vec![],
            }
            .assimilate_storage(&mut storage)
            .unwrap();
            runtime_io::TestExternalities::from(storage)
        }
    }

    mod generic_event {
        pub use super::super::Event;
    }

    impl_outer_event! {
        pub enum TestEvent for TestRuntime {
            generic_event<T>,
            balances<T>,
        }
    }

    pub type GenericEvent = Module<TestRuntime>;

    impl_outer_origin! {
        pub enum Origin for TestRuntime {}
    }

    //    Fixme:    Was not able to use these statics for the tests, always threw cannot move out of
    //              dereference of raw pointer. As copy trait not implemented for whatever reason.
    //    lazy_static! {
    //        #[derive(Clone, Copy, Encode, Decode, Default, PartialEq)]
    //        static ref ENC_1: Enclave<u64, Vec<u8>> = Enclave { pubkey: 10, url: URL.to_vec() };
    //        #[derive(Encode, Decode, Default, Clone, Copy, PartialEq)]
    //        static ref ENC_2: Enclave<u64, Vec<u8>> = Enclave { pubkey: 20, url: URL.to_vec() };
    //        #[derive(Encode, Decode, Default, Clone, Copy, PartialEq)]
    //        static ref ENC_3: Enclave<u64, Vec<u8>> = Enclave { pubkey: 30, url: URL.to_vec() };
    //    }

*/