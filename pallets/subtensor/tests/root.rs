#![allow(clippy::indexing_slicing, clippy::unwrap_used)]

use crate::mock::*;
use frame_support::{assert_err, assert_ok};
use frame_system::Config;
use frame_system::{EventRecord, Phase};
use pallet_subtensor::migration;
use pallet_subtensor::Error;
use sp_core::{Get, H256, U256};

mod mock;

#[allow(dead_code)]
fn record(event: RuntimeEvent) -> EventRecord<RuntimeEvent, H256> {
    EventRecord {
        phase: Phase::Initialization,
        event,
        topics: vec![],
    }
}

#[test]
fn test_root_register_network_exist() {
    new_test_ext(1).execute_with(|| {
        migration::migrate_create_root_network::<Test>();
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
        ));
    });
}

#[test]
fn test_set_weights_not_root_error() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;

        let dests = vec![0];
        let weights = vec![1];
        let version_key: u64 = 0;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 2143124);

        assert_err!(
            SubtensorModule::set_root_weights(
                RuntimeOrigin::signed(coldkey),
                netuid,
                hotkey,
                dests.clone(),
                weights.clone(),
                version_key,
            ),
            Error::<Test>::NotRootSubnet
        );
    });
}

#[test]
fn test_root_register_normal_on_root_fails() {
    new_test_ext(1).execute_with(|| {
        migration::migrate_create_root_network::<Test>();
        // Test fails because normal registrations are not allowed
        // on the root network.
        let root_netuid: u16 = 0;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);

        // Burn registration fails.
        SubtensorModule::set_burn(root_netuid, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1);
        assert_eq!(
            SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                root_netuid,
                hotkey_account_id
            ),
            Err(Error::<Test>::RegistrationNotPermittedOnRootSubnet.into())
        );
        // Pow registration fails.
        let block_number: u64 = SubtensorModule::get_current_block_as_u64();
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            root_netuid,
            block_number,
            0,
            &hotkey_account_id,
        );
        assert_eq!(
            SubtensorModule::register(
                <<Test as frame_system::Config>::RuntimeOrigin>::signed(hotkey_account_id),
                root_netuid,
                block_number,
                nonce,
                work,
                hotkey_account_id,
                coldkey_account_id,
            ),
            Err(Error::<Test>::RegistrationNotPermittedOnRootSubnet.into())
        );
    });
}

#[test]
fn test_root_register_stake_based_pruning_works() {
    new_test_ext(1).execute_with(|| {
        migration::migrate_create_root_network::<Test>();
        // Add two networks.
        let root_netuid: u16 = 0;
        let other_netuid: u16 = 1;
        add_network(other_netuid, 0, 0);

        // Set params to allow all registrations to subnet.
        SubtensorModule::set_burn(other_netuid, 0);
        SubtensorModule::set_max_registrations_per_block(other_netuid, 256);
        SubtensorModule::set_target_registrations_per_interval(other_netuid, 256);

        SubtensorModule::set_max_registrations_per_block(root_netuid, 1000);
        SubtensorModule::set_target_registrations_per_interval(root_netuid, 1000);

        // Register 128 accounts with stake to the other network.
        for i in 0..128 {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            // Add balance
            SubtensorModule::add_balance_to_coldkey_account(&cold, 1000 + (i as u64));
            // Register
            assert_ok!(SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                other_netuid,
                hot
            ));
            // Add stake on other network
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
                1000 + (i as u64)
            ));
            // Check successful registration.
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(other_netuid, &hot).is_ok());
            // Check that they are NOT all delegates
            assert!(!SubtensorModule::hotkey_is_delegate(&hot));
        }

        // Register the first 64 accounts with stake to the root network.
        for i in 0..64 {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
            ));
            // Check successful registration.
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
            // Check that they are all delegates
            assert!(SubtensorModule::hotkey_is_delegate(&hot));
        }

        // Register the second 64 accounts with stake to the root network.
        // Replaces the first 64
        for i in 64..128 {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
            ));
            // Check successful registration.
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
        }

        // Register the first 64 accounts again, this time failing because they
        // don't have enough stake.
        for i in 0..64 {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            assert_eq!(
                SubtensorModule::root_register(
                    <<Test as Config>::RuntimeOrigin>::signed(cold),
                    hot,
                ),
                Err(Error::<Test>::StakeTooLowForRoot.into())
            );
            // Check for unsuccessful registration.
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_err());
            // Check that they are NOT senate members
            assert!(!SubtensorModule::is_senate_member(&hot));
        }
    });
}

#[test]
fn test_root_set_weights() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        migration::migrate_create_root_network::<Test>();

        let n: usize = 10;
        let root_netuid: u16 = 0;
        SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
        SubtensorModule::set_target_registrations_per_interval(root_netuid, n as u16);
        SubtensorModule::set_max_allowed_uids(root_netuid, n as u16);
        for i in 0..n {
            let hotkey_account_id: U256 = U256::from(i);
            let coldkey_account_id: U256 = U256::from(i + 456);
            SubtensorModule::add_balance_to_coldkey_account(
                &coldkey_account_id,
                1_000_000_000_000_000,
            );
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
            ));
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                1000
            ));
        }

        log::info!("subnet limit: {:?}", SubtensorModule::get_max_subnets());
        log::info!(
            "current subnet count: {:?}",
            SubtensorModule::get_num_subnets()
        );

        // Lets create n networks
        for netuid in 1..n {
            log::debug!("Adding network with netuid: {}", netuid);
            assert_ok!(SubtensorModule::register_network(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(netuid + 456))
            ));
        }

        // Test that signing with hotkey will fail.
        for i in 0..n {
            let hotkey = U256::from(i);
            let uids: Vec<u16> = vec![i as u16];
            let values: Vec<u16> = vec![1];
            assert_err!(
                SubtensorModule::set_root_weights(
                    <<Test as Config>::RuntimeOrigin>::signed(hotkey),
                    root_netuid,
                    hotkey,
                    uids,
                    values,
                    0,
                ),
                Error::<Test>::NonAssociatedColdKey
            );
        }

        // Test that signing an unassociated coldkey will fail.
        let unassociated_coldkey = U256::from(612);
        for i in 0..n {
            let hotkey = U256::from(i);
            let uids: Vec<u16> = vec![i as u16];
            let values: Vec<u16> = vec![1];
            assert_err!(
                SubtensorModule::set_root_weights(
                    <<Test as Config>::RuntimeOrigin>::signed(unassociated_coldkey),
                    root_netuid,
                    hotkey,
                    uids,
                    values,
                    0,
                ),
                Error::<Test>::NonAssociatedColdKey
            );
        }

        // Set weights into diagonal matrix.
        for i in 0..n {
            let hotkey = U256::from(i);
            let coldkey = U256::from(i + 456);
            let uids: Vec<u16> = vec![i as u16];
            let values: Vec<u16> = vec![1];
            assert_ok!(SubtensorModule::set_root_weights(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                root_netuid,
                hotkey,
                uids,
                values,
                0,
            ));
        }
        // Run the root epoch
        log::debug!("Running Root epoch");
        SubtensorModule::set_tempo(root_netuid, 1);
        assert_ok!(SubtensorModule::root_epoch(1_000_000_000));
        // Check that the emission values have been set.
        for netuid in 1..n {
            log::debug!("check emission for netuid: {}", netuid);
            assert_eq!(
                SubtensorModule::get_subnet_emission_value(netuid as u16),
                99_999_999
            );
        }
        step_block(2);
        // Check that the pending emission values have been set.
        for netuid in 1..n {
            log::debug!(
                "check pending emission for netuid {} has pending {}",
                netuid,
                SubtensorModule::get_pending_emission(netuid as u16)
            );
            assert_eq!(
                SubtensorModule::get_pending_emission(netuid as u16),
                199_999_998
            );
        }
        step_block(1);
        for netuid in 1..n {
            log::debug!(
                "check pending emission for netuid {} has pending {}",
                netuid,
                SubtensorModule::get_pending_emission(netuid as u16)
            );
            assert_eq!(
                SubtensorModule::get_pending_emission(netuid as u16),
                299_999_997
            );
        }
        let step = SubtensorModule::blocks_until_next_epoch(
            10,
            1000,
            SubtensorModule::get_current_block_as_u64(),
        );
        step_block(step as u16);
        assert_eq!(SubtensorModule::get_pending_emission(10), 0);
    });
}

#[test]
fn test_root_set_weights_out_of_order_netuids() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        migration::migrate_create_root_network::<Test>();

        let n: usize = 10;
        let root_netuid: u16 = 0;
        SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
        SubtensorModule::set_target_registrations_per_interval(root_netuid, n as u16);
        SubtensorModule::set_max_allowed_uids(root_netuid, n as u16);
        for i in 0..n {
            let hotkey_account_id: U256 = U256::from(i);
            let coldkey_account_id: U256 = U256::from(i);
            SubtensorModule::add_balance_to_coldkey_account(
                &coldkey_account_id,
                1_000_000_000_000_000,
            );
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
            ));
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                1000
            ));
        }

        log::info!("subnet limit: {:?}", SubtensorModule::get_max_subnets());
        log::info!(
            "current subnet count: {:?}",
            SubtensorModule::get_num_subnets()
        );

        // Lets create n networks
        for netuid in 1..n {
            log::debug!("Adding network with netuid: {}", netuid);

            if netuid % 2 == 0 {
                assert_ok!(SubtensorModule::register_network(
                    <<Test as Config>::RuntimeOrigin>::signed(U256::from(netuid))
                ));
            } else {
                add_network(netuid as u16 * 10, 1000, 0)
            }
        }

        log::info!("netuids: {:?}", SubtensorModule::get_all_subnet_netuids());
        log::info!(
            "root network count: {:?}",
            SubtensorModule::get_subnetwork_n(0)
        );

        let subnets = SubtensorModule::get_all_subnet_netuids();
        // Set weights into diagonal matrix.
        for (i, netuid) in subnets.iter().enumerate() {
            let uids: Vec<u16> = vec![*netuid];
            let values: Vec<u16> = vec![1];

            let coldkey = U256::from(i);
            let hotkey = U256::from(i);
            assert_ok!(SubtensorModule::set_root_weights(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                root_netuid,
                hotkey,
                uids,
                values,
                0,
            ));
        }
        // Run the root epoch
        log::debug!("Running Root epoch");
        SubtensorModule::set_tempo(root_netuid, 1);
        assert_ok!(SubtensorModule::root_epoch(1_000_000_000));
        // Check that the emission values have been set.
        for netuid in subnets.iter() {
            log::debug!("check emission for netuid: {}", netuid);
            assert_eq!(
                SubtensorModule::get_subnet_emission_value(*netuid),
                99_999_999
            );
        }
        step_block(2);
        // Check that the pending emission values have been set.
        for netuid in subnets.iter() {
            if *netuid == 0 {
                continue;
            }

            log::debug!(
                "check pending emission for netuid {} has pending {}",
                netuid,
                SubtensorModule::get_pending_emission(*netuid)
            );
            assert_eq!(SubtensorModule::get_pending_emission(*netuid), 199_999_998);
        }
        step_block(1);
        for netuid in subnets.iter() {
            if *netuid == 0 {
                continue;
            }

            log::debug!(
                "check pending emission for netuid {} has pending {}",
                netuid,
                SubtensorModule::get_pending_emission(*netuid)
            );
            assert_eq!(SubtensorModule::get_pending_emission(*netuid), 299_999_997);
        }
        let step = SubtensorModule::blocks_until_next_epoch(
            9,
            1000,
            SubtensorModule::get_current_block_as_u64(),
        );
        step_block(step as u16);
        assert_eq!(SubtensorModule::get_pending_emission(9), 0);
    });
}

#[test]
fn test_root_subnet_creation_deletion() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        migration::migrate_create_root_network::<Test>();
        // Owner of subnets.
        let owner: U256 = U256::from(0);

        // Add a subnet.
        SubtensorModule::add_balance_to_coldkey_account(&owner, 1_000_000_000_000_000);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 0, mult: 1 lock_cost: 100000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 0, mult: 1 lock_cost: 100000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000);
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 1, mult: 1 lock_cost: 100000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 1, mult: 2 lock_cost: 200000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 200_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 2, mult: 2 lock_cost: 150000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 150_000_000_000); // Reduced by 50%
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 3, mult: 2 lock_cost: 100000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000); // Reduced another 50%
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 4, mult: 2 lock_cost: 100000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000); // Reaches min value
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 4, lock_reduction_interval: 2, current_block: 4, mult: 2 lock_cost: 200000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 200_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 4, lock_reduction_interval: 2, current_block: 5, mult: 2 lock_cost: 150000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 150000000000, min_lock: 100000000000, last_lock_block: 5, lock_reduction_interval: 2, current_block: 5, mult: 2 lock_cost: 300000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 300_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 150000000000, min_lock: 100000000000, last_lock_block: 5, lock_reduction_interval: 2, current_block: 6, mult: 2 lock_cost: 225000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 225000000000, min_lock: 100000000000, last_lock_block: 6, lock_reduction_interval: 2, current_block: 6, mult: 2 lock_cost: 450000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 450_000_000_000); // Increasing
        step_block(1);
        // last_lock: 225000000000, min_lock: 100000000000, last_lock_block: 6, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 337500000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 337500000000, min_lock: 100000000000, last_lock_block: 7, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 675000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 675_000_000_000); // Increasing.
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 337500000000, min_lock: 100000000000, last_lock_block: 7, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 675000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 1_350_000_000_000); // Double increasing.
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        assert_eq!(SubtensorModule::get_network_lock_cost(), 2_700_000_000_000); // Double increasing again.

        // Now drop it like its hot to min again.
        step_block(1);
        assert_eq!(SubtensorModule::get_network_lock_cost(), 2_025_000_000_000); // 675_000_000_000 decreasing.
        step_block(1);
        assert_eq!(SubtensorModule::get_network_lock_cost(), 1_350_000_000_000); // 675_000_000_000 decreasing.
        step_block(1);
        assert_eq!(SubtensorModule::get_network_lock_cost(), 675_000_000_000); // 675_000_000_000 decreasing.
        step_block(1);
        assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000); // 675_000_000_000 decreasing with 100000000000 min
    });
}

#[test]
fn test_network_pruning() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        migration::migrate_create_root_network::<Test>();

        assert_eq!(SubtensorModule::get_total_issuance(), 0);

        let n: usize = 10;
        let root_netuid: u16 = 0;
        SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
        SubtensorModule::set_target_registrations_per_interval(root_netuid, n as u16);
        SubtensorModule::set_max_allowed_uids(root_netuid, n as u16 + 1);
        SubtensorModule::set_tempo(root_netuid, 1);
        // No validators yet.
        assert_eq!(SubtensorModule::get_subnetwork_n(root_netuid), 0);

        for i in 0..n {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            let uids: Vec<u16> = (0..i as u16).collect();
            let values: Vec<u16> = vec![1; i];
            SubtensorModule::add_balance_to_coldkey_account(&cold, 1_000_000_000_000_000);
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot
            ));
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
                1_000
            ));
            assert_ok!(SubtensorModule::register_network(
                <<Test as Config>::RuntimeOrigin>::signed(cold)
            ));
            log::debug!("Adding network with netuid: {}", (i as u16) + 1);
            assert!(SubtensorModule::if_subnet_exist((i as u16) + 1));
            assert!(SubtensorModule::is_hotkey_registered_on_network(
                root_netuid,
                &hot
            ));
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
            assert_ok!(SubtensorModule::set_root_weights(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                root_netuid,
                hot,
                uids,
                values,
                0
            ));
            SubtensorModule::set_tempo((i as u16) + 1, 1);
            SubtensorModule::set_burn((i as u16) + 1, 0);
            assert_ok!(SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                (i as u16) + 1,
                hot
            ));
            assert_eq!(
                SubtensorModule::get_subnetwork_n(root_netuid),
                (i as u16) + 1
            );
        }
        // Stakes
        // 0 : 10_000
        // 1 : 9_000
        // 2 : 8_000
        // 3 : 7_000
        // 4 : 6_000
        // 5 : 5_000
        // 6 : 4_000
        // 7 : 3_000
        // 8 : 2_000
        // 9 : 1_000

        step_block(1);
        assert_ok!(SubtensorModule::root_epoch(1_000_000_000));
        assert_eq!(SubtensorModule::get_subnet_emission_value(0), 385_861_815);
        assert_eq!(SubtensorModule::get_subnet_emission_value(1), 249_435_914);
        assert_eq!(SubtensorModule::get_subnet_emission_value(2), 180_819_837);
        assert_eq!(SubtensorModule::get_subnet_emission_value(3), 129_362_980);
        assert_eq!(SubtensorModule::get_subnet_emission_value(4), 50_857_187);
        assert_eq!(SubtensorModule::get_subnet_emission_value(5), 3_530_356);
        step_block(1);
        assert_eq!(SubtensorModule::get_pending_emission(0), 0); // root network gets no pending emission.
        assert_eq!(SubtensorModule::get_pending_emission(1), 249_435_914);
        assert_eq!(SubtensorModule::get_pending_emission(2), 0); // This has been drained.
        assert_eq!(SubtensorModule::get_pending_emission(3), 129_362_980);
        assert_eq!(SubtensorModule::get_pending_emission(4), 0); // This network has been drained.
        assert_eq!(SubtensorModule::get_pending_emission(5), 3_530_356);
        step_block(1);
    });
}

#[test]
fn test_network_prune_results() {
    new_test_ext(1).execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        SubtensorModule::set_network_immunity_period(3);
        SubtensorModule::set_network_min_lock(0);
        SubtensorModule::set_network_rate_limit(0);

        let owner: U256 = U256::from(0);
        SubtensorModule::add_balance_to_coldkey_account(&owner, 1_000_000_000_000_000);

        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        step_block(3);

        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        step_block(3);

        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        step_block(3);

        // lowest emission
        SubtensorModule::set_emission_values(&[1u16, 2u16, 3u16], vec![5u64, 4u64, 4u64]).unwrap();
        assert_eq!(SubtensorModule::get_subnet_to_prune(), 2u16);

        // equal emission, creation date
        SubtensorModule::set_emission_values(&[1u16, 2u16, 3u16], vec![5u64, 5u64, 4u64]).unwrap();
        assert_eq!(SubtensorModule::get_subnet_to_prune(), 3u16);

        // equal emission, creation date
        SubtensorModule::set_emission_values(&[1u16, 2u16, 3u16], vec![4u64, 5u64, 5u64]).unwrap();
        assert_eq!(SubtensorModule::get_subnet_to_prune(), 1u16);
    });
}

#[test]
fn test_weights_after_network_pruning() {
    new_test_ext(1).execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        assert_eq!(SubtensorModule::get_total_issuance(), 0);

        // Set up N subnets, with max N + 1 allowed UIDs
        let n: usize = 2;
        let root_netuid: u16 = 0;
        SubtensorModule::set_network_immunity_period(3);
        SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
        SubtensorModule::set_max_subnets(n as u16);
        SubtensorModule::set_weights_set_rate_limit(root_netuid, 0_u64);

        // No validators yet.
        assert_eq!(SubtensorModule::get_subnetwork_n(root_netuid), 0);

        for i in 0..n {
            // Register a validator
            let cold: U256 = U256::from(i);

            SubtensorModule::add_balance_to_coldkey_account(&cold, 1_000_000_000_000);

            // Register a network
            assert_ok!(SubtensorModule::register_network(
                <<Test as Config>::RuntimeOrigin>::signed(cold)
            ));

            log::debug!("Adding network with netuid: {}", (i as u16) + 1);
            assert!(SubtensorModule::if_subnet_exist((i as u16) + 1));
            step_block(3);
        }

        // Register a validator in subnet 0
        let hot: U256 = U256::from((n as u64) - 1);
        let cold: U256 = U256::from((n as u64) - 1);

        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            hot
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            hot,
            1_000
        ));

        // Let's give these subnets some weights
        let uids: Vec<u16> = (0..(n as u16) + 1).collect();
        let values: Vec<u16> = vec![4u16, 2u16, 6u16];
        log::info!("uids set: {:?}", uids);
        log::info!("values set: {:?}", values);
        log::info!("In netuid: {:?}", root_netuid);
        assert_ok!(SubtensorModule::set_root_weights(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            root_netuid,
            hot,
            uids,
            values,
            0
        ));

        log::info!(
            "Root network weights before extra network registration: {:?}",
            SubtensorModule::get_root_weights()
        );
        log::info!("Max subnets: {:?}", SubtensorModule::get_max_subnets());
        let i = (n as u16) + 1;
        // let _hot: U256 = U256::from(i);
        let cold: U256 = U256::from(i);

        SubtensorModule::add_balance_to_coldkey_account(&cold, 1_000_000_000_000_000_000);
        let subnet_to_prune = SubtensorModule::get_subnet_to_prune();

        // Subnet 1 should be pruned here.
        assert_eq!(subnet_to_prune, 1);
        log::info!("Removing subnet: {:?}", subnet_to_prune);

        // Check that the weights have been set appropriately.
        let latest_weights = SubtensorModule::get_root_weights();
        log::info!("Weights before register network: {:?}", latest_weights);
        // We expect subnet 1 to be deregistered as it is oldest and has lowest emissions
        assert_eq!(latest_weights[0][1], 21845);

        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(cold)
        ));

        // Subnet should not exist, as it would replace a previous subnet.
        assert!(!SubtensorModule::if_subnet_exist(i + 1));

        log::info!(
            "Root network weights: {:?}",
            SubtensorModule::get_root_weights()
        );

        let latest_weights = SubtensorModule::get_root_weights();
        log::info!(
            "Weights after register network: {:?}",
            SubtensorModule::get_root_weights()
        );

        // Subnet 0 should be kicked, and thus its weight should be 0
        assert_eq!(latest_weights[0][1], 0);
    });
}

/// This test checks the halving mechanism of the emission schedule.
/// Run this test using the following command:
/// `cargo test --package pallet-subtensor --test root test_issance_bounds`
#[test]
fn test_issuance_bounds() {
    new_test_ext(1).execute_with(|| {
        // Simulate 100 halvings convergence to 21M. Note that the total issuance never reaches 21M because of rounding errors.
        // We converge to 20_999_999_989_500_000 (< 1 TAO away).
        let n_halvings: usize = 100;
        let mut total_issuance: u64 = 0;
        for _ in 0..n_halvings {
            let block_emission_10_500_000x: u64 =
                SubtensorModule::get_block_emission_for_issuance(total_issuance).unwrap()
                    * 10_500_000;
            total_issuance += block_emission_10_500_000x;
        }
        assert_eq!(total_issuance, 20_999_999_989_500_000);
    })
}

/// This test checks the halving mechanism of the emission schedule.
/// Run this test using the following command:
/// `cargo test --package pallet-subtensor --test root test_halving`
#[test]
fn test_halving() {
    new_test_ext(1).execute_with(|| {
        let expected_emissions: [(u64, u64); 43] = [
            (0, 1_000_000_000), // Testing at zero issuance.
            (1_776_000, 1_000_000_000),
            (1_776_000_000, 1_000_000_000),
            (1_776_000_000_000, 1_000_000_000),
            (10_500_000_000_000_000, 500_000_000), // First halving event
            (10_999_999_000_000_000, 500_000_000),
            (11_000_000_000_000_000, 500_000_000),
            (12_000_999_000_000_000, 500_000_000),
            (15_749_999_000_000_000, 500_000_000),
            (15_800_000_000_000_000, 250_000_000), // Second halving event
            (16_400_999_000_000_000, 250_000_000),
            (16_499_999_000_000_000, 250_000_000),
            (17_624_999_000_000_000, 250_000_000),
            (18_400_000_000_000_000, 125_000_000), // Third halving event
            (19_312_500_000_000_000, 125_000_000),
            (19_700_000_000_000_000, 62_500_000), // Fourth halving event
            (19_906_249_000_000_000, 62_500_000),
            (20_400_000_000_000_000, 31_250_000), // Fifth halving event
            (20_500_000_000_000_000, 31_250_000),
            (20_700_000_000_000_000, 15_625_000), // Sixth halving event
            (20_800_000_000_000_000, 15_625_000),
            (20_900_000_000_000_000, 7_812_500), // Seventh halving event
            (20_917_970_000_000_000, 3_906_250), // Eighth halving event
            (20_958_985_000_000_000, 1_953_125), // Ninth halving event
            (20_979_493_000_000_000, 976_562),   // Tenth halving event
            (20_989_747_000_000_000, 488_281),   // Eleventh halving event
            (20_994_874_000_000_000, 244_140),   // Twelfth halving event
            (20_997_437_000_000_000, 122_070),   // Thirteenth halving event
            (20_998_719_000_000_000, 61_035),    // Fourteenth halving event
            (20_999_360_000_000_000, 30_517),    // Fifteenth halving event
            (20_999_680_000_000_000, 15_258),    // Sixteenth halving event
            (20_999_840_000_000_000, 7_629),     // Seventeenth halving event
            (20_999_920_000_000_000, 3_814),     // Eighteenth halving event
            (20_999_960_000_000_000, 1_907),     // Nineteenth halving event
            (20_999_980_000_000_000, 953),       // Twentieth halving event
            (20_999_990_000_000_000, 476),       // Twenty-first halving event
            (20_999_990_500_000_000, 476),
            (20_999_995_000_000_000, 238), // Twenty-second halving event
            (20_999_998_000_000_000, 119), // Twenty-third halving event
            (20_999_999_000_000_000, 59),  // Twenty-fourth halving event
            (21_000_000_000_000_000, 0),   // Total supply reached, emissions stop
            (21_100_000_000_000_000, 0),   // Just for fun
            (u64::MAX, 0),                 // Testing bounds
        ];

        for (issuance, expected_emission) in expected_emissions.iter() {
            SubtensorModule::set_total_issuance(*issuance);
            step_block(1);

            let current_emission = SubtensorModule::get_block_emission().unwrap();
            assert_eq!(
                current_emission, *expected_emission,
                "Incorrect emission {} at total issuance {}",
                current_emission, issuance
            );
        }
    });
}

#[test]
fn test_get_emission_across_entire_issuance_range() {
    new_test_ext(1).execute_with(|| {
        let total_supply: u64 = pallet_subtensor::TotalSupply::<Test>::get();
        let original_emission: u64 = pallet_subtensor::DefaultBlockEmission::<Test>::get();
        let halving_issuance: u64 = total_supply / 2;

        let mut issuance = 0;

        // Issuance won't reach total supply.
        while issuance <= 20_900_000_000_000_000 {
            SubtensorModule::set_total_issuance(issuance);

            let issuance_f64 = issuance as f64;
            let h = f64::log2(1.0 / (1.0 - issuance_f64 / (2.0 * halving_issuance as f64)));
            let h = h.floor();
            let emission_percentage = f64::powf(2.0, -h);

            let expected_emission: u64 = if issuance < total_supply {
                (original_emission as f64 * emission_percentage) as u64
            } else {
                0
            };
            assert_eq!(
                SubtensorModule::get_block_emission().unwrap(),
                expected_emission,
                "Issuance: {}",
                issuance_f64
            );

            issuance += expected_emission;
        }
    });
}

#[test]
fn test_dissolve_network_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 30;
        let hotkey = U256::from(1);

        add_network(netuid, 0, 0);
        let owner_coldkey = SubtensorModule::get_subnet_owner(netuid);
        register_ok_neuron(netuid, hotkey, owner_coldkey, 3);

        assert!(SubtensorModule::if_subnet_exist(netuid));
        assert_ok!(SubtensorModule::dissolve_network(
            RuntimeOrigin::signed(owner_coldkey),
            netuid
        ));
        assert!(!SubtensorModule::if_subnet_exist(netuid))
    });
}

#[test]
fn test_dissolve_network_refund_coldkey_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 30;
        let hotkey = U256::from(1);
        let subnet_locked_balance = 1000;

        add_network(netuid, 0, 0);
        let owner_coldkey = SubtensorModule::get_subnet_owner(netuid);
        register_ok_neuron(netuid, hotkey, owner_coldkey, 3);

        SubtensorModule::set_subnet_locked_balance(netuid, subnet_locked_balance);
        let coldkey_balance = SubtensorModule::get_coldkey_balance(&owner_coldkey);

        assert!(SubtensorModule::if_subnet_exist(netuid));
        assert_ok!(SubtensorModule::dissolve_network(
            RuntimeOrigin::signed(owner_coldkey),
            netuid
        ));
        assert!(!SubtensorModule::if_subnet_exist(netuid));

        let coldkey_new_balance = SubtensorModule::get_coldkey_balance(&owner_coldkey);

        assert!(coldkey_new_balance > coldkey_balance);
        assert_eq!(coldkey_new_balance, coldkey_balance + subnet_locked_balance);
    });
}

#[test]
fn test_dissolve_network_not_owner_err() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 30;
        let hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let random_coldkey = U256::from(3);

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, owner_coldkey, 3);

        assert_err!(
            SubtensorModule::dissolve_network(RuntimeOrigin::signed(random_coldkey), netuid),
            Error::<Test>::NotSubnetOwner
        );
    });
}

#[test]
fn test_dissolve_network_does_not_exist_err() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 30;
        let coldkey = U256::from(2);

        assert_err!(
            SubtensorModule::dissolve_network(RuntimeOrigin::signed(coldkey), netuid),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}
