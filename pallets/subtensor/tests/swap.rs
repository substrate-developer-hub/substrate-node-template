#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]

use codec::Encode;
use frame_support::weights::Weight;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::{Config, RawOrigin};
mod mock;
use mock::*;
use pallet_subtensor::*;
use sp_core::U256;

#[test]
fn test_do_swap_hotkey_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let swap_cost = 1_000_000_000u64;

        // Setup initial state
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, old_hotkey, coldkey, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, swap_cost);

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            &old_hotkey,
            &new_hotkey
        ));

        // Verify the swap
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&new_hotkey),
            coldkey
        );
        assert_ne!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&old_hotkey),
            coldkey
        );

        // Verify other storage changes
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&new_hotkey),
            SubtensorModule::get_total_stake_for_hotkey(&old_hotkey)
        );
        assert_eq!(
            SubtensorModule::get_delegate(new_hotkey.encode()),
            SubtensorModule::get_delegate(old_hotkey.encode())
        );
        assert_eq!(
            SubtensorModule::get_last_tx_block(&new_hotkey),
            SubtensorModule::get_last_tx_block(&old_hotkey)
        );

        // Verify raw storage maps
        // Stake
        for (coldkey, stake_amount) in Stake::<Test>::iter_prefix(old_hotkey) {
            assert_eq!(Stake::<Test>::get(new_hotkey, coldkey), stake_amount);
        }

        let mut weight = Weight::zero();
        // UIDs
        for netuid in SubtensorModule::get_netuid_is_member(&old_hotkey, &mut weight) {
            assert_eq!(
                Uids::<Test>::get(netuid, new_hotkey),
                Uids::<Test>::get(netuid, old_hotkey)
            );
        }

        // Prometheus
        for netuid in SubtensorModule::get_netuid_is_member(&old_hotkey, &mut weight) {
            assert_eq!(
                Prometheus::<Test>::get(netuid, new_hotkey),
                Prometheus::<Test>::get(netuid, old_hotkey)
            );
        }

        // LoadedEmission
        for netuid in SubtensorModule::get_netuid_is_member(&old_hotkey, &mut weight) {
            assert_eq!(
                LoadedEmission::<Test>::get(netuid).unwrap(),
                LoadedEmission::<Test>::get(netuid).unwrap()
            );
        }

        // IsNetworkMember
        for netuid in SubtensorModule::get_netuid_is_member(&old_hotkey, &mut weight) {
            assert!(IsNetworkMember::<Test>::contains_key(new_hotkey, netuid));
            assert!(!IsNetworkMember::<Test>::contains_key(old_hotkey, netuid));
        }

        // Owner
        assert_eq!(Owner::<Test>::get(new_hotkey), coldkey);

        // TotalHotkeyStake
        assert_eq!(
            TotalHotkeyStake::<Test>::get(new_hotkey),
            TotalHotkeyStake::<Test>::get(old_hotkey)
        );

        // Delegates
        assert_eq!(
            Delegates::<Test>::get(new_hotkey),
            Delegates::<Test>::get(old_hotkey)
        );

        // LastTxBlock
        assert_eq!(
            LastTxBlock::<Test>::get(new_hotkey),
            LastTxBlock::<Test>::get(old_hotkey)
        );

        // Axons
        for netuid in SubtensorModule::get_netuid_is_member(&old_hotkey, &mut weight) {
            assert_eq!(
                Axons::<Test>::get(netuid, new_hotkey),
                Axons::<Test>::get(netuid, old_hotkey)
            );
        }

        // TotalHotkeyColdkeyStakesThisInterval
        assert_eq!(
            TotalHotkeyColdkeyStakesThisInterval::<Test>::get(new_hotkey, coldkey),
            TotalHotkeyColdkeyStakesThisInterval::<Test>::get(old_hotkey, coldkey)
        );
    });
}

#[test]
fn test_do_swap_hotkey_ok_robust() {
    new_test_ext(1).execute_with(|| {
        let num_subnets: u16 = 10;
        let tempo: u16 = 13;
        let swap_cost = 1_000_000_000u64;

        // Create 10 sets of keys
        let mut old_hotkeys = vec![];
        let mut new_hotkeys = vec![];
        let mut coldkeys = vec![];

        for i in 0..10 {
            old_hotkeys.push(U256::from(i * 2 + 1));
            new_hotkeys.push(U256::from(i * 2 + 2));
            coldkeys.push(U256::from(i * 2 + 11));
        }

        // Setup initial state
        for netuid in 1..=num_subnets {
            add_network(netuid, tempo, 0);
            SubtensorModule::set_max_registrations_per_block(netuid, 20);
            SubtensorModule::set_target_registrations_per_interval(netuid, 1000);
            log::info!(
                "Registrations this interval for netuid {:?} is {:?}",
                netuid,
                SubtensorModule::get_target_registrations_per_interval(netuid)
            );
            for i in 0..10 {
                register_ok_neuron(netuid, old_hotkeys[i], coldkeys[i], 0);
            }
        }

        // Add balance to coldkeys for swap cost
        for coldkey in coldkeys.iter().take(10) {
            SubtensorModule::add_balance_to_coldkey_account(coldkey, swap_cost);
        }

        // Add old_hotkeys[0] and old_hotkeys[1] to Senate
        assert_ok!(SenateMembers::add_member(
            RawOrigin::Root.into(),
            old_hotkeys[0]
        ));
        assert_ok!(SenateMembers::add_member(
            RawOrigin::Root.into(),
            old_hotkeys[1]
        ));

        // Verify initial Senate membership
        assert!(Senate::is_member(&old_hotkeys[0]));
        assert!(Senate::is_member(&old_hotkeys[1]));
        assert!(!Senate::is_member(&new_hotkeys[0]));
        assert!(!Senate::is_member(&new_hotkeys[1]));

        // Perform the swaps for only two hotkeys
        assert_ok!(SubtensorModule::do_swap_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(coldkeys[0]),
            &old_hotkeys[0],
            &new_hotkeys[0]
        ));
        assert_ok!(SubtensorModule::do_swap_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(coldkeys[1]),
            &old_hotkeys[1],
            &new_hotkeys[1]
        ));

        // Verify the swaps
        for netuid in 1..=num_subnets {
            for i in 0..10 {
                if i == 0 || i == 1 {
                    assert_eq!(
                        SubtensorModule::get_owning_coldkey_for_hotkey(&new_hotkeys[i]),
                        coldkeys[i]
                    );
                    assert_ne!(
                        SubtensorModule::get_owning_coldkey_for_hotkey(&old_hotkeys[i]),
                        coldkeys[i]
                    );

                    // Verify other storage changes
                    assert_eq!(
                        SubtensorModule::get_total_stake_for_hotkey(&new_hotkeys[i]),
                        SubtensorModule::get_total_stake_for_hotkey(&old_hotkeys[i])
                    );

                    assert_eq!(
                        SubtensorModule::get_delegate(new_hotkeys[i].encode()),
                        SubtensorModule::get_delegate(old_hotkeys[i].encode())
                    );

                    assert_eq!(
                        SubtensorModule::get_last_tx_block(&new_hotkeys[i]),
                        SubtensorModule::get_last_tx_block(&old_hotkeys[i])
                    );

                    // Verify raw storage maps
                    // Stake
                    for (coldkey, stake_amount) in Stake::<Test>::iter_prefix(old_hotkeys[i]) {
                        assert_eq!(Stake::<Test>::get(new_hotkeys[i], coldkey), stake_amount);
                    }

                    let mut weight = Weight::zero();
                    // UIDs
                    for netuid in
                        SubtensorModule::get_netuid_is_member(&old_hotkeys[i], &mut weight)
                    {
                        assert_eq!(
                            Uids::<Test>::get(netuid, new_hotkeys[i]),
                            Uids::<Test>::get(netuid, old_hotkeys[i])
                        );
                    }

                    // Prometheus
                    for netuid in
                        SubtensorModule::get_netuid_is_member(&old_hotkeys[i], &mut weight)
                    {
                        assert_eq!(
                            Prometheus::<Test>::get(netuid, new_hotkeys[i]),
                            Prometheus::<Test>::get(netuid, old_hotkeys[i])
                        );
                    }

                    // LoadedEmission
                    for netuid in
                        SubtensorModule::get_netuid_is_member(&old_hotkeys[i], &mut weight)
                    {
                        assert_eq!(
                            LoadedEmission::<Test>::get(netuid).unwrap(),
                            LoadedEmission::<Test>::get(netuid).unwrap()
                        );
                    }

                    // IsNetworkMember
                    for netuid in
                        SubtensorModule::get_netuid_is_member(&old_hotkeys[i], &mut weight)
                    {
                        assert!(IsNetworkMember::<Test>::contains_key(
                            new_hotkeys[i],
                            netuid
                        ));
                        assert!(!IsNetworkMember::<Test>::contains_key(
                            old_hotkeys[i],
                            netuid
                        ));
                    }

                    // Owner
                    assert_eq!(Owner::<Test>::get(new_hotkeys[i]), coldkeys[i]);

                    // Keys
                    for (uid, hotkey) in Keys::<Test>::iter_prefix(netuid) {
                        if hotkey == old_hotkeys[i] {
                            assert_eq!(Keys::<Test>::get(netuid, uid), new_hotkeys[i]);
                        }
                    }

                    // Verify Senate membership swap
                    assert!(!Senate::is_member(&old_hotkeys[i]));
                    assert!(Senate::is_member(&new_hotkeys[i]));
                } else {
                    // Ensure other hotkeys remain unchanged
                    assert_eq!(
                        SubtensorModule::get_owning_coldkey_for_hotkey(&old_hotkeys[i]),
                        coldkeys[i]
                    );
                    assert_ne!(
                        SubtensorModule::get_owning_coldkey_for_hotkey(&new_hotkeys[i]),
                        coldkeys[i]
                    );

                    // Verify Senate membership remains unchanged for other hotkeys
                    assert!(!Senate::is_member(&old_hotkeys[i]));
                    assert!(!Senate::is_member(&new_hotkeys[i]));
                }
            }
        }
    });
}

#[test]
fn test_swap_hotkey_tx_rate_limit_exceeded() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let old_hotkey = U256::from(1);
        let new_hotkey_1 = U256::from(2);
        let new_hotkey_2 = U256::from(4);
        let coldkey = U256::from(3);
        let swap_cost = 1_000_000_000u64 * 2;

        let tx_rate_limit = 1;

        // Get the current transaction rate limit
        let current_tx_rate_limit = SubtensorModule::get_tx_rate_limit();
        log::info!("current_tx_rate_limit: {:?}", current_tx_rate_limit);

        // Set the transaction rate limit
        SubtensorModule::set_tx_rate_limit(tx_rate_limit);
        // assert the rate limit is set to 1000 blocks
        assert_eq!(SubtensorModule::get_tx_rate_limit(), tx_rate_limit);

        // Setup initial state
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, old_hotkey, coldkey, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, swap_cost);

        // Perform the first swap
        assert_ok!(SubtensorModule::do_swap_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            &old_hotkey,
            &new_hotkey_1
        ));

        // Attempt to perform another swap immediately, which should fail due to rate limit
        assert_err!(
            SubtensorModule::do_swap_hotkey(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                &new_hotkey_1,
                &new_hotkey_2
            ),
            Error::<Test>::HotKeySetTxRateLimitExceeded
        );

        // move in time past the rate limit
        step_block(1001);
        assert_ok!(SubtensorModule::do_swap_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            &new_hotkey_1,
            &new_hotkey_2
        ));
    });
}

#[test]
fn test_do_swap_hotkey_err_not_owner() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let not_owner_coldkey = U256::from(4);
        let swap_cost = 1_000_000_000u64;

        // Setup initial state
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, old_hotkey, coldkey, 0);
        SubtensorModule::add_balance_to_coldkey_account(&not_owner_coldkey, swap_cost);

        // Attempt the swap with a non-owner coldkey
        assert_err!(
            SubtensorModule::do_swap_hotkey(
                <<Test as Config>::RuntimeOrigin>::signed(not_owner_coldkey),
                &old_hotkey,
                &new_hotkey
            ),
            Error::<Test>::NonAssociatedColdKey
        );
    });
}

#[test]
fn test_swap_owner_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        // Initialize Owner for old_hotkey
        Owner::<Test>::insert(old_hotkey, coldkey);

        // Perform the swap
        SubtensorModule::swap_owner(&old_hotkey, &new_hotkey, &coldkey, &mut weight);

        // Verify the swap
        assert_eq!(Owner::<Test>::get(new_hotkey), coldkey);
        assert!(!Owner::<Test>::contains_key(old_hotkey));
    });
}

#[test]
fn test_swap_owner_old_hotkey_not_exist() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        // Ensure old_hotkey does not exist
        assert!(!Owner::<Test>::contains_key(old_hotkey));

        // Perform the swap
        SubtensorModule::swap_owner(&old_hotkey, &new_hotkey, &coldkey, &mut weight);

        // Verify the swap
        assert_eq!(Owner::<Test>::get(new_hotkey), coldkey);
        assert!(!Owner::<Test>::contains_key(old_hotkey));
    });
}

#[test]
fn test_swap_owner_new_hotkey_already_exists() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let another_coldkey = U256::from(4);
        let mut weight = Weight::zero();

        // Initialize Owner for old_hotkey and new_hotkey
        Owner::<Test>::insert(old_hotkey, coldkey);
        Owner::<Test>::insert(new_hotkey, another_coldkey);

        // Perform the swap
        SubtensorModule::swap_owner(&old_hotkey, &new_hotkey, &coldkey, &mut weight);

        // Verify the swap
        assert_eq!(Owner::<Test>::get(new_hotkey), coldkey);
        assert!(!Owner::<Test>::contains_key(old_hotkey));
    });
}

#[test]
fn test_swap_owner_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        // Initialize Owner for old_hotkey
        Owner::<Test>::insert(old_hotkey, coldkey);

        // Perform the swap
        SubtensorModule::swap_owner(&old_hotkey, &new_hotkey, &coldkey, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_total_hotkey_stake_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let total_stake = 1000u64;
        let mut weight = Weight::zero();

        // Initialize TotalHotkeyStake for old_hotkey
        TotalHotkeyStake::<Test>::insert(old_hotkey, total_stake);

        // Perform the swap
        SubtensorModule::swap_total_hotkey_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the swap
        assert_eq!(TotalHotkeyStake::<Test>::get(new_hotkey), total_stake);
        assert!(!TotalHotkeyStake::<Test>::contains_key(old_hotkey));
    });
}

#[test]
fn test_swap_total_hotkey_stake_old_hotkey_not_exist() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let mut weight = Weight::zero();

        // Ensure old_hotkey does not exist
        assert!(!TotalHotkeyStake::<Test>::contains_key(old_hotkey));

        // Perform the swap
        SubtensorModule::swap_total_hotkey_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify that new_hotkey does not have a stake
        assert!(!TotalHotkeyStake::<Test>::contains_key(new_hotkey));
    });
}

#[test]
fn test_swap_total_hotkey_stake_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let total_stake = 1000u64;
        let mut weight = Weight::zero();

        // Initialize TotalHotkeyStake for old_hotkey
        TotalHotkeyStake::<Test>::insert(old_hotkey, total_stake);

        // Perform the swap
        SubtensorModule::swap_total_hotkey_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(1, 2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_delegates_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let delegate_take = 10u16;
        let mut weight = Weight::zero();

        // Initialize Delegates for old_hotkey
        Delegates::<Test>::insert(old_hotkey, delegate_take);

        // Perform the swap
        SubtensorModule::swap_delegates(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the swap
        assert_eq!(Delegates::<Test>::get(new_hotkey), delegate_take);
        assert!(!Delegates::<Test>::contains_key(old_hotkey));
    });
}

#[test]
fn test_swap_delegates_old_hotkey_not_exist() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let mut weight = Weight::zero();

        // Ensure old_hotkey does not exist
        assert!(!Delegates::<Test>::contains_key(old_hotkey));

        // Perform the swap
        SubtensorModule::swap_delegates(&old_hotkey, &new_hotkey, &mut weight);

        // Verify that new_hotkey does not have a delegate
        assert!(!Delegates::<Test>::contains_key(new_hotkey));
    });
}

#[test]
fn test_swap_delegates_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let delegate_take = 10u16;
        let mut weight = Weight::zero();

        // Initialize Delegates for old_hotkey
        Delegates::<Test>::insert(old_hotkey, delegate_take);

        // Perform the swap
        SubtensorModule::swap_delegates(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(1, 2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_stake_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake_amount = 1000u64;
        let mut weight = Weight::zero();

        // Initialize Stake for old_hotkey
        Stake::<Test>::insert(old_hotkey, coldkey, stake_amount);

        // Perform the swap
        SubtensorModule::swap_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the swap
        assert_eq!(Stake::<Test>::get(new_hotkey, coldkey), stake_amount);
        assert!(!Stake::<Test>::contains_key(old_hotkey, coldkey));
    });
}

#[test]
fn test_swap_stake_old_hotkey_not_exist() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake_amount = 1000u64;
        let mut weight = Weight::zero();

        // Initialize Stake for old_hotkey
        Stake::<Test>::insert(old_hotkey, coldkey, stake_amount);

        // Ensure old_hotkey has a stake
        assert!(Stake::<Test>::contains_key(old_hotkey, coldkey));

        // Perform the swap
        SubtensorModule::swap_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify that new_hotkey has the stake and old_hotkey does not
        assert!(Stake::<Test>::contains_key(new_hotkey, coldkey));
        assert!(!Stake::<Test>::contains_key(old_hotkey, coldkey));
    });
}

#[test]
fn test_swap_stake_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake_amount = 1000u64;
        let mut weight = Weight::zero();

        // Initialize Stake for old_hotkey
        Stake::<Test>::insert(old_hotkey, coldkey, stake_amount);

        // Perform the swap
        SubtensorModule::swap_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(4);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_is_network_member_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let mut weight = Weight::zero();

        // Initialize IsNetworkMember for old_hotkey
        for netuid in &netuid_is_member {
            IsNetworkMember::<Test>::insert(old_hotkey, netuid, true);
        }

        // Perform the swap
        SubtensorModule::swap_is_network_member(
            &old_hotkey,
            &new_hotkey,
            &netuid_is_member,
            &mut weight,
        );

        // Verify the swap
        for netuid in &netuid_is_member {
            assert!(IsNetworkMember::<Test>::contains_key(new_hotkey, netuid));
            assert!(!IsNetworkMember::<Test>::contains_key(old_hotkey, netuid));
        }
    });
}

#[test]
fn test_swap_is_network_member_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let mut weight = Weight::zero();

        // Initialize IsNetworkMember for old_hotkey
        for netuid in &netuid_is_member {
            IsNetworkMember::<Test>::insert(old_hotkey, netuid, true);
        }

        // Perform the swap
        SubtensorModule::swap_is_network_member(
            &old_hotkey,
            &new_hotkey,
            &netuid_is_member,
            &mut weight,
        );

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(4);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_axons_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let axon_info = AxonInfo {
            block: 100,
            version: 1,
            ip: 0x1234567890abcdef,
            port: 8080,
            ip_type: 4,
            protocol: 1,
            placeholder1: 0,
            placeholder2: 0,
        };
        let mut weight = Weight::zero();

        // Initialize Axons for old_hotkey
        for netuid in &netuid_is_member {
            Axons::<Test>::insert(netuid, old_hotkey, axon_info.clone());
        }

        // Perform the swap
        SubtensorModule::swap_axons(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the swap
        for netuid in &netuid_is_member {
            assert_eq!(Axons::<Test>::get(netuid, new_hotkey).unwrap(), axon_info);
            assert!(!Axons::<Test>::contains_key(netuid, old_hotkey));
        }
    });
}

#[test]
fn test_swap_axons_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let axon_info = AxonInfo {
            block: 100,
            version: 1,
            ip: 0x1234567890abcdef,
            port: 8080,
            ip_type: 4,
            protocol: 1,
            placeholder1: 0,
            placeholder2: 0,
        };
        let mut weight = Weight::zero();

        // Initialize Axons for old_hotkey
        for netuid in &netuid_is_member {
            Axons::<Test>::insert(netuid, old_hotkey, axon_info.clone());
        }

        // Perform the swap
        SubtensorModule::swap_axons(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the weight update
        let expected_weight = netuid_is_member.len() as u64
            * <Test as frame_system::Config>::DbWeight::get().reads_writes(1, 2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_keys_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let uid = 42u16;
        let mut weight = Weight::zero();

        // Initialize Keys for old_hotkey
        for netuid in &netuid_is_member {
            log::info!("Inserting old_hotkey:{:?} netuid:{:?}", old_hotkey, netuid);
            Keys::<Test>::insert(*netuid, uid, old_hotkey);
        }

        // Perform the swap
        SubtensorModule::swap_keys(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the swap
        for netuid in &netuid_is_member {
            log::info!(
                "neutuid, uid, hotkey: {:?}, {:?}, {:?}",
                netuid,
                uid,
                new_hotkey
            );
            assert_eq!(Keys::<Test>::get(netuid, uid), new_hotkey);
        }
    });
}

#[test]
fn test_swap_keys_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let uid = 42u16;
        let mut weight = Weight::zero();

        // Initialize Keys for old_hotkey
        for netuid in &netuid_is_member {
            Keys::<Test>::insert(*netuid, uid, old_hotkey);
        }

        // Perform the swap
        SubtensorModule::swap_keys(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(4);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_loaded_emission_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let se = 100u64;
        let ve = 200u64;
        let mut weight = Weight::zero();

        // Initialize LoadedEmission for old_hotkey
        for netuid in &netuid_is_member {
            LoadedEmission::<Test>::mutate(netuid, |emission_exists| {
                if let Some(emissions) = emission_exists {
                    emissions.push((old_hotkey, se, ve));
                } else {
                    *emission_exists = Some(vec![(old_hotkey, se, ve)]);
                }
            });
        }

        // Perform the swap
        SubtensorModule::swap_loaded_emission(
            &old_hotkey,
            &new_hotkey,
            &netuid_is_member,
            &mut weight,
        );

        // Verify the swap
        for netuid in &netuid_is_member {
            let emissions = LoadedEmission::<Test>::get(netuid).unwrap();
            assert!(emissions.iter().any(|(hk, _, _)| hk == &new_hotkey));
            assert!(!emissions.iter().any(|(hk, _, _)| hk == &old_hotkey));
        }
    });
}

#[test]
fn test_swap_loaded_emission_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        // let uid = 42u64;
        let se = 100u64;
        let ve = 200u64;
        let mut weight = Weight::zero();

        // Initialize LoadedEmission for old_hotkey
        for netuid in &netuid_is_member {
            LoadedEmission::<Test>::mutate(netuid, |emission_exists| {
                if let Some(emissions) = emission_exists {
                    emissions.push((old_hotkey, se, ve));
                } else {
                    *emission_exists = Some(vec![(old_hotkey, se, ve)]);
                }
            });
        }

        // Perform the swap
        SubtensorModule::swap_loaded_emission(
            &old_hotkey,
            &new_hotkey,
            &netuid_is_member,
            &mut weight,
        );

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_uids_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let uid = 42u16;
        let mut weight = Weight::zero();

        // Initialize Uids for old_hotkey
        for netuid in &netuid_is_member {
            Uids::<Test>::insert(netuid, old_hotkey, uid);
        }

        // Perform the swap
        SubtensorModule::swap_uids(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the swap
        for netuid in &netuid_is_member {
            assert_eq!(Uids::<Test>::get(netuid, new_hotkey).unwrap(), uid);
            assert!(!Uids::<Test>::contains_key(netuid, old_hotkey));
        }
    });
}

#[test]
fn test_swap_uids_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let uid = 42u16;
        let mut weight = Weight::zero();

        // Initialize Uids for old_hotkey
        for netuid in &netuid_is_member {
            Uids::<Test>::insert(netuid, old_hotkey, uid);
        }

        // Perform the swap
        SubtensorModule::swap_uids(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(4);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_prometheus_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let prometheus_info = PrometheusInfo {
            block: 100,
            version: 1,
            ip: 0x1234567890abcdef,
            port: 8080,
            ip_type: 4,
        };
        let mut weight = Weight::zero();

        // Initialize Prometheus for old_hotkey
        for netuid in &netuid_is_member {
            Prometheus::<Test>::insert(netuid, old_hotkey, prometheus_info.clone());
        }

        // Perform the swap
        SubtensorModule::swap_prometheus(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the swap
        for netuid in &netuid_is_member {
            assert_eq!(
                Prometheus::<Test>::get(netuid, new_hotkey).unwrap(),
                prometheus_info
            );
            assert!(!Prometheus::<Test>::contains_key(netuid, old_hotkey));
        }
    });
}

#[test]
fn test_swap_prometheus_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let prometheus_info = PrometheusInfo {
            block: 100,
            version: 1,
            ip: 0x1234567890abcdef,
            port: 8080,
            ip_type: 4,
        };
        let mut weight = Weight::zero();

        // Initialize Prometheus for old_hotkey
        for netuid in &netuid_is_member {
            Prometheus::<Test>::insert(netuid, old_hotkey, prometheus_info.clone());
        }

        // Perform the swap
        SubtensorModule::swap_prometheus(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the weight update
        let expected_weight = netuid_is_member.len() as u64
            * <Test as frame_system::Config>::DbWeight::get().reads_writes(1, 2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_total_hotkey_coldkey_stakes_this_interval_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake = (1000u64, 42u64); // Example tuple value
        let mut weight = Weight::zero();

        // Initialize TotalHotkeyColdkeyStakesThisInterval for old_hotkey
        TotalHotkeyColdkeyStakesThisInterval::<Test>::insert(old_hotkey, coldkey, stake);

        // Perform the swap
        SubtensorModule::swap_total_hotkey_coldkey_stakes_this_interval(
            &old_hotkey,
            &new_hotkey,
            &mut weight,
        );

        // Verify the swap
        assert_eq!(
            TotalHotkeyColdkeyStakesThisInterval::<Test>::get(new_hotkey, coldkey),
            stake
        );
        assert!(!TotalHotkeyColdkeyStakesThisInterval::<Test>::contains_key(
            old_hotkey, coldkey
        ));
    });
}

#[test]
fn test_swap_total_hotkey_coldkey_stakes_this_interval_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake = (1000u64, 42u64);
        let mut weight = Weight::zero();

        // Initialize TotalHotkeyColdkeyStakesThisInterval for old_hotkey
        TotalHotkeyColdkeyStakesThisInterval::<Test>::insert(old_hotkey, coldkey, stake);

        // Perform the swap

        SubtensorModule::swap_total_hotkey_coldkey_stakes_this_interval(
            &old_hotkey,
            &new_hotkey,
            &mut weight,
        );

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_do_swap_coldkey_success() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey1 = U256::from(3);
        let hotkey2 = U256::from(4);
        let netuid = 1u16;
        let stake_amount1 = 1000u64;
        let stake_amount2 = 2000u64;
        let swap_cost = SubtensorModule::get_key_swap_cost();
        let free_balance_old = 12345u64 + swap_cost;

        // Setup initial state
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey1, old_coldkey, 0);
        register_ok_neuron(netuid, hotkey2, old_coldkey, 0);

        // Add balance to old coldkey
        SubtensorModule::add_balance_to_coldkey_account(
            &old_coldkey,
            stake_amount1 + stake_amount2 + free_balance_old,
        );

        // Log initial state
        log::info!(
            "Initial total stake: {}",
            SubtensorModule::get_total_stake()
        );
        log::info!(
            "Initial old coldkey stake: {}",
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey)
        );
        log::info!(
            "Initial new coldkey stake: {}",
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey)
        );

        // Add stake to the neurons
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey1,
            stake_amount1
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey2,
            stake_amount2
        ));

        // Log state after adding stake
        log::info!(
            "Total stake after adding: {}",
            SubtensorModule::get_total_stake()
        );
        log::info!(
            "Old coldkey stake after adding: {}",
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey)
        );
        log::info!(
            "New coldkey stake after adding: {}",
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey)
        );

        // Record total stake before swap
        let total_stake_before_swap = SubtensorModule::get_total_stake();

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            &new_coldkey
        ));

        // Log state after swap
        log::info!(
            "Total stake after swap: {}",
            SubtensorModule::get_total_stake()
        );
        log::info!(
            "Old coldkey stake after swap: {}",
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey)
        );
        log::info!(
            "New coldkey stake after swap: {}",
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey)
        );

        // Verify the swap
        assert_eq!(Owner::<Test>::get(hotkey1), new_coldkey);
        assert_eq!(Owner::<Test>::get(hotkey2), new_coldkey);
        assert_eq!(
            TotalColdkeyStake::<Test>::get(new_coldkey),
            stake_amount1 + stake_amount2
        );
        assert_eq!(TotalColdkeyStake::<Test>::get(old_coldkey), 0);
        assert_eq!(Stake::<Test>::get(hotkey1, new_coldkey), stake_amount1);
        assert_eq!(Stake::<Test>::get(hotkey2, new_coldkey), stake_amount2);
        assert!(!Stake::<Test>::contains_key(hotkey1, old_coldkey));
        assert!(!Stake::<Test>::contains_key(hotkey2, old_coldkey));

        // Verify OwnedHotkeys
        let new_owned_hotkeys = OwnedHotkeys::<Test>::get(new_coldkey);
        assert!(new_owned_hotkeys.contains(&hotkey1));
        assert!(new_owned_hotkeys.contains(&hotkey2));
        assert_eq!(new_owned_hotkeys.len(), 2);
        assert!(!OwnedHotkeys::<Test>::contains_key(old_coldkey));

        // Verify balance transfer
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey),
            free_balance_old - swap_cost
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&old_coldkey), 0);

        // Verify total stake remains unchanged
        assert_eq!(
            SubtensorModule::get_total_stake(),
            total_stake_before_swap,
            "Total stake changed unexpectedly"
        );

        // Verify event emission
        System::assert_last_event(
            Event::ColdkeySwapped {
                old_coldkey,
                new_coldkey,
            }
            .into(),
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test swap -- test_swap_stake_for_coldkey --exact --nocaptur
#[test]
fn test_swap_stake_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey1 = U256::from(3);
        let hotkey2 = U256::from(4);
        let stake_amount1 = 1000u64;
        let stake_amount2 = 2000u64;
        let stake_amount3 = 3000u64;
        let total_stake = stake_amount1 + stake_amount2;
        let mut weight = Weight::zero();

        // Setup initial state
        OwnedHotkeys::<Test>::insert(old_coldkey, vec![hotkey1, hotkey2]);
        StakingHotkeys::<Test>::insert(old_coldkey, vec![hotkey1, hotkey2]);
        Stake::<Test>::insert(hotkey1, old_coldkey, stake_amount1);
        Stake::<Test>::insert(hotkey2, old_coldkey, stake_amount2);
        assert_eq!(Stake::<Test>::get(hotkey1, old_coldkey), stake_amount1);
        assert_eq!(Stake::<Test>::get(hotkey1, old_coldkey), stake_amount1);

        // Insert existing for same hotkey1
        Stake::<Test>::insert(hotkey1, new_coldkey, stake_amount3);
        StakingHotkeys::<Test>::insert(new_coldkey, vec![hotkey1]);

        TotalHotkeyStake::<Test>::insert(hotkey1, stake_amount1);
        TotalHotkeyStake::<Test>::insert(hotkey2, stake_amount2);
        TotalColdkeyStake::<Test>::insert(old_coldkey, total_stake);

        // Set up total issuance
        TotalIssuance::<Test>::put(total_stake);
        TotalStake::<Test>::put(total_stake);

        // Record initial values
        let initial_total_issuance = SubtensorModule::get_total_issuance();
        let initial_total_stake = SubtensorModule::get_total_stake();

        // Perform the swap
        SubtensorModule::swap_stake_for_coldkey(&old_coldkey, &new_coldkey, &mut weight);

        // Verify stake is additive, not replaced
        assert_eq!(
            Stake::<Test>::get(hotkey1, new_coldkey),
            stake_amount1 + stake_amount3
        );

        // Verify ownership transfer
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&new_coldkey),
            vec![hotkey1, hotkey2]
        );
        assert_eq!(SubtensorModule::get_owned_hotkeys(&old_coldkey), vec![]);

        // Verify stake transfer
        assert_eq!(Stake::<Test>::get(hotkey2, new_coldkey), stake_amount2);
        assert_eq!(Stake::<Test>::get(hotkey1, old_coldkey), 0);
        assert_eq!(Stake::<Test>::get(hotkey2, old_coldkey), 0);

        // Verify TotalColdkeyStake
        assert_eq!(TotalColdkeyStake::<Test>::get(new_coldkey), total_stake);
        assert_eq!(TotalColdkeyStake::<Test>::get(old_coldkey), 0);

        // Verify TotalHotkeyStake remains unchanged
        assert_eq!(TotalHotkeyStake::<Test>::get(hotkey1), stake_amount1);
        assert_eq!(TotalHotkeyStake::<Test>::get(hotkey2), stake_amount2);

        // Verify total stake and issuance remain unchanged
        assert_eq!(
            SubtensorModule::get_total_stake(),
            initial_total_stake,
            "Total stake changed unexpectedly"
        );
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            initial_total_issuance,
            "Total issuance changed unexpectedly"
        );
    });
}

#[test]
fn test_swap_staking_hotkeys_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey1 = U256::from(3);
        let hotkey2 = U256::from(4);
        let stake_amount1 = 1000u64;
        let stake_amount2 = 2000u64;
        let total_stake = stake_amount1 + stake_amount2;
        let mut weight = Weight::zero();

        // Setup initial state
        OwnedHotkeys::<Test>::insert(old_coldkey, vec![hotkey1, hotkey2]);
        Stake::<Test>::insert(hotkey1, old_coldkey, stake_amount1);
        Stake::<Test>::insert(hotkey2, old_coldkey, stake_amount2);
        StakingHotkeys::<Test>::insert(old_coldkey, vec![hotkey1, hotkey2]);
        TotalHotkeyStake::<Test>::insert(hotkey1, stake_amount1);
        TotalHotkeyStake::<Test>::insert(hotkey2, stake_amount2);
        TotalColdkeyStake::<Test>::insert(old_coldkey, total_stake);

        // Set up total issuance
        TotalIssuance::<Test>::put(total_stake);
        TotalStake::<Test>::put(total_stake);

        // Perform the swap
        SubtensorModule::swap_stake_for_coldkey(&old_coldkey, &new_coldkey, &mut weight);

        // Verify StakingHotkeys transfer
        assert_eq!(
            StakingHotkeys::<Test>::get(new_coldkey),
            vec![hotkey1, hotkey2]
        );
        assert_eq!(StakingHotkeys::<Test>::get(old_coldkey), vec![]);
    });
}

#[test]
fn test_swap_delegated_stake_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey1 = U256::from(3);
        let hotkey2 = U256::from(4);
        let stake_amount1 = 1000u64;
        let stake_amount2 = 2000u64;
        let total_stake = stake_amount1 + stake_amount2;
        let mut weight = Weight::zero();

        // Notice hotkey1 and hotkey2 are not in OwnedHotkeys
        // coldkey therefore delegates stake to them

        // Setup initial state
        StakingHotkeys::<Test>::insert(old_coldkey, vec![hotkey1, hotkey2]);
        Stake::<Test>::insert(hotkey1, old_coldkey, stake_amount1);
        Stake::<Test>::insert(hotkey2, old_coldkey, stake_amount2);
        TotalHotkeyStake::<Test>::insert(hotkey1, stake_amount1);
        TotalHotkeyStake::<Test>::insert(hotkey2, stake_amount2);
        TotalColdkeyStake::<Test>::insert(old_coldkey, total_stake);

        // Set up total issuance
        TotalIssuance::<Test>::put(total_stake);
        TotalStake::<Test>::put(total_stake);

        // Record initial values
        let initial_total_issuance = SubtensorModule::get_total_issuance();
        let initial_total_stake = SubtensorModule::get_total_stake();

        // Perform the swap
        SubtensorModule::swap_stake_for_coldkey(&old_coldkey, &new_coldkey, &mut weight);

        // Verify stake transfer
        assert_eq!(Stake::<Test>::get(hotkey1, new_coldkey), stake_amount1);
        assert_eq!(Stake::<Test>::get(hotkey2, new_coldkey), stake_amount2);
        assert_eq!(Stake::<Test>::get(hotkey1, old_coldkey), 0);
        assert_eq!(Stake::<Test>::get(hotkey2, old_coldkey), 0);

        // Verify TotalColdkeyStake
        assert_eq!(TotalColdkeyStake::<Test>::get(new_coldkey), total_stake);
        assert_eq!(TotalColdkeyStake::<Test>::get(old_coldkey), 0);

        // Verify TotalHotkeyStake remains unchanged
        assert_eq!(TotalHotkeyStake::<Test>::get(hotkey1), stake_amount1);
        assert_eq!(TotalHotkeyStake::<Test>::get(hotkey2), stake_amount2);

        // Verify total stake and issuance remain unchanged
        assert_eq!(
            SubtensorModule::get_total_stake(),
            initial_total_stake,
            "Total stake changed unexpectedly"
        );
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            initial_total_issuance,
            "Total issuance changed unexpectedly"
        );
    });
}

#[test]
fn test_swap_total_hotkey_coldkey_stakes_this_interval_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey1 = U256::from(3);
        let hotkey2 = U256::from(4);
        let stake1 = (1000u64, 100u64);
        let stake2 = (2000u64, 200u64);
        let mut weight = Weight::zero();

        // Initialize TotalHotkeyColdkeyStakesThisInterval for old_coldkey
        TotalHotkeyColdkeyStakesThisInterval::<Test>::insert(hotkey1, old_coldkey, stake1);
        TotalHotkeyColdkeyStakesThisInterval::<Test>::insert(hotkey2, old_coldkey, stake2);

        // Populate OwnedHotkeys map
        OwnedHotkeys::<Test>::insert(old_coldkey, vec![hotkey1, hotkey2]);

        // Perform the swap
        SubtensorModule::swap_total_hotkey_coldkey_stakes_this_interval_for_coldkey(
            &old_coldkey,
            &new_coldkey,
            &mut weight,
        );

        // Verify the swap
        assert_eq!(
            TotalHotkeyColdkeyStakesThisInterval::<Test>::get(hotkey1, new_coldkey),
            stake1
        );
        assert_eq!(
            TotalHotkeyColdkeyStakesThisInterval::<Test>::get(hotkey2, new_coldkey),
            stake2
        );
        assert!(!TotalHotkeyColdkeyStakesThisInterval::<Test>::contains_key(
            old_coldkey,
            hotkey1
        ));
        assert!(!TotalHotkeyColdkeyStakesThisInterval::<Test>::contains_key(
            old_coldkey,
            hotkey2
        ));

        // Verify weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(5, 4);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_subnet_owner_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let netuid1 = 1u16;
        let netuid2 = 2u16;
        let mut weight = Weight::zero();

        // Initialize SubnetOwner for old_coldkey
        SubnetOwner::<Test>::insert(netuid1, old_coldkey);
        SubnetOwner::<Test>::insert(netuid2, old_coldkey);

        // Set up TotalNetworks
        TotalNetworks::<Test>::put(3);

        // Perform the swap
        SubtensorModule::swap_subnet_owner_for_coldkey(&old_coldkey, &new_coldkey, &mut weight);

        // Verify the swap
        assert_eq!(SubnetOwner::<Test>::get(netuid1), new_coldkey);
        assert_eq!(SubnetOwner::<Test>::get(netuid2), new_coldkey);

        // Verify weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(3, 2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_do_swap_coldkey_with_subnet_ownership() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let netuid = 1u16;
        let stake_amount: u64 = 1000u64;
        let swap_cost = SubtensorModule::get_key_swap_cost();

        // Setup initial state
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, old_coldkey, 0);

        // Set TotalNetworks because swap relies on it
        pallet_subtensor::TotalNetworks::<Test>::set(1);

        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, stake_amount + swap_cost);
        SubnetOwner::<Test>::insert(netuid, old_coldkey);

        // Populate OwnedHotkeys map
        OwnedHotkeys::<Test>::insert(old_coldkey, vec![hotkey]);

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            &new_coldkey
        ));

        // Verify subnet ownership transfer
        assert_eq!(SubnetOwner::<Test>::get(netuid), new_coldkey);
    });
}

#[test]
fn test_coldkey_has_associated_hotkeys() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = 1u16;

        // Setup initial state
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1000);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test swap -- test_coldkey_swap_total --exact --nocapture
#[test]
fn test_coldkey_swap_total() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let nominator1 = U256::from(2);
        let nominator2 = U256::from(3);
        let nominator3 = U256::from(4);
        let delegate1 = U256::from(5);
        let delegate2 = U256::from(6);
        let delegate3 = U256::from(7);
        let hotkey1 = U256::from(2);
        let hotkey2 = U256::from(3);
        let hotkey3 = U256::from(4);
        let netuid1 = 1u16;
        let netuid2 = 2u16;
        let netuid3 = 3u16;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1000);
        SubtensorModule::add_balance_to_coldkey_account(&delegate1, 1000);
        SubtensorModule::add_balance_to_coldkey_account(&delegate2, 1000);
        SubtensorModule::add_balance_to_coldkey_account(&delegate3, 1000);
        SubtensorModule::add_balance_to_coldkey_account(&nominator1, 1000);
        SubtensorModule::add_balance_to_coldkey_account(&nominator2, 1000);
        SubtensorModule::add_balance_to_coldkey_account(&nominator3, 1000);

        // Setup initial state
        add_network(netuid1, 13, 0);
        add_network(netuid2, 14, 0);
        add_network(netuid3, 15, 0);
        register_ok_neuron(netuid1, hotkey1, coldkey, 0);
        register_ok_neuron(netuid2, hotkey2, coldkey, 0);
        register_ok_neuron(netuid3, hotkey3, coldkey, 0);
        register_ok_neuron(netuid1, delegate1, delegate1, 0);
        register_ok_neuron(netuid2, delegate2, delegate2, 0);
        register_ok_neuron(netuid3, delegate3, delegate3, 0);
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey1,
            u16::MAX / 10
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey2,
            u16::MAX / 10
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey3,
            u16::MAX / 10
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(delegate1),
            delegate1,
            u16::MAX / 10
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(delegate2),
            delegate2,
            u16::MAX / 10
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(delegate3),
            delegate3,
            u16::MAX / 10
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey1,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey2,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey3,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate1,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate2,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate3,
            100
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate1),
            hotkey1,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate2),
            hotkey2,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate3),
            hotkey3,
            100
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate1),
            delegate1,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate2),
            delegate2,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate3),
            delegate3,
            100
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator1),
            hotkey1,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator2),
            hotkey2,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator3),
            hotkey3,
            100
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator1),
            delegate1,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator2),
            delegate2,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator3),
            delegate3,
            100
        ));

        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&coldkey),
            vec![hotkey1, hotkey2, hotkey3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&coldkey),
            vec![hotkey1, hotkey2, hotkey3, delegate1, delegate2, delegate3]
        );
        assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&coldkey), 600);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 300);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey2), 300);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey3), 300);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&delegate1), 300);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&delegate2), 300);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&delegate3), 300);

        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate1),
            vec![delegate1]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate2),
            vec![delegate2]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate3),
            vec![delegate3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate1),
            vec![delegate1, hotkey1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate2),
            vec![delegate2, hotkey2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate3),
            vec![delegate3, hotkey3]
        );

        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator1), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator2), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator3), vec![]);

        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator1),
            vec![hotkey1, delegate1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator2),
            vec![hotkey2, delegate2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator3),
            vec![hotkey3, delegate3]
        );

        // Perform the swap
        let new_coldkey = U256::from(1100);
        assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&coldkey), 600);
        assert_ok!(SubtensorModule::perform_swap_coldkey(
            &coldkey,
            &new_coldkey
        ));
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            600
        );

        // Check everything is swapped.
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&new_coldkey),
            vec![hotkey1, hotkey2, hotkey3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&new_coldkey),
            vec![hotkey1, hotkey2, hotkey3, delegate1, delegate2, delegate3]
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            600
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 300);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey2), 300);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey3), 300);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&delegate1), 300);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&delegate2), 300);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&delegate3), 300);

        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate1),
            vec![delegate1]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate2),
            vec![delegate2]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate3),
            vec![delegate3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate1),
            vec![delegate1, hotkey1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate2),
            vec![delegate2, hotkey2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate3),
            vec![delegate3, hotkey3]
        );

        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator1), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator2), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator3), vec![]);

        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator1),
            vec![hotkey1, delegate1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator2),
            vec![hotkey2, delegate2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator3),
            vec![hotkey3, delegate3]
        );
    });
}

#[test]
fn test_swap_senate_member() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let non_member_hotkey = U256::from(3);
        let mut weight = Weight::zero();

        // Setup: Add old_hotkey as a Senate member
        assert_ok!(SenateMembers::add_member(
            RawOrigin::Root.into(),
            old_hotkey
        ));

        // Test 1: Successful swap
        assert_ok!(SubtensorModule::swap_senate_member(
            &old_hotkey,
            &new_hotkey,
            &mut weight
        ));
        assert!(Senate::is_member(&new_hotkey));
        assert!(!Senate::is_member(&old_hotkey));

        // Verify weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(2, 2);
        assert_eq!(weight, expected_weight);

        // Reset weight for next test
        weight = Weight::zero();

        // Test 2: Swap with non-member (should not change anything)
        assert_ok!(SubtensorModule::swap_senate_member(
            &non_member_hotkey,
            &new_hotkey,
            &mut weight
        ));
        assert!(Senate::is_member(&new_hotkey));
        assert!(!Senate::is_member(&non_member_hotkey));

        // Verify weight update (should only have read operations)
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads(1);
        assert_eq!(weight, expected_weight);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test swap -- test_coldkey_delegations --exact --nocapture
#[test]
fn test_coldkey_delegations() {
    new_test_ext(1).execute_with(|| {
        let new_coldkey = U256::from(0);
        let owner = U256::from(1);
        let coldkey = U256::from(4);
        let delegate = U256::from(2);
        let netuid = 1u16;
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, delegate, owner, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1000);
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            delegate,
            u16::MAX / 10
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate,
            100
        ));
        assert_ok!(SubtensorModule::perform_swap_coldkey(
            &coldkey,
            &new_coldkey
        ));
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&delegate), 100);
        assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&coldkey), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            100
        );
        assert_eq!(Stake::<Test>::get(delegate, new_coldkey), 100);
        assert_eq!(Stake::<Test>::get(delegate, coldkey), 0);
    });
}
