use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

#[test]
fn patient_can_create_account() {
	new_test_ext().execute_with(|| {
		let o = RuntimeOrigin::signed(1);

		assert_ok!(MedicalRecord::create_patient_account(o.clone()));
		let mb_raw_origin: Result<RawOrigin<u64>, RuntimeOrigin> = o.clone().into();
		match mb_raw_origin.ok().unwrap() {
			RawOrigin::Signed(account_id) => {
				let account_created = match MedicalRecord::patient_records(account_id) {
					None => false,
					Some(_) => true,
				};
				assert!(account_created, "failed to create an account");
			},
			_ => (),
		}

		assert_noop!(MedicalRecord::create_patient_account(o), Error::<Test>::AccountAlreadyExist);
	});
}
