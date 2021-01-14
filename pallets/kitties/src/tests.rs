use super::*;
use crate::{mock::*, Error};
use frame_support::assert_noop;

#[test]
fn owned_kitties_can_append_values() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        let _ = Balances::deposit_creating(&1, 5_000);
        assert_eq!(KittiesModule::create(Origin::signed(1),), Ok(()));
    })
}

#[test]
fn create_kitties_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(KittiesModule::kitties_count(), 0);

        let _ = Balances::deposit_creating(&1, 5_000);
        let _ = Balances::deposit_creating(&2, 5_000);
        let _ = Balances::deposit_creating(&3, 5_000);
        let _ = Balances::deposit_creating(&4, 5_000);

        let _ = KittiesModule::create(Origin::signed(1));
        let _ = KittiesModule::create(Origin::signed(2));
        let _ = KittiesModule::create(Origin::signed(3));
        let _ = KittiesModule::create(Origin::signed(4));

        assert_eq!(KittiesModule::kitties_count(), 4);

        assert_eq!(KittiesModule::kitty_owner(0), Some(1));
        assert_eq!(KittiesModule::kitty_owner(1), Some(2));
        assert_eq!(KittiesModule::kitty_owner(2), Some(3));
        assert_eq!(KittiesModule::kitty_owner(3), Some(4));

        assert_eq!(KittiesModule::owned_kitties(1), vec![0]);
        assert_eq!(KittiesModule::owned_kitties(2), vec![1]);
        assert_eq!(KittiesModule::owned_kitties(3), vec![2]);
        assert_eq!(KittiesModule::owned_kitties(4), vec![3]);

        assert_eq!(last_event(), TestEvent::kitties(RawEvent::Created(4, 3)));
    })
}

#[test]
fn create_kitties_failed_with_notenoughbalance() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(KittiesModule::kitties_count(), 0);

        let _ = Balances::deposit_creating(&1, 100);
        assert_noop!(
            KittiesModule::create(Origin::signed(1)),
            Error::<Test>::NotEnoughBalance
        );
    })
}

#[test]
fn transfer_kitties_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(KittiesModule::kitties_count(), 0);
        let _ = Balances::deposit_creating(&1, 5_000);
        let _ = KittiesModule::create(Origin::signed(1));
        assert_eq!(KittiesModule::kitties_count(), 1);
        assert_eq!(KittiesModule::kitty_owner(0), Some(1));
        assert_eq!(KittiesModule::owned_kitties(1), vec![0]);

        let _ = Balances::deposit_creating(&2, 5_000);
        let _ = KittiesModule::transfer(Origin::signed(1), 2, 0);
        assert_eq!(KittiesModule::kitties_count(), 1);
        assert_eq!(KittiesModule::kitty_owner(0), Some(2));
        assert_eq!(KittiesModule::owned_kitties(1).len(), 0);
        assert_eq!(KittiesModule::owned_kitties(2), vec![0]);

        assert_eq!(
            last_event(),
            TestEvent::kitties(RawEvent::Transferred(1, 2, 0))
        );
    })
}

#[test]
fn transfer_kitties_failed_with_invalidkittyid() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        let _ = KittiesModule::create(Origin::signed(1));

        assert_noop!(
            KittiesModule::transfer(Origin::signed(1), 2, 1),
            Error::<Test>::NotKittyOwner
        );
    })
}

#[test]
fn transfer_kitties_failed_with_indenticalaccountid() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        let _ = Balances::deposit_creating(&1, 5_000);
        let _ = KittiesModule::create(Origin::signed(1));

        assert_noop!(
            KittiesModule::transfer(Origin::signed(1), 1, 0),
            Error::<Test>::IndenticalAccountId
        );
    })
}

#[test]
fn transfer_kitties_failed_with_notenoughbalance() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        let _ = Balances::deposit_creating(&1, 5_000);
        let _ = KittiesModule::create(Origin::signed(1));

        let _ = Balances::deposit_creating(&2, 100);
        assert_noop!(
            KittiesModule::transfer(Origin::signed(1), 2, 0),
            Error::<Test>::NotEnoughBalance
        );
    })
}

#[test]
fn breed_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(KittiesModule::kitties_count(), 0);
        let _ = Balances::deposit_creating(&1, 5_000);
        let _ = Balances::deposit_creating(&2, 5_000);
        let _ = KittiesModule::create(Origin::signed(1));
        let _ = KittiesModule::create(Origin::signed(2));
        assert_eq!(KittiesModule::kitties_count(), 2);
        assert_eq!(KittiesModule::kitty_owner(0), Some(1));
        assert_eq!(KittiesModule::owned_kitties(1), vec![0]);

        let _ = KittiesModule::breed(Origin::signed(1), 0, 1);
        assert_eq!(KittiesModule::kitties_count(), 3);
        assert_eq!(KittiesModule::kitty_owner(2), Some(1));
        assert_eq!(KittiesModule::kitty_parents(2), vec![0, 1]);
        assert_eq!(KittiesModule::kitty_children(0), vec![2]);
        assert_eq!(KittiesModule::kitty_children(1), vec![2]);

        assert_eq!(
            last_event(),
            TestEvent::kitties(RawEvent::Breed(1, 0, 1, 2))
        );
    })
}

#[test]
fn breed_kitties_failed_with_notenoughbalance() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        let _ = Balances::deposit_creating(&1, 1_000);
        let _ = Balances::deposit_creating(&2, 5_000);
        let _ = KittiesModule::create(Origin::signed(1));
        let _ = KittiesModule::create(Origin::signed(2));

        let _ = Balances::deposit_creating(&2, 100);
        assert_noop!(
            KittiesModule::breed(Origin::signed(1), 0, 1),
            Error::<Test>::NotEnoughBalance
        );
    })
}
