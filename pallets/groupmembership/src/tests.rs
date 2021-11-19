use crate as pallet_template;
use frame_support::parameter_types;
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
		GroupMembershipModule: groupmembership::{Pallet, Call, Storage, Event<T>},
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

impl pallet_template::Config for Test {
	type Event = Event;
}

//  Build genesis storage according to the mock runntime 
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
	t.execute_with(|| System::set_block_number(1) );
	t
}

#[test]
fn members_can_join() {
	new_test_ext.execute_with(|| { 
		//	Origin Sign In 
		assert_ok!(GroupMembershipModule::add_member(Origin::signed(1)));
		//	Check if you can duplicate memebers inside the storage database
		assert_noop!(GroupMembershipModule::add_member(Origin::signed(1)), 
			Error::<Test>::NotMember
		);
		//	Check values inside the storage value 
		assert_eq!(GroupMembershipModule::members(), vec![1])
		//	emit expected event 
		System::assert_last_event(Event::GroupMembershipModule(crate::Event::<Test>::MemberAdded(1)));
	})
}
#[test]
fn members_can_join_group() {
	new_test_ext.execute_with(|| { 
		let idx = 2;
		let scr = 3;
		//	Add a member into the data 
		assert_ok!(GroupMembershipModule::add_member(Origin::signed(1)));
		//	Check storage values: group_membership, memberscore 
		assert_ok!(GroupMembershipModule::join_a_group(Origin::singed(1), idx, scr));
		//	Account 2 isnt part of this AllMembers group 
		assert_noop!(GroupMembershipModule::join_a_group(Origin::signed(2), idx, scr));
		//	Emit expected event 
		System::assert_last_event(Event::GroupMembershipModule(
			//	account, group id, score 
			crate::Event::<Test>::MembersJoinGroup(1, idx, scr)));
		
		//	Check Storage Values AllMembers
		assert_eq!(GroupMembershipModule::members(), vec![1]);
		//	Check Storage Map membership
		assert_eq!(GroupMembershipModule::membership(1), idx);
		//	Check storage double member_score
		assert_eq!(GroupMembershipModule::member_score(idx, 1), scr);
	})
} 
#[test]
fn can_remove_member() {
	new_test_ext.execute_with(|| { 
		assert_ok!(GroupMembershipModule::add_member(Origin::signed(1)));
		
		assert_ok!(GroupMembershipModule::join_a_group(Origin::signed(1), 2, 3));
		
		assert_ok!(GroupMembershipModule::remove_member(Orign::signed(1)));
		
		assert_noop!(GroupMembershipModule::remove_member(2), Error::<T>::NotMember);
		//	Check Accoount 1 in Group 2 with score 3
		assert!(!<GroupMembership<Test>>::contains_key(1));
		
		assert!(!<MembershipScore<Test>>::contains_key(2, 1));
		
		System::assert_last_event(Event::GroupMembershipModule(
			//	account, group id, score 
			crate::Event::<Test>::RemoveMember(1)));
	
	})
}
#[test]
fn can_remove_group_score() {
	new_test_ext.execute_with(|| { 
		assert_ok!(GroupMembershipModule::add_member(Origin::signed(1)));
		assert_ok!(GroupMembershipModule::add_member(Origin::signed(2)));
		assert_ok!(GroupMembershipModule::add_member(Origin::signed(3)));
		assert_ok!(GroupMembershipModule::join_a_group(Origin::signed(1), 2, 3));
		assert_ok!(GroupMembershipModule::join_a_group(Origin::signed(2), 2, 3));
		assert_ok!(GroupMembershipModule::join_a_group(Origin::signed(3), 2, 3));

		assert_noop!(GroupMembershipModule::remove_group_score(Origin::signed(1), 3, 3), "Member 1 isnt in Groupindex 1");
		
		assert_ok!(GroupMembershipModule::join_a_group(Origin::signed(1), 2, 3));

		assert_ok!(GroupMembershipModule::remove_group_score(Origin::signed(1), 3));
		
		System::assert_last_event(Event::GroupMembershipModule(
			// score 
			crate::Event::<Test>::RemoveGroup(3)));
	})
}
