#![allow(clippy::unwrap_used)]

mod mock;
use frame_support::{assert_ok, weights::Weight};
use frame_system::Config;
use mock::*;
use pallet_subtensor::*;
use sp_core::U256;

#[test]
fn test_migration_fix_total_stake_maps() {
    new_test_ext(1).execute_with(|| {
        let ck1 = U256::from(1);
        let ck2 = U256::from(2);
        let ck3 = U256::from(3);

        let hk1 = U256::from(1 + 100);
        let hk2 = U256::from(2 + 100);

        let mut total_stake_amount = 0;

        // Give each coldkey some stake in the maps
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&ck1, &hk1, 100);
        total_stake_amount += 100;

        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&ck2, &hk1, 10_101);
        total_stake_amount += 10_101;

        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&ck3, &hk2, 100_000_000);
        total_stake_amount += 100_000_000;

        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&ck1, &hk2, 1_123_000_000);
        total_stake_amount += 1_123_000_000;

        // Check that the total stake is correct
        assert_eq!(SubtensorModule::get_total_stake(), total_stake_amount);

        // Check that the total coldkey stake is correct
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&ck1),
            100 + 1_123_000_000
        );
        assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck2), 10_101);
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&ck3),
            100_000_000
        );

        // Check that the total hotkey stake is correct
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hk1),
            100 + 10_101
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hk2),
            100_000_000 + 1_123_000_000
        );

        // Mess up the total coldkey stake
        pallet_subtensor::TotalColdkeyStake::<Test>::insert(ck1, 0);
        // Verify that the total coldkey stake is now 0 for ck1
        assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck1), 0);

        // Mess up the total stake
        pallet_subtensor::TotalStake::<Test>::put(123_456_789);
        // Verify that the total stake is now wrong
        assert_ne!(SubtensorModule::get_total_stake(), total_stake_amount);

        // Run the migration to fix the total stake maps
        pallet_subtensor::migration::migrate_to_v2_fixed_total_stake::<Test>();

        // Verify that the total stake is now correct
        assert_eq!(SubtensorModule::get_total_stake(), total_stake_amount);
        // Verify that the total coldkey stake is now correct for each coldkey
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&ck1),
            100 + 1_123_000_000
        );
        assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck2), 10_101);
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&ck3),
            100_000_000
        );

        // Verify that the total hotkey stake is STILL correct for each hotkey
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hk1),
            100 + 10_101
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hk2),
            100_000_000 + 1_123_000_000
        );

        // Verify that the Stake map has no extra entries
        assert_eq!(pallet_subtensor::Stake::<Test>::iter().count(), 4); // 4 entries total
        assert_eq!(
            pallet_subtensor::Stake::<Test>::iter_key_prefix(hk1).count(),
            2
        ); // 2 stake entries for hk1
        assert_eq!(
            pallet_subtensor::Stake::<Test>::iter_key_prefix(hk2).count(),
            2
        ); // 2 stake entries for hk2
    })
}

#[test]
// To run this test with cargo, use the following command:
// cargo test --package pallet-subtensor --test migration test_migration5_total_issuance
fn test_migration5_total_issuance() {
    new_test_ext(1).execute_with(|| {
        // Run the migration to check total issuance.
        let test: bool = true;

        assert_eq!(SubtensorModule::get_total_issuance(), 0);
        pallet_subtensor::migration::migration5_total_issuance::<Test>(test);
        assert_eq!(SubtensorModule::get_total_issuance(), 0);

        SubtensorModule::add_balance_to_coldkey_account(&U256::from(1), 10000);
        assert_eq!(SubtensorModule::get_total_issuance(), 0);
        pallet_subtensor::migration::migration5_total_issuance::<Test>(test);
        assert_eq!(SubtensorModule::get_total_issuance(), 10000);

        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &U256::from(1),
            &U256::from(1),
            30000,
        );
        assert_eq!(SubtensorModule::get_total_issuance(), 10000);
        pallet_subtensor::migration::migration5_total_issuance::<Test>(test);
        assert_eq!(SubtensorModule::get_total_issuance(), 10000 + 30000);
    })
}

#[test]
// To run this test with cargo, use the following command:
// cargo test --package pallet-subtensor --test migration test_total_issuance_global
fn test_total_issuance_global() {
    new_test_ext(0).execute_with(|| {
        // Initialize network unique identifier and keys for testing.
        let netuid: u16 = 1; // Network unique identifier set to 1 for testing.
        let coldkey = U256::from(0); // Coldkey initialized to 0, representing an account's public key for non-transactional operations.
        let hotkey = U256::from(0); // Hotkey initialized to 0, representing an account's public key for transactional operations.
        let owner: U256 = U256::from(0);

        let lockcost: u64 = SubtensorModule::get_network_lock_cost();
        SubtensorModule::add_balance_to_coldkey_account(&owner, lockcost); // Add a balance of 20000 to the coldkey account.
        assert_eq!(SubtensorModule::get_total_issuance(), 0); // initial is zero.
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        SubtensorModule::set_max_allowed_uids(netuid, 1); // Set the maximum allowed unique identifiers for the network to 1.
        assert_eq!(SubtensorModule::get_total_issuance(), 0); // initial is zero.
        pallet_subtensor::migration::migration5_total_issuance::<Test>(true); // Pick up lock.
        assert_eq!(SubtensorModule::get_total_issuance(), lockcost); // Verify the total issuance is updated to 20000 after migration.
        assert!(SubtensorModule::if_subnet_exist(netuid));

        // Test the migration's effect on total issuance after adding balance to a coldkey account.
        let account_balance: u64 = 20000;
        let _hotkey_account_id_1 = U256::from(1); // Define a hotkey account ID for further operations.
        let _coldkey_account_id_1 = U256::from(1); // Define a coldkey account ID for further operations.
        assert_eq!(SubtensorModule::get_total_issuance(), lockcost); // Ensure the total issuance starts at 0 before the migration.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, account_balance); // Add a balance of 20000 to the coldkey account.
        pallet_subtensor::migration::migration5_total_issuance::<Test>(true); // Execute the migration to update total issuance.
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            account_balance + lockcost
        ); // Verify the total issuance is updated to 20000 after migration.

        // Test the effect of burning on total issuance.
        let burn_cost: u64 = 10000;
        SubtensorModule::set_burn(netuid, burn_cost); // Set the burn amount to 10000 for the network.
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            account_balance + lockcost
        ); // Confirm the total issuance remains 20000 before burning.
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey),
            netuid,
            hotkey
        )); // Execute the burn operation, reducing the total issuance.
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1); // Ensure the subnetwork count increases to 1 after burning
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            account_balance + lockcost - burn_cost
        ); // Verify the total issuance is reduced to 10000 after burning.
        pallet_subtensor::migration::migration5_total_issuance::<Test>(true); // Execute the migration to update total issuance.
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            account_balance + lockcost - burn_cost
        ); // Verify the total issuance is updated to 10000 nothing changes

        // Test staking functionality and its effect on total issuance.
        let new_stake: u64 = 10000;
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            account_balance + lockcost - burn_cost
        ); // Same
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, new_stake); // Stake an additional 10000 to the coldkey-hotkey account. This is i
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            account_balance + lockcost - burn_cost
        ); // Same
        pallet_subtensor::migration::migration5_total_issuance::<Test>(true); // Fix issuance
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            account_balance + lockcost - burn_cost + new_stake
        ); // New

        // Set emission values for the network and verify.
        let emission: u64 = 1_000_000_000;
        SubtensorModule::set_tempo(netuid, 1);
        SubtensorModule::set_emission_values(&[netuid], vec![emission]).unwrap(); // Set the emission value for the network to 1_000_000_000.
        assert_eq!(SubtensorModule::get_subnet_emission_value(netuid), emission); // Verify the emission value is set correctly for the network.
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            account_balance + lockcost - burn_cost + new_stake
        );
        run_to_block(2); // Advance to block number 2 to trigger the emission through the subnet.
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            account_balance + lockcost - burn_cost + new_stake + emission
        ); // Verify the total issuance reflects the staked amount and emission value that has been put through the epoch.
        pallet_subtensor::migration::migration5_total_issuance::<Test>(true); // Test migration does not change amount.
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            account_balance + lockcost - burn_cost + new_stake + emission
        ); // Verify the total issuance reflects the staked amount and emission value that has been put through the epoch.
    })
}

#[test]
fn test_migration_transfer_nets_to_foundation() {
    new_test_ext(1).execute_with(|| {
        // Create subnet 1
        add_network(1, 1, 0);
        // Create subnet 11
        add_network(11, 1, 0);

        log::info!("{:?}", SubtensorModule::get_subnet_owner(1));
        //assert_eq!(SubtensorModule::<T>::get_subnet_owner(1), );

        // Run the migration to transfer ownership
        let hex =
            hex_literal::hex!["feabaafee293d3b76dae304e2f9d885f77d2b17adab9e17e921b321eccd61c77"];
        pallet_subtensor::migration::migrate_transfer_ownership_to_foundation::<Test>(hex);

        log::info!("new owner: {:?}", SubtensorModule::get_subnet_owner(1));
    })
}

#[test]
fn test_migration_delete_subnet_3() {
    new_test_ext(1).execute_with(|| {
        // Create subnet 3
        add_network(3, 1, 0);
        assert!(SubtensorModule::if_subnet_exist(3));

        // Run the migration to transfer ownership
        pallet_subtensor::migration::migrate_delete_subnet_3::<Test>();

        assert!(!SubtensorModule::if_subnet_exist(3));
    })
}

#[test]
fn test_migration_delete_subnet_21() {
    new_test_ext(1).execute_with(|| {
        // Create subnet 21
        add_network(21, 1, 0);
        assert!(SubtensorModule::if_subnet_exist(21));

        // Run the migration to transfer ownership
        pallet_subtensor::migration::migrate_delete_subnet_21::<Test>();

        assert!(!SubtensorModule::if_subnet_exist(21));
    })
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test migration -- test_migrate_fix_total_coldkey_stake --exact --nocapture
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test migration -- test_migrate_fix_total_coldkey_stake --exact --nocapture
#[test]
fn test_migrate_fix_total_coldkey_stake() {
    new_test_ext(1).execute_with(|| {
        let migration_name = "fix_total_coldkey_stake_v7";
        let coldkey = U256::from(0);
        TotalColdkeyStake::<Test>::insert(coldkey, 0);
        StakingHotkeys::<Test>::insert(coldkey, vec![U256::from(1), U256::from(2), U256::from(3)]);
        Stake::<Test>::insert(U256::from(1), U256::from(0), 10000);
        Stake::<Test>::insert(U256::from(2), U256::from(0), 10000);
        Stake::<Test>::insert(U256::from(3), U256::from(0), 10000);

        let weight = run_migration_and_check(migration_name);
        assert!(weight != Weight::zero());
        assert_eq!(TotalColdkeyStake::<Test>::get(coldkey), 30000);

        let second_weight = run_migration_and_check(migration_name);
        assert_eq!(second_weight, Weight::zero());
        assert_eq!(TotalColdkeyStake::<Test>::get(coldkey), 30000);
    })
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test migration -- test_migrate_fix_total_coldkey_stake_value_already_in_total --exact --nocapture
#[test]
fn test_migrate_fix_total_coldkey_stake_value_already_in_total() {
    new_test_ext(1).execute_with(|| {
        let migration_name = "fix_total_coldkey_stake_v7";
        let coldkey = U256::from(0);
        TotalColdkeyStake::<Test>::insert(coldkey, 100000000);
        StakingHotkeys::<Test>::insert(coldkey, vec![U256::from(1), U256::from(2), U256::from(3)]);
        Stake::<Test>::insert(U256::from(1), U256::from(0), 10000);
        Stake::<Test>::insert(U256::from(2), U256::from(0), 10000);
        Stake::<Test>::insert(U256::from(3), U256::from(0), 10000);

        let weight = run_migration_and_check(migration_name);
        assert!(weight != Weight::zero());
        assert_eq!(TotalColdkeyStake::<Test>::get(coldkey), 30000);
    })
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test migration -- test_migrate_fix_total_coldkey_stake_no_entry --exact --nocapture
#[test]
fn test_migrate_fix_total_coldkey_stake_no_entry() {
    new_test_ext(1).execute_with(|| {
        let migration_name = "fix_total_coldkey_stake_v7";
        let coldkey = U256::from(0);
        StakingHotkeys::<Test>::insert(coldkey, vec![U256::from(1), U256::from(2), U256::from(3)]);
        Stake::<Test>::insert(U256::from(1), U256::from(0), 10000);
        Stake::<Test>::insert(U256::from(2), U256::from(0), 10000);
        Stake::<Test>::insert(U256::from(3), U256::from(0), 10000);

        let weight = run_migration_and_check(migration_name);
        assert!(weight != Weight::zero());
        assert_eq!(TotalColdkeyStake::<Test>::get(coldkey), 30000);
    })
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test migration -- test_migrate_fix_total_coldkey_stake_no_entry_in_hotkeys --exact --nocapture
#[test]
fn test_migrate_fix_total_coldkey_stake_no_entry_in_hotkeys() {
    new_test_ext(1).execute_with(|| {
        let migration_name = "fix_total_coldkey_stake_v7";
        let coldkey = U256::from(0);
        TotalColdkeyStake::<Test>::insert(coldkey, 100000000);
        StakingHotkeys::<Test>::insert(coldkey, vec![U256::from(1), U256::from(2), U256::from(3)]);

        let weight = run_migration_and_check(migration_name);
        assert!(weight != Weight::zero());
        assert_eq!(TotalColdkeyStake::<Test>::get(coldkey), 0);
    })
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test migration -- test_migrate_fix_total_coldkey_stake_one_hotkey_stake_missing --exact --nocapture
#[test]
fn test_migrate_fix_total_coldkey_stake_one_hotkey_stake_missing() {
    new_test_ext(1).execute_with(|| {
        let migration_name = "fix_total_coldkey_stake_v7";
        let coldkey = U256::from(0);
        TotalColdkeyStake::<Test>::insert(coldkey, 100000000);
        StakingHotkeys::<Test>::insert(coldkey, vec![U256::from(1), U256::from(2), U256::from(3)]);
        Stake::<Test>::insert(U256::from(1), U256::from(0), 10000);
        Stake::<Test>::insert(U256::from(2), U256::from(0), 10000);

        let weight = run_migration_and_check(migration_name);
        assert!(weight != Weight::zero());
        assert_eq!(TotalColdkeyStake::<Test>::get(coldkey), 20000);
    })
}

// New test to check if migration runs only once
#[test]
fn test_migrate_fix_total_coldkey_stake_runs_once() {
    new_test_ext(1).execute_with(|| {
        let migration_name = "fix_total_coldkey_stake_v7";
        let coldkey = U256::from(0);
        TotalColdkeyStake::<Test>::insert(coldkey, 0);
        StakingHotkeys::<Test>::insert(coldkey, vec![U256::from(1), U256::from(2), U256::from(3)]);
        Stake::<Test>::insert(U256::from(1), U256::from(0), 10000);
        Stake::<Test>::insert(U256::from(2), U256::from(0), 10000);
        Stake::<Test>::insert(U256::from(3), U256::from(0), 10000);

        // First run
        let first_weight = run_migration_and_check(migration_name);
        assert!(first_weight != Weight::zero());
        assert_eq!(TotalColdkeyStake::<Test>::get(coldkey), 30000);

        // Second run
        let second_weight = run_migration_and_check(migration_name);
        assert_eq!(second_weight, Weight::zero());
        assert_eq!(TotalColdkeyStake::<Test>::get(coldkey), 30000);
    })
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test migration -- test_migrate_fix_total_coldkey_stake_starts_with_value_no_stake_map_entries --exact --nocapture
#[test]
fn test_migrate_fix_total_coldkey_stake_starts_with_value_no_stake_map_entries() {
    new_test_ext(1).execute_with(|| {
        let migration_name = "fix_total_coldkey_stake_v7";
        let coldkey = U256::from(0);
        TotalColdkeyStake::<Test>::insert(coldkey, 123_456_789);

        // Notably, coldkey has no stake map or staking_hotkeys map entries

        let weight = run_migration_and_check(migration_name);
        assert!(weight != Weight::zero());
        // Therefore 0
        assert_eq!(TotalColdkeyStake::<Test>::get(coldkey), 0);
    })
}

fn run_migration_and_check(migration_name: &'static str) -> frame_support::weights::Weight {
    // Execute the migration and store its weight
    let weight: frame_support::weights::Weight =
        pallet_subtensor::migration::migrate_fix_total_coldkey_stake::<Test>();

    // Check if the migration has been marked as completed
    assert!(HasMigrationRun::<Test>::get(
        migration_name.as_bytes().to_vec()
    ));

    // Return the weight of the executed migration
    weight
}
