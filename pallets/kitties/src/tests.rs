use crate::{Error, mock::*};
// use frame_support::{assert_ok, assert_noop};
use frame_support::{traits::OnFinalize, traits::OnInitialize};
// use super::*;

fn run_to_block(n: u64) {
	while System::block_number() < n {
		Kitties::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Kitties::on_initialize(System::block_number());
	}
}

#[test]
fn owned_kitties_can_append_values() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));
	})
}

#[test]
fn create_kitties_to_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_eq!(Kitties::kitties_count(), 0);
		let _ = Kitties::create(Origin::signed(1));
		let _ = Kitties::create(Origin::signed(2));
		let _ = Kitties::create(Origin::signed(2));
		let _ = Kitties::create(Origin::signed(4));
		assert_eq!(Kitties::kitties_count(), 4);
	})
}