use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		assert_eq!(MyPallet::init_storage(), Some(8888));
		assert_eq!(MyPallet::stored_value(), None);
		assert_ok!(MyPallet::simple(Origin::signed(1), 123));
		assert_eq!(MyPallet::stored_value(), Some(123));
		MyPallet::output_something(Origin::signed(1)).unwrap();
		// // Read pallet storage and assert an expected result.
		// assert_eq!(TemplateModule::something(), Some(42));
		assert_eq!(1, 1);
	});
}
