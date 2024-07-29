#![allow(clippy::unwrap_used)]

mod mock;
use frame_support::assert_ok;
use frame_system::Config;
use mock::*;
use sp_core::U256;

#[test]
fn test_loaded_emission() {
    new_test_ext(1).execute_with(|| {
        let n: u16 = 100;
        let netuid: u16 = 1;
        let tempo: u16 = 10;
        let netuids: Vec<u16> = vec![1];
        let emission: Vec<u64> = vec![1000000000];
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_emission_values(&netuids, emission).unwrap();
        for i in 0..n {
            SubtensorModule::append_neuron(netuid, &U256::from(i), 0);
        }
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid).is_none());

        // Try loading at block 0
        let block: u64 = 0;
        assert_eq!(
            SubtensorModule::blocks_until_next_epoch(netuid, tempo, block),
            8
        );
        SubtensorModule::generate_emission(block);
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid).is_none());

        // Try loading at block = 9;
        let block: u64 = 8;
        assert_eq!(
            SubtensorModule::blocks_until_next_epoch(netuid, tempo, block),
            0
        );
        SubtensorModule::generate_emission(block);
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid).is_some());
        assert_eq!(
            SubtensorModule::get_loaded_emission_tuples(netuid)
                .unwrap()
                .len(),
            n as usize
        );

        // Try draining the emission tuples
        // None remaining because we are at epoch.
        let block: u64 = 8;
        SubtensorModule::drain_emission(block);
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid).is_none());

        // Generate more emission.
        SubtensorModule::generate_emission(8);
        assert_eq!(
            SubtensorModule::get_loaded_emission_tuples(netuid)
                .unwrap()
                .len(),
            n as usize
        );

        for block in 9..19 {
            let mut n_remaining: usize = 0;
            let mut n_to_drain: usize = 0;
            if let Some(tuples) = SubtensorModule::get_loaded_emission_tuples(netuid) {
                n_remaining = tuples.len();
                n_to_drain =
                    SubtensorModule::tuples_to_drain_this_block(netuid, tempo, block, tuples.len());
            }
            SubtensorModule::drain_emission(block); // drain it with 9 more blocks to go
            if let Some(tuples) = SubtensorModule::get_loaded_emission_tuples(netuid) {
                assert_eq!(tuples.len(), n_remaining - n_to_drain);
            }
            log::info!("n_to_drain: {:?}", n_to_drain);
            log::info!(
                "SubtensorModule::get_loaded_emission_tuples( netuid ).len(): {:?}",
                n_remaining - n_to_drain
            );
        }
    })
}

#[test]
fn test_tuples_to_drain_this_block() {
    new_test_ext(1).execute_with(|| {
        // pub fn tuples_to_drain_this_block( netuid: u16, tempo: u16, block_number: u64, n_remaining: usize ) -> usize {
        assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 1, 0, 10), 10); // drain all epoch block.
        assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 0, 0, 10), 10); // drain all no tempo.
        assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 10, 0, 10), 2); // drain 10 / ( 10 / 2 ) = 2
        assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 20, 0, 10), 1); // drain 10 / ( 20 / 2 ) = 1
        assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 10, 0, 20), 5); // drain 20 / ( 9 / 2 ) = 5
        assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 20, 0, 0), 0); // nothing to drain.
        assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 10, 1, 20), 5); // drain 19 / ( 10 / 2 ) = 4
        assert_eq!(
            SubtensorModule::tuples_to_drain_this_block(0, 10, 10, 20),
            4
        ); // drain 19 / ( 10 / 2 ) = 4
        assert_eq!(
            SubtensorModule::tuples_to_drain_this_block(0, 10, 15, 20),
            10
        ); // drain 19 / ( 10 / 2 ) = 4
        assert_eq!(
            SubtensorModule::tuples_to_drain_this_block(0, 10, 19, 20),
            20
        ); // drain 19 / ( 10 / 2 ) = 4
        assert_eq!(
            SubtensorModule::tuples_to_drain_this_block(0, 10, 20, 20),
            20
        ); // drain 19 / ( 10 / 2 ) = 4
        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    for l in 0..10 {
                        assert!(SubtensorModule::tuples_to_drain_this_block(i, j, k, l) <= 10);
                    }
                }
            }
        }
    })
}

#[test]
fn test_blocks_until_epoch() {
    new_test_ext(1).execute_with(|| {
        // Check tempo = 0 block = * netuid = *
        assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 0, 0), 1000);

        // Check tempo = 1 block = * netuid = *
        assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 1, 0), 0);
        assert_eq!(SubtensorModule::blocks_until_next_epoch(1, 1, 0), 1);
        assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 1, 1), 1);
        assert_eq!(SubtensorModule::blocks_until_next_epoch(1, 1, 1), 0);
        assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 1, 2), 0);
        assert_eq!(SubtensorModule::blocks_until_next_epoch(1, 1, 2), 1);
        for i in 0..100 {
            if i % 2 == 0 {
                assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 1, i), 0);
                assert_eq!(SubtensorModule::blocks_until_next_epoch(1, 1, i), 1);
            } else {
                assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 1, i), 1);
                assert_eq!(SubtensorModule::blocks_until_next_epoch(1, 1, i), 0);
            }
        }

        // Check general case.
        for netuid in 0..30_u16 {
            for block in 0..30_u64 {
                for tempo in 1..30_u16 {
                    assert_eq!(
                        SubtensorModule::blocks_until_next_epoch(netuid, tempo, block),
                        tempo as u64 - (block + netuid as u64 + 1) % (tempo as u64 + 1)
                    );
                }
            }
        }
    });
}

// /********************************************
//     block_step::adjust_registration_terms_for_networks tests
// *********************************************/
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
        assert_eq!(
            SubtensorModule::get_adjustment_interval(netuid),
            adjustment_interval
        ); // Sanity check the adjustment interval.

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
fn test_burn_adjustment_with_moving_average() {
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
        // Set alpha here.
        SubtensorModule::set_adjustment_alpha(netuid, u64::MAX / 2);

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
        // 0.5 * 1000 + 0.5 * 1500 = 1250
        assert_eq!(SubtensorModule::get_burn_as_u64(netuid), 1250);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_a() {
    // Test case A of the difficulty and burn adjustment algorithm.
    // ====================
    // There are too many registrations this interval and most of them are pow registrations
    // this triggers an increase in the pow difficulty.
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 1;
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_min_difficulty(netuid, start_diff);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1. This is a burn registration.
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            hotkey_account_id_1
        ));

        // Register key 2. This is a POW registration
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            0,
            &hotkey_account_id_2,
        );
        let result0 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            curr_block_num,
            nonce0,
            work0,
            hotkey_account_id_2,
            coldkey_account_id_2,
        );
        assert_ok!(result0);

        // Register key 3. This is a POW registration
        let hotkey_account_id_3 = U256::from(3);
        let coldkey_account_id_3 = U256::from(3);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            11231312312,
            &hotkey_account_id_3,
        );
        let result1 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
            netuid,
            curr_block_num,
            nonce1,
            work1,
            hotkey_account_id_3,
            coldkey_account_id_3,
        );
        assert_ok!(result1);

        // We are over the number of regs allowed this interval.
        // Most of them are POW registrations (2 out of 3)
        // Step the block and trigger the adjustment.
        step_block(1);
        curr_block_num += 1;

        // Check the adjusted POW difficulty has INCREASED.
        //   and the burn has not changed.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert_eq!(adjusted_burn, burn_cost);

        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert!(adjusted_diff > start_diff);
        assert_eq!(adjusted_diff, 20_000);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_b() {
    // Test case B of the difficulty and burn adjustment algorithm.
    // ====================
    // There are too many registrations this interval and most of them are burn registrations
    // this triggers an increase in the burn cost.
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 1;
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
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

        // Register key 3. This one is a POW registration
        let hotkey_account_id_3 = U256::from(3);
        let coldkey_account_id_3 = U256::from(3);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            0,
            &hotkey_account_id_3,
        );
        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
            netuid,
            curr_block_num,
            nonce,
            work,
            hotkey_account_id_3,
            coldkey_account_id_3,
        );
        assert_ok!(result);

        // We are over the number of regs allowed this interval.
        // Most of them are burn registrations (2 out of 3)
        // Step the block and trigger the adjustment.
        step_block(1);
        curr_block_num += 1;

        // Check the adjusted burn has INCREASED.
        //   and the difficulty has not changed.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert!(adjusted_burn > burn_cost);
        assert_eq!(adjusted_burn, 2_000);

        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert_eq!(adjusted_diff, start_diff);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_c() {
    // Test case C of the difficulty and burn adjustment algorithm.
    // ====================
    // There are not enough registrations this interval and most of them are POW registrations
    // this triggers a decrease in the burn cost
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 4; // Needs registrations < 4 to trigger
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1. This is a BURN registration
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            hotkey_account_id_1
        ));

        // Register key 2. This is a POW registration
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            0,
            &hotkey_account_id_2,
        );
        let result0 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            curr_block_num,
            nonce0,
            work0,
            hotkey_account_id_2,
            coldkey_account_id_2,
        );
        assert_ok!(result0);

        // Register key 3. This is a POW registration
        let hotkey_account_id_3 = U256::from(3);
        let coldkey_account_id_3 = U256::from(3);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            11231312312,
            &hotkey_account_id_3,
        );
        let result1 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
            netuid,
            curr_block_num,
            nonce1,
            work1,
            hotkey_account_id_3,
            coldkey_account_id_3,
        );
        assert_ok!(result1);

        // We are UNDER the number of regs allowed this interval.
        // Most of them are POW registrations (2 out of 3)
        // Step the block and trigger the adjustment.
        step_block(1);
        curr_block_num += 1;

        // Check the adjusted burn has DECREASED.
        //   and the difficulty has not changed.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert!(adjusted_burn < burn_cost);
        assert_eq!(adjusted_burn, 875);

        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert_eq!(adjusted_diff, start_diff);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_d() {
    // Test case D of the difficulty and burn adjustment algorithm.
    // ====================
    // There are not enough registrations this interval and most of them are BURN registrations
    // this triggers a decrease in the POW difficulty
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 4; // Needs registrations < 4 to trigger
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_min_difficulty(netuid, 1);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1. This is a BURN registration
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            hotkey_account_id_1
        ));

        // Register key 2. This is a BURN registration
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            hotkey_account_id_2
        ));

        // Register key 3. This is a POW registration
        let hotkey_account_id_3 = U256::from(3);
        let coldkey_account_id_3 = U256::from(3);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            11231312312,
            &hotkey_account_id_3,
        );
        let result1 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
            netuid,
            curr_block_num,
            nonce1,
            work1,
            hotkey_account_id_3,
            coldkey_account_id_3,
        );
        assert_ok!(result1);

        // We are UNDER the number of regs allowed this interval.
        // Most of them are BURN registrations (2 out of 3)
        // Step the block and trigger the adjustment.
        step_block(1);
        curr_block_num += 1;

        // Check the adjusted POW difficulty has DECREASED.
        //   and the burn has not changed.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert_eq!(adjusted_burn, burn_cost);

        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert!(adjusted_diff < start_diff);
        assert_eq!(adjusted_diff, 8750);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_e() {
    // Test case E of the difficulty and burn adjustment algorithm.
    // ====================
    // There are not enough registrations this interval and nobody registered either POW or BURN
    // this triggers a decrease in the BURN cost and POW difficulty
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval: u16 = 3;
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_min_difficulty(netuid, 1);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1. This is a POW registration
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            11231312312,
            &hotkey_account_id_1,
        );
        let result1 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            curr_block_num,
            nonce1,
            work1,
            hotkey_account_id_1,
            coldkey_account_id_1,
        );
        assert_ok!(result1);

        // Register key 2. This is a BURN registration
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            hotkey_account_id_2
        ));

        step_block(1);
        curr_block_num += 1;

        // We are UNDER the number of regs allowed this interval.
        // And the number of regs of each type is equal

        // Check the adjusted BURN has DECREASED.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert!(adjusted_burn < burn_cost);
        assert_eq!(adjusted_burn, 833);

        // Check the adjusted POW difficulty has DECREASED.
        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert!(adjusted_diff < start_diff);
        assert_eq!(adjusted_diff, 8_333);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_f() {
    // Test case F of the difficulty and burn adjustment algorithm.
    // ====================
    // There are too many registrations this interval and the pow and burn registrations are equal
    // this triggers an increase in the burn cost and pow difficulty
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval: u16 = 1;
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_min_difficulty(netuid, start_diff);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1. This is a POW registration
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            11231312312,
            &hotkey_account_id_1,
        );
        let result1 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            curr_block_num,
            nonce1,
            work1,
            hotkey_account_id_1,
            coldkey_account_id_1,
        );
        assert_ok!(result1);

        // Register key 2. This is a BURN registration
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            hotkey_account_id_2
        ));

        step_block(1);
        curr_block_num += 1;
        // We are OVER the number of regs allowed this interval.
        // And the number of regs of each type is equal

        // Check the adjusted BURN has INCREASED.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert!(adjusted_burn > burn_cost);
        assert_eq!(adjusted_burn, 1_500);

        // Check the adjusted POW difficulty has INCREASED.
        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert!(adjusted_diff > start_diff);
        assert_eq!(adjusted_diff, 15_000);
    });
}

#[test]
fn test_burn_adjustment_case_e_zero_registrations() {
    // Test case E of the difficulty and burn adjustment algorithm.
    // ====================
    // There are not enough registrations this interval and nobody registered either POW or BURN
    // this triggers a decrease in the BURN cost and POW difficulty

    // BUT there are zero registrations this interval.
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval: u16 = 1;
        let start_diff: u64 = 10_000;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_min_difficulty(netuid, 1);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // No registrations this interval of any kind.
        step_block(1);

        // We are UNDER the number of regs allowed this interval.
        // And the number of regs of each type is equal

        // Check the adjusted BURN has DECREASED.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert!(adjusted_burn < burn_cost);
        assert_eq!(adjusted_burn, 500);

        // Check the adjusted POW difficulty has DECREASED.
        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert!(adjusted_diff < start_diff);
        assert_eq!(adjusted_diff, 5_000);
    });
}

#[test]
fn test_emission_based_on_registration_status() {
    new_test_ext(1).execute_with(|| {
        let n: u16 = 100;
        let netuid_off: u16 = 1;
        let netuid_on: u16 = 2;
        let tempo: u16 = 1;
        let netuids: Vec<u16> = vec![netuid_off, netuid_on];
        let emissions: Vec<u64> = vec![1000000000, 1000000000];

        // Add subnets with registration turned off and on
        add_network(netuid_off, tempo, 0);
        add_network(netuid_on, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid_off, n);
        SubtensorModule::set_max_allowed_uids(netuid_on, n);
        SubtensorModule::set_emission_values(&netuids, emissions).unwrap();
        SubtensorModule::set_network_registration_allowed(netuid_off, false);
        SubtensorModule::set_network_registration_allowed(netuid_on, true);

        // Populate the subnets with neurons
        for i in 0..n {
            SubtensorModule::append_neuron(netuid_off, &U256::from(i), 0);
            SubtensorModule::append_neuron(netuid_on, &U256::from(i), 0);
        }

        // Generate emission at block 0
        let block: u64 = 0;
        SubtensorModule::generate_emission(block);

        // Verify that no emission tuples are loaded for the subnet with registration off
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid_off).is_none());

        // Verify that emission tuples are loaded for the subnet with registration on
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid_on).is_some());
        assert_eq!(
            SubtensorModule::get_loaded_emission_tuples(netuid_on)
                .unwrap()
                .len(),
            n as usize
        );

        // Step to the next epoch block
        let epoch_block: u16 = tempo;
        step_block(epoch_block);

        // Verify that no emission tuples are loaded for the subnet with registration off
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid_off).is_none());
        log::info!(
            "Emissions for netuid with registration off: {:?}",
            SubtensorModule::get_loaded_emission_tuples(netuid_off)
        );

        // Verify that emission tuples are loaded for the subnet with registration on
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid_on).is_some());
        log::info!(
            "Emissions for netuid with registration on: {:?}",
            SubtensorModule::get_loaded_emission_tuples(netuid_on)
        );
        assert_eq!(
            SubtensorModule::get_loaded_emission_tuples(netuid_on)
                .unwrap()
                .len(),
            n as usize
        );

        let block: u64 = 0;
        // drain the emission tuples for the subnet with registration on
        SubtensorModule::drain_emission(block);
        // Turn on registration for the subnet with registration off
        SubtensorModule::set_network_registration_allowed(netuid_off, true);
        SubtensorModule::set_network_registration_allowed(netuid_on, false);

        // Generate emission at the next block
        let next_block: u64 = block + 1;
        SubtensorModule::generate_emission(next_block);

        // Verify that emission tuples are now loaded for the subnet with registration turned on
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid_off).is_some());
        log::info!(
            "Emissions for netuid with registration on: {:?}",
            SubtensorModule::get_loaded_emission_tuples(netuid_on)
        );
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid_on).is_none());
        assert_eq!(
            SubtensorModule::get_loaded_emission_tuples(netuid_off)
                .unwrap()
                .len(),
            n as usize
        );
    });
}

#[test]
fn test_epoch_runs_when_registration_disabled() {
    new_test_ext(1).execute_with(|| {
        let n: u16 = 100;
        let netuid_off: u16 = 1;
        let tempo: u16 = 1;
        let netuids: Vec<u16> = vec![netuid_off];
        let emissions: Vec<u64> = vec![1000000000];

        // Add subnets with registration turned off and on
        add_network(netuid_off, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid_off, n);
        SubtensorModule::set_emission_values(&netuids, emissions).unwrap();
        SubtensorModule::set_network_registration_allowed(netuid_off, false);

        // Populate the subnets with neurons
        for i in 0..n {
            SubtensorModule::append_neuron(netuid_off, &U256::from(i), 0);
        }

        // Generate emission at block 1
        let block: u64 = 1;
        SubtensorModule::generate_emission(block);

        step_block(1); // Now block 2

        // Verify blocks since last step was set
        assert_eq!(SubtensorModule::get_blocks_since_last_step(netuid_off), 1);

        // Step to the next epoch block
        let epoch_block: u16 = tempo;
        step_block(epoch_block);

        // Verify blocks since last step was set, this indicates we ran the epoch
        assert_eq!(
            SubtensorModule::get_blocks_since_last_step(netuid_off),
            0_u64
        );
        assert!(SubtensorModule::get_loaded_emission_tuples(netuid_off).is_some());
    });
}
