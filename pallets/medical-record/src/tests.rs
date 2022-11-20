use crate::{mock::*, Error, UserType};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use frame_system::RawOrigin;

#[test]
fn user_can_create_account() {
	new_test_ext().execute_with(|| {
		let o = RuntimeOrigin::signed(1);
		assert_ok!(MedicalRecord::create_account(o.clone(), UserType::Patient));

		let ro: Result<RawOrigin<u64>, RuntimeOrigin> = o.clone().into();
		match ro.ok().unwrap() {
			RawOrigin::Signed(account_id) => {
				let account_created =
					MedicalRecord::records(account_id, UserType::Patient).is_some();
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

#[test]
fn patient_can_add_record() {
	new_test_ext().execute_with(|| {
		let o = RuntimeOrigin::signed(1);
		assert_ok!(MedicalRecord::create_account(o.clone(), UserType::Patient));
		assert_ok!(MedicalRecord::patient_adds_record(o.clone(), BoundedVec::with_max_capacity()));

		let ro: Result<RawOrigin<u64>, RuntimeOrigin> = o.clone().into();
		match ro.ok().unwrap() {
			RawOrigin::Signed(account_id) => {
				match MedicalRecord::records(account_id, UserType::Patient) {
					Some(a) => {
						assert_eq!(a.len(), 1);
					},
					None => (),
				}
			},
			_ => (),
		}

		assert_noop!(
			MedicalRecord::patient_adds_record(o.clone(), BoundedVec::with_max_capacity()),
			Error::<Test>::ExceedsMaxRecordLength
		);

		assert_noop!(
			MedicalRecord::patient_adds_record(
				RuntimeOrigin::signed(2),
				BoundedVec::with_max_capacity()
			),
			Error::<Test>::AccountNotFound
		);
	});
}
