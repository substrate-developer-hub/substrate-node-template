use super::*;


use crate as membershipset;
use frame_support::{
    parameter_types, assert_ok, assert_noop,
    construct_runtime,   

};
use sp_io::TestExternalities;
use frame_system as system;
use sp_core::H256;
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
		MembershipModule: membershipset::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

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
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

impl Config for Test { 
    type Event = Event;
}

struct ExternalityBuilder;

//  Build genesis storage according to the mock runntime 
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
	t.execute_with(|| System::set_block_number(1) );
	t
}

#[test]
fn can_add_member() { 
    //  Set up environment for testing purposes 
    new_test_ext.execute_with(|| { 
        //  Origin sign
        assert_ok!(MembershipModule::add_member(Origin::singed(1)));
        assert_eq!(MembershipModule::member(1), ());
        //  Member COunt is 1
        assert_eq!(MembershipModule::member_count(), 1);
        //  Check for member contains_key()
        assert!(<Member<Test>>::contains_key(1));
        //  Expected Event 
        System::assert_last_event(Event::MembershipModule(crate::Event::<Test>::MemberAdded(1)));       
    })
}

#[test]
fn cant_exceed_max_members() { 
    new_test_ext.execute_with(|| { 
        //  Origin sign with 16 new members
        for new_member in 0..16 { 
            assert_ok!(MembershipModule::add_member(Origin::signed(new_member)));
        }
        //  Expect 16 members to be added       
        assert_eq(MembershipModule::member_count(), 16);
        //  Check what happens if error occurs when we add the 17th member
        assert_noop(MembershipModule::add_member(Origin::singed(16)), Error::<Test>::MembershipLimitReached);        
    })
}

#[test]
fn can_remove_member() { 
    new_test_ext.execute_with(|| { 
        //  Origin sign 
        //  Add member
        assert_ok!(MembershipModule::add_member(Origin::signed(1)));
        //  Remove member
        assert_ok!(MembershupModule::remove_member(Origin::signed(1)));
        //  Check for storage changes, if the member no longer contains the key  
        assert!(!<Member<Test>>::contains_key(1));        
        //  Check Member count, should be zero since we got rid of it in the second line 
        assert_eq!(MembershipModule::member_count(), 0);
        //  Emit event 
        System::assert_last_event(Event::MembershipModule(crate::Event::<Test>::MemberRemoved(1)));
    })
}
#[test]
fn can_check_for_duplicates() { 
    new_test_ext.execute_with(|| { 
        assert_ok!(MembershipModule::add_member(Origin::signed(1)));
        assert_noop!(
            MembershipModule::add_member(Origin::signed(1), Error::<Test>::AlreadyMember);
        );
    })
}
#[test]
fn remove_member_handles_errors() { 
    new_test_ext.execute_with(|| { 
        //  No member is found
        assert_noop!(
            MembershipModule::remove(Origin::signed(2), Error::<Test>::NotMember)
        )
    })
}