use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok};

#[test]
fn create_proof_normal() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(PoeModule::create_claim(Origin::signed(1), vec![0x1;8]));
		assert_eq!(PoeModule::get_proof(&vec![0x1;8]).0, 1);
	});
}

#[test]
fn create_proof_too_short() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_err!(PoeModule::create_claim(Origin::signed(1), vec![0x1;1]), Error::<Test>::Prooftooshort);
	});
}

#[test]
fn create_proof_too_long() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_err!(PoeModule::create_claim(Origin::signed(1), vec![0x1;100]), Error::<Test>::Prooftoolong);
	});
}

#[test]
fn revoke_proof_normal() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(PoeModule::create_claim(Origin::signed(1), vec![0x1;8]));
		assert_eq!(PoeModule::get_proof(&vec![0x1;8]).0, 1);
		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), vec![0x1;8]));
		assert_ne!(PoeModule::get_proof(&vec![0x1;8]).0, 1);
	});
}

#[test]
fn revoke_proof_twice() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(PoeModule::create_claim(Origin::signed(1), vec![0x1;8]));
		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), vec![0x1;8]));
		assert_err!(PoeModule::revoke_claim(Origin::signed(1), vec![0x1;8]),  Error::<Test>::NoSuchProof);

	});
}


#[test]
fn revoke_proof_unexisted() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_err!(PoeModule::revoke_claim(Origin::signed(1), vec![0x1;8]),  Error::<Test>::NoSuchProof);
	});
}

#[test]
fn revoke_proof_not_owner() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic
		assert_ok!(PoeModule::create_claim(Origin::signed(2), vec![0x1;8]));
		assert_err!(PoeModule::revoke_claim(Origin::signed(1), vec![0x1;8]), Error::<Test>::NotProofOwner);
	});
}

#[test]
fn transfer_proof_normal() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic
		assert_ok!(PoeModule::create_claim(Origin::signed(1), vec![0x1;8]));
		assert_ok!(PoeModule::transfer_to(Origin::signed(1), vec![0x1;8],2));
	});
}


#[test]
fn transfer_proof_not_exists() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic
		assert_err!(PoeModule::transfer_to(Origin::signed(1), vec![0x1;8],2), Error::<Test>::NoSuchProof);
	});
}

#[test]
fn transfer_proof_non_owner() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic
		assert_ok!(PoeModule::create_claim(Origin::signed(2), vec![0x1;8]));
		assert_err!(PoeModule::transfer_to(Origin::signed(1), vec![0x1;8],2), Error::<Test>::NotProofOwner);
	});
}



