#![allow(clippy::indexing_slicing)]

mod mock;
use frame_support::{
    assert_err, assert_ok,
    dispatch::{DispatchClass, DispatchInfo, DispatchResult, GetDispatchInfo, Pays},
    pallet_prelude::{InvalidTransaction, TransactionValidityError},
};
use mock::*;
use pallet_subtensor::{Error, Owner};
use sp_core::{H256, U256};
use sp_runtime::{
    traits::{BlakeTwo256, DispatchInfoOf, Hash, SignedExtension},
    DispatchError,
};
use substrate_fixed::types::I32F32;

/***************************
  pub fn set_weights() tests
*****************************/

// Test the call passes through the subtensor module.
#[test]
#[cfg(not(tarpaulin))]
fn test_set_weights_dispatch_info_ok() {
    new_test_ext(0).execute_with(|| {
        let dests = vec![1, 1];
        let weights = vec![1, 1];
        let netuid: u16 = 1;
        let version_key: u64 = 0;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::set_weights {
            netuid,
            dests,
            weights,
            version_key,
        });
        let dispatch_info = call.get_dispatch_info();

        assert_eq!(dispatch_info.class, DispatchClass::Normal);
        assert_eq!(dispatch_info.pays_fee, Pays::No);
    });
}
#[test]
#[cfg(not(tarpaulin))]
fn test_set_rootweights_dispatch_info_ok() {
    new_test_ext(0).execute_with(|| {
        let dests = vec![1, 1];
        let weights = vec![1, 1];
        let netuid: u16 = 1;
        let version_key: u64 = 0;
        let hotkey: U256 = U256::from(1); // Add the hotkey field
        let call = RuntimeCall::SubtensorModule(SubtensorCall::set_root_weights {
            netuid,
            dests,
            weights,
            version_key,
            hotkey, // Include the hotkey field
        });
        let dispatch_info = call.get_dispatch_info();

        assert_eq!(dispatch_info.class, DispatchClass::Normal);
        assert_eq!(dispatch_info.pays_fee, Pays::No);
    });
}

#[test]
fn test_set_rootweights_validate() {
    // Testing the signed extension validate function
    // correctly filters this transaction.

    new_test_ext(0).execute_with(|| {
        let dests = vec![1, 1];
        let weights = vec![1, 1];
        let netuid: u16 = 1;
        let version_key: u64 = 0;
        let coldkey = U256::from(0);
        let hotkey: U256 = U256::from(1); // Add the hotkey field
        assert_ne!(hotkey, coldkey); // Ensure hotkey is NOT the same as coldkey !!!

        let who = coldkey; // The coldkey signs this transaction

        let call = RuntimeCall::SubtensorModule(SubtensorCall::set_root_weights {
            netuid,
            dests,
            weights,
            version_key,
            hotkey, // Include the hotkey field
        });

        // Create netuid
        add_network(netuid, 0, 0);
        // Register the hotkey
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        Owner::<Test>::insert(hotkey, coldkey);

        let min_stake = 500_000_000_000;
        // Set the minimum stake
        SubtensorModule::set_weights_min_stake(min_stake);

        // Verify stake is less than minimum
        assert!(SubtensorModule::get_total_stake_for_hotkey(&hotkey) < min_stake);
        let info: DispatchInfo =
            DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();

        let extension = pallet_subtensor::SubtensorSignedExtension::<Test>::new();
        // Submit to the signed extension validate function
        let result_no_stake = extension.validate(&who, &call.clone(), &info, 10);
        // Should fail
        assert_err!(
            // Should get an invalid transaction error
            result_no_stake,
            TransactionValidityError::Invalid(InvalidTransaction::Call,)
        );

        // Increase the stake to be equal to the minimum
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey, min_stake);

        // Verify stake is equal to minimum
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            min_stake
        );

        // Submit to the signed extension validate function
        let result_min_stake = extension.validate(&who, &call.clone(), &info, 10);
        // Now the call should pass
        assert_ok!(result_min_stake);

        // Try with more stake than minimum
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey, 1);

        // Verify stake is more than minimum
        assert!(SubtensorModule::get_total_stake_for_hotkey(&hotkey) > min_stake);

        let result_more_stake = extension.validate(&who, &call.clone(), &info, 10);
        // The call should still pass
        assert_ok!(result_more_stake);
    });
}

#[test]
fn test_commit_weights_dispatch_info_ok() {
    new_test_ext(0).execute_with(|| {
        let dests = vec![1, 1];
        let weights = vec![1, 1];
        let netuid: u16 = 1;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;
        let hotkey: U256 = U256::from(1);

        let commit_hash: H256 =
            BlakeTwo256::hash_of(&(hotkey, netuid, dests, weights, salt, version_key));

        let call = RuntimeCall::SubtensorModule(SubtensorCall::commit_weights {
            netuid,
            commit_hash,
        });
        let dispatch_info = call.get_dispatch_info();

        assert_eq!(dispatch_info.class, DispatchClass::Normal);
        assert_eq!(dispatch_info.pays_fee, Pays::No);
    });
}

#[test]
fn test_commit_weights_validate() {
    // Testing the signed extension validate function
    // correctly filters this transaction.

    new_test_ext(0).execute_with(|| {
        let dests = vec![1, 1];
        let weights = vec![1, 1];
        let netuid: u16 = 1;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;
        let coldkey = U256::from(0);
        let hotkey: U256 = U256::from(1); // Add the hotkey field
        assert_ne!(hotkey, coldkey); // Ensure hotkey is NOT the same as coldkey !!!

        let who = hotkey; // The hotkey signs this transaction

        let commit_hash: H256 =
            BlakeTwo256::hash_of(&(hotkey, netuid, dests, weights, salt, version_key));

        let call = RuntimeCall::SubtensorModule(SubtensorCall::commit_weights {
            netuid,
            commit_hash,
        });

        // Create netuid
        add_network(netuid, 0, 0);
        // Register the hotkey
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        Owner::<Test>::insert(hotkey, coldkey);

        let min_stake = 500_000_000_000;
        // Set the minimum stake
        SubtensorModule::set_weights_min_stake(min_stake);

        // Verify stake is less than minimum
        assert!(SubtensorModule::get_total_stake_for_hotkey(&hotkey) < min_stake);
        let info: DispatchInfo =
            DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();

        let extension = pallet_subtensor::SubtensorSignedExtension::<Test>::new();
        // Submit to the signed extension validate function
        let result_no_stake = extension.validate(&who, &call.clone(), &info, 10);
        // Should fail
        assert_err!(
            // Should get an invalid transaction error
            result_no_stake,
            TransactionValidityError::Invalid(InvalidTransaction::Call,)
        );

        // Increase the stake to be equal to the minimum
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey, min_stake);

        // Verify stake is equal to minimum
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            min_stake
        );

        // Submit to the signed extension validate function
        let result_min_stake = extension.validate(&who, &call.clone(), &info, 10);
        // Now the call should pass
        assert_ok!(result_min_stake);

        // Try with more stake than minimum
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey, 1);

        // Verify stake is more than minimum
        assert!(SubtensorModule::get_total_stake_for_hotkey(&hotkey) > min_stake);

        let result_more_stake = extension.validate(&who, &call.clone(), &info, 10);
        // The call should still pass
        assert_ok!(result_more_stake);
    });
}

#[test]
fn test_reveal_weights_dispatch_info_ok() {
    new_test_ext(0).execute_with(|| {
        let dests = vec![1, 1];
        let weights = vec![1, 1];
        let netuid: u16 = 1;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;

        let call = RuntimeCall::SubtensorModule(SubtensorCall::reveal_weights {
            netuid,
            uids: dests,
            values: weights,
            salt,
            version_key,
        });
        let dispatch_info = call.get_dispatch_info();

        assert_eq!(dispatch_info.class, DispatchClass::Normal);
        assert_eq!(dispatch_info.pays_fee, Pays::No);
    });
}

#[test]
fn test_reveal_weights_validate() {
    // Testing the signed extension validate function
    // correctly filters this transaction.

    new_test_ext(0).execute_with(|| {
        let dests = vec![1, 1];
        let weights = vec![1, 1];
        let netuid: u16 = 1;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;
        let coldkey = U256::from(0);
        let hotkey: U256 = U256::from(1); // Add the hotkey field
        assert_ne!(hotkey, coldkey); // Ensure hotkey is NOT the same as coldkey !!!

        let who = hotkey; // The hotkey signs this transaction

        let call = RuntimeCall::SubtensorModule(SubtensorCall::reveal_weights {
            netuid,
            uids: dests,
            values: weights,
            salt,
            version_key,
        });

        // Create netuid
        add_network(netuid, 0, 0);
        // Register the hotkey
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        Owner::<Test>::insert(hotkey, coldkey);

        let min_stake = 500_000_000_000;
        // Set the minimum stake
        SubtensorModule::set_weights_min_stake(min_stake);

        // Verify stake is less than minimum
        assert!(SubtensorModule::get_total_stake_for_hotkey(&hotkey) < min_stake);
        let info: DispatchInfo =
            DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();

        let extension = pallet_subtensor::SubtensorSignedExtension::<Test>::new();
        // Submit to the signed extension validate function
        let result_no_stake = extension.validate(&who, &call.clone(), &info, 10);
        // Should fail
        assert_err!(
            // Should get an invalid transaction error
            result_no_stake,
            TransactionValidityError::Invalid(InvalidTransaction::Call,)
        );

        // Increase the stake to be equal to the minimum
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey, min_stake);

        // Verify stake is equal to minimum
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            min_stake
        );

        // Submit to the signed extension validate function
        let result_min_stake = extension.validate(&who, &call.clone(), &info, 10);
        // Now the call should pass
        assert_ok!(result_min_stake);

        // Try with more stake than minimum
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey, 1);

        // Verify stake is more than minimum
        assert!(SubtensorModule::get_total_stake_for_hotkey(&hotkey) > min_stake);

        let result_more_stake = extension.validate(&who, &call.clone(), &info, 10);
        // The call should still pass
        assert_ok!(result_more_stake);
    });
}

#[test]
fn test_set_weights_is_root_error() {
    new_test_ext(0).execute_with(|| {
        let root_netuid: u16 = 0;

        let uids = vec![0];
        let weights = vec![1];
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;
        let hotkey = U256::from(1);

        assert_err!(
            commit_reveal_set_weights(hotkey, root_netuid, uids, weights, salt, version_key),
            Error::<Test>::CanNotSetRootNetworkWeights
        );
    });
}

// Test ensures that uid has validator permit to set non-self weights.
#[test]
fn test_weights_err_no_validator_permit() {
    new_test_ext(0).execute_with(|| {
        let hotkey_account_id = U256::from(55);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_min_allowed_weights(netuid, 0);
        SubtensorModule::set_max_allowed_uids(netuid, 3);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);
        register_ok_neuron(netuid, hotkey_account_id, U256::from(66), 0);
        register_ok_neuron(netuid, U256::from(1), U256::from(1), 65555);
        register_ok_neuron(netuid, U256::from(2), U256::from(2), 75555);

        let weights_keys: Vec<u16> = vec![1, 2];
        let weight_values: Vec<u16> = vec![1, 2];
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let result = commit_reveal_set_weights(
            hotkey_account_id,
            netuid,
            weights_keys,
            weight_values,
            salt.clone(),
            0,
        );
        assert_eq!(result, Err(Error::<Test>::NeuronNoValidatorPermit.into()));

        let weights_keys: Vec<u16> = vec![1, 2];
        let weight_values: Vec<u16> = vec![1, 2];
        let neuron_uid: u16 =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id)
                .expect("Not registered.");
        SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
        let result = commit_reveal_set_weights(
            hotkey_account_id,
            netuid,
            weights_keys,
            weight_values,
            salt,
            0,
        );
        assert_ok!(result);
    });
}

// To execute this test: cargo test --package pallet-subtensor --test weights test_set_weights_min_stake_failed -- --nocapture`
#[test]
#[cfg(not(tarpaulin))]
fn test_set_weights_min_stake_failed() {
    new_test_ext(0).execute_with(|| {
        let dests = vec![0];
        let weights = vec![1];
        let netuid: u16 = 1;
        let version_key: u64 = 0;
        let hotkey = U256::from(0);
        let coldkey = U256::from(0);
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 2143124);
        SubtensorModule::set_weights_min_stake(20_000_000_000_000);

        // Check the signed extension function.
        assert_eq!(SubtensorModule::get_weights_min_stake(), 20_000_000_000_000);
        assert!(!SubtensorModule::check_weights_min_stake(&hotkey));
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey, 19_000_000_000_000);
        assert!(!SubtensorModule::check_weights_min_stake(&hotkey));
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey, 20_000_000_000_000);
        assert!(SubtensorModule::check_weights_min_stake(&hotkey));

        // Check that it fails at the pallet level.
        SubtensorModule::set_weights_min_stake(100_000_000_000_000);
        assert_eq!(
            commit_reveal_set_weights(
                hotkey,
                netuid,
                dests.clone(),
                weights.clone(),
                salt.clone(),
                version_key
            ),
            Err(Error::<Test>::NotEnoughStakeToSetWeights.into())
        );
        // Now passes
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey, 100_000_000_000_000);
        assert_ok!(commit_reveal_set_weights(
            hotkey,
            netuid,
            dests.clone(),
            weights.clone(),
            salt.clone(),
            version_key
        ));
    });
}

// Test ensures that a uid can only set weights if it has the valid weights set version key.
#[test]
fn test_weights_version_key() {
    new_test_ext(0).execute_with(|| {
        let hotkey = U256::from(55);
        let coldkey = U256::from(66);
        let netuid0: u16 = 1;
        let netuid1: u16 = 2;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        add_network(netuid0, 0, 0);
        add_network(netuid1, 0, 0);
        register_ok_neuron(netuid0, hotkey, coldkey, 2143124);
        register_ok_neuron(netuid1, hotkey, coldkey, 3124124);

        let weights_keys: Vec<u16> = vec![0];
        let weight_values: Vec<u16> = vec![1];
        assert_ok!(commit_reveal_set_weights(
            hotkey,
            netuid0,
            weights_keys.clone(),
            weight_values.clone(),
            salt.clone(),
            0
        ));
        assert_ok!(commit_reveal_set_weights(
            hotkey,
            netuid1,
            weights_keys.clone(),
            weight_values.clone(),
            salt.clone(),
            0
        ));

        // Set version keys.
        let key0: u64 = 12312;
        let key1: u64 = 20313;
        SubtensorModule::set_weights_version_key(netuid0, key0);
        SubtensorModule::set_weights_version_key(netuid1, key1);

        // Setting works with version key.
        assert_ok!(commit_reveal_set_weights(
            hotkey,
            netuid0,
            weights_keys.clone(),
            weight_values.clone(),
            salt.clone(),
            key0
        ));
        assert_ok!(commit_reveal_set_weights(
            hotkey,
            netuid1,
            weights_keys.clone(),
            weight_values.clone(),
            salt.clone(),
            key1
        ));

        // validator:20313 >= network:12312 (accepted: validator newer)
        assert_ok!(commit_reveal_set_weights(
            hotkey,
            netuid0,
            weights_keys.clone(),
            weight_values.clone(),
            salt.clone(),
            key1
        ));

        // Setting fails with incorrect keys.
        // validator:12312 < network:20313 (rejected: validator not updated)
        assert_eq!(
            commit_reveal_set_weights(
                hotkey,
                netuid1,
                weights_keys.clone(),
                weight_values.clone(),
                salt.clone(),
                key0
            ),
            Err(Error::<Test>::IncorrectWeightVersionKey.into())
        );
    });
}

// Test ensures that uid has validator permit to set non-self weights.
#[test]
fn test_weights_err_setting_weights_too_fast() {
    new_test_ext(0).execute_with(|| {
        let hotkey_account_id = U256::from(55);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_min_allowed_weights(netuid, 0);
        SubtensorModule::set_max_allowed_uids(netuid, 3);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);
        register_ok_neuron(netuid, hotkey_account_id, U256::from(66), 0);
        register_ok_neuron(netuid, U256::from(1), U256::from(1), 65555);
        register_ok_neuron(netuid, U256::from(2), U256::from(2), 75555);

        let neuron_uid: u16 =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id)
                .expect("Not registered.");
        SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
        SubtensorModule::set_weights_set_rate_limit(netuid, 10);
        assert_eq!(SubtensorModule::get_weights_set_rate_limit(netuid), 10);

        let weights_keys: Vec<u16> = vec![1, 2];
        let weight_values: Vec<u16> = vec![1, 2];

        // Note that LastUpdate has default 0 for new uids, but if they have actually set weights on block 0
        // then they are allowed to set weights again once more without a wait restriction, to accommodate the default.
        let result = SubtensorModule::set_weights(
            RuntimeOrigin::signed(hotkey_account_id),
            netuid,
            weights_keys.clone(),
            weight_values.clone(),
            0,
        );
        assert_ok!(result);
        run_to_block(1);

        for i in 1..100 {
            let result = SubtensorModule::set_weights(
                RuntimeOrigin::signed(hotkey_account_id),
                netuid,
                weights_keys.clone(),
                weight_values.clone(),
                0,
            );
            if i % 10 == 1 {
                assert_ok!(result);
            } else {
                assert_eq!(result, Err(Error::<Test>::SettingWeightsTooFast.into()));
            }
            run_to_block(i + 1);
        }
    });
}

// Test ensures that uids -- weights must have the same size.
#[test]
fn test_weights_err_weights_vec_not_equal_size() {
    new_test_ext(0).execute_with(|| {
        let hotkey_account_id = U256::from(55);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        add_network(netuid, tempo, 0);
        register_ok_neuron(1, hotkey_account_id, U256::from(66), 0);
        let neuron_uid: u16 =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id)
                .expect("Not registered.");
        SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
        let weights_keys: Vec<u16> = vec![1, 2, 3, 4, 5, 6];
        let weight_values: Vec<u16> = vec![1, 2, 3, 4, 5]; // Uneven sizes
        let result = commit_reveal_set_weights(
            hotkey_account_id,
            1,
            weights_keys.clone(),
            weight_values.clone(),
            salt.clone(),
            0,
        );
        assert_eq!(result, Err(Error::<Test>::WeightVecNotEqualSize.into()));
    });
}

// Test ensures that uids can have not duplicates
#[test]
fn test_weights_err_has_duplicate_ids() {
    new_test_ext(0).execute_with(|| {
        let hotkey_account_id = U256::from(666);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        add_network(netuid, tempo, 0);

        SubtensorModule::set_max_allowed_uids(netuid, 100); // Allow many registrations per block.
        SubtensorModule::set_max_registrations_per_block(netuid, 100); // Allow many registrations per block.
        SubtensorModule::set_target_registrations_per_interval(netuid, 100); // Allow many registrations per block.
                                                                             // uid 0
        register_ok_neuron(netuid, hotkey_account_id, U256::from(77), 0);
        let neuron_uid: u16 =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id)
                .expect("Not registered.");
        SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);

        // uid 1
        register_ok_neuron(netuid, U256::from(1), U256::from(1), 100_000);
        SubtensorModule::get_uid_for_net_and_hotkey(netuid, &U256::from(1))
            .expect("Not registered.");

        // uid 2
        register_ok_neuron(netuid, U256::from(2), U256::from(1), 200_000);
        SubtensorModule::get_uid_for_net_and_hotkey(netuid, &U256::from(2))
            .expect("Not registered.");

        // uid 3
        register_ok_neuron(netuid, U256::from(3), U256::from(1), 300_000);
        SubtensorModule::get_uid_for_net_and_hotkey(netuid, &U256::from(3))
            .expect("Not registered.");

        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 4);

        let weights_keys: Vec<u16> = vec![1, 1, 1]; // Contains duplicates
        let weight_values: Vec<u16> = vec![1, 2, 3];
        let result = commit_reveal_set_weights(
            hotkey_account_id,
            netuid,
            weights_keys.clone(),
            weight_values.clone(),
            salt.clone(),
            0,
        );
        assert_eq!(result, Err(Error::<Test>::DuplicateUids.into()));
    });
}

// Test ensures weights cannot exceed max weight limit.
#[test]
fn test_weights_err_max_weight_limit() {
    //TO DO SAM: uncomment when we implement run_to_block fn
    new_test_ext(0).execute_with(|| {
        // Add network.
        let netuid: u16 = 1;
        let tempo: u16 = 100;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        add_network(netuid, tempo, 0);

        // Set params.
        SubtensorModule::set_max_allowed_uids(netuid, 5);
        SubtensorModule::set_target_registrations_per_interval(netuid, 5);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX / 5);
        SubtensorModule::set_min_allowed_weights(netuid, 0);

        // Add 5 accounts.
        println!("+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 0, 0);
        register_ok_neuron(netuid, U256::from(0), U256::from(0), 55555);
        let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &U256::from(0))
            .expect("Not registered.");
        SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        assert!(SubtensorModule::is_hotkey_registered_on_network(
            netuid,
            &U256::from(0)
        ));
        step_block(1);

        println!("+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 1, 1);
        register_ok_neuron(netuid, U256::from(1), U256::from(1), 65555);
        assert!(SubtensorModule::is_hotkey_registered_on_network(
            netuid,
            &U256::from(1)
        ));
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 2);
        step_block(1);

        println!("+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 2, 2);
        register_ok_neuron(netuid, U256::from(2), U256::from(2), 75555);
        assert!(SubtensorModule::is_hotkey_registered_on_network(
            netuid,
            &U256::from(2)
        ));
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 3);
        step_block(1);

        println!("+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 3, 3);
        register_ok_neuron(netuid, U256::from(3), U256::from(3), 95555);
        assert!(SubtensorModule::is_hotkey_registered_on_network(
            netuid,
            &U256::from(3)
        ));
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 4);
        step_block(1);

        println!("+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 4, 4);
        register_ok_neuron(netuid, U256::from(4), U256::from(4), 35555);
        assert!(SubtensorModule::is_hotkey_registered_on_network(
            netuid,
            &U256::from(4)
        ));
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 5);
        step_block(1);

        // Non self-weight fails.
        let uids: Vec<u16> = vec![1, 2, 3, 4];
        let values: Vec<u16> = vec![u16::MAX / 4, u16::MAX / 4, u16::MAX / 54, u16::MAX / 4];
        let result = commit_reveal_set_weights(U256::from(0), 1, uids, values, salt.clone(), 0);
        assert_eq!(result, Err(Error::<Test>::MaxWeightExceeded.into()));

        // Self-weight is a success.
        let uids: Vec<u16> = vec![0]; // Self.
        let values: Vec<u16> = vec![u16::MAX]; // normalizes to u32::MAX
        assert_ok!(commit_reveal_set_weights(
            U256::from(0),
            1,
            uids,
            values,
            salt.clone(),
            0
        ));
    });
}

// Tests the call requires a valid origin.
#[test]
fn test_no_signature() {
    new_test_ext(0).execute_with(|| {
        let uids: Vec<u16> = vec![];
        let values: Vec<u16> = vec![];
        let result = SubtensorModule::set_weights(RuntimeOrigin::none(), 1, uids, values, 0);
        assert_eq!(result, Err(DispatchError::BadOrigin));
    });
}

// Tests that weights cannot be set BY non-registered hotkeys.
#[test]
fn test_set_weights_err_not_active() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        add_network(netuid, tempo, 0);

        // Register one neuron. Should have uid 0
        register_ok_neuron(1, U256::from(666), U256::from(2), 100000);
        SubtensorModule::get_uid_for_net_and_hotkey(netuid, &U256::from(666))
            .expect("Not registered.");

        let weights_keys: Vec<u16> = vec![0]; // Uid 0 is valid.
        let weight_values: Vec<u16> = vec![1];
        // This hotkey is NOT registered.
        let result =
            commit_reveal_set_weights(U256::from(1), 1, weights_keys, weight_values, salt, 0);
        assert_eq!(
            result,
            Err(Error::<Test>::HotKeyNotRegisteredInSubNet.into())
        );
    });
}

// Tests that set weights fails if you pass invalid uids.
#[test]
fn test_set_weights_err_invalid_uid() {
    new_test_ext(0).execute_with(|| {
        let hotkey_account_id = U256::from(55);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        add_network(netuid, tempo, 0);
        register_ok_neuron(1, hotkey_account_id, U256::from(66), 0);
        let neuron_uid: u16 =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id)
                .expect("Not registered.");
        SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
        let weight_keys: Vec<u16> = vec![9999]; // Does not exist
        let weight_values: Vec<u16> = vec![88]; // random value
        let result =
            commit_reveal_set_weights(hotkey_account_id, 1, weight_keys, weight_values, salt, 0);
        assert_eq!(result, Err(Error::<Test>::UidVecContainInvalidOne.into()));
    });
}

// Tests that set weights fails if you don't pass enough values.
#[test]
fn test_set_weight_not_enough_values() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let account_id = U256::from(1);
        add_network(netuid, tempo, 0);

        register_ok_neuron(1, account_id, U256::from(2), 100000);
        let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &U256::from(1))
            .expect("Not registered.");
        SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);

        register_ok_neuron(1, U256::from(3), U256::from(4), 300000);
        SubtensorModule::set_min_allowed_weights(netuid, 2);

        // Should fail because we are only setting a single value and its not the self weight.
        let weight_keys: Vec<u16> = vec![1]; // not weight.
        let weight_values: Vec<u16> = vec![88]; // random value.
        let result =
            commit_reveal_set_weights(account_id, 1, weight_keys, weight_values, salt.clone(), 0);
        assert_eq!(result, Err(Error::<Test>::WeightVecLengthIsLow.into()));

        // Shouldnt fail because we setting a single value but it is the self weight.
        let weight_keys: Vec<u16> = vec![0]; // self weight.
        let weight_values: Vec<u16> = vec![88]; // random value.
        assert_ok!(commit_reveal_set_weights(
            account_id,
            1,
            weight_keys,
            weight_values,
            salt.clone(),
            0
        ));

        // Should pass because we are setting enough values.
        let weight_keys: Vec<u16> = vec![0, 1]; // self weight.
        let weight_values: Vec<u16> = vec![10, 10]; // random value.
        SubtensorModule::set_min_allowed_weights(netuid, 1);
        assert_ok!(commit_reveal_set_weights(
            account_id,
            1,
            weight_keys,
            weight_values,
            salt,
            0
        ));
    });
}

// Tests that the weights set fails if you pass too many uids for the subnet
#[test]
fn test_set_weight_too_many_uids() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        add_network(netuid, tempo, 0);

        register_ok_neuron(1, U256::from(1), U256::from(2), 100_000);
        let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &U256::from(1))
            .expect("Not registered.");
        SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);

        register_ok_neuron(1, U256::from(3), U256::from(4), 300_000);
        SubtensorModule::set_min_allowed_weights(1, 2);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);

        // Should fail because we are setting more weights than there are neurons.
        let weight_keys: Vec<u16> = vec![0, 1, 2, 3, 4]; // more uids than neurons in subnet.
        let weight_values: Vec<u16> = vec![88, 102, 303, 1212, 11]; // random value.
        let result = commit_reveal_set_weights(
            U256::from(1),
            1,
            weight_keys,
            weight_values,
            salt.clone(),
            0,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::UidsLengthExceedUidsInSubNet.into())
        );

        // Shouldnt fail because we are setting less weights than there are neurons.
        let weight_keys: Vec<u16> = vec![0, 1]; // Only on neurons that exist.
        let weight_values: Vec<u16> = vec![10, 10]; // random value.
        assert_ok!(commit_reveal_set_weights(
            U256::from(1),
            1,
            weight_keys,
            weight_values,
            salt,
            0
        ));
    });
}

// Tests that the weights set doesn't panic if you pass weights that sum to larger than u16 max.
#[test]
fn test_set_weights_sum_larger_than_u16_max() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        add_network(netuid, tempo, 0);

        register_ok_neuron(1, U256::from(1), U256::from(2), 100_000);
        let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &U256::from(1))
            .expect("Not registered.");
        SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);

        register_ok_neuron(1, U256::from(3), U256::from(4), 300_000);
        SubtensorModule::set_min_allowed_weights(1, 2);

        // Shouldn't fail because we are setting the right number of weights.
        let weight_keys: Vec<u16> = vec![0, 1];
        let weight_values: Vec<u16> = vec![u16::MAX, u16::MAX];
        // sum of weights is larger than u16 max.
        assert!(weight_values.iter().map(|x| *x as u64).sum::<u64>() > (u16::MAX as u64));

        let result =
            commit_reveal_set_weights(U256::from(1), 1, weight_keys, weight_values, salt, 0);
        assert_ok!(result);

        // Get max-upscaled unnormalized weights.
        let all_weights: Vec<Vec<I32F32>> = SubtensorModule::get_weights(netuid);
        let weights_set: &[I32F32] = &all_weights[neuron_uid as usize];
        assert_eq!(weights_set[0], I32F32::from_num(u16::MAX));
        assert_eq!(weights_set[1], I32F32::from_num(u16::MAX));
    });
}

/// Check _truthy_ path for self weight
#[test]
fn test_check_length_allows_singleton() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;

        let max_allowed: u16 = 1;
        let min_allowed_weights = max_allowed;

        SubtensorModule::set_min_allowed_weights(netuid, min_allowed_weights);

        let uids: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));
        let uid: u16 = uids[0];
        let weights: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));

        let expected = true;
        let result = SubtensorModule::check_length(netuid, uid, &uids, &weights);

        assert_eq!(expected, result, "Failed get expected result");
    });
}

/// Check _truthy_ path for weights within allowed range
#[test]
fn test_check_length_weights_length_exceeds_min_allowed() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;

        let max_allowed: u16 = 3;
        let min_allowed_weights = max_allowed;

        SubtensorModule::set_min_allowed_weights(netuid, min_allowed_weights);

        let uids: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));
        let uid: u16 = uids[0];
        let weights: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));

        let expected = true;
        let result = SubtensorModule::check_length(netuid, uid, &uids, &weights);

        assert_eq!(expected, result, "Failed get expected result");
    });
}

/// Check _falsey_ path for weights outside allowed range
#[test]
fn test_check_length_to_few_weights() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;

        let min_allowed_weights = 3;

        add_network(netuid, 1, 0);
        SubtensorModule::set_target_registrations_per_interval(netuid, 100);
        SubtensorModule::set_max_registrations_per_block(netuid, 100);
        // register morw than min allowed
        register_ok_neuron(1, U256::from(1), U256::from(1), 300_000);
        register_ok_neuron(1, U256::from(2), U256::from(2), 300_001);
        register_ok_neuron(1, U256::from(3), U256::from(3), 300_002);
        register_ok_neuron(1, U256::from(4), U256::from(4), 300_003);
        register_ok_neuron(1, U256::from(5), U256::from(5), 300_004);
        register_ok_neuron(1, U256::from(6), U256::from(6), 300_005);
        register_ok_neuron(1, U256::from(7), U256::from(7), 300_006);
        SubtensorModule::set_min_allowed_weights(netuid, min_allowed_weights);

        let uids: Vec<u16> = Vec::from_iter((0..2).map(|id| id + 1));
        let weights: Vec<u16> = Vec::from_iter((0..2).map(|id| id + 1));
        let uid: u16 = uids[0];

        let expected = false;
        let result = SubtensorModule::check_length(netuid, uid, &uids, &weights);

        assert_eq!(expected, result, "Failed get expected result");
    });
}

/// Check do nothing path
#[test]
fn test_normalize_weights_does_not_mutate_when_sum_is_zero() {
    new_test_ext(0).execute_with(|| {
        let max_allowed: u16 = 3;

        let weights: Vec<u16> = Vec::from_iter((0..max_allowed).map(|_| 0));

        let expected = weights.clone();
        let result = SubtensorModule::normalize_weights(weights);

        assert_eq!(
            expected, result,
            "Failed get expected result when everything _should_ be fine"
        );
    });
}

/// Check do something path
#[test]
fn test_normalize_weights_does_not_mutate_when_sum_not_zero() {
    new_test_ext(0).execute_with(|| {
        let max_allowed: u16 = 3;

        let weights: Vec<u16> = Vec::from_iter(0..max_allowed);

        let expected = weights.clone();
        let result = SubtensorModule::normalize_weights(weights);

        assert_eq!(expected.len(), result.len(), "Length of weights changed?!");
    });
}

/// Check _truthy_ path for weights length
#[test]
fn test_max_weight_limited_allow_self_weights_to_exceed_max_weight_limit() {
    new_test_ext(0).execute_with(|| {
        let max_allowed: u16 = 1;

        let netuid: u16 = 1;
        let uids: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));
        let uid: u16 = uids[0];
        let weights: Vec<u16> = vec![0];

        let expected = true;
        let result = SubtensorModule::max_weight_limited(netuid, uid, &uids, &weights);

        assert_eq!(
            expected, result,
            "Failed get expected result when everything _should_ be fine"
        );
    });
}

/// Check _truthy_ path for max weight limit
#[test]
fn test_max_weight_limited_when_weight_limit_is_u16_max() {
    new_test_ext(0).execute_with(|| {
        let max_allowed: u16 = 3;

        let netuid: u16 = 1;
        let uids: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));
        let uid: u16 = uids[0];
        let weights: Vec<u16> = Vec::from_iter((0..max_allowed).map(|_id| u16::MAX));

        let expected = true;
        let result = SubtensorModule::max_weight_limited(netuid, uid, &uids, &weights);

        assert_eq!(
            expected, result,
            "Failed get expected result when everything _should_ be fine"
        );
    });
}

/// Check _truthy_ path for max weight limit
#[test]
fn test_max_weight_limited_when_max_weight_is_within_limit() {
    new_test_ext(0).execute_with(|| {
        let max_allowed: u16 = 1;
        let max_weight_limit = u16::MAX / 5;

        let netuid: u16 = 1;
        let uids: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));
        let uid: u16 = uids[0];
        let weights: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| max_weight_limit - id));

        SubtensorModule::set_max_weight_limit(netuid, max_weight_limit);

        let expected = true;
        let result = SubtensorModule::max_weight_limited(netuid, uid, &uids, &weights);

        assert_eq!(
            expected, result,
            "Failed get expected result when everything _should_ be fine"
        );
    });
}

/// Check _falsey_ path
#[test]
fn test_max_weight_limited_when_guard_checks_are_not_triggered() {
    new_test_ext(0).execute_with(|| {
        let max_allowed: u16 = 3;
        let max_weight_limit = u16::MAX / 5;

        let netuid: u16 = 1;
        let uids: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));
        let uid: u16 = uids[0];
        let weights: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| max_weight_limit + id));

        SubtensorModule::set_max_weight_limit(netuid, max_weight_limit);

        let expected = false;
        let result = SubtensorModule::max_weight_limited(netuid, uid, &uids, &weights);

        assert_eq!(
            expected, result,
            "Failed get expected result when guard-checks were not triggered"
        );
    });
}

/// Check _falsey_ path for weights length
#[test]
fn test_is_self_weight_weights_length_not_one() {
    new_test_ext(0).execute_with(|| {
        let max_allowed: u16 = 3;

        let uids: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));
        let uid: u16 = uids[0];
        let weights: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));

        let expected = false;
        let result = SubtensorModule::is_self_weight(uid, &uids, &weights);

        assert_eq!(
            expected, result,
            "Failed get expected result when `weights.len() != 1`"
        );
    });
}

/// Check _falsey_ path for uid vs uids[0]
#[test]
fn test_is_self_weight_uid_not_in_uids() {
    new_test_ext(0).execute_with(|| {
        let max_allowed: u16 = 3;

        let uids: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));
        let uid: u16 = uids[1];
        let weights: Vec<u16> = vec![0];

        let expected = false;
        let result = SubtensorModule::is_self_weight(uid, &uids, &weights);

        assert_eq!(
            expected, result,
            "Failed get expected result when `uid != uids[0]`"
        );
    });
}

/// Check _truthy_ path
/// @TODO: double-check if this really be desired behavior
#[test]
fn test_is_self_weight_uid_in_uids() {
    new_test_ext(0).execute_with(|| {
        let max_allowed: u16 = 1;

        let uids: Vec<u16> = Vec::from_iter((0..max_allowed).map(|id| id + 1));
        let uid: u16 = uids[0];
        let weights: Vec<u16> = vec![0];

        let expected = true;
        let result = SubtensorModule::is_self_weight(uid, &uids, &weights);

        assert_eq!(
            expected, result,
            "Failed get expected result when everything _should_ be fine"
        );
    });
}

/// Check _truthy_ path
#[test]
fn test_check_len_uids_within_allowed_within_network_pool() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;

        let tempo: u16 = 13;
        let modality: u16 = 0;

        let max_registrations_per_block: u16 = 100;

        add_network(netuid, tempo, modality);

        /* @TODO: use a loop maybe */
        register_ok_neuron(netuid, U256::from(1), U256::from(1), 0);
        register_ok_neuron(netuid, U256::from(3), U256::from(3), 65555);
        register_ok_neuron(netuid, U256::from(5), U256::from(5), 75555);
        let max_allowed: u16 = SubtensorModule::get_subnetwork_n(netuid);

        SubtensorModule::set_max_allowed_uids(netuid, max_allowed);
        SubtensorModule::set_max_registrations_per_block(netuid, max_registrations_per_block);

        let uids: Vec<u16> = Vec::from_iter(0..max_allowed);

        let expected = true;
        let result = SubtensorModule::check_len_uids_within_allowed(netuid, &uids);
        assert_eq!(
            expected, result,
            "netuid network length and uids length incompatible"
        );
    });
}

/// Check _falsey_ path
#[test]
fn test_check_len_uids_within_allowed_not_within_network_pool() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;

        let tempo: u16 = 13;
        let modality: u16 = 0;

        let max_registrations_per_block: u16 = 100;

        add_network(netuid, tempo, modality);

        /* @TODO: use a loop maybe */
        register_ok_neuron(netuid, U256::from(1), U256::from(1), 0);
        register_ok_neuron(netuid, U256::from(3), U256::from(3), 65555);
        register_ok_neuron(netuid, U256::from(5), U256::from(5), 75555);
        let max_allowed: u16 = SubtensorModule::get_subnetwork_n(netuid);

        SubtensorModule::set_max_allowed_uids(netuid, max_allowed);
        SubtensorModule::set_max_registrations_per_block(netuid, max_registrations_per_block);

        let uids: Vec<u16> = Vec::from_iter(0..(max_allowed + 1));

        let expected = false;
        let result = SubtensorModule::check_len_uids_within_allowed(netuid, &uids);
        assert_eq!(
            expected, result,
            "Failed to detect incompatible uids for network"
        );
    });
}

#[test]
fn test_set_weights_commit_reveal_enabled_error() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, U256::from(1), U256::from(2), 10);

        let uids = vec![0];
        let weights = vec![1];
        let version_key: u64 = 0;
        let hotkey = U256::from(1);

        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);

        assert_err!(
            SubtensorModule::set_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                weights.clone(),
                version_key
            ),
            Error::<Test>::CommitRevealEnabled
        );

        SubtensorModule::set_commit_reveal_weights_enabled(netuid, false);

        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            uids,
            weights,
            version_key
        ));
    });
}

#[test]
fn test_commit_reveal_weights_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let uids: Vec<u16> = vec![0, 1];
        let weight_values: Vec<u16> = vec![10, 10];
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;
        let hotkey: U256 = U256::from(1);

        let commit_hash: H256 = BlakeTwo256::hash_of(&(
            hotkey,
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        ));

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, U256::from(3), U256::from(4), 300000);
        register_ok_neuron(netuid, U256::from(1), U256::from(2), 100000);
        SubtensorModule::set_weights_set_rate_limit(netuid, 5);
        SubtensorModule::set_validator_permit_for_uid(netuid, 0, true);
        SubtensorModule::set_validator_permit_for_uid(netuid, 1, true);

        SubtensorModule::set_commit_reveal_weights_interval(netuid, 5);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);

        assert_ok!(SubtensorModule::commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            commit_hash
        ));

        step_block(5);

        assert_ok!(SubtensorModule::reveal_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            uids,
            weight_values,
            salt,
            version_key,
        ));
    });
}

#[test]
fn test_commit_reveal_interval() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let uids: Vec<u16> = vec![0, 1];
        let weight_values: Vec<u16> = vec![10, 10];
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;
        let hotkey: U256 = U256::from(1);

        let commit_hash: H256 = BlakeTwo256::hash_of(&(
            hotkey,
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        ));

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, U256::from(3), U256::from(4), 300000);
        register_ok_neuron(netuid, U256::from(1), U256::from(2), 100000);
        SubtensorModule::set_weights_set_rate_limit(netuid, 5);
        SubtensorModule::set_validator_permit_for_uid(netuid, 0, true);
        SubtensorModule::set_validator_permit_for_uid(netuid, 1, true);

        SubtensorModule::set_commit_reveal_weights_interval(netuid, 100);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);
        System::set_block_number(0);

        assert_ok!(SubtensorModule::commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            commit_hash
        ));
        assert_err!(
            SubtensorModule::commit_weights(RuntimeOrigin::signed(hotkey), netuid, commit_hash),
            Error::<Test>::WeightsCommitNotAllowed
        );
        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                weight_values.clone(),
                salt.clone(),
                version_key,
            ),
            Error::<Test>::InvalidRevealCommitTempo
        );
        step_block(99);
        assert_err!(
            SubtensorModule::commit_weights(RuntimeOrigin::signed(hotkey), netuid, commit_hash),
            Error::<Test>::WeightsCommitNotAllowed
        );
        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                weight_values.clone(),
                salt.clone(),
                version_key,
            ),
            Error::<Test>::InvalidRevealCommitTempo
        );
        step_block(1);
        assert_ok!(SubtensorModule::reveal_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        ));
        assert_ok!(SubtensorModule::commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            commit_hash
        ));
        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                weight_values.clone(),
                salt.clone(),
                version_key,
            ),
            Error::<Test>::InvalidRevealCommitTempo
        );
        step_block(100);
        assert_ok!(SubtensorModule::reveal_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        ));

        // Testing that if you miss the next tempo you cannot reveal it.
        assert_ok!(SubtensorModule::commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            commit_hash
        ));
        step_block(205);
        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                weight_values.clone(),
                salt.clone(),
                version_key,
            ),
            Error::<Test>::InvalidRevealCommitTempo
        );

        // Testing when you commit but do not reveal until later intervals
        assert_ok!(SubtensorModule::commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            commit_hash
        ));
        step_block(425);
        let commit_hash_2: H256 = BlakeTwo256::hash_of(&(
            hotkey,
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key + 1,
        ));
        assert_ok!(SubtensorModule::commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            commit_hash_2
        ));
        step_block(100);
        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                weight_values.clone(),
                salt.clone(),
                version_key,
            ),
            Error::<Test>::InvalidRevealCommitHashNotMatch
        );
        assert_ok!(SubtensorModule::reveal_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key + 1,
        ));
    });
}

#[test]
fn test_commit_reveal_hash() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let uids: Vec<u16> = vec![0, 1];
        let weight_values: Vec<u16> = vec![10, 10];
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;
        let hotkey: U256 = U256::from(1);

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, U256::from(3), U256::from(4), 300000);
        register_ok_neuron(netuid, U256::from(1), U256::from(2), 100000);
        SubtensorModule::set_weights_set_rate_limit(netuid, 5);
        SubtensorModule::set_validator_permit_for_uid(netuid, 0, true);
        SubtensorModule::set_validator_permit_for_uid(netuid, 1, true);

        SubtensorModule::set_commit_reveal_weights_interval(netuid, 5);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);

        let commit_hash: H256 = BlakeTwo256::hash_of(&(
            hotkey,
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        ));

        assert_ok!(SubtensorModule::commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            commit_hash
        ));

        step_block(5);

        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                vec![0, 2],
                weight_values.clone(),
                salt.clone(),
                version_key
            ),
            Error::<Test>::InvalidRevealCommitHashNotMatch
        );
        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                weight_values.clone(),
                salt.clone(),
                7,
            ),
            Error::<Test>::InvalidRevealCommitHashNotMatch
        );
        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                vec![10, 9],
                salt.clone(),
                version_key,
            ),
            Error::<Test>::InvalidRevealCommitHashNotMatch
        );
        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                vec![0, 1, 2],
                vec![10, 10, 33],
                salt.clone(),
                9,
            ),
            Error::<Test>::InvalidRevealCommitHashNotMatch
        );

        assert_ok!(SubtensorModule::reveal_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            uids,
            weight_values,
            salt.clone(),
            version_key,
        ));
    });
}

#[test]
fn test_commit_reveal_disabled_or_enabled() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let uids: Vec<u16> = vec![0, 1];
        let weight_values: Vec<u16> = vec![10, 10];
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;
        let hotkey: U256 = U256::from(1);

        let commit_hash: H256 = BlakeTwo256::hash_of(&(
            hotkey,
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        ));

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, U256::from(3), U256::from(4), 300000);
        register_ok_neuron(netuid, U256::from(1), U256::from(2), 100000);
        SubtensorModule::set_weights_set_rate_limit(netuid, 5);
        SubtensorModule::set_validator_permit_for_uid(netuid, 0, true);
        SubtensorModule::set_validator_permit_for_uid(netuid, 1, true);

        SubtensorModule::set_commit_reveal_weights_interval(netuid, 5);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, false);

        assert_err!(
            SubtensorModule::commit_weights(RuntimeOrigin::signed(hotkey), netuid, commit_hash),
            Error::<Test>::CommitRevealDisabled
        );

        step_block(5);

        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                weight_values.clone(),
                salt.clone(),
                version_key,
            ),
            Error::<Test>::CommitRevealDisabled
        );

        SubtensorModule::set_commit_reveal_weights_enabled(netuid + 1, true);

        //Should still fail because bad netuid
        assert_err!(
            SubtensorModule::commit_weights(RuntimeOrigin::signed(hotkey), netuid, commit_hash),
            Error::<Test>::CommitRevealDisabled
        );

        step_block(5);

        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                weight_values.clone(),
                salt.clone(),
                version_key,
            ),
            Error::<Test>::CommitRevealDisabled
        );

        // Enable and should pass
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);

        assert_ok!(SubtensorModule::commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            commit_hash
        ));

        step_block(5);

        assert_ok!(SubtensorModule::reveal_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            uids,
            weight_values,
            salt.clone(),
            version_key,
        ));
    });
}

#[test]
fn test_toggle_commit_reveal_weights_and_set_weights() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let uids: Vec<u16> = vec![0, 1];
        let weight_values: Vec<u16> = vec![10, 10];
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;
        let hotkey: U256 = U256::from(1);

        let commit_hash: H256 = BlakeTwo256::hash_of(&(
            hotkey,
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        ));

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, U256::from(3), U256::from(4), 300000);
        register_ok_neuron(netuid, U256::from(1), U256::from(2), 100000);
        SubtensorModule::set_validator_permit_for_uid(netuid, 0, true);
        SubtensorModule::set_validator_permit_for_uid(netuid, 1, true);

        SubtensorModule::set_weights_set_rate_limit(netuid, 5);
        SubtensorModule::set_commit_reveal_weights_interval(netuid, 5);

        step_block(5);

        // Set weights OK
        let result = SubtensorModule::set_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            uids.clone(),
            weight_values.clone(),
            0,
        );
        assert_ok!(result);

        // Enable Commit/Reveal
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);

        // Commit is enabled the same block
        assert_ok!(SubtensorModule::commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            commit_hash
        ));

        step_block(5); //Step to the next commit/reveal tempo

        // Reveal OK
        assert_ok!(SubtensorModule::reveal_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        ));

        // Disable Commit/Reveal
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, false);

        // Cannot set weights the same block due to WeightsRateLimit
        step_block(5); //step to avoid settingweightstofast

        let result = SubtensorModule::set_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            uids.clone(),
            weight_values.clone(),
            0,
        );
        assert_ok!(result);
    });
}

#[test]
fn test_commit_reveal_bad_salt_fail() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let uids: Vec<u16> = vec![0, 1];
        let weight_values: Vec<u16> = vec![10, 10];
        let salt: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let bad_salt: Vec<u16> = vec![0, 2, 3, 4, 5, 6, 7, 8];
        let version_key: u64 = 0;
        let hotkey: U256 = U256::from(1);

        let commit_hash: H256 = BlakeTwo256::hash_of(&(
            hotkey,
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        ));

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, U256::from(3), U256::from(4), 300000);
        register_ok_neuron(netuid, U256::from(1), U256::from(2), 100000);
        SubtensorModule::set_weights_set_rate_limit(netuid, 5);
        SubtensorModule::set_validator_permit_for_uid(netuid, 0, true);
        SubtensorModule::set_validator_permit_for_uid(netuid, 1, true);

        SubtensorModule::set_commit_reveal_weights_interval(netuid, 5);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);

        assert_ok!(SubtensorModule::commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            commit_hash
        ));

        step_block(5);

        assert_err!(
            SubtensorModule::reveal_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                uids.clone(),
                weight_values.clone(),
                bad_salt.clone(),
                version_key,
            ),
            Error::<Test>::InvalidRevealCommitHashNotMatch
        );
    });
}

fn commit_reveal_set_weights(
    hotkey: U256,
    netuid: u16,
    uids: Vec<u16>,
    weights: Vec<u16>,
    salt: Vec<u16>,
    version_key: u64,
) -> DispatchResult {
    SubtensorModule::set_commit_reveal_weights_interval(netuid, 5);
    SubtensorModule::set_weights_set_rate_limit(netuid, 5);
    SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);

    let commit_hash: H256 = BlakeTwo256::hash_of(&(
        hotkey,
        netuid,
        uids.clone(),
        weights.clone(),
        salt.clone(),
        version_key,
    ));

    SubtensorModule::commit_weights(RuntimeOrigin::signed(hotkey), netuid, commit_hash)?;

    step_block(5);

    SubtensorModule::reveal_weights(
        RuntimeOrigin::signed(hotkey),
        netuid,
        uids,
        weights,
        salt,
        version_key,
    )?;

    Ok(())
}
