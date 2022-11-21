use crate::{mock::*, Error, Record, UserType};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use frame_system::RawOrigin;
use sp_core::Get;

#[test]
fn user_can_create_account() {
	new_test_ext().execute_with(|| {
		let o = RuntimeOrigin::signed(1);
		assert_ok!(MedicalRecord::create_account(o.clone(), UserType::Patient));

		let account_created =
			MedicalRecord::records(origin_to_account_id(o.clone()), UserType::Patient).is_some();
		assert!(account_created, "failed to create an account");

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
		assert_noop!(
			MedicalRecord::patient_adds_record(o.clone(), BoundedVec::with_max_capacity()),
			Error::<Test>::AccountNotFound
		);
		assert_ok!(MedicalRecord::create_account(o.clone(), UserType::Patient));
		let max_record_len = <MockMaxRecordLength as Get<u32>>::get() as usize;
		for _ in 0..max_record_len {
			assert_ok!(MedicalRecord::patient_adds_record(
				o.clone(),
				BoundedVec::with_max_capacity()
			));
		}

		let records = MedicalRecord::records(origin_to_account_id(o.clone()), UserType::Patient)
			.expect("The record should exist");
		assert_eq!(records.len(), max_record_len);
		assert_eq!(
			records
				.into_iter()
				.filter(|r| match r {
					Record::UnverifiedRecord(_, _, _) => true,
					_ => false,
				})
				.collect::<Vec<_>>()
				.len(),
			max_record_len
		);

		assert_noop!(
			MedicalRecord::patient_adds_record(o.clone(), BoundedVec::with_max_capacity()),
			Error::<Test>::ExceedsMaxRecordLength
		);
	});
}

#[test]
fn doctor_can_add_record_for_patient() {
	new_test_ext().execute_with(|| {
		let doctor = RuntimeOrigin::signed(1);
		let patient = RuntimeOrigin::signed(2);
		assert_ok!(MedicalRecord::create_account(doctor.clone(), UserType::Doctor));
		assert_ok!(MedicalRecord::create_account(patient.clone(), UserType::Patient));

		let max_record_len = <MockMaxRecordLength as Get<u32>>::get() as usize;
		for _ in 0..max_record_len {
			assert_ok!(MedicalRecord::doctor_adds_record(
				doctor.clone(),
				origin_to_account_id(patient.clone()),
				BoundedVec::with_max_capacity(),
				BoundedVec::with_max_capacity(),
			));
		}

		let patient_records =
			MedicalRecord::records(origin_to_account_id(patient.clone()), UserType::Patient)
				.expect("the record should exist");
		assert_eq!(patient_records.len(), max_record_len);
		assert_eq!(
			patient_records
				.into_iter()
				.filter(|r| match r {
					Record::VerifiedRecord(_, _, _, _) => true,
					_ => false,
				})
				.collect::<Vec<_>>()
				.len(),
			max_record_len
		);

		assert_noop!(
			MedicalRecord::patient_adds_record(patient.clone(), BoundedVec::with_max_capacity()),
			Error::<Test>::ExceedsMaxRecordLength
		);
	});
}

fn origin_to_account_id(o: RuntimeOrigin) -> u64 {
	let ro: Result<RawOrigin<u64>, RuntimeOrigin> = o.clone().into();
	match ro.ok().expect("Assume remote origin is signed") {
		RawOrigin::Signed(account_id) => account_id,
		_ => unreachable!(),
	}
}
