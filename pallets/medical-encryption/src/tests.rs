use crate::mock::new_test_ext;

#[test]
fn placeholder_test() {
	new_test_ext().execute_with(|| {
		assert!(true);
	});
}
