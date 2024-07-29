#![allow(clippy::unwrap_used)]

use frame_support::traits::Currency;

use crate::mock::*;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::{transaction_validity::InvalidTransaction, DispatchError};
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::Config;
use pallet_subtensor::{AxonInfoOf, Error, SubtensorSignedExtension};
use sp_core::U256;
use sp_runtime::traits::{DispatchInfoOf, SignedExtension};

mod mock;

/********************************************
    subscribing::subscribe() tests
*********************************************/

// Tests a basic registration dispatch passes.
#[test]
fn test_registration_subscribe_ok_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let nonce: u64 = 0;
        let netuid: u16 = 1;
        let work: Vec<u8> = vec![0; 32];
        let hotkey: U256 = U256::from(0);
        let coldkey: U256 = U256::from(0);
        let call = RuntimeCall::SubtensorModule(SubtensorCall::register {
            netuid,
            block_number,
            nonce,
            work,
            hotkey,
            coldkey,
        });
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(2_992_000_000, 0),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
    });
}

#[test]
fn test_registration_difficulty() {
    new_test_ext(1).execute_with(|| {
        assert_eq!(SubtensorModule::get_difficulty(1).as_u64(), 10000);
    });
}

#[test]
fn test_registration_invalid_seal_hotkey() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id_1: U256 = U256::from(1);
        let hotkey_account_id_2: U256 = U256::from(2);
        let coldkey_account_id: U256 = U256::from(667); // Neighbour of the beast, har har
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &hotkey_account_id_1,
        );
        let (nonce2, work2): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &hotkey_account_id_1,
        );

        //add network
        add_network(netuid, tempo, 0);

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            block_number,
            nonce,
            work.clone(),
            hotkey_account_id_1,
            coldkey_account_id
        ));
        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            block_number,
            nonce2,
            work2.clone(),
            hotkey_account_id_2,
            coldkey_account_id,
        );
        assert_eq!(result, Err(Error::<Test>::InvalidSeal.into()));
    });
}

#[test]
fn test_registration_ok() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            129123813,
            &hotkey_account_id,
        );

        //add network
        add_network(netuid, tempo, 0);

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id
        ));

        // Check if neuron has added to the specified network(netuid)
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);

        //check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
            coldkey_account_id
        );

        // Check if the neuron has added to the Keys
        let neuron_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).unwrap();

        assert!(SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).is_ok());
        // Check if neuron has added to Uids
        let neuro_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).unwrap();
        assert_eq!(neuro_uid, neuron_uid);

        // Check if the balance of this hotkey account for this subnetwork == 0
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            0
        );
    });
}

#[test]
fn test_registration_without_neuron_slot() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            129123813,
            &hotkey_account_id,
        );

        //add network
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, 0);

        assert_noop!(
            SubtensorModule::register(
                <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
                netuid,
                block_number,
                nonce,
                work,
                hotkey_account_id,
                coldkey_account_id
            ),
            Error::<Test>::NoNeuronIdAvailable
        );
    });
}

#[test]
fn test_registration_under_limit() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let block_number: u64 = 0;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let who: <Test as frame_system::Config>::AccountId = hotkey_account_id;

        let max_registrants = 2;
        SubtensorModule::set_target_registrations_per_interval(netuid, max_registrants);

        let (nonce, work) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            129123813,
            &hotkey_account_id,
        );
        let work_clone = work.clone();
        let call = pallet_subtensor::Call::register {
            netuid,
            block_number,
            nonce,
            work: work_clone,
            hotkey: hotkey_account_id,
            coldkey: coldkey_account_id,
        };
        let info: DispatchInfo =
            DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorSignedExtension::<Test>::new();
        //does not actually call register
        let result = extension.validate(&who, &call.into(), &info, 10);
        assert_ok!(result);

        //actually call register
        add_network(netuid, 13, 0);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id
        ));

        let current_registrants = SubtensorModule::get_registrations_this_interval(netuid);
        let target_registrants = SubtensorModule::get_target_registrations_per_interval(netuid);
        assert!(current_registrants <= target_registrants);
    });
}

#[test]
fn test_registration_rate_limit_exceeded() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let block_number: u64 = 0;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let who: <Test as frame_system::Config>::AccountId = hotkey_account_id;

        let target_registrants = 1;
        let max_registrants = target_registrants * 3;
        SubtensorModule::set_target_registrations_per_interval(netuid, target_registrants);
        SubtensorModule::set_registrations_this_interval(netuid, max_registrants);

        let (nonce, work) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            129123813,
            &hotkey_account_id,
        );
        let call = pallet_subtensor::Call::register {
            netuid,
            block_number,
            nonce,
            work,
            hotkey: hotkey_account_id,
            coldkey: coldkey_account_id,
        };
        let info: DispatchInfo =
            DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorSignedExtension::<Test>::new();
        let result = extension.validate(&who, &call.into(), &info, 10);

        // Expectation: The transaction should be rejected
        assert_err!(result, InvalidTransaction::ExhaustsResources);

        let current_registrants = SubtensorModule::get_registrations_this_interval(netuid);
        assert!(current_registrants <= max_registrants);
    });
}

/********************************************
    registration::do_burned_registration tests
*********************************************/

#[test]
fn test_burned_registration_under_limit() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let who: <Test as frame_system::Config>::AccountId = coldkey_account_id;
        let burn_cost = 1000;
        // Set the burn cost
        SubtensorModule::set_burn(netuid, burn_cost);

        add_network(netuid, 13, 0); // Add the network
                                    // Give it some TAO to the coldkey balance; more than the burn cost
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, burn_cost + 10_000);

        let target_registrants = 2;
        let max_registrants = target_registrants * 3; // Maximum is 3 times the target
        SubtensorModule::set_target_registrations_per_interval(netuid, target_registrants);

        let call_burned_register: pallet_subtensor::Call<Test> =
            pallet_subtensor::Call::burned_register {
                netuid,
                hotkey: hotkey_account_id,
            };

        let info: DispatchInfo =
            DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorSignedExtension::<Test>::new();
        //does not actually call register
        let burned_register_result =
            extension.validate(&who, &call_burned_register.into(), &info, 10);
        assert_ok!(burned_register_result);

        //actually call register
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id,
        ));

        let current_registrants = SubtensorModule::get_registrations_this_interval(netuid);
        assert!(current_registrants <= max_registrants);
    });
}

#[test]
fn test_burned_registration_rate_limit_exceeded() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let who: <Test as frame_system::Config>::AccountId = coldkey_account_id;

        let target_registrants = 1;
        let max_registrants = target_registrants * 3; // Maximum is 3 times the target

        SubtensorModule::set_target_registrations_per_interval(netuid, target_registrants);
        // Set the current registrations to the maximum; should not be able to register more
        SubtensorModule::set_registrations_this_interval(netuid, max_registrants);

        let call_burned_register: pallet_subtensor::Call<Test> =
            pallet_subtensor::Call::burned_register {
                netuid,
                hotkey: hotkey_account_id,
            };

        let info: DispatchInfo =
            DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorSignedExtension::<Test>::new();
        let burned_register_result =
            extension.validate(&who, &call_burned_register.into(), &info, 10);

        // Expectation: The transaction should be rejected
        assert_err!(
            burned_register_result,
            InvalidTransaction::ExhaustsResources
        );

        let current_registrants = SubtensorModule::get_registrations_this_interval(netuid);
        assert!(current_registrants <= max_registrants);
    });
}

#[test]
fn test_burned_registration_rate_allows_burn_adjustment() {
    // We need to be able to register more than the *target* registrations per interval
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let who: <Test as frame_system::Config>::AccountId = coldkey_account_id;

        let burn_cost = 1000;
        // Set the burn cost
        SubtensorModule::set_burn(netuid, burn_cost);

        add_network(netuid, 13, 0); // Add the network
                                    // Give it some TAO to the coldkey balance; more than the burn cost
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, burn_cost + 10_000);

        let target_registrants = 1; // Target is 1, but we can register more than that, up to some maximum.
        SubtensorModule::set_target_registrations_per_interval(netuid, target_registrants);
        // Set the current registrations to above the target; we should be able to register at least 1 more
        SubtensorModule::set_registrations_this_interval(netuid, target_registrants);

        // Register one more, so the current registrations are above the target
        let call_burned_register: pallet_subtensor::Call<Test> =
            pallet_subtensor::Call::burned_register {
                netuid,
                hotkey: hotkey_account_id,
            };

        let info: DispatchInfo =
            DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorSignedExtension::<Test>::new();
        //does not actually call register
        let burned_register_result =
            extension.validate(&who, &call_burned_register.into(), &info, 10);
        assert_ok!(burned_register_result);

        //actually call register
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));

        let current_registrants = SubtensorModule::get_registrations_this_interval(netuid);
        assert!(current_registrants > target_registrants); // Should be able to register more than the target
    });
}

#[test]
fn test_burned_registration_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let burn_cost = 1000;
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har
                                                  //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network(netuid, tempo, 0);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);
        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));
        // Check if balance has  decreased to pay for the burn.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            10000 - burn_cost
        ); // funds drained on reg.
           // Check if neuron has added to the specified network(netuid)
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        //check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
            coldkey_account_id
        );
        // Check if the neuron has added to the Keys
        let neuron_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).unwrap();
        assert!(SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).is_ok());
        // Check if neuron has added to Uids
        let neuro_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).unwrap();
        assert_eq!(neuro_uid, neuron_uid);
        // Check if the balance of this hotkey account for this subnetwork == 0
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            0
        );
    });
}

#[test]
fn test_burn_registration_without_neuron_slot() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let burn_cost = 1000;
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har
                                                  //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network(netuid, tempo, 0);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);
        SubtensorModule::set_max_allowed_uids(netuid, 0);

        assert_noop!(
            SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                netuid,
                hotkey_account_id
            ),
            Error::<Test>::NoNeuronIdAvailable
        );
    });
}

#[test]
fn test_burn_adjustment() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 1;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1.
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            hotkey_account_id_1
        ));

        // Register key 2.
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            hotkey_account_id_2
        ));

        // We are over the number of regs allowed this interval.
        // Step the block and trigger the adjustment.
        step_block(1);

        // Check the adjusted burn.
        assert_eq!(SubtensorModule::get_burn_as_u64(netuid), 1500);
    });
}

#[test]
#[cfg(not(tarpaulin))]
fn test_registration_too_many_registrations_per_block() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        SubtensorModule::set_target_registrations_per_interval(netuid, 10);
        assert_eq!(SubtensorModule::get_max_registrations_per_block(netuid), 10);

        let block_number: u64 = 0;
        let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            3942084,
            &U256::from(0),
        );
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            11231312312,
            &U256::from(1),
        );
        let (nonce2, work2): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            212312414,
            &U256::from(2),
        );
        let (nonce3, work3): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            21813123,
            &U256::from(3),
        );
        let (nonce4, work4): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            148141209,
            &U256::from(4),
        );
        let (nonce5, work5): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            1245235534,
            &U256::from(5),
        );
        let (nonce6, work6): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            256234,
            &U256::from(6),
        );
        let (nonce7, work7): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            6923424,
            &U256::from(7),
        );
        let (nonce8, work8): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            124242,
            &U256::from(8),
        );
        let (nonce9, work9): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            153453,
            &U256::from(9),
        );
        let (nonce10, work10): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            345923888,
            &U256::from(10),
        );
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), 10000);

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
            netuid,
            block_number,
            nonce0,
            work0,
            U256::from(0),
            U256::from(0)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 1);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
            netuid,
            block_number,
            nonce1,
            work1,
            U256::from(1),
            U256::from(1)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 2);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(2)),
            netuid,
            block_number,
            nonce2,
            work2,
            U256::from(2),
            U256::from(2)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 3);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(3)),
            netuid,
            block_number,
            nonce3,
            work3,
            U256::from(3),
            U256::from(3)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 4);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(4)),
            netuid,
            block_number,
            nonce4,
            work4,
            U256::from(4),
            U256::from(4)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 5);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(5)),
            netuid,
            block_number,
            nonce5,
            work5,
            U256::from(5),
            U256::from(5)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 6);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(6)),
            netuid,
            block_number,
            nonce6,
            work6,
            U256::from(6),
            U256::from(6)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 7);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(7)),
            netuid,
            block_number,
            nonce7,
            work7,
            U256::from(7),
            U256::from(7)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 8);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(8)),
            netuid,
            block_number,
            nonce8,
            work8,
            U256::from(8),
            U256::from(8)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 9);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(9)),
            netuid,
            block_number,
            nonce9,
            work9,
            U256::from(9),
            U256::from(9)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 10);
        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(10)),
            netuid,
            block_number,
            nonce10,
            work10,
            U256::from(10),
            U256::from(10),
        );
        assert_eq!(
            result,
            Err(Error::<Test>::TooManyRegistrationsThisBlock.into())
        );
    });
}

#[test]
fn test_registration_too_many_registrations_per_interval() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 11);
        assert_eq!(SubtensorModule::get_max_registrations_per_block(netuid), 11);
        SubtensorModule::set_target_registrations_per_interval(netuid, 3);
        assert_eq!(
            SubtensorModule::get_target_registrations_per_interval(netuid),
            3
        );
        // Then the max is 3 * 3 = 9

        let block_number: u64 = 0;
        let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            3942084,
            &U256::from(0),
        );
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            11231312312,
            &U256::from(1),
        );
        let (nonce2, work2): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            212312414,
            &U256::from(2),
        );
        let (nonce3, work3): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            21813123,
            &U256::from(3),
        );
        let (nonce4, work4): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            148141209,
            &U256::from(4),
        );
        let (nonce5, work5): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            1245235534,
            &U256::from(5),
        );
        let (nonce6, work6): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            256234,
            &U256::from(6),
        );
        let (nonce7, work7): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            6923424,
            &U256::from(7),
        );
        let (nonce8, work8): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            124242,
            &U256::from(8),
        );
        let (nonce9, work9): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            153453,
            &U256::from(9),
        );
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), 10000);

        // Subscribe and check extrinsic output
        // Try 10 registrations, this is less than the max per block, but more than the max per interval
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
            netuid,
            block_number,
            nonce0,
            work0,
            U256::from(0),
            U256::from(0)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid), 1);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
            netuid,
            block_number,
            nonce1,
            work1,
            U256::from(1),
            U256::from(1)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid), 2);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(2)),
            netuid,
            block_number,
            nonce2,
            work2,
            U256::from(2),
            U256::from(2)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid), 3);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(3)),
            netuid,
            block_number,
            nonce3,
            work3,
            U256::from(3),
            U256::from(3)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid), 4);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(4)),
            netuid,
            block_number,
            nonce4,
            work4,
            U256::from(4),
            U256::from(4)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid), 5);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(5)),
            netuid,
            block_number,
            nonce5,
            work5,
            U256::from(5),
            U256::from(5)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid), 6);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(6)),
            netuid,
            block_number,
            nonce6,
            work6,
            U256::from(6),
            U256::from(6)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid), 7);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(7)),
            netuid,
            block_number,
            nonce7,
            work7,
            U256::from(7),
            U256::from(7)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid), 8);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(8)),
            netuid,
            block_number,
            nonce8,
            work8,
            U256::from(8),
            U256::from(8)
        ));
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid), 9);
        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(9)),
            netuid,
            block_number,
            nonce9,
            work9,
            U256::from(9),
            U256::from(9),
        );
        assert_eq!(
            result,
            Err(Error::<Test>::TooManyRegistrationsThisInterval.into())
        );
    });
}

#[test]
fn test_registration_immunity_period() { //impl this test when epoch impl and calculating pruning score is done
                                         /* TO DO */
}

#[test]
fn test_registration_already_active_hotkey() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &hotkey_account_id,
        );

        //add network
        add_network(netuid, tempo, 0);

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id
        ));

        let block_number: u64 = 0;
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &hotkey_account_id,
        );
        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::HotKeyAlreadyRegisteredInSubNet.into())
        );
    });
}

#[test]
fn test_registration_invalid_seal() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let (nonce, work): (u64, Vec<u8>) =
            SubtensorModule::create_work_for_block_number(netuid, 1, 0, &hotkey_account_id);

        //add network
        add_network(netuid, tempo, 0);

        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id,
        );
        assert_eq!(result, Err(Error::<Test>::InvalidSeal.into()));
    });
}

#[test]
fn test_registration_invalid_block_number() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        let block_number: u64 = 1;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &hotkey_account_id,
        );

        //add network
        add_network(netuid, tempo, 0);

        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id,
        );
        assert_eq!(result, Err(Error::<Test>::InvalidWorkBlock.into()));
    });
}

#[test]
fn test_registration_invalid_difficulty() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &hotkey_account_id,
        );

        //add network
        add_network(netuid, tempo, 0);

        SubtensorModule::set_difficulty(netuid, 18_446_744_073_709_551_615u64);

        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id,
        );
        assert_eq!(result, Err(Error::<Test>::InvalidDifficulty.into()));
    });
}

#[test]
fn test_registration_failed_no_signature() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 1;
        let netuid: u16 = 1;
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &hotkey_account_id,
        );

        // Subscribe and check extrinsic output
        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::none(),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id,
        );
        assert_eq!(result, Err(DispatchError::BadOrigin));
    });
}

#[test]
fn test_registration_get_uid_to_prune_all_in_immunity_period() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        let netuid: u16 = 1;
        add_network(netuid, 0, 0);
        log::info!("add network");
        register_ok_neuron(netuid, U256::from(0), U256::from(0), 39420842);
        register_ok_neuron(netuid, U256::from(1), U256::from(1), 12412392);
        SubtensorModule::set_pruning_score_for_uid(netuid, 0, 100);
        SubtensorModule::set_pruning_score_for_uid(netuid, 1, 110);
        SubtensorModule::set_immunity_period(netuid, 2);
        assert_eq!(SubtensorModule::get_pruning_score_for_uid(netuid, 0), 100);
        assert_eq!(SubtensorModule::get_pruning_score_for_uid(netuid, 1), 110);
        assert_eq!(SubtensorModule::get_immunity_period(netuid), 2);
        assert_eq!(SubtensorModule::get_current_block_as_u64(), 0);
        assert_eq!(
            SubtensorModule::get_neuron_block_at_registration(netuid, 0),
            0
        );
        assert_eq!(SubtensorModule::get_neuron_to_prune(0), 0);
    });
}

#[test]
fn test_registration_get_uid_to_prune_none_in_immunity_period() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        let netuid: u16 = 1;
        add_network(netuid, 0, 0);
        log::info!("add network");
        register_ok_neuron(netuid, U256::from(0), U256::from(0), 39420842);
        register_ok_neuron(netuid, U256::from(1), U256::from(1), 12412392);
        SubtensorModule::set_pruning_score_for_uid(netuid, 0, 100);
        SubtensorModule::set_pruning_score_for_uid(netuid, 1, 110);
        SubtensorModule::set_immunity_period(netuid, 2);
        assert_eq!(SubtensorModule::get_pruning_score_for_uid(netuid, 0), 100);
        assert_eq!(SubtensorModule::get_pruning_score_for_uid(netuid, 1), 110);
        assert_eq!(SubtensorModule::get_immunity_period(netuid), 2);
        assert_eq!(SubtensorModule::get_current_block_as_u64(), 0);
        assert_eq!(
            SubtensorModule::get_neuron_block_at_registration(netuid, 0),
            0
        );
        step_block(3);
        assert_eq!(SubtensorModule::get_current_block_as_u64(), 3);
        assert_eq!(SubtensorModule::get_neuron_to_prune(0), 0);
    });
}

#[test]
fn test_registration_pruning() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let block_number: u64 = 0;
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            3942084,
            &hotkey_account_id,
        );

        //add network
        add_network(netuid, tempo, 0);

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce0,
            work0,
            hotkey_account_id,
            coldkey_account_id
        ));
        //
        let neuron_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).unwrap();
        SubtensorModule::set_pruning_score_for_uid(netuid, neuron_uid, 2);
        //
        let hotkey_account_id1 = U256::from(2);
        let coldkey_account_id1 = U256::from(668);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            11231312312,
            &hotkey_account_id1,
        );

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id1),
            netuid,
            block_number,
            nonce1,
            work1,
            hotkey_account_id1,
            coldkey_account_id1
        ));
        //
        let neuron_uid1 =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id1).unwrap();
        SubtensorModule::set_pruning_score_for_uid(netuid, neuron_uid1, 3);
        //
        let hotkey_account_id2 = U256::from(3);
        let coldkey_account_id2 = U256::from(669);
        let (nonce2, work2): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            212312414,
            &hotkey_account_id2,
        );

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id2),
            netuid,
            block_number,
            nonce2,
            work2,
            hotkey_account_id2,
            coldkey_account_id2
        ));
    });
}

#[test]
fn test_registration_get_neuron_metadata() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let block_number: u64 = 0;
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            3942084,
            &hotkey_account_id,
        );

        add_network(netuid, tempo, 0);

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce0,
            work0,
            hotkey_account_id,
            coldkey_account_id
        ));
        //
        //let neuron_id = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id);
        // let neuron_uid = SubtensorModule::get_uid_for_net_and_hotkey( netuid, &hotkey_account_id ).unwrap();
        let neuron: AxonInfoOf = SubtensorModule::get_axon_info(netuid, &hotkey_account_id);
        assert_eq!(neuron.ip, 0);
        assert_eq!(neuron.version, 0);
        assert_eq!(neuron.port, 0);
    });
}

#[test]
fn test_registration_add_network_size() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let netuid2: u16 = 2;
        let block_number: u64 = 0;
        let hotkey_account_id = U256::from(1);
        let hotkey_account_id1 = U256::from(2);
        let hotkey_account_id2 = U256::from(3);
        let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            3942084,
            &hotkey_account_id,
        );
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid2,
            block_number,
            11231312312,
            &hotkey_account_id1,
        );
        let (nonce2, work2): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid2,
            block_number,
            21813123,
            &hotkey_account_id2,
        );
        let coldkey_account_id = U256::from(667);

        add_network(netuid, 13, 0);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 0);

        add_network(netuid2, 13, 0);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid2), 0);

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce0,
            work0,
            hotkey_account_id,
            coldkey_account_id
        ));
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid), 1);

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id1),
            netuid2,
            block_number,
            nonce1,
            work1,
            hotkey_account_id1,
            coldkey_account_id
        ));
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id2),
            netuid2,
            block_number,
            nonce2,
            work2,
            hotkey_account_id2,
            coldkey_account_id
        ));
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid2), 2);
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid2), 2);
    });
}

#[test]
fn test_burn_registration_increase_recycled_rao() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let netuid2: u16 = 2;

        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(667);

        // Give funds for burn. 1000 TAO
        let _ =
            Balances::deposit_creating(&coldkey_account_id, Balance::from(1_000_000_000_000_u64));

        add_network(netuid, 13, 0);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 0);

        add_network(netuid2, 13, 0);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid2), 0);

        run_to_block(1);

        let burn_amount = SubtensorModule::get_burn_as_u64(netuid);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            hotkey_account_id
        ));
        assert_eq!(SubtensorModule::get_rao_recycled(netuid), burn_amount);

        run_to_block(2);

        let burn_amount2 = SubtensorModule::get_burn_as_u64(netuid2);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid2,
            hotkey_account_id
        ));
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(2)),
            netuid2,
            U256::from(2)
        ));
        assert_eq!(SubtensorModule::get_rao_recycled(netuid2), burn_amount2 * 2);
        // Validate netuid is not affected.
        assert_eq!(SubtensorModule::get_rao_recycled(netuid), burn_amount);
    });
}

#[test]
fn test_full_pass_through() {
    new_test_ext(1).execute_with(|| {
        // Create 3 networks.
        let netuid0: u16 = 1;
        let netuid1: u16 = 2;
        let netuid2: u16 = 3;

        // With 3 tempos
        let tempo0: u16 = 2;
        let tempo1: u16 = 2;
        let tempo2: u16 = 2;

        // Create 3 keys.
        let hotkey0 = U256::from(0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);

        // With 3 different coldkeys.
        let coldkey0 = U256::from(0);
        let coldkey1 = U256::from(1);
        let coldkey2 = U256::from(2);

        // Add the 3 networks.
        add_network(netuid0, tempo0, 0);
        add_network(netuid1, tempo1, 0);
        add_network(netuid2, tempo2, 0);

        // Check their tempo.
        assert_eq!(SubtensorModule::get_tempo(netuid0), tempo0);
        assert_eq!(SubtensorModule::get_tempo(netuid1), tempo1);
        assert_eq!(SubtensorModule::get_tempo(netuid2), tempo2);

        // Check their emission value.
        assert_eq!(SubtensorModule::get_emission_value(netuid0), 0);
        assert_eq!(SubtensorModule::get_emission_value(netuid1), 0);
        assert_eq!(SubtensorModule::get_emission_value(netuid2), 0);

        // Set their max allowed uids.
        SubtensorModule::set_max_allowed_uids(netuid0, 2);
        SubtensorModule::set_max_allowed_uids(netuid1, 2);
        SubtensorModule::set_max_allowed_uids(netuid2, 2);

        // Check their max allowed.
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid0), 2);
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid0), 2);
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid0), 2);

        // Set the max registration per block.
        SubtensorModule::set_max_registrations_per_block(netuid0, 3);
        SubtensorModule::set_max_registrations_per_block(netuid1, 3);
        SubtensorModule::set_max_registrations_per_block(netuid2, 3);
        assert_eq!(SubtensorModule::get_max_registrations_per_block(netuid0), 3);
        assert_eq!(SubtensorModule::get_max_registrations_per_block(netuid1), 3);
        assert_eq!(SubtensorModule::get_max_registrations_per_block(netuid2), 3);

        // Check that no one has registered yet.
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid0), 0);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid1), 0);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid2), 0);

        // Registered the keys to all networks.
        register_ok_neuron(netuid0, hotkey0, coldkey0, 39420842);
        register_ok_neuron(netuid0, hotkey1, coldkey1, 12412392);
        register_ok_neuron(netuid1, hotkey0, coldkey0, 21813123);
        register_ok_neuron(netuid1, hotkey1, coldkey1, 25755207);
        register_ok_neuron(netuid2, hotkey0, coldkey0, 251232207);
        register_ok_neuron(netuid2, hotkey1, coldkey1, 159184122);

        // Check uids.
        // n0 [ h0, h1 ]
        // n1 [ h0, h1 ]
        // n2 [ h0, h1 ]
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid0, 0).unwrap(),
            hotkey0
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid1, 0).unwrap(),
            hotkey0
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid2, 0).unwrap(),
            hotkey0
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid0, 1).unwrap(),
            hotkey1
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid1, 1).unwrap(),
            hotkey1
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid2, 1).unwrap(),
            hotkey1
        );

        // Check registered networks.
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey0 ).contains( &netuid0 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey0 ).contains( &netuid1 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey0 ).contains( &netuid2 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey1 ).contains( &netuid0 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey1 ).contains( &netuid1 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey1 ).contains( &netuid2 ) );
        // assert!( !SubtensorModule::get_registered_networks_for_hotkey( &hotkey2 ).contains( &netuid0 ) );
        // assert!( !SubtensorModule::get_registered_networks_for_hotkey( &hotkey2 ).contains( &netuid1 ) );
        // assert!( !SubtensorModule::get_registered_networks_for_hotkey( &hotkey2 ).contains( &netuid2 ) );

        // Check the number of registrations.
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid0), 2);
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid1), 2);
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid2), 2);

        // Get the number of uids in each network.
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid0), 2);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid1), 2);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid2), 2);

        // Check the uids exist.
        assert!(SubtensorModule::is_uid_exist_on_network(netuid0, 0));
        assert!(SubtensorModule::is_uid_exist_on_network(netuid1, 0));
        assert!(SubtensorModule::is_uid_exist_on_network(netuid2, 0));

        // Check the other exists.
        assert!(SubtensorModule::is_uid_exist_on_network(netuid0, 1));
        assert!(SubtensorModule::is_uid_exist_on_network(netuid1, 1));
        assert!(SubtensorModule::is_uid_exist_on_network(netuid2, 1));

        // Get the hotkey under each uid.
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid0, 0).unwrap(),
            hotkey0
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid1, 0).unwrap(),
            hotkey0
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid2, 0).unwrap(),
            hotkey0
        );

        // Get the hotkey under the other uid.
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid0, 1).unwrap(),
            hotkey1
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid1, 1).unwrap(),
            hotkey1
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid2, 1).unwrap(),
            hotkey1
        );

        // Check for replacement.
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid0), 2);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid1), 2);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid2), 2);

        // Register the 3rd hotkey.
        register_ok_neuron(netuid0, hotkey2, coldkey2, 59420842);
        register_ok_neuron(netuid1, hotkey2, coldkey2, 31813123);
        register_ok_neuron(netuid2, hotkey2, coldkey2, 451232207);

        // Check for replacement.
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid0), 2);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid1), 2);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid2), 2);

        // Check uids.
        // n0 [ h0, h1 ]
        // n1 [ h0, h1 ]
        // n2 [ h0, h1 ]
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid0, 0).unwrap(),
            hotkey2
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid1, 0).unwrap(),
            hotkey2
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid2, 0).unwrap(),
            hotkey2
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid0, 1).unwrap(),
            hotkey1
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid1, 1).unwrap(),
            hotkey1
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid2, 1).unwrap(),
            hotkey1
        );

        // Check registered networks.
        // hotkey0 has been deregistered.
        // assert!( !SubtensorModule::get_registered_networks_for_hotkey( &hotkey0 ).contains( &netuid0 ) );
        // assert!( !SubtensorModule::get_registered_networks_for_hotkey( &hotkey0 ).contains( &netuid1 ) );
        // assert!( !SubtensorModule::get_registered_networks_for_hotkey( &hotkey0 ).contains( &netuid2 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey1 ).contains( &netuid0 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey1 ).contains( &netuid1 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey1 ).contains( &netuid2 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey2 ).contains( &netuid0 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey2 ).contains( &netuid1 ) );
        // assert!( SubtensorModule::get_registered_networks_for_hotkey( &hotkey2 ).contains( &netuid2 ) );

        // Check the registration counters.
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid0), 3);
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid1), 3);
        assert_eq!(SubtensorModule::get_registrations_this_interval(netuid2), 3);

        // Check the hotkeys are expected.
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid0, 0).unwrap(),
            hotkey2
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid1, 0).unwrap(),
            hotkey2
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid2, 0).unwrap(),
            hotkey2
        );
    });
}

// DEPRECATED #[test]
// fn test_network_connection_requirement() {
//     new_test_ext(1).execute_with(|| {
//         // Add a networks and connection requirements.
//         let netuid_a: u16 = 0;
//         let netuid_b: u16 = 1;
//         add_network(netuid_a, 10, 0);
//         add_network(netuid_b, 10, 0);

//         // Bulk values.
//         let hotkeys: Vec<U256> = (0..=10).map(|x| U256::from(x)).collect();
//         let coldkeys: Vec<U256> = (0..=10).map(|x| U256::from(x)).collect();

//         // Add a connection requirement between the A and B. A requires B.
//         SubtensorModule::add_connection_requirement(netuid_a, netuid_b, u16::MAX);
//         SubtensorModule::set_max_registrations_per_block(netuid_a, 10); // Enough for the below tests.
//         SubtensorModule::set_max_registrations_per_block(netuid_b, 10); // Enough for the below tests.
//         SubtensorModule::set_max_allowed_uids(netuid_a, 10); // Enough for the below tests.
//         SubtensorModule::set_max_allowed_uids(netuid_b, 10); // Enough for the below tests.

//         // Attempt registration on A fails because the hotkey is not registered on network B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_a, 0, 3942084, &U256::from(0));
//         assert_eq!(
//             SubtensorModule::register(
//                 <<Test as Config>::RuntimeOrigin>::signed(hotkeys[0]),
//                 netuid_a,
//                 0,
//                 nonce,
//                 work,
//                 hotkeys[0],
//                 coldkeys[0]
//             ),
//             Err(Error::<Test>::DidNotPassConnectedNetworkRequirement.into())
//         );

//         // Attempt registration on B passes because there is no exterior requirement.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_b, 0, 5942084, &U256::from(0));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[0]),
//             netuid_b,
//             0,
//             nonce,
//             work,
//             hotkeys[0],
//             coldkeys[0]
//         ));

//         // Attempt registration on A passes because this key is in the top 100 of keys on network B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_a, 0, 6942084, &U256::from(0));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[0]),
//             netuid_a,
//             0,
//             nonce,
//             work,
//             hotkeys[0],
//             coldkeys[0]
//         ));

//         // Lets attempt the key registration on A. Fails because we are not in B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_a, 0, 634242084, &U256::from(1));
//         assert_eq!(
//             SubtensorModule::register(
//                 <<Test as Config>::RuntimeOrigin>::signed(hotkeys[1]),
//                 netuid_a,
//                 0,
//                 nonce,
//                 work,
//                 hotkeys[1],
//                 coldkeys[1]
//             ),
//             Err(Error::<Test>::DidNotPassConnectedNetworkRequirement.into())
//         );

//         // Lets register the next key on B. Passes, np.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_b, 0, 7942084, &U256::from(1));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[1]),
//             netuid_b,
//             0,
//             nonce,
//             work,
//             hotkeys[1],
//             coldkeys[1]
//         ));

//         // Lets make the connection requirement harder. Top 0th percentile.
//         SubtensorModule::add_connection_requirement(netuid_a, netuid_b, 0);

//         // Attempted registration passes because the prunning score for hotkey_1 is the top keys on network B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_a, 0, 8942084, &U256::from(1));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[1]),
//             netuid_a,
//             0,
//             nonce,
//             work,
//             hotkeys[1],
//             coldkeys[1]
//         ));

//         // Lets register key 3 with lower prunning score.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_b, 0, 9942084, &U256::from(2));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[2]),
//             netuid_b,
//             0,
//             nonce,
//             work,
//             hotkeys[2],
//             coldkeys[2]
//         ));
//         SubtensorModule::set_pruning_score_for_uid(
//             netuid_b,
//             SubtensorModule::get_uid_for_net_and_hotkey(netuid_b, &hotkeys[2]).unwrap(),
//             0,
//         ); // Set prunning score to 0.
//         SubtensorModule::set_pruning_score_for_uid(
//             netuid_b,
//             SubtensorModule::get_uid_for_net_and_hotkey(netuid_b, &hotkeys[1]).unwrap(),
//             0,
//         ); // Set prunning score to 0.
//         SubtensorModule::set_pruning_score_for_uid(
//             netuid_b,
//             SubtensorModule::get_uid_for_net_and_hotkey(netuid_b, &hotkeys[0]).unwrap(),
//             0,
//         ); // Set prunning score to 0.

//         // Lets register key 4 with higher prunining score.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_b, 0, 10142084, &U256::from(3));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[3]),
//             netuid_b,
//             0,
//             nonce,
//             work,
//             hotkeys[3],
//             coldkeys[3]
//         ));
//         SubtensorModule::set_pruning_score_for_uid(
//             netuid_b,
//             SubtensorModule::get_uid_for_net_and_hotkey(netuid_b, &hotkeys[3]).unwrap(),
//             1,
//         ); // Set prunning score to 1.

//         // Attempted register of key 3 fails because of bad prunning score on B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_a, 0, 11142084, &U256::from(2));
//         assert_eq!(
//             SubtensorModule::register(
//                 <<Test as Config>::RuntimeOrigin>::signed(hotkeys[2]),
//                 netuid_a,
//                 0,
//                 nonce,
//                 work,
//                 hotkeys[2],
//                 coldkeys[2]
//             ),
//             Err(Error::<Test>::DidNotPassConnectedNetworkRequirement.into())
//         );

//         // Attempt to register key 4 passes because of best prunning score on B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_b, 0, 12142084, &U256::from(3));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[3]),
//             netuid_a,
//             0,
//             nonce,
//             work,
//             hotkeys[3],
//             coldkeys[3]
//         ));
//     });
// }

#[test]
fn test_registration_origin_hotkey_mismatch() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id_1: U256 = U256::from(1);
        let hotkey_account_id_2: U256 = U256::from(2);
        let coldkey_account_id: U256 = U256::from(668);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &hotkey_account_id_1,
        );

        //add network
        add_network(netuid, tempo, 0);

        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            block_number,
            nonce,
            work.clone(),
            hotkey_account_id_2, // Not the same as the origin.
            coldkey_account_id,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::TransactorAccountShouldBeHotKey.into())
        );
    });
}

#[test]
fn test_registration_disabled() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id: U256 = U256::from(668);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &hotkey_account_id,
        );

        //add network
        add_network(netuid, tempo, 0);
        SubtensorModule::set_network_registration_allowed(netuid, false);
        SubtensorModule::set_network_pow_registration_allowed(netuid, false);

        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work.clone(),
            hotkey_account_id,
            coldkey_account_id,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::SubNetRegistrationDisabled.into())
        );
    });
}

// #[ignore]
// #[test]
// fn test_hotkey_swap_ok() {
//     new_test_ext(1).execute_with(|| {
//         let netuid: u16 = 1;
//         let tempo: u16 = 13;
//         let hotkey_account_id = U256::from(1);
//         let burn_cost = 1000;
//         let coldkey_account_id = U256::from(667);

//         SubtensorModule::set_burn(netuid, burn_cost);
//         add_network(netuid, tempo, 0);

//         // Give it some $$$ in his coldkey balance
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10_000_000_000);

//         // Subscribe and check extrinsic output
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid,
//             hotkey_account_id
//         ));

//         let new_hotkey = U256::from(1337);
//         assert_ok!(SubtensorModule::swap_hotkey(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             hotkey_account_id,
//             new_hotkey
//         ));
//         assert_ne!(
//             SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
//             coldkey_account_id
//         );
//         assert_eq!(
//             SubtensorModule::get_owning_coldkey_for_hotkey(&new_hotkey),
//             coldkey_account_id
//         );
//     });
// }

// #[ignore]
// #[test]
// fn test_hotkey_swap_not_owner() {
//     new_test_ext(1).execute_with(|| {
//         let netuid: u16 = 1;
//         let tempo: u16 = 13;
//         let hotkey_account_id = U256::from(1);
//         let burn_cost = 1000;
//         let coldkey_account_id = U256::from(2);
//         let not_owner_coldkey = U256::from(3);

//         SubtensorModule::set_burn(netuid, burn_cost);
//         add_network(netuid, tempo, 0);

//         // Give it some $$$ in his coldkey balance
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

//         // Subscribe and check extrinsic output
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid,
//             hotkey_account_id
//         ));

//         let new_hotkey = U256::from(4);
//         assert_err!(
//             SubtensorModule::swap_hotkey(
//                 <<Test as Config>::RuntimeOrigin>::signed(not_owner_coldkey),
//                 hotkey_account_id,
//                 new_hotkey
//             ),
//             Error::<Test>::NonAssociatedColdKey
//         );
//     });
// }

// #[ignore]
// #[test]
// fn test_hotkey_swap_same_key() {
//     new_test_ext(1).execute_with(|| {
//         let netuid: u16 = 1;
//         let tempo: u16 = 13;
//         let hotkey_account_id = U256::from(1);
//         let burn_cost = 1000;
//         let coldkey_account_id = U256::from(2);

//         SubtensorModule::set_burn(netuid, burn_cost);
//         add_network(netuid, tempo, 0);

//         // Give it some $$$ in his coldkey balance
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

//         // Subscribe and check extrinsic output
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid,
//             hotkey_account_id
//         ));

//         assert_err!(
//             SubtensorModule::swap_hotkey(
//                 <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//                 hotkey_account_id,
//                 hotkey_account_id
//             ),
//             Error::<Test>::HotKeyAlreadyRegisteredInSubNet
//         );
//     });
// }

// #[ignore]
// #[test]
// fn test_hotkey_swap_registered_key() {
//     new_test_ext(1).execute_with(|| {
//         let netuid: u16 = 1;
//         let tempo: u16 = 13;
//         let hotkey_account_id = U256::from(1);
//         let burn_cost = 1000;
//         let coldkey_account_id = U256::from(2);

//         SubtensorModule::set_burn(netuid, burn_cost);
//         add_network(netuid, tempo, 0);

//         // Give it some $$$ in his coldkey balance
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 100_000_000_000);

//         // Subscribe and check extrinsic output
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid,
//             hotkey_account_id
//         ));

//         let new_hotkey = U256::from(3);
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid,
//             new_hotkey
//         ));

//         assert_err!(
//             SubtensorModule::swap_hotkey(
//                 <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//                 hotkey_account_id,
//                 new_hotkey
//             ),
//             Error::<Test>::HotKeyAlreadyRegisteredInSubNet
//         );
//     });
// }
