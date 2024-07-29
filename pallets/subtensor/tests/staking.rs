#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

use frame_support::pallet_prelude::{
    InvalidTransaction, TransactionValidity, TransactionValidityError,
};
use frame_support::traits::{OnFinalize, OnIdle, OnInitialize};
use frame_support::weights::Weight;
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use frame_system::Config;
mod mock;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use mock::*;
use pallet_balances::Call as BalancesCall;
use pallet_subtensor::*;
use sp_core::{H256, U256};
use sp_runtime::traits::SignedExtension;

/***********************************************************
    staking::add_stake() tests
************************************************************/

#[test]
#[cfg(not(tarpaulin))]
fn test_add_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let amount_staked = 5000;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
            hotkey,
            amount_staked,
        });
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(1_074_000_000, 0),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
    });
}
#[test]
fn test_add_stake_ok_no_emission() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        // Check we have zero staked before transfer
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Transfer to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            10000
        ));

        // Check if stake has increased
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            9999
        );

        // Check if balance has decreased
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 1);

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), 9999);
    });
}

#[test]
fn test_dividends_with_run_to_block() {
    new_test_ext(1).execute_with(|| {
        let neuron_src_hotkey_id = U256::from(1);
        let neuron_dest_hotkey_id = U256::from(2);
        let coldkey_account_id = U256::from(667);
        let netuid: u16 = 1;

        let initial_stake: u64 = 5000;

        //add network
        add_network(netuid, 13, 0);

        // Register neuron, this will set a self weight
        SubtensorModule::set_max_registrations_per_block(netuid, 3);
        SubtensorModule::set_max_allowed_uids(1, 5);

        register_ok_neuron(netuid, U256::from(0), coldkey_account_id, 2112321);
        register_ok_neuron(netuid, neuron_src_hotkey_id, coldkey_account_id, 192213123);
        register_ok_neuron(netuid, neuron_dest_hotkey_id, coldkey_account_id, 12323);

        // Add some stake to the hotkey account, so we can test for emission before the transfer takes place
        SubtensorModule::increase_stake_on_hotkey_account(&neuron_src_hotkey_id, initial_stake);

        // Check if the initial stake has arrived
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&neuron_src_hotkey_id),
            initial_stake
        );

        // Check if all three neurons are registered
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 3);

        // Run a couple of blocks to check if emission works
        run_to_block(2);

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&neuron_src_hotkey_id),
            initial_stake
        );

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&neuron_dest_hotkey_id),
            0
        );
    });
}

#[test]
fn test_add_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(654); // bogus
        let amount = 20000; // Not used

        let result = SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::none(),
            hotkey_account_id,
            amount,
        );
        assert_eq!(result, DispatchError::BadOrigin.into());
    });
}

#[test]
fn test_add_stake_not_registered_key_pair() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(435445);
        let hotkey_account_id = U256::from(54544);
        let amount = 1337;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1800);
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                amount
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
    });
}

#[test]
fn test_add_stake_err_neuron_does_not_belong_to_coldkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let other_cold_key = U256::from(99498);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&other_cold_key, 100000);

        // Perform the request which is signed by a different cold key
        let result = SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(other_cold_key),
            hotkey_id,
            1000,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
    });
}

#[test]
fn test_add_stake_err_not_enough_belance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // Lets try to stake with 0 balance in cold key account
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_id), 0);
        let result = SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_id),
            hotkey_id,
            60000,
        );

        assert_eq!(result, Err(Error::<Test>::NotEnoughBalanceToStake.into()));
    });
}

#[test]
#[ignore]
fn test_add_stake_total_balance_no_change() {
    // When we add stake, the total balance of the coldkey account should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(551337);
        let coldkey_account_id = U256::from(51337);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Give it some $$$ in his coldkey balance
        let initial_balance = 10000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);

        // Check we have zero staked before transfer
        let initial_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(initial_stake, 0);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            10000
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(new_stake, 10000);

        // Check if free balance has decreased
        let new_free_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        assert_eq!(new_free_balance, 0);

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), 10000);

        // Check if total balance has remained the same. (no fee, includes reserved/locked balance)
        let total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(total_balance, initial_total_balance);
    });
}

#[test]
#[ignore]
fn test_add_stake_total_issuance_no_change() {
    // When we add stake, the total issuance of the balances pallet should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Give it some $$$ in his coldkey balance
        let initial_balance = 10000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);

        // Check we have zero staked before transfer
        let initial_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(initial_stake, 0);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Check total issuance is equal to initial balance
        let initial_total_issuance = Balances::total_issuance();
        assert_eq!(initial_total_issuance, initial_balance);

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            10000
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(new_stake, 10000);

        // Check if free balance has decreased
        let new_free_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        assert_eq!(new_free_balance, 0);

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), 10000);

        // Check if total issuance has remained the same. (no fee, includes reserved/locked balance)
        let total_issuance = Balances::total_issuance();
        assert_eq!(total_issuance, initial_total_issuance);
    });
}

#[test]
fn test_reset_stakes_per_interval() {
    new_test_ext(0).execute_with(|| {
        let coldkey = U256::from(561330);
        let hotkey = U256::from(561337);

        SubtensorModule::set_stake_interval(7);
        SubtensorModule::set_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey, 5, 1);
        step_block(1);

        assert_eq!(
            SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey),
            5
        );

        // block: 7 interval not yet passed
        step_block(6);
        assert_eq!(
            SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey),
            5
        );

        // block 8: interval passed
        step_block(1);
        assert_eq!(
            SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey),
            0
        );
    });
}

#[test]
fn test_add_stake_under_limit() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;
        let max_stakes = 2;

        SubtensorModule::set_target_stakes_per_interval(max_stakes);
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            1,
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            1,
        ));

        let current_stakes = SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
        );
        assert!(current_stakes <= max_stakes);
    });
}

#[test]
fn test_add_stake_rate_limit_exceeded() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;
        let max_stakes = 2;
        let block_number = 1;

        SubtensorModule::set_target_stakes_per_interval(max_stakes);
        SubtensorModule::set_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
            max_stakes,
            block_number,
        );

        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
        assert_err!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                1,
            ),
            Error::<Test>::StakeRateLimitExceeded
        );

        let current_stakes = SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
        );
        assert_eq!(current_stakes, max_stakes);
    });
}

// /***********************************************************
// 	staking::remove_stake() tests
// ************************************************************/
#[test]
fn test_remove_stake_under_limit() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;
        let max_unstakes = 2;

        SubtensorModule::set_target_stakes_per_interval(max_unstakes);
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, 2);

        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            1,
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            1,
        ));

        let current_unstakes = SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
        );
        assert!(current_unstakes <= max_unstakes);
    });
}

#[test]
fn test_remove_stake_rate_limit_exceeded() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;
        let max_unstakes = 1;
        let block_number = 1;

        SubtensorModule::set_target_stakes_per_interval(max_unstakes);
        SubtensorModule::set_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
            max_unstakes,
            block_number,
        );

        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, 2);
        assert_err!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                2,
            ),
            Error::<Test>::UnstakeRateLimitExceeded
        );

        let current_unstakes = SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
        );
        assert_eq!(current_unstakes, max_unstakes);
    });
}

#[test]
#[cfg(not(tarpaulin))]
fn test_remove_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let amount_unstaked = 5000;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::remove_stake {
            hotkey,
            amount_unstaked,
        });
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(1_061_000_000, 0)
                    .add_proof_size(43991),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
    });
}

#[test]
fn test_remove_stake_ok_no_emission() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(4343);
        let hotkey_account_id = U256::from(4968585);
        let amount = 10000;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Let's spin up a neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, amount);

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            amount
        ));

        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 0);
    });
}

#[test]
fn test_remove_stake_amount_zero() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(4343);
        let hotkey_account_id = U256::from(4968585);
        let amount = 10000;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Let's spin up a neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, amount);

        // Do the magic
        assert_noop!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                0
            ),
            Error::<Test>::StakeToWithdrawIsZero
        );
    });
}

#[test]
fn test_remove_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(4968585);
        let amount = 10000; // Amount to be removed

        let result = SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::none(),
            hotkey_account_id,
            amount,
        );
        assert_eq!(result, DispatchError::BadOrigin.into());
    });
}

#[test]
fn test_remove_stake_err_hotkey_does_not_belong_to_coldkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let other_cold_key = U256::from(99498);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // Perform the request which is signed by a different cold key
        let result = SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(other_cold_key),
            hotkey_id,
            1000,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
    });
}

#[test]
fn test_remove_stake_no_enough_stake() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let amount = 10000;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

        let result = SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_id),
            hotkey_id,
            amount,
        );
        assert_eq!(result, Err(Error::<Test>::NotEnoughStakeToWithdraw.into()));
    });
}

#[test]
fn test_remove_stake_total_balance_no_change() {
    // When we remove stake, the total balance of the coldkey account should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    //    then the removed stake just becomes free balance
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(571337);
        let coldkey_account_id = U256::from(71337);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        let amount = 10000;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, amount);

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            amount
        ));

        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Check total balance is equal to the added stake. Even after remove stake (no fee, includes reserved/locked balance)
        let total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(total_balance, amount);
    });
}

#[test]
#[ignore]
fn test_remove_stake_total_issuance_no_change() {
    // When we remove stake, the total issuance of the balances pallet should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    //    then the removed stake just becomes free balance
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(581337);
        let coldkey_account_id = U256::from(81337);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        let amount = 10000;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);
        let inital_total_issuance = Balances::total_issuance();
        assert_eq!(inital_total_issuance, 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, amount);

        let total_issuance_after_stake = Balances::total_issuance();

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            amount
        ));

        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Check if total issuance is equal to the added stake, even after remove stake (no fee, includes reserved/locked balance)
        // Should also be equal to the total issuance after adding stake
        let total_issuance = Balances::total_issuance();
        assert_eq!(total_issuance, total_issuance_after_stake);
        assert_eq!(total_issuance, amount);
    });
}

/***********************************************************
    staking::get_coldkey_balance() tests
************************************************************/
#[test]
fn test_get_coldkey_balance_no_balance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(5454); // arbitrary
        let result = SubtensorModule::get_coldkey_balance(&coldkey_account_id);

        // Arbitrary account should have 0 balance
        assert_eq!(result, 0);
    });
}

#[test]
fn test_get_coldkey_balance_with_balance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(5454); // arbitrary
        let amount = 1337;

        // Put the balance on the account
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        let result = SubtensorModule::get_coldkey_balance(&coldkey_account_id);

        // Arbitrary account should have 0 balance
        assert_eq!(result, amount);
    });
}

// /***********************************************************
// 	staking::add_stake_to_hotkey_account() tests
// ************************************************************/
#[test]
fn test_add_stake_to_hotkey_account_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(5445);
        let coldkey_id = U256::from(5443433);
        let amount: u64 = 10000;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // There is not stake in the system at first, so result should be 0;
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, amount);

        // The stake that is now in the account, should equal the amount
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount
        );

        // The total stake should have been increased by the amount -> 0 + amount = amount
        assert_eq!(SubtensorModule::get_total_stake(), amount);
    });
}

/************************************************************
    staking::remove_stake_from_hotkey_account() tests
************************************************************/
#[test]
fn test_remove_stake_from_hotkey_account() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(5445);
        let coldkey_id = U256::from(5443433);
        let amount: u64 = 10000;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // Add some stake that can be removed
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, amount);

        // Prelimiary checks
        assert_eq!(SubtensorModule::get_total_stake(), amount);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount
        );

        // Remove stake
        SubtensorModule::decrease_stake_on_hotkey_account(&hotkey_id, amount);

        // The stake on the hotkey account should be 0
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

        // The total amount of stake should be 0
        assert_eq!(SubtensorModule::get_total_stake(), 0);
    });
}

#[test]
fn test_remove_stake_from_hotkey_account_registered_in_various_networks() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(5445);
        let coldkey_id = U256::from(5443433);
        let amount: u64 = 10000;
        let netuid: u16 = 1;
        let netuid_ex = 2;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        //
        add_network(netuid, tempo, 0);
        add_network(netuid_ex, tempo, 0);
        //
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        register_ok_neuron(netuid_ex, hotkey_id, coldkey_id, 48141209);

        //let neuron_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id);

        let neuron_uid = match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(k) => k,
            Err(e) => panic!("Error: {:?}", e),
        };
        //let neuron_uid_ex = SubtensorModule::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id);

        let neuron_uid_ex = match SubtensorModule::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id)
        {
            Ok(k) => k,
            Err(e) => panic!("Error: {:?}", e),
        };
        //Add some stake that can be removed
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, amount);

        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            amount
        );

        // Remove stake
        SubtensorModule::decrease_stake_on_hotkey_account(&hotkey_id, amount);
        //
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            0
        );
    });
}

// /************************************************************
// 	staking::increase_total_stake() tests
// ************************************************************/
#[test]
fn test_increase_total_stake_ok() {
    new_test_ext(1).execute_with(|| {
        let increment = 10000;
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        SubtensorModule::increase_total_stake(increment);
        assert_eq!(SubtensorModule::get_total_stake(), increment);
    });
}

// /************************************************************
// 	staking::decrease_total_stake() tests
// ************************************************************/
#[test]
fn test_decrease_total_stake_ok() {
    new_test_ext(1).execute_with(|| {
        let initial_total_stake = 10000;
        let decrement = 5000;

        SubtensorModule::increase_total_stake(initial_total_stake);
        SubtensorModule::decrease_total_stake(decrement);

        // The total stake remaining should be the difference between the initial stake and the decrement
        assert_eq!(
            SubtensorModule::get_total_stake(),
            initial_total_stake - decrement
        );
    });
}

// /************************************************************
// 	staking::add_balance_to_coldkey_account() tests
// ************************************************************/
#[test]
fn test_add_balance_to_coldkey_account_ok() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(4444322);
        let amount = 50000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_id, amount);
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_id), amount);
    });
}

// /***********************************************************
// 	staking::remove_balance_from_coldkey_account() tests
// ************************************************************/
#[test]
fn test_remove_balance_from_coldkey_account_ok() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(434324); // Random
        let ammount = 10000; // Arbitrary
                             // Put some $$ on the bank
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, ammount);
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            ammount
        );
        // Should be able to withdraw without hassle
        let result =
            SubtensorModule::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
        assert!(result.is_ok());
    });
}

#[test]
fn test_remove_balance_from_coldkey_account_failed() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(434324); // Random
        let ammount = 10000; // Arbitrary

        // Try to remove stake from the coldkey account. This should fail,
        // as there is no balance, nor does the account exist
        let result =
            SubtensorModule::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
        assert_eq!(result, Err(Error::<Test>::ZeroBalanceAfterWithdrawn.into()));
    });
}

//************************************************************
// 	staking::hotkey_belongs_to_coldkey() tests
// ************************************************************/
#[test]
fn test_hotkey_belongs_to_coldkey_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(4434334);
        let coldkey_id = U256::from(34333);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_id),
            coldkey_id
        );
    });
}
// /************************************************************
// 	staking::can_remove_balance_from_coldkey_account() tests
// ************************************************************/
#[test]
fn test_can_remove_balane_from_coldkey_account_ok() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(87987984);
        let initial_amount = 10000;
        let remove_amount = 5000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
        assert!(SubtensorModule::can_remove_balance_from_coldkey_account(
            &coldkey_id,
            remove_amount
        ));
    });
}

#[test]
fn test_can_remove_balance_from_coldkey_account_err_insufficient_balance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(87987984);
        let initial_amount = 10000;
        let remove_amount = 20000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
        assert!(!SubtensorModule::can_remove_balance_from_coldkey_account(
            &coldkey_id,
            remove_amount
        ));
    });
}
/************************************************************
    staking::has_enough_stake() tests
************************************************************/
#[test]
fn test_has_enough_stake_yes() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(4334);
        let coldkey_id = U256::from(87989);
        let intial_amount = 10000;
        let netuid = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, intial_amount);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            10000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id),
            10000
        );
        assert!(SubtensorModule::has_enough_stake(
            &coldkey_id,
            &hotkey_id,
            5000
        ));
    });
}

#[test]
fn test_has_enough_stake_no() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(4334);
        let coldkey_id = U256::from(87989);
        let intial_amount = 0;
        let netuid = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, intial_amount);
        assert!(!SubtensorModule::has_enough_stake(
            &coldkey_id,
            &hotkey_id,
            5000
        ));
    });
}

#[test]
fn test_non_existent_account() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &U256::from(0),
            &(U256::from(0)),
            10,
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&U256::from(0), &U256::from(0)),
            10
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&(U256::from(0))),
            10
        );
    });
}

/************************************************************
    staking::delegating
************************************************************/

#[test]
fn test_delegate_stake_division_by_zero_check() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(3);
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 2341312);
        assert_ok!(SubtensorModule::become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey
        ));
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey, 0, 1000);
    });
}

#[test]
#[cfg(not(tarpaulin))]
fn test_full_with_delegating() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        // Make two accounts.
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);

        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);
        add_network(netuid, 0, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 4);
        SubtensorModule::set_target_registrations_per_interval(netuid, 4);
        SubtensorModule::set_max_allowed_uids(netuid, 4); // Allow all 4 to be registered at once
        SubtensorModule::set_target_stakes_per_interval(10); // Increase max stakes per interval

        // Neither key can add stake because they dont have fundss.
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 60000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 60000);

        // We have enough, but the keys are not registered.
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );

        // Cant remove either.
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );

        // Neither key can become a delegate either because we are not registered.
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );

        // Register the 2 neurons to a new network.
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey0),
            coldkey0
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey1),
            coldkey1
        );
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey0, &hotkey0));
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey1, &hotkey1));

        // We try to delegate stake but niether are allowing delegation.
        assert!(!SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(!SubtensorModule::hotkey_is_delegate(&hotkey1));
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                100
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                100
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        // We stake and all is ok.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            100
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            100
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 100);
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey0 ), 100 );
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey1 ), 100 );
        assert_eq!(SubtensorModule::get_total_stake(), 200);

        // Cant remove these funds because we are not delegating.
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 100);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 200);

        // Try allowing the keys to become delegates, fails because of incorrect coldkeys.
        // Set take to be 0.
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                0
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                0
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        // Become delegates all is ok.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            SubtensorModule::get_min_take()
        ));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey1));

        // Cant become a delegate twice.
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                SubtensorModule::get_min_take()
            ),
            Err(Error::<Test>::HotKeyAlreadyDelegate.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                u16::MAX / 10
            ),
            Err(Error::<Test>::HotKeyAlreadyDelegate.into())
        );

        // This add stake works for delegates.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            200
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            200
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            300
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            300
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            200
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 500);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 400);
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey0 ), 400 );
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey1 ), 500 );
        assert_eq!(SubtensorModule::get_total_stake(), 900);

        // Lets emit inflation through the hot and coldkeys.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 1000);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 1000);

        // validator_take = take * validator_emission = 10% * 1000 = 100
        // old_stake + (validator_emission - validator_take) * stake_for_coldkey_and_hotkey / total_stake_for_hotkey + validator_take
        // =
        // 200 + 900 * 200 / 500 + 100 = 660
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            654
        );
        // validator_take = take * validator_emission = 9% * 1000 = 90
        // old_stake + (validator_emission - validator_take) * stake_for_coldkey_and_hotkey / total_stake_for_hotkey
        // =
        // 200 + (1000 - 90) * 200 / 400 = 655 ~ 654
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            655
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            846
        ); // 300 + 910 x ( 300 / 500 ) = 300 + 546 = 846
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            745
        ); // 200 + 1090 x ( 200 / 400 )  = 300 + 545 = 745
        assert_eq!(SubtensorModule::get_total_stake(), 2900); // 600 + 700 + 900 + 750 = 2900

        // // Try unstaking too much.
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );

        // unstaking is ok.
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            100
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            100
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            100
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            100
        ));

        // All the amounts have been decreased.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            554
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            555
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            746
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            645
        );

        // Lets register and stake a new key.
        let hotkey2 = U256::from(5);
        let coldkey2 = U256::from(6);
        register_ok_neuron(netuid, hotkey2, coldkey2, 248_123);
        assert!(SubtensorModule::is_hotkey_registered_on_any_network(
            &hotkey0
        ));
        assert!(SubtensorModule::is_hotkey_registered_on_any_network(
            &hotkey1
        ));

        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 60_000);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            1000
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            900
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey2,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey2,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        // Lets make this new key a delegate with a 10% take.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            SubtensorModule::get_min_take()
        ));

        // Add nominate some stake.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey2,
            1_000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey2,
            1_000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            1_000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2),
            1_000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2),
            1_000
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey2), 3_000);
        assert_eq!(SubtensorModule::get_total_stake(), 5_500);

        // Lets emit inflation through this new key with distributed ownership.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, 0, 1000);
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            1_394
        ); // 1000 + 94 + 900 * (1000/3000) = 1400
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2),
            1_303
        ); // 1000 + 900 * (1000/3000) = 1300
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2),
            1_303
        ); // 1000 + 900 * (1000/3000) = 1300
        assert_eq!(SubtensorModule::get_total_stake(), 6_500); // before + 1_000 = 5_500 + 1_000 = 6_500

        step_block(1);

        // Lets register and stake a new key.
        let hotkey3 = U256::from(7);
        let coldkey3 = U256::from(8);
        register_ok_neuron(netuid, hotkey3, coldkey3, 4124124);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey3, 60000);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey3),
            hotkey3,
            1000
        ));

        step_block(3);

        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey3),
            hotkey3,
            SubtensorModule::get_min_take()
        )); // Full take.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey3,
            1000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey3,
            1000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey3,
            1000
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey3),
            1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey3),
            1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey3),
            1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey3, &hotkey3),
            1000
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey3), 4000);
        assert_eq!(SubtensorModule::get_total_stake(), 10_500);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey3, 0, 1000);
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey3),
            1227
        ); // 1000 + 90% * 1000 * 1000/4000 = 1225
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey3),
            1227
        ); // 1000 + 90% * 1000 * 1000/4000 = 1225
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey3),
            1227
        ); // 1000 + 90% * 1000 * 1000/4000 = 1225
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey3, &hotkey3),
            1319
        ); // 1000 + 25 * 3 + 1000 * 1000/4000 = 1325
        assert_eq!(SubtensorModule::get_total_stake(), 11_500); // before + 1_000 = 10_500 + 1_000 = 11_500
    });
}

// Verify delegates with servers get the full server inflation.
#[test]
fn test_full_with_delegating_some_servers() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        // Make two accounts.
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);

        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);
        SubtensorModule::set_max_registrations_per_block(netuid, 4);
        SubtensorModule::set_max_allowed_uids(netuid, 10); // Allow at least 10 to be registered at once, so no unstaking occurs
        SubtensorModule::set_target_stakes_per_interval(10); // Increase max stakes per interval

        // Neither key can add stake because they dont have fundss.
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 60000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 60000);

        // Register the 2 neurons to a new network.
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey0),
            coldkey0
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey1),
            coldkey1
        );
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey0, &hotkey0));
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey1, &hotkey1));

        // We stake and all is ok.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            100
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            100
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 100);
        assert_eq!(SubtensorModule::get_total_stake(), 200);

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 100);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 200);

        // Become delegates all is ok.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            SubtensorModule::get_min_take()
        ));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey1));

        // This add stake works for delegates.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            200
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            200
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            300
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            300
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            200
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 500);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 400);
        assert_eq!(SubtensorModule::get_total_stake(), 900);

        // Lets emit inflation through the hot and coldkeys.
        // fist emission arg is for a server. This should only go to the owner of the hotkey.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 200, 1_000); // 1_200 total emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 123, 2_000); // 2_123 total emission.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            854
        ); // 200 + (200 + 910 x ( 200 / 500 ))  = 200 + (200 + 400) + 60 = 854
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            846
        ); // 300 + 910 x ( 300 / 500 ) = 300 + 546 = 846
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 1_700); // initial + server emission + validator emission = 799 + 899 = 1_698

        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            1_110
        ); // 200 + (0 + 2000 x ( 200 / 400 )) - 100 = 200 + (1000) - 100= 1_110
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            1_413
        ); // 200 + (123 + 2000 x ( 200 / 400 )) + 100 = 200 + (1_200)+ 100 = 1_423
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 2_523); // 400 + 2_123
        assert_eq!(SubtensorModule::get_total_stake(), 4_223); // 2_100 + 2_123 = 4_223

        // Lets emit MORE inflation through the hot and coldkeys.
        // This time only server emission. This should go to the owner of the hotkey.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 350, 0);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 150, 0);
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            1_204
        ); // + 350 + 54 = 1_204
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            1_110
        ); // No change.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            846
        ); // No change.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            1_563
        ); // 1_323 + 150 + 90 = 1_573
        assert_eq!(SubtensorModule::get_total_stake(), 4_723); // 4_223 + 500 = 4_823

        // Lets register and stake a new key.
        let hotkey2 = U256::from(5);
        let coldkey2 = U256::from(6);
        register_ok_neuron(netuid, hotkey2, coldkey2, 248123);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 60_000);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            1_000
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            900
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey2,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey2,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        assert_eq!(SubtensorModule::get_total_stake(), 5_623); // 4_723 + 900 = 5_623

        // Lets make this new key a delegate with a 9% take.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            SubtensorModule::get_min_take()
        ));

        // Add nominate some stake.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey2,
            1000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey2,
            1000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2),
            1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2),
            1000
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey2), 3_000);
        assert_eq!(SubtensorModule::get_total_stake(), 7_723); // 5_623 + (1_000 + 1_000 + 100) = 7_723

        // Lets emit inflation through this new key with distributed ownership.
        // We will emit 100 server emission, which should go in-full to the owner of the hotkey.
        // We will emit 1000 validator emission, which should be distributed in-part to the nominators.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, 100, 1000);
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            1_494
        ); // 1000 + 100 + 94 + 900 * (1000/3000) = 1000 + 200 + 300 = 1_494
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2),
            1_303
        ); // 1000 + 900 * (1000/3000) = 1000 + 300 = 1_303
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2),
            1_303
        ); // 1000 + 900 * (1000/3000) = 1000 + 300 = 1300
        assert_eq!(SubtensorModule::get_total_stake(), 8_823); // 7_723 + 1_100 = 8_823

        // Lets emit MORE inflation through this new key with distributed ownership.
        // This time we do ONLY server emission
        // We will emit 123 server emission, which should go in-full to the owner of the hotkey.
        // We will emit *0* validator emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, 123, 0);
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            1_617
        ); // 1_500 + 117 = 1_617
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2),
            1_303
        ); // No change.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2),
            1_303
        ); // No change.
        assert_eq!(SubtensorModule::get_total_stake(), 8_946); // 8_823 + 123 = 8_946
    });
}

#[test]
fn test_full_block_emission_occurs() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        // Make two accounts.
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);

        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);
        SubtensorModule::set_max_registrations_per_block(netuid, 4);
        SubtensorModule::set_max_allowed_uids(netuid, 10); // Allow at least 10 to be registered at once, so no unstaking occurs
        SubtensorModule::set_target_stakes_per_interval(10); // Increase max stakes per interval

        // Neither key can add stake because they dont have fundss.
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 60000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 60000);

        // Register the 2 neurons to a new network.
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey0),
            coldkey0
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey1),
            coldkey1
        );
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey0, &hotkey0));
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey1, &hotkey1));

        // We stake and all is ok.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            100
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            100
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 100);
        assert_eq!(SubtensorModule::get_total_stake(), 200);

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 111);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 234);
        // Verify the full emission occurs.
        assert_eq!(SubtensorModule::get_total_stake(), 200 + 111 + 234); // 200 + 111 + 234 = 545

        // Become delegates all is ok.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            SubtensorModule::get_min_take()
        ));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey1));

        // Add some delegate stake
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            200
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            300
        ));

        assert_eq!(SubtensorModule::get_total_stake(), 545 + 500); // 545 + 500 = 1045

        // Lets emit inflation with delegatees, with both validator and server emission
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 200, 1_000); // 1_200 total emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 123, 2_000); // 2_123 total emission.

        assert_eq!(SubtensorModule::get_total_stake(), 1045 + 1_200 + 2_123); // before + 1_200 + 2_123 = 4_368

        // Lets emit MORE inflation through the hot and coldkeys.
        // This time JUSt server emission
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 350, 0);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 150, 0);

        assert_eq!(SubtensorModule::get_total_stake(), 4_368 + 350 + 150); // before + 350 + 150 = 4_868

        // Lastly, do only validator emission

        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 12_948);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 1_874);

        assert_eq!(SubtensorModule::get_total_stake(), 4_868 + 12_948 + 1_874); // before + 12_948 + 1_874 = 19_690
    });
}

/************************************************************
    staking::unstake_all_coldkeys_from_hotkey_account() tests
************************************************************/

#[test]
fn test_unstake_all_coldkeys_from_hotkey_account() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(123570);
        let coldkey0_id = U256::from(123560);

        let coldkey1_id = U256::from(123561);
        let coldkey2_id = U256::from(123562);
        let coldkey3_id = U256::from(123563);

        let amount: u64 = 10000;

        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        // Make subnet
        add_network(netuid, tempo, 0);
        // Register delegate
        register_ok_neuron(netuid, hotkey_id, coldkey0_id, start_nonce);

        match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(_k) => (),
            Err(e) => panic!("Error: {:?}", e),
        }

        //Add some stake that can be removed
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0_id, &hotkey_id, amount);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey1_id,
            &hotkey_id,
            amount + 2,
        );
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey2_id,
            &hotkey_id,
            amount + 3,
        );
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey3_id,
            &hotkey_id,
            amount + 4,
        );

        // Verify free balance is 0 for all coldkeys
        assert_eq!(Balances::free_balance(coldkey0_id), 0);
        assert_eq!(Balances::free_balance(coldkey1_id), 0);
        assert_eq!(Balances::free_balance(coldkey2_id), 0);
        assert_eq!(Balances::free_balance(coldkey3_id), 0);

        // Verify total stake is correct
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount * 4 + (2 + 3 + 4)
        );

        // Run unstake_all_coldkeys_from_hotkey_account
        SubtensorModule::unstake_all_coldkeys_from_hotkey_account(&hotkey_id);

        // Verify total stake is 0
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

        // Vefify stake for all coldkeys is 0
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0_id, &hotkey_id),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1_id, &hotkey_id),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2_id, &hotkey_id),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey3_id, &hotkey_id),
            0
        );

        // Verify free balance is correct for all coldkeys
        assert_eq!(Balances::free_balance(coldkey0_id), amount);
        assert_eq!(Balances::free_balance(coldkey1_id), amount + 2);
        assert_eq!(Balances::free_balance(coldkey2_id), amount + 3);
        assert_eq!(Balances::free_balance(coldkey3_id), amount + 4);
    });
}

#[test]
fn test_unstake_all_coldkeys_from_hotkey_account_single_staker() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(123570);
        let coldkey0_id = U256::from(123560);

        let amount: u64 = 891011;

        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        // Make subnet
        add_network(netuid, tempo, 0);
        // Register delegate
        register_ok_neuron(netuid, hotkey_id, coldkey0_id, start_nonce);

        match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(_) => (),
            Err(e) => panic!("Error: {:?}", e),
        }

        //Add some stake that can be removed
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0_id, &hotkey_id, amount);

        // Verify free balance is 0 for coldkey
        assert_eq!(Balances::free_balance(coldkey0_id), 0);

        // Verify total stake is correct
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount
        );

        // Run unstake_all_coldkeys_from_hotkey_account
        SubtensorModule::unstake_all_coldkeys_from_hotkey_account(&hotkey_id);

        // Verify total stake is 0
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

        // Vefify stake for single coldkey is 0
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0_id, &hotkey_id),
            0
        );

        // Verify free balance is correct for single coldkey
        assert_eq!(Balances::free_balance(coldkey0_id), amount);
    });
}

#[test]
fn test_faucet_ok() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(123560);

        log::info!("Creating work for submission to faucet...");

        let block_number = SubtensorModule::get_current_block_as_u64();
        let difficulty: U256 = U256::from(10_000_000);
        let mut nonce: u64 = 0;
        let mut work: H256 = SubtensorModule::create_seal_hash(block_number, nonce, &coldkey);
        while !SubtensorModule::hash_meets_difficulty(&work, difficulty) {
            nonce += 1;
            work = SubtensorModule::create_seal_hash(block_number, nonce, &coldkey);
        }
        let vec_work: Vec<u8> = SubtensorModule::hash_to_vec(work);

        log::info!("Faucet state: {}", cfg!(feature = "pow-faucet"));

        #[cfg(feature = "pow-faucet")]
        assert_ok!(SubtensorModule::do_faucet(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            block_number,
            nonce,
            vec_work
        ));

        #[cfg(not(feature = "pow-faucet"))]
        assert_ok!(SubtensorModule::do_faucet(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            block_number,
            nonce,
            vec_work
        ));
    });
}

/// This test ensures that the clear_small_nominations function works as expected.
/// It creates a network with two hotkeys and two coldkeys, and then registers a nominator account for each hotkey.
/// When we call set_nominator_min_required_stake, it should clear all small nominations that are below the minimum required stake.
/// Run this test using: cargo test --package pallet-subtensor --test staking test_clear_small_nominations
#[test]
fn test_clear_small_nominations() {
    new_test_ext(0).execute_with(|| {
        System::set_block_number(1);

        // Create accounts.
        let netuid = 1;
        let hot1 = U256::from(1);
        let hot2 = U256::from(2);
        let cold1 = U256::from(3);
        let cold2 = U256::from(4);

        SubtensorModule::set_target_stakes_per_interval(10);
        // Register hot1 and hot2 .
        add_network(netuid, 0, 0);

        // Register hot1.
        register_ok_neuron(netuid, hot1, cold1, 0);
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            hot1,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot1), cold1);

        // Register hot2.
        register_ok_neuron(netuid, hot2, cold2, 0);
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot2,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot2), cold2);

        // Add stake cold1 --> hot1 (non delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold1, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            hot1,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot1),
            1
        );
        assert_eq!(Balances::free_balance(cold1), 4);

        // Add stake cold2 --> hot1 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot1,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot1),
            1
        );
        assert_eq!(Balances::free_balance(cold2), 4);

        // Add stake cold1 --> hot2 (non delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold1, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            hot2,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot2),
            1
        );
        assert_eq!(Balances::free_balance(cold1), 8);

        // Add stake cold2 --> hot2 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot2,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot2),
            1
        );
        assert_eq!(Balances::free_balance(cold2), 8);

        // Run clear all small nominations when min stake is zero (noop)
        SubtensorModule::set_nominator_min_required_stake(0);
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot1),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot2),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot1),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot2),
            1
        );

        // Set min nomination to 10
        let total_cold1_stake_before = TotalColdkeyStake::<Test>::get(cold1);
        let total_cold2_stake_before = TotalColdkeyStake::<Test>::get(cold2);
        let total_hot1_stake_before = TotalHotkeyStake::<Test>::get(hot1);
        let total_hot2_stake_before = TotalHotkeyStake::<Test>::get(hot2);
        let _ = Stake::<Test>::try_get(hot2, cold1).unwrap(); // ensure exists before
        let _ = Stake::<Test>::try_get(hot1, cold2).unwrap(); // ensure exists before
        let total_stake_before = TotalStake::<Test>::get();
        SubtensorModule::set_nominator_min_required_stake(10);

        // Run clear all small nominations (removes delegations under 10)
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot1),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot2),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot2),
            1
        );

        // Balances have been added back into accounts.
        assert_eq!(Balances::free_balance(cold1), 9);
        assert_eq!(Balances::free_balance(cold2), 9);

        // Internal storage is updated
        assert_eq!(
            TotalColdkeyStake::<Test>::get(cold2),
            total_cold2_stake_before - 1
        );
        assert_eq!(
            TotalHotkeyStake::<Test>::get(hot2),
            total_hot2_stake_before - 1
        );
        Stake::<Test>::try_get(hot2, cold1).unwrap_err();
        Stake::<Test>::try_get(hot1, cold2).unwrap_err();
        assert_eq!(
            TotalColdkeyStake::<Test>::get(cold1),
            total_cold1_stake_before - 1
        );
        assert_eq!(
            TotalHotkeyStake::<Test>::get(hot1),
            total_hot1_stake_before - 1
        );
        Stake::<Test>::try_get(hot2, cold1).unwrap_err();
        assert_eq!(TotalStake::<Test>::get(), total_stake_before - 2);
    });
}

/// Test that the nominator minimum staking threshold is enforced when stake is added.
#[test]
fn test_add_stake_below_minimum_threshold() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey1 = U256::from(0);
        let hotkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let minimum_threshold = 10_000_000;
        let amount_below = 50_000;

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 100_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 100_000);
        SubtensorModule::set_nominator_min_required_stake(minimum_threshold);
        SubtensorModule::set_target_stakes_per_interval(10);

        // Create network
        add_network(netuid, 0, 0);

        // Register the neuron to a new network.
        register_ok_neuron(netuid, hotkey1, coldkey1, 0);
        assert_ok!(SubtensorModule::become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1
        ));

        // Coldkey staking on its own hotkey can stake below min threshold.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            amount_below
        ));

        // Nomination stake cannot stake below min threshold.
        assert_noop!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
                hotkey1,
                amount_below
            ),
            pallet_subtensor::Error::<Test>::NomStakeBelowMinimumThreshold
        );
    });
}

/// Test that the nominator minimum staking threshold is enforced when stake is removed.
#[test]
fn test_remove_stake_below_minimum_threshold() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey1 = U256::from(0);
        let hotkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let initial_balance = 200_000_000;
        let initial_stake = 100_000;
        let minimum_threshold = 50_000;
        let stake_amount_to_remove = 51_000;

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, initial_balance);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, initial_balance);
        SubtensorModule::set_nominator_min_required_stake(minimum_threshold);
        SubtensorModule::set_target_stakes_per_interval(10);

        // Create network
        add_network(netuid, 0, 0);

        // Register the neuron to a new network.
        register_ok_neuron(netuid, hotkey1, coldkey1, 0);
        assert_ok!(SubtensorModule::become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            initial_stake
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey1,
            initial_stake
        ));

        // Coldkey staking on its own hotkey can unstake below min threshold.
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            stake_amount_to_remove
        ));

        // Nomination stake cannot unstake below min threshold,
        // without unstaking all and removing the nomination.
        let total_hotkey_stake_before = SubtensorModule::get_total_stake_for_hotkey(&hotkey1);
        let bal_before = Balances::free_balance(coldkey2);
        let staked_before = SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey1);
        let total_network_stake_before = SubtensorModule::get_total_stake();
        let total_issuance_before = SubtensorModule::get_total_issuance();
        // check the premise of the test is correct
        assert!(initial_stake - stake_amount_to_remove < minimum_threshold);
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey1,
            stake_amount_to_remove
        ));

        // Has no stake now
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey1),
            0
        );
        let stake_removed = staked_before; // All stake was removed
                                           // Has the full balance
        assert_eq!(Balances::free_balance(coldkey2), bal_before + stake_removed);

        // Stake map entry is removed
        assert!(Stake::<Test>::try_get(hotkey1, coldkey2).is_err(),);
        // Stake tracking is updated
        assert_eq!(
            TotalColdkeyStake::<Test>::try_get(coldkey2).unwrap(),
            0 // Did not have any stake before; Entry is NOT removed
        );
        assert_eq!(
            TotalHotkeyStake::<Test>::try_get(hotkey1).unwrap(),
            total_hotkey_stake_before - stake_removed // Stake was removed from hotkey1 tracker
        );
        assert_eq!(
            TotalStake::<Test>::try_get().unwrap(),
            total_network_stake_before - stake_removed
        );

        // Total issuance is the same
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            total_issuance_before // Nothing was issued
        );
    });
}

// Verify delegate take can be decreased
#[test]
fn test_delegate_take_can_be_decreased() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );

        // Coldkey / hotkey 0 decreases take to 5%. This should fail as the minimum take is 9%
        assert_err!(
            SubtensorModule::do_decrease_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                u16::MAX / 20
            ),
            Error::<Test>::DelegateTakeTooLow
        );
    });
}

// Verify delegate take can be decreased
#[test]
fn test_can_set_min_take_ok() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            u16::MAX / 10
        ));

        // Coldkey / hotkey 0 decreases take to min
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );
    });
}

// Verify delegate take can not be increased with do_decrease_take
#[test]
fn test_delegate_take_can_not_be_increased_with_decrease_take() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 10% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );

        // Coldkey / hotkey 0 tries to increase take to 12.5%
        assert_eq!(
            SubtensorModule::do_decrease_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                u16::MAX / 8
            ),
            Err(Error::<Test>::DelegateTakeTooLow.into())
        );
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );
    });
}

// Verify delegate take can be increased
#[test]
fn test_delegate_take_can_be_increased() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Coldkey / hotkey 0 decreases take to 12.5%
        assert_ok!(SubtensorModule::do_increase_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            u16::MAX / 8
        ));
        assert_eq!(SubtensorModule::get_hotkey_take(&hotkey0), u16::MAX / 8);
    });
}

// Verify delegate take can not be decreased with increase_take
#[test]
fn test_delegate_take_can_not_be_decreased_with_increase_take() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );

        // Coldkey / hotkey 0 tries to decrease take to 5%
        assert_eq!(
            SubtensorModule::do_increase_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                u16::MAX / 20
            ),
            Err(Error::<Test>::DelegateTakeTooLow.into())
        );
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );
    });
}

// Verify delegate take can be increased up to InitialDefaultTake (18%)
#[test]
fn test_delegate_take_can_be_increased_to_limit() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Coldkey / hotkey 0 tries to increase take to InitialDefaultTake+1
        assert_ok!(SubtensorModule::do_increase_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            InitialDefaultTake::get()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            InitialDefaultTake::get()
        );
    });
}

// Verify delegate take can not be set above InitialDefaultTake
#[test]
fn test_delegate_take_can_not_be_set_beyond_limit() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        let before = SubtensorModule::get_hotkey_take(&hotkey0);

        // Coldkey / hotkey 0 attempt to become delegates with take above maximum
        // (Disable this check if InitialDefaultTake is u16::MAX)
        if InitialDefaultTake::get() != u16::MAX {
            assert_eq!(
                SubtensorModule::do_become_delegate(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                    hotkey0,
                    InitialDefaultTake::get() + 1
                ),
                Err(Error::<Test>::DelegateTakeTooHigh.into())
            );
        }
        assert_eq!(SubtensorModule::get_hotkey_take(&hotkey0), before);
    });
}

// Verify delegate take can not be increased above InitialDefaultTake (18%)
#[test]
fn test_delegate_take_can_not_be_increased_beyond_limit() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );

        // Coldkey / hotkey 0 tries to increase take to InitialDefaultTake+1
        // (Disable this check if InitialDefaultTake is u16::MAX)
        if InitialDefaultTake::get() != u16::MAX {
            assert_eq!(
                SubtensorModule::do_increase_take(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                    hotkey0,
                    InitialDefaultTake::get() + 1
                ),
                Err(Error::<Test>::DelegateTakeTooHigh.into())
            );
        }
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );
    });
}

// Test rate-limiting on increase_take
#[test]
fn test_rate_limits_enforced_on_increase_take() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );

        // Coldkey / hotkey 0 increases take to 12.5%
        assert_eq!(
            SubtensorModule::do_increase_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                u16::MAX / 8
            ),
            Err(Error::<Test>::DelegateTxRateLimitExceeded.into())
        );
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Can increase after waiting
        assert_ok!(SubtensorModule::do_increase_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            u16::MAX / 8
        ));
        assert_eq!(SubtensorModule::get_hotkey_take(&hotkey0), u16::MAX / 8);
    });
}

// Helper function to set up a test environment
fn setup_test_environment() -> (AccountId, AccountId, AccountId) {
    let current_coldkey = U256::from(1);
    let hotkey = U256::from(2);
    let new_coldkey = U256::from(3);
    // Register the neuron to a new network
    let netuid = 1;
    add_network(netuid, 0, 0);

    // Register the hotkey and associate it with the current coldkey
    register_ok_neuron(1, hotkey, current_coldkey, 0);

    // Add some balance to the hotkey
    SubtensorModule::add_balance_to_coldkey_account(&current_coldkey, 1000);

    // Stake some amount
    assert_ok!(SubtensorModule::add_stake(
        RuntimeOrigin::signed(current_coldkey),
        hotkey,
        500
    ));

    (current_coldkey, hotkey, new_coldkey)
}

/// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking -- test_arbitrated_coldkey_swap_success --exact --nocapture
#[test]
fn test_arbitrated_coldkey_swap_success() {
    new_test_ext(1).execute_with(|| {
        let (current_coldkey, hotkey, new_coldkey) = setup_test_environment();

        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work, nonce) = generate_valid_pow(
            &current_coldkey,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &current_coldkey,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &current_coldkey.clone(),
            &new_coldkey,
            work.to_fixed_bytes().to_vec(),
            current_block,
            nonce
        ));

        // Check that ColdkeySwapDestinations is populated correctly
        assert_eq!(
            pallet_subtensor::ColdkeySwapDestinations::<Test>::get(current_coldkey),
            vec![new_coldkey]
        );

        // Check that drain block is set correctly
        let drain_block: u64 = 7200 * 3 + 1;

        log::info!(
            "ColdkeysToSwapAtBlock before scheduling: {:?}",
            pallet_subtensor::ColdkeysToSwapAtBlock::<Test>::get(drain_block)
        );

        assert_eq!(
            pallet_subtensor::ColdkeysToSwapAtBlock::<Test>::get(drain_block),
            vec![current_coldkey]
        );
        log::info!("Drain block set correctly: {:?}", drain_block);
        log::info!(
            "Drain block {:?}",
            pallet_subtensor::ColdkeysToSwapAtBlock::<Test>::get(drain_block)
        );

        // Make 5400 blocks pass
        run_to_block(drain_block);

        // Run unstaking
        SubtensorModule::swap_coldkeys_this_block(&BlockWeights::get().max_block).unwrap();
        log::info!(
            "Arbitrated coldkeys for block: {:?}",
            SubtensorModule::get_current_block_as_u64()
        );

        // Check the hotkey stake.
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey), 500);

        // Get the owner of the hotkey now new key.
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey),
            new_coldkey
        );

        // Check that the balance has been transferred to the new coldkey
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey),
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP + 500
        ); // The new key as the 500
    });
}

/// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking -- test_arbitrated_coldkey_swap_same_coldkey --exact --nocapture
#[test]
fn test_arbitrated_coldkey_swap_same_coldkey() {
    new_test_ext(1).execute_with(|| {
        let (current_coldkey, _hotkey, _) = setup_test_environment();

        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work, nonce) = generate_valid_pow(
            &current_coldkey,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );

        assert_noop!(
            SubtensorModule::do_schedule_coldkey_swap(
                &current_coldkey.clone(),
                &current_coldkey,
                work.to_fixed_bytes().to_vec(),
                current_block,
                nonce
            ),
            Error::<Test>::SameColdkey
        );
    });
}

/// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking -- test_arbitrated_coldkey_swap_no_balance --exact --nocapture
#[test]
fn test_arbitrated_coldkey_swap_no_balance() {
    new_test_ext(1).execute_with(|| {
        // Create accounts manually
        let current_coldkey: AccountId = U256::from(1);
        let hotkey: AccountId = U256::from(2);
        let new_coldkey: AccountId = U256::from(3);

        add_network(1, 0, 0);

        // Register the hotkey and associate it with the current coldkey
        register_ok_neuron(1, hotkey, current_coldkey, 0);

        // Print initial balances
        log::info!(
            "Initial current_coldkey balance: {:?}",
            Balances::total_balance(&current_coldkey)
        );
        log::info!(
            "Initial hotkey balance: {:?}",
            Balances::total_balance(&hotkey)
        );
        log::info!(
            "Initial new_coldkey balance: {:?}",
            Balances::total_balance(&new_coldkey)
        );

        // Ensure there's no balance in any of the accounts
        assert_eq!(Balances::total_balance(&current_coldkey), 0);
        assert_eq!(Balances::total_balance(&hotkey), 0);
        assert_eq!(Balances::total_balance(&new_coldkey), 0);

        // Generate valid PoW
        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work, nonce) = generate_valid_pow(
            &current_coldkey,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );

        // Try to schedule coldkey swap
        let result = SubtensorModule::do_schedule_coldkey_swap(
            &current_coldkey.clone(),
            &new_coldkey,
            work.to_fixed_bytes().to_vec(),
            current_block,
            nonce,
        );

        // Print the result
        log::info!("Result of arbitrated_coldkey_swap: {:?}", result);

        // Verify that the operation failed due to insufficient balance
        assert_noop!(
            result,
            Error::<Test>::InsufficientBalanceToPerformColdkeySwap
        );

        // Print final balances
        log::info!(
            "Final current_coldkey balance: {:?}",
            Balances::total_balance(&current_coldkey)
        );
        log::info!(
            "Final hotkey balance: {:?}",
            Balances::total_balance(&hotkey)
        );
        log::info!(
            "Final new_coldkey balance: {:?}",
            Balances::total_balance(&new_coldkey)
        );

        // Verify that no balance was transferred
        assert_eq!(Balances::total_balance(&current_coldkey), 0);
        assert_eq!(Balances::total_balance(&hotkey), 0);
        assert_eq!(Balances::total_balance(&new_coldkey), 0);
    });
}

// To run this test, use the following command:
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking -- test_arbitrated_coldkey_swap_with_no_stake --exact --nocapture
#[test]
fn test_arbitrated_coldkey_swap_with_no_stake() {
    new_test_ext(1).execute_with(|| {
        // Create accounts manually
        let current_coldkey: AccountId = U256::from(1);
        let hotkey: AccountId = U256::from(2);
        let new_coldkey: AccountId = U256::from(3);

        add_network(1, 0, 0);

        // Register the hotkey and associate it with the current coldkey
        register_ok_neuron(1, hotkey, current_coldkey, 0);

        // Add balance to the current coldkey without staking
        Balances::make_free_balance_be(&current_coldkey, MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP);

        // Print initial balances
        log::info!(
            "Initial current_coldkey balance: {:?}",
            Balances::total_balance(&current_coldkey)
        );
        log::info!(
            "Initial hotkey balance: {:?}",
            Balances::total_balance(&hotkey)
        );
        log::info!(
            "Initial new_coldkey balance: {:?}",
            Balances::total_balance(&new_coldkey)
        );

        // Ensure initial balances are correct
        assert_eq!(
            Balances::total_balance(&current_coldkey),
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP
        );
        assert_eq!(Balances::total_balance(&hotkey), 0);
        assert_eq!(Balances::total_balance(&new_coldkey), 0);

        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work, nonce) = generate_valid_pow(
            &current_coldkey,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );

        // Schedule coldkey swap
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &current_coldkey.clone(),
            &new_coldkey,
            work.to_fixed_bytes().to_vec(),
            current_block,
            nonce
        ));

        // Make 5400 blocks pass, simulating on_idle for each block
        let drain_block: u64 = 7200 * 3 + 1;
        for _ in 0..drain_block {
            next_block();
            SubtensorModule::on_idle(System::block_number(), Weight::MAX);
        }

        // Print final balances
        log::info!(
            "Final current_coldkey balance: {:?}",
            Balances::total_balance(&current_coldkey)
        );
        log::info!(
            "Final hotkey balance: {:?}",
            Balances::total_balance(&hotkey)
        );
        log::info!(
            "Final new_coldkey balance: {:?}",
            Balances::total_balance(&new_coldkey)
        );

        // Check that the balance has been transferred to the new coldkey
        assert_eq!(
            Balances::total_balance(&new_coldkey),
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP
        );
        assert_eq!(Balances::total_balance(&current_coldkey), 0);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_arbitrated_coldkey_swap_with_multiple_stakes --exact --nocapture
#[test]
fn test_arbitrated_coldkey_swap_with_multiple_stakes() {
    new_test_ext(1).execute_with(|| {
        let (current_coldkey, hotkey, new_coldkey) = setup_test_environment();

        SubtensorModule::set_target_stakes_per_interval(10);
        SubtensorModule::add_balance_to_coldkey_account(
            &current_coldkey,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );

        // Add more stake
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(current_coldkey),
            hotkey,
            300
        ));

        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work, nonce) = generate_valid_pow(
            &current_coldkey,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );

        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &current_coldkey.clone(),
            &new_coldkey,
            work.to_fixed_bytes().to_vec(),
            current_block,
            nonce
        ));

        // Make 5400 blocks pass, simulating on_idle for each block
        let drain_block: u64 = 7200 * 3 + 1;
        for _ in 0..drain_block {
            next_block();
            SubtensorModule::on_idle(System::block_number(), Weight::MAX);
        }

        // Check that all stake has been removed
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey), 800);

        // Owner has changed
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey),
            new_coldkey
        );

        // Check that the full balance has been transferred to the new coldkey
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey),
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP + 200
        );

        // Check that the full balance has been transferred to the new coldkey
        assert_eq!(SubtensorModule::get_coldkey_balance(&current_coldkey), 0);
    });
}
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_arbitrated_coldkey_swap_multiple_arbitrations --exact --nocapture
#[test]
fn test_arbitrated_coldkey_swap_multiple_arbitrations() {
    new_test_ext(1).execute_with(|| {
        // Set a very low base difficulty for testing
        BaseDifficulty::<Test>::put(1);

        // Create coldkey with three choices.
        let coldkey: AccountId = U256::from(1);
        let new_coldkey1: AccountId = U256::from(2);
        let new_coldkey2: AccountId = U256::from(3);
        let new_coldkey3: AccountId = U256::from(4);
        let hotkey: AccountId = U256::from(5);

        // Setup network state.
        add_network(1, 0, 0);
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );
        ArbitrationPeriod::<Test>::put(5); // Set arbitration period to 5 blocks
        register_ok_neuron(1, hotkey, coldkey, 0);

        let current_block = SubtensorModule::get_current_block_as_u64();

        // Generate valid PoW for each swap attempt
        let (work1, nonce1) = generate_valid_pow(&coldkey, current_block, U256::from(1));
        let (work2, nonce2) = generate_valid_pow(&coldkey, current_block, U256::from(2));
        let (work3, nonce3) = generate_valid_pow(&coldkey, current_block, U256::from(4));

        // Schedule three swaps
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &coldkey.clone(),
            &new_coldkey1,
            work1.to_fixed_bytes().to_vec(),
            current_block,
            nonce1
        ));
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &coldkey.clone(),
            &new_coldkey2,
            work2.to_fixed_bytes().to_vec(),
            current_block,
            nonce2
        ));
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &coldkey.clone(),
            &new_coldkey3,
            work3.to_fixed_bytes().to_vec(),
            current_block,
            nonce3
        ));

        // All three keys are added in swap destinations.
        assert_eq!(
            pallet_subtensor::ColdkeySwapDestinations::<Test>::get(coldkey),
            vec![new_coldkey1, new_coldkey2, new_coldkey3]
        );

        // Simulate the passage of blocks and on_idle calls
        for i in 0..(7200 * 3 + 1) {
            next_block();
            SubtensorModule::on_idle(System::block_number(), Weight::MAX);

            log::info!(
                "Block {}: Coldkey in arbitration: {}, Swap destinations: {:?}",
                i + 1,
                SubtensorModule::coldkey_in_arbitration(&coldkey),
                pallet_subtensor::ColdkeySwapDestinations::<Test>::get(coldkey)
            );
        }

        // Check that the swap destinations remain unchanged due to multiple (>2) swap calls
        assert_eq!(
            pallet_subtensor::ColdkeySwapDestinations::<Test>::get(coldkey),
            vec![new_coldkey1, new_coldkey2, new_coldkey3],
            "ColdkeySwapDestinations should remain unchanged with more than two swap calls"
        );

        // Key remains in arbitration due to multiple (>2) swap calls
        assert!(
            SubtensorModule::coldkey_in_arbitration(&coldkey),
            "Coldkey should remain in arbitration with more than two swap calls"
        );

        // Check that no balance has been transferred
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey),
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
            "Original coldkey balance should remain unchanged"
        );
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey1),
            0,
            "New coldkey1 should not receive any balance"
        );
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey2),
            0,
            "New coldkey2 should not receive any balance"
        );
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey3),
            0,
            "New coldkey3 should not receive any balance"
        );
    });
}

// TODO: Verify that we never want more than 2 destinations for a coldkey
#[test]
fn test_arbitrated_coldkey_swap_existing_destination() {
    new_test_ext(1).execute_with(|| {
        let (current_coldkey, _hotkey, new_coldkey) = setup_test_environment();
        let another_coldkey = U256::from(4);
        let third_coldkey = U256::from(5);

        let current_block = SubtensorModule::get_current_block_as_u64();

        SubtensorModule::add_balance_to_coldkey_account(
            &current_coldkey,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );

        // First swap attempt (0 existing destinations)
        let difficulty1 = SubtensorModule::calculate_pow_difficulty(0);
        let (work1, nonce1) = generate_valid_pow(&current_coldkey, current_block, difficulty1);

        // Schedule a swap to new_coldkey
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &current_coldkey,
            &new_coldkey,
            work1.to_fixed_bytes().to_vec(),
            current_block,
            nonce1
        ));

        // Second swap attempt (1 existing destination)
        let difficulty2 = SubtensorModule::calculate_pow_difficulty(1);
        let (work2, nonce2) = generate_valid_pow(&current_coldkey, current_block, difficulty2);

        // Attempt to schedule a swap to the same new_coldkey again
        assert_noop!(
            SubtensorModule::do_schedule_coldkey_swap(
                &current_coldkey.clone(),
                &new_coldkey,
                work2.to_fixed_bytes().to_vec(),
                current_block,
                nonce2
            ),
            Error::<Test>::DuplicateColdkey
        );

        // Schedule a swap to another_coldkey (still 1 existing destination)
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &current_coldkey.clone(),
            &another_coldkey,
            work2.to_fixed_bytes().to_vec(),
            current_block,
            nonce2
        ));

        // Third swap attempt (2 existing destinations)
        let difficulty3 = SubtensorModule::calculate_pow_difficulty(2);
        let (work3, nonce3) = generate_valid_pow(&current_coldkey, current_block, difficulty3);

        // Attempt to schedule a third swap
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &current_coldkey.clone(),
            &third_coldkey,
            work3.to_fixed_bytes().to_vec(),
            current_block,
            nonce3
        ));
    });
}

#[test]
fn test_arbitration_period_extension() {
    new_test_ext(1).execute_with(|| {
        let (current_coldkey, _hotkey, new_coldkey) = setup_test_environment();
        let another_coldkey = U256::from(4);

        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work1, nonce1) = generate_valid_pow(
            &current_coldkey,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );
        let (work2, nonce2) =
            generate_valid_pow(&current_coldkey, current_block, U256::from(20_000_000u64));
        SubtensorModule::add_balance_to_coldkey_account(
            &current_coldkey,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );

        // Schedule a swap to new_coldkey
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &current_coldkey.clone(),
            &new_coldkey,
            work1.to_fixed_bytes().to_vec(),
            current_block,
            nonce1
        ));

        // Schedule a swap to another_coldkey
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &current_coldkey.clone(),
            &another_coldkey,
            work2.to_fixed_bytes().to_vec(),
            current_block,
            nonce2
        ));

        // Check that the arbitration period is extended
        let arbitration_block =
            SubtensorModule::get_current_block_as_u64() + ArbitrationPeriod::<Test>::get();
        assert_eq!(
            pallet_subtensor::ColdkeyArbitrationBlock::<Test>::get(current_coldkey),
            arbitration_block
        );
    });
}

#[test]
fn test_concurrent_arbitrated_coldkey_swaps() {
    new_test_ext(1).execute_with(|| {
        // Manually create accounts
        let coldkey1: AccountId = U256::from(1);
        let hotkey1: AccountId = U256::from(2);
        let new_coldkey1: AccountId = U256::from(3);

        let coldkey2: AccountId = U256::from(4);
        let hotkey2: AccountId = U256::from(5);
        let new_coldkey2: AccountId = U256::from(6);

        // Add networks
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        add_network(netuid1, 13, 0);
        add_network(netuid2, 13, 0);

        // Register neurons in different networks
        register_ok_neuron(netuid1, hotkey1, coldkey1, 0);
        register_ok_neuron(netuid2, hotkey2, coldkey2, 0);

        // Add balance to coldkeys
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey1,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey2,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );

        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work1, nonce1) = generate_valid_pow(
            &coldkey1,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );
        let (work2, nonce2) = generate_valid_pow(
            &coldkey2,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );
        // Schedule swaps for both coldkeys
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &coldkey1.clone(),
            &new_coldkey1,
            work1.to_fixed_bytes().to_vec(),
            current_block,
            nonce1
        ));
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &coldkey2.clone(),
            &new_coldkey2,
            work2.to_fixed_bytes().to_vec(),
            current_block,
            nonce2
        ));
        // Make 5400 blocks pass
        let drain_block: u64 = 7200 * 3 + 1;
        run_to_block(drain_block);

        // Run arbitration
        SubtensorModule::swap_coldkeys_this_block(&BlockWeights::get().max_block).unwrap();

        // Check that the balances have been transferred correctly
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey1),
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP
        );
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey2),
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP
        );
    });
}

// #[test]
// fn test_get_remaining_arbitration_period() {
//     new_test_ext(1).execute_with(|| {
//         let coldkey_account_id = U256::from(12345); // arbitrary coldkey
//         let new_coldkey_account_id = U256::from(54321); // arbitrary new coldkey

//         let current_block = SubtensorModule::get_current_block_as_u64();
//         let (work, nonce) = generate_valid_pow(
//             &coldkey_account_id,
//             current_block,
//             U256::from(BaseDifficulty::<Test>::get()),
//         );

//         SubtensorModule::add_balance_to_coldkey_account(
//             &coldkey_account_id,
//             MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
//         );

//         // Schedule a coldkey swap to set the arbitration block
//         assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
//             &coldkey_account_id.clone(),
//             &new_coldkey_account_id,
//             work.to_fixed_bytes().to_vec(),
//             current_block,
//             nonce
//         ));

//         // Get the current block number and arbitration period
//         let current_block: u64 = SubtensorModule::get_current_block_as_u64();
//         let arbitration_period: u64 = ArbitrationPeriod::<Test>::get();
//         log::info!("arbitration_period: {:?}", arbitration_period);
//         let arbitration_block: u64 = current_block + arbitration_period;
//         log::info!("arbitration_block: {:?}", arbitration_block);

//         // Check if the remaining arbitration period is correct
//         let remaining_period =
//             SubtensorModule::get_remaining_arbitration_period(&coldkey_account_id);
//         assert_eq!(remaining_period, arbitration_period);

//         // Move the current block forward and check again
//         step_block(50);
//         let remaining_period =
//             SubtensorModule::get_remaining_arbitration_period(&coldkey_account_id);
//         assert_eq!(remaining_period, arbitration_period - 50);

//         // Move the current block beyond the arbitration block and check again
//         step_block((arbitration_period as u16) - 50 + 1);
//         let remaining_period =
//             SubtensorModule::get_remaining_arbitration_period(&coldkey_account_id);
//         assert_eq!(remaining_period, 0);
//     });
// }

#[test]
fn test_transfer_coldkey_in_arbitration() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(1);
        let recipient_account_id = U256::from(2);
        let new_coldkey_account_id = U256::from(3);

        // Add balance to coldkey
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey_account_id,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );

        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work, nonce) = generate_valid_pow(
            &coldkey_account_id,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );

        // Schedule a coldkey swap to put the coldkey in arbitration
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &coldkey_account_id.clone(),
            &new_coldkey_account_id,
            work.to_fixed_bytes().to_vec(),
            current_block,
            nonce
        ));

        // Try to transfer balance
        let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: recipient_account_id,
            value: 1000,
        });

        assert_eq!(
            validate_transaction(&coldkey_account_id, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
        );
    });
}

#[test]
fn test_add_stake_coldkey_in_arbitration() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let new_coldkey_account_id = U256::from(71337);
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;

        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey_account_id,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );

        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work, nonce) = generate_valid_pow(
            &coldkey_account_id,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );

        // Schedule a coldkey swap to put the coldkey in arbitration
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &coldkey_account_id.clone(),
            &new_coldkey_account_id,
            work.to_fixed_bytes().to_vec(),
            current_block,
            nonce
        ));
        let call = RuntimeCall::SubtensorModule(crate::Call::add_stake {
            hotkey: hotkey_account_id,
            amount_staked: 1000,
        });

        // This should now be Ok
        assert!(validate_transaction(&coldkey_account_id, &call).is_ok());
    })
}

#[test]
fn test_remove_stake_coldkey_in_arbitration() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let new_coldkey_account_id = U256::from(71337);
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;

        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey_account_id,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, 1000);

        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work, nonce) = generate_valid_pow(
            &coldkey_account_id,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );

        // Schedule a coldkey swap to put the coldkey in arbitration
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &coldkey_account_id.clone(),
            &new_coldkey_account_id,
            work.to_fixed_bytes().to_vec(),
            current_block,
            nonce
        ));

        let call = RuntimeCall::SubtensorModule(crate::Call::remove_stake {
            hotkey: hotkey_account_id,
            amount_unstaked: 500,
        });

        // This should now be Ok
        assert!(validate_transaction(&coldkey_account_id, &call).is_ok());
    });
}

#[test]
fn test_transfer_coldkey_not_in_arbitration() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(61337);
        let recipient_account_id = U256::from(71337);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);

        let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: recipient_account_id,
            value: 1000,
        });

        // This should be Ok
        assert!(validate_transaction(&coldkey_account_id, &call).is_ok());
    });
}

fn validate_transaction(who: &AccountId, call: &RuntimeCall) -> TransactionValidity {
    SubtensorSignedExtension::<Test>::new().validate(who, call, &DispatchInfo::default(), 0)
}

// Helper function to generate valid PoW
fn generate_valid_pow(coldkey: &U256, block_number: u64, difficulty: U256) -> (H256, u64) {
    let mut nonce: u64 = 0;
    loop {
        let work = SubtensorModule::create_seal_hash(block_number, nonce, coldkey);
        if SubtensorModule::hash_meets_difficulty(&work, difficulty) {
            return (work, nonce);
        }
        nonce += 1;
    }
}

// Helper function to advance to the next block and run hooks
fn next_block() {
    let current_block = System::block_number();
    System::on_finalize(current_block);
    System::set_block_number(current_block + 1);
    System::on_initialize(System::block_number());
    SubtensorModule::on_initialize(System::block_number());
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_coldkey_meets_enough --exact --nocapture
#[test]
fn test_coldkey_meets_enough() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(2);
        let netuid = 1u16;
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        let current_block = SubtensorModule::get_current_block_as_u64();
        let (work1, nonce1) = generate_valid_pow(
            &coldkey,
            current_block,
            U256::from(BaseDifficulty::<Test>::get()),
        );
        assert_err!(
            SubtensorModule::do_schedule_coldkey_swap(
                &coldkey.clone(),
                &new_coldkey,
                work1.to_fixed_bytes().to_vec(),
                current_block,
                nonce1
            ),
            Error::<Test>::InsufficientBalanceToPerformColdkeySwap
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );
        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &coldkey.clone(),
            &new_coldkey,
            work1.to_fixed_bytes().to_vec(),
            current_block,
            nonce1
        ));
    });
}

#[test]
fn test_comprehensive_coldkey_swap_scenarios() {
    new_test_ext(1).execute_with(|| {
        // Set arbitration period to 5 blocks
        ArbitrationPeriod::<Test>::put(5);

        let subnet_owner1 = U256::from(1);
        let subnet_owner2 = U256::from(2);
        let regular_user = U256::from(3);
        let new_coldkey1 = U256::from(4);
        let new_coldkey2 = U256::from(5);
        let new_coldkey3 = U256::from(6);
        let netuid1 = 1;
        let netuid2 = 2;

        // Add networks and register subnet owners
        add_network(netuid1, 13, 0);
        add_network(netuid2, 13, 0);
        SubnetOwner::<Test>::insert(netuid1, subnet_owner1);
        SubnetOwner::<Test>::insert(netuid2, subnet_owner2);

        // Add balance to subnet owners and regular user
        SubtensorModule::add_balance_to_coldkey_account(
            &subnet_owner1,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &subnet_owner2,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &regular_user,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP * 2,
        );

        // Set a very low base difficulty for testing
        BaseDifficulty::<Test>::put(1);

        let current_block = SubtensorModule::get_current_block_as_u64();

        // Schedule swaps for subnet owners and regular user
        let (work1, nonce1) = generate_valid_pow(&subnet_owner1, current_block, U256::from(BaseDifficulty::<Test>::get()));
        let (work2, nonce2) = generate_valid_pow(&subnet_owner2, current_block, U256::from(BaseDifficulty::<Test>::get()));
        let (work3, nonce3) = generate_valid_pow(&regular_user, current_block,  U256::from(BaseDifficulty::<Test>::get()));

        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &subnet_owner1,
            &new_coldkey1,
            work1.to_fixed_bytes().to_vec(),
            current_block,
            nonce1
        ));

        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &subnet_owner2,
            &new_coldkey2,
            work2.to_fixed_bytes().to_vec(),
            current_block,
            nonce2
        ));

        assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
            &regular_user,
            &new_coldkey3,
            work3.to_fixed_bytes().to_vec(),
            current_block,
            nonce3
        ));

        // Check if swaps were scheduled correctly
        assert_eq!(
            ColdkeySwapDestinations::<Test>::get(subnet_owner1),
            vec![new_coldkey1]
        );
        assert_eq!(
            ColdkeySwapDestinations::<Test>::get(subnet_owner2),
            vec![new_coldkey2]
        );
        assert_eq!(
            ColdkeySwapDestinations::<Test>::get(regular_user),
            vec![new_coldkey3]
        );

        // Run through the arbitration period plus one block
        for i in 0..6 {
            next_block();
            SubtensorModule::on_idle(System::block_number(), Weight::MAX);

            log::info!(
                "Block {}: Coldkey in arbitration: {}, Swap destinations: {:?}",
                i + 1,
                SubtensorModule::coldkey_in_arbitration(&subnet_owner1),
                ColdkeySwapDestinations::<Test>::get(subnet_owner1)
            );

            // Test edge case: try to schedule another swap during arbitration
            if i == 2 {
                let (work4, nonce4) = generate_valid_pow(
                    &subnet_owner1,
                    current_block + i as u64,
                    U256::from(4) * U256::from(BaseDifficulty::<Test>::get()),
                );
                assert_ok!(SubtensorModule::do_schedule_coldkey_swap(
                    &subnet_owner1,
                    &new_coldkey2,
                    work4.to_fixed_bytes().to_vec(),
                    current_block + i as u64,
                    nonce4
                ));
                // This should add new_coldkey2 to subnet_owner1's destinations
                assert_eq!(
                    ColdkeySwapDestinations::<Test>::get(subnet_owner1),
                    vec![new_coldkey1, new_coldkey2]
                );
            }
        }

        // Check if swaps have been executed
        log::info!(
            "After arbitration period - Swap destinations for subnet_owner1: {:?}",
            ColdkeySwapDestinations::<Test>::get(subnet_owner1)
        );
        assert_eq!(
            ColdkeySwapDestinations::<Test>::get(subnet_owner1),
            vec![new_coldkey1, new_coldkey2],
            "ColdkeySwapDestinations for subnet_owner1 should still contain two destinations after arbitration period"
        );
        assert!(ColdkeySwapDestinations::<Test>::get(subnet_owner2).is_empty());
        assert!(ColdkeySwapDestinations::<Test>::get(regular_user).is_empty());

        // Verify that subnet ownerships have NOT been transferred for subnet_owner1
        assert_eq!(SubnetOwner::<Test>::get(netuid1), subnet_owner1);
        // But subnet_owner2's ownership should have been transferred
        assert_eq!(SubnetOwner::<Test>::get(netuid2), new_coldkey2);

        // Verify regular user's balance has been transferred
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey3),
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP * 2
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&regular_user), 0);
    });
}

#[test]
fn test_get_total_delegated_stake_after_unstaking() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let initial_stake = 2000;
        let unstake_amount = 500;
        let existential_deposit = 1; // Account for the existential deposit

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Make the account a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey
        ));

        // Add balance to delegator
        SubtensorModule::add_balance_to_coldkey_account(&delegator, initial_stake);

        // Delegate stake
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            initial_stake
        ));

        // Check initial delegated stake
        assert_eq!(
            SubtensorModule::get_total_delegated_stake(&delegate_coldkey),
            initial_stake - existential_deposit,
            "Initial delegated stake is incorrect"
        );

        // Unstake part of the delegation
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            unstake_amount
        ));

        // Calculate the expected delegated stake
        let expected_delegated_stake = initial_stake - unstake_amount - existential_deposit;

        // Debug prints
        println!("Initial stake: {}", initial_stake);
        println!("Unstake amount: {}", unstake_amount);
        println!("Existential deposit: {}", existential_deposit);
        println!("Expected delegated stake: {}", expected_delegated_stake);
        println!(
            "Actual delegated stake: {}",
            SubtensorModule::get_total_delegated_stake(&delegate_coldkey)
        );

        // Check the total delegated stake after unstaking
        assert_eq!(
            SubtensorModule::get_total_delegated_stake(&delegate_coldkey),
            expected_delegated_stake,
            "Delegated stake mismatch after unstaking"
        );
    });
}

#[test]
fn test_get_total_delegated_stake_no_delegations() {
    new_test_ext(1).execute_with(|| {
        let delegate = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = 1u16;

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate, coldkey, 0);

        // Make the delegate a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(coldkey),
            delegate
        ));

        // Check that there's no delegated stake
        assert_eq!(SubtensorModule::get_total_delegated_stake(&delegate), 0);
    });
}

#[test]
fn test_get_total_delegated_stake_single_delegator() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let stake_amount = 999;
        let existential_deposit = 1; // Account for the existential deposit

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Make the account a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey
        ));

        // Add stake from delegator
        SubtensorModule::add_balance_to_coldkey_account(&delegator, stake_amount);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            stake_amount
        ));

        // Debug prints
        println!("Delegate coldkey: {:?}", delegate_coldkey);
        println!("Delegate hotkey: {:?}", delegate_hotkey);
        println!("Delegator: {:?}", delegator);
        println!("Stake amount: {}", stake_amount);
        println!("Existential deposit: {}", existential_deposit);
        println!("Total stake for hotkey: {}", SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey));
        println!("Delegated stake for coldkey: {}", SubtensorModule::get_total_delegated_stake(&delegate_coldkey));

        // Calculate expected delegated stake
        let expected_delegated_stake = stake_amount - existential_deposit;
        let actual_delegated_stake = SubtensorModule::get_total_delegated_stake(&delegate_coldkey);

        assert_eq!(
            actual_delegated_stake,
            expected_delegated_stake,
            "Total delegated stake should match the delegator's stake minus existential deposit. Expected: {}, Actual: {}",
            expected_delegated_stake,
            actual_delegated_stake
        );
    });
}

#[test]
fn test_get_total_delegated_stake_multiple_delegators() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator1 = U256::from(3);
        let delegator2 = U256::from(4);
        let stake1 = 1000;
        let stake2 = 1999;
        let existential_deposit = 1; // Account for the existential deposit

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Make the account a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey
        ));

        // Add stake from delegator1
        SubtensorModule::add_balance_to_coldkey_account(&delegator1, stake1);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator1),
            delegate_hotkey,
            stake1
        ));

        // Add stake from delegator2
        SubtensorModule::add_balance_to_coldkey_account(&delegator2, stake2);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator2),
            delegate_hotkey,
            stake2
        ));

        // Debug prints
        println!("Delegator1 stake: {}", stake1);
        println!("Delegator2 stake: {}", stake2);
        println!("Existential deposit: {}", existential_deposit);
        println!("Total stake for hotkey: {}", SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey));
        println!("Delegated stake for coldkey: {}", SubtensorModule::get_total_delegated_stake(&delegate_coldkey));

        // Calculate expected total delegated stake
        let expected_total_delegated = stake1 + stake2 - 2 * existential_deposit;
        let actual_total_delegated = SubtensorModule::get_total_delegated_stake(&delegate_coldkey);

        assert_eq!(
            actual_total_delegated,
            expected_total_delegated,
            "Total delegated stake should match the sum of delegators' stakes minus existential deposits. Expected: {}, Actual: {}",
            expected_total_delegated,
            actual_total_delegated
        );
    });
}

#[test]
fn test_get_total_delegated_stake_exclude_owner_stake() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let owner_stake = 1000;
        let delegator_stake = 999;
        let existential_deposit = 1; // Account for the existential deposit

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Make the account a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey
        ));

        // Add owner stake
        SubtensorModule::add_balance_to_coldkey_account(&delegate_coldkey, owner_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey,
            owner_stake
        ));

        // Add delegator stake
        SubtensorModule::add_balance_to_coldkey_account(&delegator, delegator_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            delegator_stake
        ));

        // Debug prints
        println!("Owner stake: {}", owner_stake);
        println!("Delegator stake: {}", delegator_stake);
        println!("Existential deposit: {}", existential_deposit);
        println!(
            "Total stake for hotkey: {}",
            SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey)
        );
        println!(
            "Delegated stake for coldkey: {}",
            SubtensorModule::get_total_delegated_stake(&delegate_coldkey)
        );

        // Check the total delegated stake (should exclude owner's stake)
        let expected_delegated_stake = delegator_stake - existential_deposit;
        let actual_delegated_stake = SubtensorModule::get_total_delegated_stake(&delegate_coldkey);

        assert_eq!(
            actual_delegated_stake, expected_delegated_stake,
            "Delegated stake should exclude owner's stake. Expected: {}, Actual: {}",
            expected_delegated_stake, actual_delegated_stake
        );
    });
}

#[test]
fn test_do_schedule_coldkey_swap_subnet_owner_skips_min_balance() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let subnet_owner = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let current_block = 0u64;

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, subnet_owner, 0);

        // Make subnet_owner the owner of the subnet
        SubnetOwner::<Test>::insert(netuid, subnet_owner);

        // Ensure subnet_owner has less than minimum balance
        assert!(
            SubtensorModule::get_coldkey_balance(&subnet_owner)
                < MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP
        );

        // Generate valid PoW
        let difficulty = U256::from(4) * U256::from(BaseDifficulty::<Test>::get());
        let (work, nonce) = generate_valid_pow(&subnet_owner, current_block, difficulty);

        // Debug prints
        println!("Subnet owner: {:?}", subnet_owner);
        println!("New coldkey: {:?}", new_coldkey);
        println!("Current block: {}", current_block);
        println!("Difficulty: {:?}", difficulty);
        println!("Work: {:?}", work);
        println!("Nonce: {}", nonce);

        // Verify the PoW
        let seal = SubtensorModule::create_seal_hash(current_block, nonce, &subnet_owner);
        println!("Calculated seal: {:?}", seal);
        println!("Work matches seal: {}", work == seal);
        println!(
            "Seal meets difficulty: {}",
            SubtensorModule::hash_meets_difficulty(&seal, difficulty)
        );

        // Attempt to schedule coldkey swap
        let result = SubtensorModule::do_schedule_coldkey_swap(
            &subnet_owner,
            &new_coldkey,
            work.to_fixed_bytes().to_vec(),
            current_block,
            nonce,
        );

        // Print the result
        println!("Swap result: {:?}", result);

        assert_ok!(result);

        // Verify that the swap was scheduled
        assert_eq!(
            ColdkeySwapDestinations::<Test>::get(subnet_owner),
            vec![new_coldkey]
        );
    });
}

#[test]
fn test_do_schedule_coldkey_swap_delegate_with_500_tao_skips_min_balance() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let new_coldkey = U256::from(3);
        let delegator = U256::from(4);
        let current_block = 0u64;

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Make delegate a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey
        ));

        // Add more than 500 TAO of stake to the delegate's hotkey
        let stake_amount = 501_000_000_000; // 501 TAO in RAO
        SubtensorModule::add_balance_to_coldkey_account(&delegator, stake_amount);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            stake_amount
        ));

        // Debug prints
        println!(
            "Delegator balance: {}",
            SubtensorModule::get_coldkey_balance(&delegator)
        );
        println!(
            "Delegate coldkey balance: {}",
            SubtensorModule::get_coldkey_balance(&delegate_coldkey)
        );
        println!("Stake amount: {}", stake_amount);
        println!(
            "Delegate hotkey total stake: {}",
            SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey)
        );
        println!(
            "Delegate coldkey delegated stake: {}",
            SubtensorModule::get_total_delegated_stake(&delegate_coldkey)
        );

        // Ensure delegate's coldkey has less than minimum balance
        assert!(
            SubtensorModule::get_coldkey_balance(&delegate_coldkey)
                < MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
            "Delegate coldkey balance should be less than minimum required"
        );

        // Ensure the delegate's hotkey has more than 500 TAO delegated
        assert!(
            SubtensorModule::get_total_delegated_stake(&delegate_coldkey) >= 500_000_000_000,
            "Delegate hotkey should have at least 500 TAO delegated"
        );

        // Generate valid PoW
        let (work, nonce) = generate_valid_pow(
            &delegate_coldkey,
            current_block,
            U256::from(4) * U256::from(BaseDifficulty::<Test>::get()),
        );

        // Debug prints
        println!("Work: {:?}", work);
        println!("Nonce: {}", nonce);

        // Attempt to schedule coldkey swap
        let result = SubtensorModule::do_schedule_coldkey_swap(
            &delegate_coldkey,
            &new_coldkey,
            work.to_fixed_bytes().to_vec(),
            current_block,
            nonce,
        );

        // Print the result
        println!("Swap result: {:?}", result);

        assert_ok!(result);

        // Verify that the swap was scheduled
        assert_eq!(
            ColdkeySwapDestinations::<Test>::get(delegate_coldkey),
            vec![new_coldkey]
        );

        // Additional debug prints after swap
        println!(
            "Coldkey swap destinations: {:?}",
            ColdkeySwapDestinations::<Test>::get(delegate_coldkey)
        );
        println!(
            "Is coldkey in arbitration: {}",
            SubtensorModule::coldkey_in_arbitration(&delegate_coldkey)
        );
    });
}

#[test]
fn test_do_schedule_coldkey_swap_regular_user_fails_min_balance() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let regular_user = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let current_block = 0u64;
        let nonce = 0u64;

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, regular_user, 0);

        // Ensure regular_user has less than minimum balance
        assert!(
            SubtensorModule::get_coldkey_balance(&regular_user)
                < MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP
        );

        let (work, _) = generate_valid_pow(
            &regular_user,
            current_block,
            U256::from(4) * U256::from(BaseDifficulty::<Test>::get()),
        );

        // Attempt to schedule coldkey swap
        assert_noop!(
            SubtensorModule::do_schedule_coldkey_swap(
                &regular_user,
                &new_coldkey,
                work.to_fixed_bytes().to_vec(),
                current_block,
                nonce
            ),
            Error::<Test>::InsufficientBalanceToPerformColdkeySwap
        );

        // Verify that the swap was not scheduled
        assert!(ColdkeySwapDestinations::<Test>::get(regular_user).is_empty());
    });
}

#[test]
fn test_do_schedule_coldkey_swap_regular_user_passes_min_balance() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let regular_user = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let current_block = 0u64;

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, regular_user, 0);

        // Ensure regular_user has more than minimum balance
        SubtensorModule::add_balance_to_coldkey_account(
            &regular_user,
            MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP + 1,
        );
        assert!(
            SubtensorModule::get_coldkey_balance(&regular_user)
                > MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP
        );

        // Generate valid PoW
        let (work, nonce) = generate_valid_pow(
            &regular_user,
            current_block,
            U256::from(4) * U256::from(BaseDifficulty::<Test>::get()),
        );

        // Debug prints
        println!("Regular user: {:?}", regular_user);
        println!("New coldkey: {:?}", new_coldkey);
        println!("Current block: {}", current_block);
        println!("Work: {:?}", work);
        println!("Nonce: {}", nonce);

        // Attempt to schedule coldkey swap
        let result = SubtensorModule::do_schedule_coldkey_swap(
            &regular_user,
            &new_coldkey,
            work.to_fixed_bytes().to_vec(),
            current_block,
            nonce,
        );

        // Print the result
        println!("Swap result: {:?}", result);

        assert_ok!(result);

        // Verify that the swap was scheduled
        assert_eq!(
            ColdkeySwapDestinations::<Test>::get(regular_user),
            vec![new_coldkey]
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_emission_creates_staking_hotkeys_entry --exact --nocapture
#[test]
fn test_emission_creates_staking_hotkeys_entry() {
    new_test_ext(1).execute_with(|| {
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);

        let coldkey = U256::from(3);

        // Add to Owner map
        Owner::<Test>::insert(hotkey0, coldkey);
        Owner::<Test>::insert(hotkey1, coldkey);
        OwnedHotkeys::<Test>::insert(coldkey, vec![hotkey0, hotkey1]);

        // Emit through hotkey
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 1_000);

        // Verify StakingHotkeys has an entry
        assert_eq!(StakingHotkeys::<Test>::get(coldkey).len(), 1);
        assert!(StakingHotkeys::<Test>::get(coldkey).contains(&hotkey0));

        // Try again with another emission on hotkey1
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 2_000);

        // Verify both hotkeys are now in the map
        assert_eq!(StakingHotkeys::<Test>::get(coldkey).len(), 2);
        let final_map = StakingHotkeys::<Test>::get(coldkey);
        assert!(final_map.contains(&hotkey0));
        assert!(final_map.contains(&hotkey1));
    })
}
