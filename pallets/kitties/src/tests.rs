#![cfg(test)]

use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn create_kitty_should_work() {
	new_test_ext().execute_with(|| {
		// create a kitty with account #10.
		assert_ok!(Kitties::create_kitty(Origin::signed(10)));

		// check that there is now 1 kitty in storage
		assert_eq!(Kitties::all_kitties_count(), 3);

		// check that account #10 owns 1 kitty
		assert_eq!(Kitties::owned_kitty_count(10), 1);

		// check that some random account #5 does not own a kitty
		assert_eq!(Kitties::owned_kitty_count(5), 0);

		// check that this kitty is specifically owned by account #10
		let hash = Kitties::kitty_by_index(2);
		assert_eq!(Kitties::owner_of(hash), Some(10));

		let other_hash = Kitties::kitty_of_owner_by_index((10, 0));
		assert_eq!(hash, other_hash);
	});
}

#[test]
fn transfer_kitty_should_work() {
	new_test_ext().execute_with(|| {
		// check that 10 own a kitty
		assert_ok!(Kitties::create_kitty(Origin::signed(10)));

		assert_eq!(Kitties::owned_kitty_count(10), 1);
		let hash = Kitties::kitty_of_owner_by_index((10, 0));

		// send kitty to 3
		assert_ok!(Kitties::transfer(Origin::signed(10), 3, hash));

		// 10 now has nothing
		assert_eq!(Kitties::owned_kitty_count(10), 0);
		// but 3 does
		assert_eq!(Kitties::owned_kitty_count(3), 1);
		let new_hash = Kitties::kitty_of_owner_by_index((3, 0));
		// and it has the same hash
		assert_eq!(hash, new_hash);
	});
}

#[test]
fn transfer_not_owned_kitty_should_fail() {
	new_test_ext().execute_with(|| {
		// check that 10 own a kitty
		assert_ok!(Kitties::create_kitty(Origin::signed(10)));
		let hash = Kitties::kitty_of_owner_by_index((10, 0));

		// account 0 cannot transfer a kitty with this hash.
		assert_noop!(
			Kitties::transfer(Origin::signed(9), 1, hash),
			"You do not own this kitty"
		);
	});
}

#[test]
fn should_build_genesis_kitties() {
	new_test_ext().execute_with(|| {
		// Check that 2nd kitty exists at genesis, with value 100
		let kitty0 = Kitties::kitty_by_index(0);
		let kitty1 = Kitties::kitty_by_index(1);

		// Check we have 2 kitties, as specified
		assert_eq!(Kitties::all_kitties_count(), 2);

		// Check that they are owned correctly
		assert_eq!(Kitties::owner_of(kitty0), Some(0));
		assert_eq!(Kitties::owner_of(kitty1), Some(1));

		// Check owners own the correct amount of kitties
		assert_eq!(Kitties::owned_kitty_count(0), 1);
		assert_eq!(Kitties::owned_kitty_count(2), 0);
	});
}
