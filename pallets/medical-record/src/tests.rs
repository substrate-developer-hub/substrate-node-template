use crate::{mock::*, Error, UserType};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

#[test]
fn user_can_create_account() {
	new_test_ext().execute_with(|| {
		let o = RuntimeOrigin::signed(1);

		assert_ok!(MedicalRecord::create_account(o.clone(), UserType::Patient));
		let ro: Result<RawOrigin<u64>, RuntimeOrigin> = o.clone().into();

		match ro.ok().unwrap() {
			RawOrigin::Signed(account_id) => {
				let account_created = match MedicalRecord::records(account_id, UserType::Patient) {
					None => false,
					Some(_) => true,
				};
				assert!(account_created, "failed to create an account");
			},
			_ => (),
		}

		assert_noop!(
			MedicalRecord::create_account(o, UserType::Patient),
			Error::<Test>::AccountAlreadyExist
		);
	});
}
