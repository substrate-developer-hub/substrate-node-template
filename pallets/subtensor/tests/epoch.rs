#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

use crate::mock::*;
use frame_support::{assert_err, assert_ok};
use frame_system::Config;
use pallet_subtensor::math::safe_exp;
use pallet_subtensor::*;
use rand::{distributions::Uniform, rngs::StdRng, seq::SliceRandom, thread_rng, Rng, SeedableRng};
use sp_core::U256;
use sp_runtime::DispatchError;
use std::time::Instant;
use substrate_fixed::types::I32F32;

mod mock;

pub fn fixed(val: f32) -> I32F32 {
    I32F32::from_num(val)
}

pub fn fixed_to_u16(x: I32F32) -> u16 {
    x.to_num::<u16>()
}

pub fn fixed_proportion_to_u16(x: I32F32) -> u16 {
    fixed_to_u16(x * I32F32::from_num(u16::MAX))
}

// Normalizes (sum to 1 except 0) the input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize(x: &mut [I32F32]) {
    let x_sum: I32F32 = x.iter().sum();
    if x_sum == I32F32::from_num(0.0_f32) {
        return;
    }
    for i in x.iter_mut() {
        *i /= x_sum;
    }
}

// Inplace normalize the passed positive integer weights so that they sum to u16 max value.
fn normalize_weights(mut weights: Vec<u16>) -> Vec<u16> {
    let sum: u64 = weights.iter().map(|x| *x as u64).sum();
    if sum == 0 {
        return weights;
    }
    weights.iter_mut().for_each(|x| {
        *x = (*x as u64 * u16::MAX as u64 / sum) as u16;
    });
    weights
}

// // Return as usize an I32F32 ratio of a usize input, avoiding the 0% and 100% extremes.
// fn non_extreme_fixed_ratio(ratio: I32F32, total: usize) -> usize {
//     if total == 0 {
//         return total;
//     }
//     let mut subset: usize = (ratio * I32F32::from_num(total)).to_num::<usize>();
//     if subset == 0 {
//         subset = 1;
//     } else if subset == total {
//         subset = total - 1;
//     }
//     return subset;
// }

// // Box-Muller Transform converting two uniform random samples to a normal random sample.
// fn normal(size: usize, rng: &mut StdRng, dist: &Uniform<u16>) -> Vec<I32F32> {
//     let max: I32F32 = I32F32::from_num(u16::MAX);
//     let two: I32F32 = I32F32::from_num(2);
//     let eps: I32F32 = I32F32::from_num(0.000001);
//     let pi: I32F32 = I32F32::from_num(PI);

//     let uniform_u16: Vec<u16> = (0..(2 * size)).map(|_| rng.sample(&dist)).collect();
//     let uniform: Vec<I32F32> = uniform_u16
//         .iter()
//         .map(|&x| I32F32::from_num(x) / max)
//         .collect();
//     let mut normal: Vec<I32F32> = vec![I32F32::from_num(0); size as usize];

//     for i in 0..size {
//         let u1: I32F32 = uniform[i] + eps;
//         let u2: I32F32 = uniform[i + size] + eps;
//         normal[i] = sqrt::<I32F32, I32F32>(-two * ln::<I32F32, I32F32>(u1).expect("")).expect("")
//             * cos(two * pi * u2);
//     }
//     normal
// }

// Returns validators and servers uids with either blockwise, regular, or random interleaving.
fn distribute_nodes(
    validators_n: usize,
    network_n: usize,
    interleave: usize,
) -> (Vec<u16>, Vec<u16>) {
    let mut validators: Vec<u16> = vec![];
    let mut servers: Vec<u16> = vec![];

    if interleave == 0 {
        // blockwise [validator_block, server_block]
        validators = (0..validators_n as u16).collect();
        servers = (validators_n as u16..network_n as u16).collect();
    } else if interleave == 1 {
        // regular interleaving [val, srv, srv, ..., srv, val, srv, srv, ..., srv, val, srv, ..., srv]
        (validators, servers) = (0..network_n as u16)
            .collect::<Vec<u16>>()
            .iter()
            .partition(|&i| *i as usize % (network_n / validators_n) == 0);
    } else if interleave == 2 {
        // random interleaving
        let mut permuted_uids: Vec<u16> = (0..network_n as u16).collect();
        permuted_uids.shuffle(&mut thread_rng());
        validators = permuted_uids[0..validators_n].into();
        servers = permuted_uids[validators_n..network_n].into();
    }

    (validators, servers)
}

#[allow(dead_code)]
fn uid_stats(netuid: u16, uid: u16) {
    log::info!(
        "stake: {:?}",
        SubtensorModule::get_total_stake_for_hotkey(&(U256::from(uid)))
    );
    log::info!("rank: {:?}", SubtensorModule::get_rank_for_uid(netuid, uid));
    log::info!(
        "trust: {:?}",
        SubtensorModule::get_trust_for_uid(netuid, uid)
    );
    log::info!(
        "consensus: {:?}",
        SubtensorModule::get_consensus_for_uid(netuid, uid)
    );
    log::info!(
        "incentive: {:?}",
        SubtensorModule::get_incentive_for_uid(netuid, uid)
    );
    log::info!(
        "dividend: {:?}",
        SubtensorModule::get_dividends_for_uid(netuid, uid)
    );
    log::info!(
        "emission: {:?}",
        SubtensorModule::get_emission_for_uid(netuid, uid)
    );
}

#[allow(clippy::too_many_arguments)]
fn init_run_epochs(
    netuid: u16,
    n: u16,
    validators: &[u16],
    servers: &[u16],
    epochs: u16,
    stake_per_validator: u64,
    server_self: bool,
    input_stake: &[u64],
    use_input_stake: bool,
    input_weights: &[Vec<(u16, u16)>],
    use_input_weights: bool,
    random_weights: bool,
    random_seed: u64,
    sparse: bool,
) {
    // === Create the network
    add_network(netuid, u16::MAX - 1, 0); // set higher tempo to avoid built-in epoch, then manual epoch instead

    // === Register uids
    SubtensorModule::set_max_allowed_uids(netuid, n);
    for key in 0..n {
        let stake = if use_input_stake {
            input_stake[key as usize]
        } else if validators.contains(&key) {
            stake_per_validator
        } else {
            // only validators receive stake
            0
        };

        // let stake: u64 = 1; // alternative test: all nodes receive stake, should be same outcome, except stake
        SubtensorModule::add_balance_to_coldkey_account(&(U256::from(key)), stake);
        SubtensorModule::append_neuron(netuid, &(U256::from(key)), 0);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &U256::from(key),
            &U256::from(key),
            stake,
        );
    }
    assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

    // === Issue validator permits
    SubtensorModule::set_max_allowed_validators(netuid, validators.len() as u16);
    assert_eq!(
        SubtensorModule::get_max_allowed_validators(netuid),
        validators.len() as u16
    );
    SubtensorModule::epoch(netuid, 1_000_000_000); // run first epoch to set allowed validators
    run_to_block(1); // run to next block to ensure weights are set on nodes after their registration block

    // === Set weights
    let mut rng = StdRng::seed_from_u64(random_seed); // constant seed so weights over multiple runs are equal
    let range = Uniform::new(0, u16::MAX);
    let mut weights: Vec<u16> = vec![u16::MAX / n; servers.len()];
    for uid in validators {
        if random_weights {
            weights = (0..servers.len()).map(|_| rng.sample(range)).collect();
            weights = normalize_weights(weights);
            // assert_eq!(weights.iter().map(|x| *x as u64).sum::<u64>(), u16::MAX as u64); // normalized weight sum not always u16::MAX
        }
        if use_input_weights {
            let sparse_weights = input_weights[*uid as usize].clone();
            weights = sparse_weights.iter().map(|(_, w)| *w).collect();
            let srvs: Vec<u16> = sparse_weights.iter().map(|(s, _)| *s).collect();
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(*uid as u64)),
                netuid,
                srvs,
                weights.clone(),
                0
            ));
        } else {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(*uid as u64)),
                netuid,
                servers.to_vec(),
                weights.clone(),
                0
            ));
        }
    }
    if server_self {
        for uid in servers {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(*uid as u64)),
                netuid,
                vec![*uid],
                vec![u16::MAX],
                0
            )); // server self-weight
        }
    }

    // === Run the epochs.
    log::info!("Start {epochs} epoch(s)");
    let start = Instant::now();
    for _ in 0..epochs {
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
    }
    let duration = start.elapsed();
    log::info!(
        "Time elapsed in (sparse={sparse}) epoch() is: {:?}",
        duration
    );

    // let bonds = SubtensorModule::get_bonds( netuid );
    // for (uid, node) in vec![ (validators[0], "validator"), (servers[0], "server") ] {
    // 	log::info!("\n{node}" );
    // 	uid_stats(netuid, uid);
    // 	log::info!("bonds: {:?} (on validator), {:?} (on server)", bonds[uid as usize][0], bonds[uid as usize][servers[0] as usize]);
    // }
}

// // Generate a random graph that is split into a major and minor set, each setting specific weight on itself and the complement on the other.
// fn split_graph(
//     major_stake: I32F32,
//     major_weight: I32F32,
//     minor_weight: I32F32,
//     weight_stddev: I32F32,
//     validators_n: usize,
//     network_n: usize,
//     interleave: usize,
// ) -> (
//     Vec<u16>,
//     Vec<u16>,
//     Vec<u16>,
//     Vec<u16>,
//     Vec<u16>,
//     Vec<u16>,
//     Vec<u64>,
//     Vec<Vec<(u16, u16)>>,
//     I32F32,
// ) {
//     let servers_n: usize = network_n - validators_n;
//     let major_servers_n: usize = non_extreme_fixed_ratio(major_stake, servers_n);
//     let major_validators_n: usize = non_extreme_fixed_ratio(major_stake, validators_n);

//     let (validators, servers) = distribute_nodes(validators_n, network_n, interleave as usize);
//     let major_validators: Vec<u16> = (0..major_validators_n).map(|i| validators[i]).collect();
//     let minor_validators: Vec<u16> = (major_validators_n..validators_n)
//         .map(|i| validators[i])
//         .collect();
//     let major_servers: Vec<u16> = (0..major_servers_n).map(|i| servers[i]).collect();
//     let minor_servers: Vec<u16> = (major_servers_n..servers_n).map(|i| servers[i]).collect();

//     let zero: I32F32 = I32F32::from_num(0);
//     let one: I32F32 = I32F32::from_num(1);
//     let stddev: I32F32 = I32F32::from_num(0.3);
//     let total_stake: I64F64 = I64F64::from_num(21_000_000_000_000_000 as u64);
//     let mut rng = StdRng::seed_from_u64(0); // constant seed so weights over multiple runs are equal
//     let dist = Uniform::new(0, u16::MAX);

//     let mut stake: Vec<u64> = vec![0; network_n];
//     let mut stake_fixed: Vec<I32F32> = vec![zero; network_n];
//     for (ratio, vals) in vec![
//         (major_stake, &major_validators),
//         (one - major_stake, &minor_validators),
//     ] {
//         let mut sample = normal(vals.len(), &mut rng, &dist)
//             .iter()
//             .map(|x: &I32F32| {
//                 let v: I32F32 = (stddev * x) + one;
//                 if v < zero {
//                     zero
//                 } else {
//                     v
//                 }
//             })
//             .collect();
//         inplace_normalize(&mut sample);
//         for (i, &val) in vals.iter().enumerate() {
//             stake[val as usize] =
//                 (I64F64::from_num(ratio) * I64F64::from_num(sample[i]) * total_stake)
//                     .to_num::<u64>();
//             stake_fixed[val as usize] =
//                 I32F32::from_num(I64F64::from_num(ratio) * I64F64::from_num(sample[i]));
//         }
//     }

//     let mut weights: Vec<Vec<(u16, u16)>> = vec![vec![]; network_n as usize];
//     let mut weights_fixed: Vec<Vec<I32F32>> = vec![vec![zero; network_n]; network_n];
//     for (first, second, vals) in vec![
//         (major_weight, one - major_weight, &major_validators),
//         (one - minor_weight, minor_weight, &minor_validators),
//     ] {
//         for &val in vals {
//             for (weight, srvs) in vec![(first, &major_servers), (second, &minor_servers)] {
//                 let mut sample: Vec<I32F32> = normal(srvs.len(), &mut rng, &dist)
//                     .iter()
//                     .map(|x: &I32F32| {
//                         let v: I32F32 = (weight_stddev * x) + one;
//                         if v < zero {
//                             zero
//                         } else {
//                             v
//                         }
//                     })
//                     .collect();
//                 inplace_normalize(&mut sample);

//                 for (i, &srv) in srvs.iter().enumerate() {
//                     weights[val as usize].push((srv, fixed_proportion_to_u16(weight * sample[i])));
//                     weights_fixed[val as usize][srv as usize] = weight * sample[i];
//                 }
//             }
//             inplace_normalize(&mut weights_fixed[val as usize]);
//         }
//     }

//     inplace_normalize(&mut stake_fixed);

//     // Calculate stake-weighted mean per server
//     let mut weight_mean: Vec<I32F32> = vec![zero; network_n];
//     for val in 0..network_n {
//         if stake_fixed[val] > zero {
//             for srv in 0..network_n {
//                 weight_mean[srv] += stake_fixed[val] * weights_fixed[val][srv];
//             }
//         }
//     }

//     // Calculate stake-weighted absolute standard deviation
//     let mut weight_dev: Vec<I32F32> = vec![zero; network_n];
//     for val in 0..network_n {
//         if stake_fixed[val] > zero {
//             for srv in 0..network_n {
//                 weight_dev[srv] +=
//                     stake_fixed[val] * (weight_mean[srv] - weights_fixed[val][srv]).abs();
//             }
//         }
//     }

//     // Calculate rank-weighted mean of weight_dev
//     let avg_weight_dev: I32F32 =
//         weight_dev.iter().sum::<I32F32>() / weight_mean.iter().sum::<I32F32>();

//     (
//         validators,
//         servers,
//         major_validators,
//         minor_validators,
//         major_servers,
//         minor_servers,
//         stake,
//         weights,
//         avg_weight_dev,
//     )
// }

// Test consensus guarantees with an epoch on a graph with 4096 nodes, of which the first 128 are validators, the graph is split into a major and minor set, each setting specific weight on itself and the complement on the other. Asserts that the major emission ratio >= major stake ratio.
// #[test]
// fn test_consensus_guarantees() {
//     let netuid: u16 = 0;
//     let network_n: u16 = 512;
//     let validators_n: u16 = 64;
//     let epochs: u16 = 1;
//     let interleave = 2;
//     log::info!("test_consensus_guarantees ({network_n:?}, {validators_n:?} validators)");
//     for (major_stake, major_weight, minor_weight, weight_stddev) in vec![
//         (0.51, 1., 1., 0.001),
//         (0.51, 0.03, 0., 0.001),
//         (0.51, 0.51, 0.49, 0.001),
//         (0.51, 0.51, 1., 0.001),
//         (0.51, 0.61, 0.8, 0.1),
//         (0.6, 0.67, 0.65, 0.2),
//         (0.6, 0.74, 0.77, 0.4),
//         (0.6, 0.76, 0.8, 0.4),
//         (0.6, 0.76, 1., 0.4),
//         (0.6, 0.92, 1., 0.4),
//         (0.6, 0.94, 1., 0.4),
//         (0.65, 0.78, 0.85, 0.6),
//         (0.7, 0.81, 0.85, 0.8),
//         (0.7, 0.83, 0.85, 1.),
//     ] {
//         let (
//             validators,
//             servers,
//             major_validators,
//             minor_validators,
//             major_servers,
//             minor_servers,
//             stake,
//             weights,
//             _avg_weight_dev,
//         ) = split_graph(
//             fixed(major_stake),
//             fixed(major_weight),
//             fixed(minor_weight),
//             fixed(weight_stddev),
//             validators_n as usize,
//             network_n as usize,
//             interleave as usize,
//         );

//         new_test_ext(1).execute_with(|| {
//             init_run_epochs(
//                 netuid,
//                 network_n,
//                 &validators,
//                 &servers,
//                 epochs,
//                 1,
//                 true,
//                 &stake,
//                 true,
//                 &weights,
//                 true,
//                 false,
//                 0,
//                 false,
//             );

//             let mut major_emission: I64F64 = I64F64::from_num(0);
//             let mut minor_emission: I64F64 = I64F64::from_num(0);
//             for set in vec![major_validators, major_servers] {
//                 for uid in set {
//                     major_emission +=
//                         I64F64::from_num(SubtensorModule::get_emission_for_uid(netuid, uid));
//                 }
//             }
//             for set in vec![minor_validators, minor_servers] {
//                 for uid in set {
//                     minor_emission +=
//                         I64F64::from_num(SubtensorModule::get_emission_for_uid(netuid, uid));
//                 }
//             }
//             let major_ratio: I32F32 =
//                 I32F32::from_num(major_emission / (major_emission + minor_emission));
//             assert!(major_stake <= major_ratio);
//         });
//     }
// }

// Test an epoch on an empty graph.
// #[test]
// fn test_overflow() {
//     new_test_ext(1).execute_with(|| {
//         log::info!("test_overflow:");
//         let netuid: u16 = 1;
//         add_network(netuid, 0, 0);
//         SubtensorModule::set_max_allowed_uids(netuid, 3);
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(
//             &U256::from(0),
//             &U256::from(0),
//             10,
//         );
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(
//             &U256::from(1),
//             &U256::from(1),
//             10,
//         );
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(
//             &U256::from(2),
//             &U256::from(2),
//             10,
//         );
//         SubtensorModule::append_neuron(netuid, &U256::from(0), 0);
//         SubtensorModule::append_neuron(netuid, &U256::from(1), 0);
//         SubtensorModule::append_neuron(netuid, &U256::from(2), 0);
//         SubtensorModule::set_validator_permit_for_uid(0, 0, true);
//         SubtensorModule::set_validator_permit_for_uid(0, 1, true);
//         SubtensorModule::set_validator_permit_for_uid(0, 2, true);
//         assert_ok!(SubtensorModule::set_weights(
//             RuntimeOrigin::signed(U256::from(0)),
//             netuid,
//             vec![0, 1, 2],
//             vec![u16::MAX / 3, u16::MAX / 3, u16::MAX],
//             0
//         ));
//         assert_ok!(SubtensorModule::set_weights(
//             RuntimeOrigin::signed(U256::from(1)),
//             netuid,
//             vec![1, 2],
//             vec![u16::MAX / 2, u16::MAX / 2],
//             0
//         ));
//         assert_ok!(SubtensorModule::set_weights(
//             RuntimeOrigin::signed(U256::from(2)),
//             netuid,
//             vec![2],
//             vec![u16::MAX],
//             0
//         ));
//         SubtensorModule::epoch(0, u64::MAX);
//     });
// }

// Test an epoch on an empty graph.
// #[test]
// fn test_nill_epoch_subtensor() {
//     new_test_ext(1).execute_with(|| {
//         log::info!("test_nill_epoch:");
//         SubtensorModule::epoch(0, 0);
//     });
// }

// Test an epoch on a graph with a single item.
#[test]
fn test_1_graph() {
    new_test_ext(1).execute_with(|| {
        log::info!("test_1_graph:");
        let netuid: u16 = 1;
        let coldkey = U256::from(0);
        let hotkey = U256::from(0);
        let uid: u16 = 0;
        let stake_amount: u64 = 1;
        add_network(netuid, u16::MAX - 1, 0); // set higher tempo to avoid built-in epoch, then manual epoch instead
        SubtensorModule::set_max_allowed_uids(netuid, 1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake_amount);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake_amount);
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        run_to_block(1); // run to next block to ensure weights are set on nodes after their registration block
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid)),
            netuid,
            vec![uid],
            vec![u16::MAX],
            0
        ));
        // SubtensorModule::set_weights_for_testing( netuid, i as u16, vec![ ( 0, u16::MAX )]); // doesn't set update status
        // SubtensorModule::set_bonds_for_testing( netuid, uid, vec![ ( 0, u16::MAX )]); // rather, bonds are calculated in epoch
        SubtensorModule::set_emission_values(&[netuid], vec![1_000_000_000]).unwrap();
        assert_eq!(
            SubtensorModule::get_subnet_emission_value(netuid),
            1_000_000_000
        );
        SubtensorModule::epoch(netuid, 1_000_000_000);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            stake_amount
        );
        assert_eq!(SubtensorModule::get_rank_for_uid(netuid, uid), 0);
        assert_eq!(SubtensorModule::get_trust_for_uid(netuid, uid), 0);
        assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, uid), 0);
        assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, uid), 0);
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 0);
        assert_eq!(
            SubtensorModule::get_emission_for_uid(netuid, uid),
            1_000_000_000
        );
    });
}

// Test an epoch on a graph with two items.
#[test]
fn test_10_graph() {
    new_test_ext(1).execute_with(|| {
        log::info!("test_10_graph");
        // Function for adding a nodes to the graph.
        pub fn add_node(netuid: u16, coldkey: U256, hotkey: U256, uid: u16, stake_amount: u64) {
            log::info!(
                "+Add net:{:?} coldkey:{:?} hotkey:{:?} uid:{:?} stake_amount: {:?} subn: {:?}",
                netuid,
                coldkey,
                hotkey,
                uid,
                stake_amount,
                SubtensorModule::get_subnetwork_n(netuid),
            );
            SubtensorModule::increase_stake_on_coldkey_hotkey_account(
                &coldkey,
                &hotkey,
                stake_amount,
            );
            SubtensorModule::append_neuron(netuid, &hotkey, 0);
            assert_eq!(SubtensorModule::get_subnetwork_n(netuid) - 1, uid);
        }
        // Build the graph with 10 items
        // each with 1 stake and self weights.
        let n: usize = 10;
        let netuid: u16 = 1;
        add_network(netuid, u16::MAX - 1, 0); // set higher tempo to avoid built-in epoch, then manual epoch instead
        SubtensorModule::set_max_allowed_uids(netuid, n as u16);
        for i in 0..10 {
            add_node(netuid, U256::from(i), U256::from(i), i as u16, 1)
        }
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 10);
        run_to_block(1); // run to next block to ensure weights are set on nodes after their registration block
        for i in 0..10 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(i)),
                netuid,
                vec![i as u16],
                vec![u16::MAX],
                0
            ));
        }
        // Run the epoch.
        SubtensorModule::epoch(netuid, 1_000_000_000);
        // Check return values.
        for i in 0..n {
            assert_eq!(
                SubtensorModule::get_total_stake_for_hotkey(&(U256::from(i))),
                1
            );
            assert_eq!(SubtensorModule::get_rank_for_uid(netuid, i as u16), 0);
            assert_eq!(SubtensorModule::get_trust_for_uid(netuid, i as u16), 0);
            assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, i as u16), 0);
            assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, i as u16), 0);
            assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, i as u16), 0);
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, i as u16),
                99999999
            );
        }
    });
}

// Test an epoch on a graph with 512 nodes, of which the first 64 are validators setting non-self weights, and the rest servers setting only self-weights.
#[test]
fn test_512_graph() {
    let netuid: u16 = 1;
    let network_n: u16 = 512;
    let validators_n: u16 = 64;
    let max_stake_per_validator: u64 = 328_125_000_000_000; // 21_000_000_000_000_000 / 64
    let epochs: u16 = 3;
    log::info!("test_{network_n:?}_graph ({validators_n:?} validators)");
    for interleave in 0..3 {
        for server_self in [false, true] {
            // server-self weight off/on
            let (validators, servers) = distribute_nodes(
                validators_n as usize,
                network_n as usize,
                interleave as usize,
            );
            let server: usize = servers[0] as usize;
            let validator: usize = validators[0] as usize;
            new_test_ext(1).execute_with(|| {
                init_run_epochs(
                    netuid,
                    network_n,
                    &validators,
                    &servers,
                    epochs,
                    max_stake_per_validator,
                    server_self,
                    &[],
                    false,
                    &[],
                    false,
                    false,
                    0,
                    false,
                );
                let bonds = SubtensorModule::get_bonds(netuid);
                for uid in validators {
                    assert_eq!(
                        SubtensorModule::get_total_stake_for_hotkey(&(U256::from(uid))),
                        max_stake_per_validator
                    );
                    assert_eq!(SubtensorModule::get_rank_for_uid(netuid, uid), 0);
                    assert_eq!(SubtensorModule::get_trust_for_uid(netuid, uid), 0);
                    assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, uid), 0);
                    assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, uid), 0);
                    assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 1023); // Note D = floor(1 / 64 * 65_535) = 1023
                    assert_eq!(SubtensorModule::get_emission_for_uid(netuid, uid), 7812500); // Note E = 0.5 / 200 * 1_000_000_000 = 7_812_500
                    assert_eq!(bonds[uid as usize][validator], 0.0);
                    assert_eq!(bonds[uid as usize][server], I32F32::from_num(65_535));
                    // Note B_ij = floor(1 / 64 * 65_535) / 65_535 = 1023 / 65_535, then max-upscaled to 65_535
                }
                for uid in servers {
                    assert_eq!(
                        SubtensorModule::get_total_stake_for_hotkey(&(U256::from(uid))),
                        0
                    );
                    assert_eq!(SubtensorModule::get_rank_for_uid(netuid, uid), 146); // Note R = floor(1 / (512 - 64) * 65_535) = 146
                    assert_eq!(SubtensorModule::get_trust_for_uid(netuid, uid), 65535);
                    assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, uid), 146); // Note C = floor(1 / (512 - 64) * 65_535) = 146
                    assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, uid), 146); // Note I = floor(1 / (512 - 64) * 65_535) = 146
                    assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 0);
                    assert_eq!(SubtensorModule::get_emission_for_uid(netuid, uid), 1116071); // Note E = floor(0.5 / (512 - 64) * 1_000_000_000) = 1_116_071
                    assert_eq!(bonds[uid as usize][validator], 0.0);
                    assert_eq!(bonds[uid as usize][server], 0.0);
                }
            });
        }
    }
}

// Test an epoch on a graph with 4096 nodes, of which the first 256 are validators setting random non-self weights, and the rest servers setting only self-weights.
#[test]
fn test_512_graph_random_weights() {
    let netuid: u16 = 1;
    let network_n: u16 = 512;
    let validators_n: u16 = 64;
    let epochs: u16 = 1;
    log::info!("test_{network_n:?}_graph_random_weights ({validators_n:?} validators)");
    for interleave in 0..3 {
        for server_self in [false, true] {
            // server-self weight off/on
            let (validators, servers) = distribute_nodes(
                validators_n as usize,
                network_n as usize,
                interleave as usize,
            );
            let server: usize = servers[0] as usize;
            let validator: usize = validators[0] as usize;
            #[allow(clippy::type_complexity)]
            let (mut rank, mut incentive, mut dividend, mut emission, mut bondv, mut bonds): (
                Vec<u16>,
                Vec<u16>,
                Vec<u16>,
                Vec<u64>,
                Vec<I32F32>,
                Vec<I32F32>,
            ) = (vec![], vec![], vec![], vec![], vec![], vec![]);

            // Dense epoch
            new_test_ext(1).execute_with(|| {
                init_run_epochs(
                    netuid,
                    network_n,
                    &validators,
                    &servers,
                    epochs,
                    1,
                    server_self,
                    &[],
                    false,
                    &[],
                    false,
                    true,
                    interleave as u64,
                    false,
                );

                let bond = SubtensorModule::get_bonds(netuid);
                for uid in 0..network_n {
                    rank.push(SubtensorModule::get_rank_for_uid(netuid, uid));
                    incentive.push(SubtensorModule::get_incentive_for_uid(netuid, uid));
                    dividend.push(SubtensorModule::get_dividends_for_uid(netuid, uid));
                    emission.push(SubtensorModule::get_emission_for_uid(netuid, uid));
                    bondv.push(bond[uid as usize][validator]);
                    bonds.push(bond[uid as usize][server]);
                }
            });

            // Sparse epoch (same random seed as dense)
            new_test_ext(1).execute_with(|| {
                init_run_epochs(
                    netuid,
                    network_n,
                    &validators,
                    &servers,
                    epochs,
                    1,
                    server_self,
                    &[],
                    false,
                    &[],
                    false,
                    true,
                    interleave as u64,
                    true,
                );
                // Assert that dense and sparse epoch results are equal
                let bond = SubtensorModule::get_bonds(netuid);
                for uid in 0..network_n {
                    assert_eq!(
                        SubtensorModule::get_rank_for_uid(netuid, uid),
                        rank[uid as usize]
                    );
                    assert_eq!(
                        SubtensorModule::get_incentive_for_uid(netuid, uid),
                        incentive[uid as usize]
                    );
                    assert_eq!(
                        SubtensorModule::get_dividends_for_uid(netuid, uid),
                        dividend[uid as usize]
                    );
                    assert_eq!(
                        SubtensorModule::get_emission_for_uid(netuid, uid),
                        emission[uid as usize]
                    );
                    assert_eq!(bond[uid as usize][validator], bondv[uid as usize]);
                    assert_eq!(bond[uid as usize][server], bonds[uid as usize]);
                }
            });
        }
    }
}

// Test an epoch on a graph with 4096 nodes, of which the first 256 are validators setting non-self weights, and the rest servers setting only self-weights.
// #[test]
#[allow(dead_code)]
fn test_4096_graph() {
    let netuid: u16 = 1;
    let network_n: u16 = 4096;
    let validators_n: u16 = 256;
    let epochs: u16 = 1;
    let max_stake_per_validator: u64 = 82_031_250_000_000; // 21_000_000_000_000_000 / 256
    log::info!("test_{network_n:?}_graph ({validators_n:?} validators)");
    for interleave in 0..3 {
        let (validators, servers) = distribute_nodes(
            validators_n as usize,
            network_n as usize,
            interleave as usize,
        );
        let server: usize = servers[0] as usize;
        let validator: usize = validators[0] as usize;
        for server_self in [false, true] {
            // server-self weight off/on
            new_test_ext(1).execute_with(|| {
                init_run_epochs(
                    netuid,
                    network_n,
                    &validators,
                    &servers,
                    epochs,
                    max_stake_per_validator,
                    server_self,
                    &[],
                    false,
                    &[],
                    false,
                    false,
                    0,
                    true,
                );
                assert_eq!(SubtensorModule::get_total_stake(), 21_000_000_000_000_000);
                let bonds = SubtensorModule::get_bonds(netuid);
                for uid in &validators {
                    assert_eq!(
                        SubtensorModule::get_total_stake_for_hotkey(&(U256::from(*uid as u64))),
                        max_stake_per_validator
                    );
                    assert_eq!(SubtensorModule::get_rank_for_uid(netuid, *uid), 0);
                    assert_eq!(SubtensorModule::get_trust_for_uid(netuid, *uid), 0);
                    assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, *uid), 0);
                    assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, *uid), 0);
                    assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, *uid), 255); // Note D = floor(1 / 256 * 65_535)
                    assert_eq!(SubtensorModule::get_emission_for_uid(netuid, *uid), 1953125); // Note E = 0.5 / 256 * 1_000_000_000 = 1953125
                    assert_eq!(bonds[*uid as usize][validator], 0.0);
                    assert_eq!(
                        bonds[*uid as usize][server],
                        I32F32::from_num(255) / I32F32::from_num(65_535)
                    ); // Note B_ij = floor(1 / 256 * 65_535) / 65_535
                }
                for uid in &servers {
                    assert_eq!(
                        SubtensorModule::get_total_stake_for_hotkey(&(U256::from(*uid as u64))),
                        0
                    );
                    assert_eq!(SubtensorModule::get_rank_for_uid(netuid, *uid), 17); // Note R = floor(1 / (4096 - 256) * 65_535) = 17
                    assert_eq!(SubtensorModule::get_trust_for_uid(netuid, *uid), 65535);
                    assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, *uid), 17); // Note C = floor(1 / (4096 - 256) * 65_535) = 17
                    assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, *uid), 17); // Note I = floor(1 / (4096 - 256) * 65_535) = 17
                    assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, *uid), 0);
                    assert_eq!(SubtensorModule::get_emission_for_uid(netuid, *uid), 130208); // Note E = floor(0.5 / (4096 - 256) * 1_000_000_000) = 130208
                    assert_eq!(bonds[*uid as usize][validator], 0.0);
                    assert_eq!(bonds[*uid as usize][server], 0.0);
                }
            });
        }
    }
}

// Test an epoch_sparse on a graph with 16384 nodes, of which the first 512 are validators setting non-self weights, and the rest servers setting only self-weights.
// #[test]
#[allow(dead_code)]
fn test_16384_graph_sparse() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let n: u16 = 16384;
        let validators_n: u16 = 512;
        let validators: Vec<u16> = (0..validators_n).collect();
        let servers: Vec<u16> = (validators_n..n).collect();
        let server: u16 = servers[0];
        let epochs: u16 = 1;
        log::info!("test_{n:?}_graph ({validators_n:?} validators)");
        init_run_epochs(
            netuid,
            n,
            &validators,
            &servers,
            epochs,
            1,
            false,
            &[],
            false,
            &[],
            false,
            false,
            0,
            true,
        );
        let bonds = SubtensorModule::get_bonds(netuid);
        for uid in validators {
            assert_eq!(
                SubtensorModule::get_total_stake_for_hotkey(&(U256::from(uid))),
                1
            );
            assert_eq!(SubtensorModule::get_rank_for_uid(netuid, uid), 0);
            assert_eq!(SubtensorModule::get_trust_for_uid(netuid, uid), 0);
            assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, uid), 438); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
            assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, uid), 0);
            assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 127); // Note D = floor(1 / 512 * 65_535) = 127
            assert_eq!(SubtensorModule::get_emission_for_uid(netuid, uid), 976085); // Note E = 0.5 / 512 * 1_000_000_000 = 976_562 (discrepancy)
            assert_eq!(bonds[uid as usize][0], 0.0);
            assert_eq!(
                bonds[uid as usize][server as usize],
                I32F32::from_num(127) / I32F32::from_num(65_535)
            ); // Note B_ij = floor(1 / 512 * 65_535) / 65_535 = 127 / 65_535
        }
        for uid in servers {
            assert_eq!(
                SubtensorModule::get_total_stake_for_hotkey(&(U256::from(uid))),
                0
            );
            assert_eq!(SubtensorModule::get_rank_for_uid(netuid, uid), 4); // Note R = floor(1 / (16384 - 512) * 65_535) = 4
            assert_eq!(SubtensorModule::get_trust_for_uid(netuid, uid), 65535);
            assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, uid), 4); // Note C = floor(1 / (16384 - 512) * 65_535) = 4
            assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, uid), 4); // Note I = floor(1 / (16384 - 512) * 65_535) = 4
            assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 0);
            assert_eq!(SubtensorModule::get_emission_for_uid(netuid, uid), 31517); // Note E = floor(0.5 / (16384 - 512) * 1_000_000_000) = 31502 (discrepancy)
            assert_eq!(bonds[uid as usize][0], 0.0);
            assert_eq!(bonds[uid as usize][server as usize], 0.0);
        }
    });
}

// Test bonds exponential moving average over a sequence of epochs.
#[test]
fn test_bonds() {
    new_test_ext(1).execute_with(|| {
		let sparse: bool = true;
		let n: u16 = 8;
		let netuid: u16 = 1;
		let tempo: u16 = u16::MAX - 1;  // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
		let max_stake: u64 = 4;
		let stakes: Vec<u64> = vec![1, 2, 3, 4, 0, 0, 0, 0];
        let block_number = System::block_number();
		add_network(netuid, tempo, 0);
		SubtensorModule::set_max_allowed_uids( netuid, n );
		assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
		SubtensorModule::set_max_registrations_per_block( netuid, n );
		SubtensorModule::set_target_registrations_per_interval(netuid, n);
		SubtensorModule::set_weights_set_rate_limit( netuid, 0 );
        SubtensorModule::set_min_allowed_weights( netuid, 1 );
        SubtensorModule::set_max_weight_limit( netuid, u16::MAX );

		// === Register [validator1, validator2, validator3, validator4, server1, server2, server3, server4]
		for key in 0..n as u64 {
			SubtensorModule::add_balance_to_coldkey_account( &U256::from(key), max_stake );
			let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, block_number, key * 1_000_000, &U256::from(key));
			assert_ok!(SubtensorModule::register(<<Test as Config>::RuntimeOrigin>::signed(U256::from(key)), netuid, block_number, nonce, work, U256::from(key), U256::from(key)));
			SubtensorModule::increase_stake_on_coldkey_hotkey_account( &U256::from(key), &U256::from(key), stakes[key as usize] );
		}
		assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

		// === Issue validator permits
		SubtensorModule::set_max_allowed_validators(netuid, n);
		assert_eq!( SubtensorModule::get_max_allowed_validators(netuid), n);
		SubtensorModule::epoch( netuid, 1_000_000_000 ); // run first epoch to set allowed validators
        next_block(); // run to next block to ensure weights are set on nodes after their registration block

		// === Set weights [val->srv1: 0.1, val->srv2: 0.2, val->srv3: 0.3, val->srv4: 0.4]
		for uid in 0..(n/2) as u64 {
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(U256::from(uid)), netuid, ((n/2)..n).collect(), vec![ u16::MAX/4, u16::MAX/2, (u16::MAX/4)*3, u16::MAX], 0));
		}
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  n: 8
			current_block: 1; activity_cutoff: 5000; Last update: [1, 1, 1, 1, 0, 0, 0, 0]
			Inactive: [false, false, false, false, false, false, false, false]
			Block at registration: [0, 0, 0, 0, 0, 0, 0, 0]
			hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
			S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
			validator_permits: [true, true, true, true, true, true, true, true]
			max_allowed_validators: 8
			new_validator_permits: [true, true, true, true, true, true, true, true]
			S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
			W: [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit+diag): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit+diag+outdate): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (mask+norm): [[(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
			R (before): [0, 0, 0, 0, 0.099997558, 0.2000012202, 0.2999926745, 0.4000085443]
			C: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
			W: [[(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
			Tv: [0.9999999995, 0.9999999995, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
			R (after): [0, 0, 0, 0, 0.099997558, 0.2000012202, 0.2999926745, 0.4000085443]
			T: [0, 0, 0, 0, 1, 1, 1, 1]
			I (=R): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926752, 0.4000085455]
			B: [[], [], [], [], [], [], [], []]
			B (outdatedmask): [[], [], [], [], [], [], [], []]
			B (mask+norm): [[], [], [], [], [], [], [], []]
			ΔB: [[(4, 0.0099997558), (5, 0.020000122), (6, 0.0299992673), (7, 0.0400008543)], [(4, 0.0199995115), (5, 0.040000244), (6, 0.0599985349), (7, 0.0800017088)], [(4, 0.0299992673), (5, 0.060000366), (6, 0.0899978024), (7, 0.1200025633)], [(4, 0.0399990233), (5, 0.080000488), (6, 0.11999707), (7, 0.1600034179)], [], [], [], []]
			ΔB (norm): [[(4, 0.0999999996), (5, 0.0999999999), (6, 0.0999999994), (7, 0.0999999996)], [(4, 0.1999999995), (5, 0.2), (6, 0.1999999997), (7, 0.1999999997)], [(4, 0.299999999), (5, 0.2999999998), (6, 0.3), (7, 0.3)], [(4, 0.4000000013), (5, 0.4), (6, 0.4000000004), (7, 0.4000000001)], [], [], [], []]
			emaB: [[(4, 0.0999999982), (5, 0.0999999985), (6, 0.099999998), (7, 0.099999998)], [(4, 0.199999999), (5, 0.1999999995), (6, 0.1999999986), (7, 0.1999999986)], [(4, 0.2999999996), (5, 0.3000000003), (6, 0.3000000012), (7, 0.3000000012)], [(4, 0.4000000027), (5, 0.4000000013), (6, 0.4000000018), (7, 0.4000000018)], [], [], [], []]
			D: [0.0999999978, 0.1999999983, 0.3000000012, 0.4000000022, 0, 0, 0, 0]
			nE: [0.0499999989, 0.0999999992, 0.1500000006, 0.2000000011, 0.049998779, 0.1000006103, 0.1499963375, 0.2000042726]
			E: [49999998, 99999999, 150000000, 200000001, 49998779, 100000610, 149996337, 200004272]
			P: [0.0499999989, 0.0999999992, 0.1500000006, 0.2000000011, 0.049998779, 0.1000006103, 0.1499963375, 0.2000042726]
			emaB: [[(4, 0.2499999937), (5, 0.2499999953), (6, 0.2499999937), (7, 0.2499999937)], [(4, 0.4999999942), (5, 0.499999997), (6, 0.4999999942), (7, 0.4999999942)], [(4, 0.7499999937), (5, 0.7499999981), (6, 0.7499999995), (7, 0.7499999995)], [(4, 1), (5, 1), (6, 1), (7, 1)], [], [], [], []] */
		let bonds = SubtensorModule::get_bonds( netuid );
		assert_eq!(bonds[0][4], 16383);
		assert_eq!(bonds[1][4], 32767);
		assert_eq!(bonds[2][4], 49151);
		assert_eq!(bonds[3][4], 65535);

		// === Set self-weight only on val1
		let uid = 0;
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(U256::from(uid)), netuid, vec![uid], vec![u16::MAX], 0));
        next_block();
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  n: 8
			current_block: 2
			activity_cutoff: 5000
			Last update: [1, 1, 1, 1, 0, 0, 0, 0]
			Inactive: [false, false, false, false, false, false, false, false]
			Block at registration: [0, 0, 0, 0, 0, 0, 0, 0]
			hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
			S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
			validator_permits: [true, true, true, true, true, true, true, true]
			max_allowed_validators: 8
			new_validator_permits: [true, true, true, true, true, true, true, true]
			S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
			W: [[(0, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit): [[(0, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit+diag): [[], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit+diag+outdate): [[], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (mask+norm): [[], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
			R (before): [0, 0, 0, 0, 0.0899978022, 0.1800010982, 0.2699934072, 0.36000769]
			C: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
			W: [[], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
			Tv: [0, 0.9999999995, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
			R (after): [0, 0, 0, 0, 0.0899978022, 0.1800010982, 0.2699934072, 0.36000769]
			T: [0, 0, 0, 0, 1, 1, 1, 1]
			I (=R): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
			B: [[(4, 16383), (5, 16383), (6, 16383), (7, 16383)], [(4, 32767), (5, 32767), (6, 32767), (7, 32767)], [(4, 49151), (5, 49151), (6, 49151), (7, 49151)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (outdatedmask): [[(4, 16383), (5, 16383), (6, 16383), (7, 16383)], [(4, 32767), (5, 32767), (6, 32767), (7, 32767)], [(4, 49151), (5, 49151), (6, 49151), (7, 49151)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (mask+norm): [[(4, 0.0999963377), (5, 0.0999963377), (6, 0.0999963377), (7, 0.0999963377)], [(4, 0.1999987792), (5, 0.1999987792), (6, 0.1999987792), (7, 0.1999987792)], [(4, 0.3000012205), (5, 0.3000012205), (6, 0.3000012205), (7, 0.3000012205)], [(4, 0.400003662), (5, 0.400003662), (6, 0.400003662), (7, 0.400003662)], [], [], [], []]
			ΔB: [[], [(4, 0.0199995115), (5, 0.040000244), (6, 0.0599985349), (7, 0.0800017088)], [(4, 0.0299992673), (5, 0.060000366), (6, 0.0899978024), (7, 0.1200025633)], [(4, 0.0399990233), (5, 0.080000488), (6, 0.11999707), (7, 0.1600034179)], [], [], [], []]
			ΔB (norm): [[], [(4, 0.2222222215), (5, 0.222222222), (6, 0.2222222218), (7, 0.2222222218)], [(4, 0.3333333323), (5, 0.3333333333), (6, 0.3333333333), (7, 0.3333333333)], [(4, 0.4444444457), (5, 0.4444444443), (6, 0.4444444447), (7, 0.4444444445)], [], [], [], []]
			emaB: [[(4, 0.0899967037), (5, 0.0899967037), (6, 0.0899967037), (7, 0.0899967037)], [(4, 0.2022211235), (5, 0.2022211235), (6, 0.2022211235), (7, 0.2022211235)], [(4, 0.3033344317), (5, 0.3033344317), (6, 0.3033344317), (7, 0.3033344317)], [(4, 0.4044477409), (5, 0.4044477406), (6, 0.4044477406), (7, 0.4044477406)], [], [], [], []]
			D: [0.0899967032, 0.2022211233, 0.303334432, 0.404447741, 0, 0, 0, 0]
			nE: [0.0449983515, 0.1011105615, 0.1516672159, 0.2022238704, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
			E: [44998351, 101110561, 151667215, 202223870, 49998779, 100000610, 149996337, 200004272]
			P: [0.0449983515, 0.1011105615, 0.1516672159, 0.2022238704, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
			emaB: [[(4, 0.2225175085), (5, 0.2225175085), (6, 0.2225175085), (7, 0.2225175085)], [(4, 0.499993208), (5, 0.4999932083), (6, 0.4999932083), (7, 0.4999932083)], [(4, 0.7499966028), (5, 0.7499966032), (6, 0.7499966032), (7, 0.7499966032)], [(4, 1), (5, 1), (6, 1), (7, 1)], [], [], [], []] */
		let bonds = SubtensorModule::get_bonds( netuid );
		assert_eq!(bonds[0][4], 14582);
		assert_eq!(bonds[1][4], 32767);
		assert_eq!(bonds[2][4], 49151);
		assert_eq!(bonds[3][4], 65535);

		// === Set self-weight only on val2
		let uid = 1;
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(U256::from(uid)), netuid, vec![uid], vec![u16::MAX], 0));
        next_block();
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  current_block: 3
			W: [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit): [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit+diag): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit+diag+outdate): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (mask+norm): [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
			R (before): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
			C: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
			W: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
			Tv: [0, 0, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
			R (after): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
			T: [0, 0, 0, 0, 1, 1, 1, 1]
			I (=R): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
			B: [[(4, 14582), (5, 14582), (6, 14582), (7, 14582)], [(4, 32767), (5, 32767), (6, 32767), (7, 32767)], [(4, 49151), (5, 49151), (6, 49151), (7, 49151)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (outdatedmask): [[(4, 14582), (5, 14582), (6, 14582), (7, 14582)], [(4, 32767), (5, 32767), (6, 32767), (7, 32767)], [(4, 49151), (5, 49151), (6, 49151), (7, 49151)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (mask+norm): [[(4, 0.0899929027), (5, 0.0899929027), (6, 0.0899929027), (7, 0.0899929027)], [(4, 0.2022217421), (5, 0.2022217421), (6, 0.2022217421), (7, 0.2022217421)], [(4, 0.303335699), (5, 0.303335699), (6, 0.303335699), (7, 0.303335699)], [(4, 0.404449656), (5, 0.404449656), (6, 0.404449656), (7, 0.404449656)], [], [], [], []]
			ΔB: [[], [], [(4, 0.0299992673), (5, 0.060000366), (6, 0.0899978024), (7, 0.1200025633)], [(4, 0.0399990233), (5, 0.080000488), (6, 0.11999707), (7, 0.1600034179)], [], [], [], []]
			ΔB (norm): [[], [], [(4, 0.428571427), (5, 0.4285714284), (6, 0.4285714284), (7, 0.4285714284)], [(4, 0.5714285728), (5, 0.5714285714), (6, 0.5714285714), (7, 0.5714285714)], [], [], [], []]
			emaB: [[(4, 0.0809936123), (5, 0.0809936123), (6, 0.0809936123), (7, 0.0809936123)], [(4, 0.181999568), (5, 0.181999568), (6, 0.181999568), (7, 0.181999568)], [(4, 0.3158592717), (5, 0.315859272), (6, 0.315859272), (7, 0.315859272)], [(4, 0.4211475477), (5, 0.4211475474), (6, 0.4211475474), (7, 0.4211475474)], [], [], [], []]
			D: [0.0809936118, 0.1819995677, 0.3158592721, 0.421147548, 0, 0, 0, 0]
			nE: [0.040496806, 0.0909997837, 0.157929636, 0.2105737738, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
			E: [40496805, 90999783, 157929636, 210573773, 49998779, 100000610, 149996337, 200004272]
			P: [0.040496806, 0.0909997837, 0.157929636, 0.2105737738, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
			emaB: [[(4, 0.192316476), (5, 0.192316476), (6, 0.192316476), (7, 0.192316476)], [(4, 0.4321515555), (5, 0.4321515558), (6, 0.4321515558), (7, 0.4321515558)], [(4, 0.7499967015), (5, 0.7499967027), (6, 0.7499967027), (7, 0.7499967027)], [(4, 1), (5, 1), (6, 1), (7, 1)], [], [], [], []] */
		let bonds = SubtensorModule::get_bonds( netuid );
		assert_eq!(bonds[0][4], 12603);
		assert_eq!(bonds[1][4], 28321);
		assert_eq!(bonds[2][4], 49151);
		assert_eq!(bonds[3][4], 65535);

		// === Set self-weight only on val3
		let uid = 2;
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(U256::from(uid)), netuid, vec![uid], vec![u16::MAX], 0));
        next_block();
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  current_block: 4
			W: [[(0, 65535)], [(1, 65535)], [(2, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit): [[(0, 65535)], [(1, 65535)], [(2, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit+diag): [[], [], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit+diag+outdate): [[], [], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (mask+norm): [[], [], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
			R (before): [0, 0, 0, 0, 0.0399990233, 0.080000488, 0.11999707, 0.1600034179]
			C: [0, 0, 0, 0, 0, 0, 0, 0]
			W: [[], [], [], [], [], [], [], []]
			Tv: [0, 0, 0, 0, 0, 0, 0, 0]
			R (after): [0, 0, 0, 0, 0, 0, 0, 0]
			T: [0, 0, 0, 0, 0, 0, 0, 0]
			I (=R): [0, 0, 0, 0, 0, 0, 0, 0]
			B: [[(4, 12603), (5, 12603), (6, 12603), (7, 12603)], [(4, 28321), (5, 28321), (6, 28321), (7, 28321)], [(4, 49151), (5, 49151), (6, 49151), (7, 49151)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (outdatedmask): [[(4, 12603), (5, 12603), (6, 12603), (7, 12603)], [(4, 28321), (5, 28321), (6, 28321), (7, 28321)], [(4, 49151), (5, 49151), (6, 49151), (7, 49151)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (mask+norm): [[(4, 0.0809909387), (5, 0.0809909387), (6, 0.0809909387), (7, 0.0809909387)], [(4, 0.1819998713), (5, 0.1819998713), (6, 0.1819998713), (7, 0.1819998713)], [(4, 0.3158601632), (5, 0.3158601632), (6, 0.3158601632), (7, 0.3158601632)], [(4, 0.4211490264), (5, 0.4211490264), (6, 0.4211490264), (7, 0.4211490264)], [], [], [], []]
			ΔB: [[], [], [], [], [], [], [], []]
			ΔB (norm): [[], [], [], [], [], [], [], []]
			emaB: [[(4, 0.0809909385), (5, 0.0809909385), (6, 0.0809909385), (7, 0.0809909385)], [(4, 0.1819998713), (5, 0.1819998713), (6, 0.1819998713), (7, 0.1819998713)], [(4, 0.3158601632), (5, 0.3158601632), (6, 0.3158601632), (7, 0.3158601632)], [(4, 0.4211490266), (5, 0.4211490266), (6, 0.4211490266), (7, 0.4211490266)], [], [], [], []]
			D: [0, 0, 0, 0, 0, 0, 0, 0]
			nE: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
			E: [99999999, 199999999, 299999999, 399999999, 0, 0, 0, 0]
			P: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
			emaB: [[(4, 0.1923094518), (5, 0.1923094518), (6, 0.1923094518), (7, 0.1923094518)], [(4, 0.4321507583), (5, 0.4321507583), (6, 0.4321507583), (7, 0.4321507583)], [(4, 0.7499961846), (5, 0.7499961846), (6, 0.7499961846), (7, 0.7499961846)], [(4, 1), (5, 1), (6, 1), (7, 1)], [], [], [], []] */
		let bonds = SubtensorModule::get_bonds( netuid );
		assert_eq!(bonds[0][7], 12602);
		assert_eq!(bonds[1][7], 28320);
		assert_eq!(bonds[2][7], 49150);
		assert_eq!(bonds[3][7], 65535);

		// === Set val3->srv4: 1
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(U256::from(2)), netuid, vec![7], vec![u16::MAX], 0));
        next_block();
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  current_block: 5
			W: [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit): [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit+diag): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (permit+diag+outdate): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
			W (mask+norm): [[], [], [(7, 1)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
			R (before): [0, 0, 0, 0, 0.0399990233, 0.080000488, 0.11999707, 0.4600034177]
			C: [0, 0, 0, 0, 0, 0, 0, 0.400008545]
			W: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
			Tv: [0, 0, 0.400008545, 0.400008545, 0, 0, 0, 0]
			R (after): [0, 0, 0, 0, 0, 0, 0, 0.2800059812]
			T: [0, 0, 0, 0, 0, 0, 0, 0.6087041323]
			I (=R): [0, 0, 0, 0, 0, 0, 0, 1]
			B: [[(4, 12602), (5, 12602), (6, 12602), (7, 12602)], [(4, 28320), (5, 28320), (6, 28320), (7, 28320)], [(4, 49150), (5, 49150), (6, 49150), (7, 49150)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (outdatedmask): [[(4, 12602), (5, 12602), (6, 12602), (7, 12602)], [(4, 28320), (5, 28320), (6, 28320), (7, 28320)], [(4, 49150), (5, 49150), (6, 49150), (7, 49150)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (mask+norm): [[(4, 0.0809860737), (5, 0.0809860737), (6, 0.0809860737), (7, 0.0809860737)], [(4, 0.1819969537), (5, 0.1819969537), (6, 0.1819969537), (7, 0.1819969537)], [(4, 0.3158598263), (5, 0.3158598263), (6, 0.3158598263), (7, 0.3158598263)], [(4, 0.4211571459), (5, 0.4211571459), (6, 0.4211571459), (7, 0.4211571459)], [], [], [], []]
			ΔB: [[], [], [(7, 0.1200025633)], [(7, 0.1600034179)], [], [], [], []]
			ΔB (norm): [[], [], [(7, 0.4285714284)], [(7, 0.5714285714)], [], [], [], []]
			emaB: [[(4, 0.0809860737), (5, 0.0809860737), (6, 0.0809860737), (7, 0.0728874663)], [(4, 0.1819969537), (5, 0.1819969537), (6, 0.1819969537), (7, 0.1637972582)], [(4, 0.3158598263), (5, 0.3158598263), (6, 0.3158598263), (7, 0.3271309866)], [(4, 0.421157146), (5, 0.421157146), (6, 0.421157146), (7, 0.4361842885)], [], [], [], []]
			D: [0.0728874663, 0.1637972582, 0.3271309866, 0.4361842885, 0, 0, 0, 0]
			nE: [0.0364437331, 0.081898629, 0.1635654932, 0.2180921442, 0, 0, 0, 0.5]
			E: [36443733, 81898628, 163565493, 218092144, 0, 0, 0, 500000000]
			P: [0.0364437331, 0.081898629, 0.1635654932, 0.2180921442, 0, 0, 0, 0.5]
			emaB: [[(4, 0.1922941932), (5, 0.1922941932), (6, 0.1922941932), (7, 0.1671024568)], [(4, 0.4321354993), (5, 0.4321354993), (6, 0.4321354993), (7, 0.3755230587)], [(4, 0.7499809256), (5, 0.7499809256), (6, 0.7499809256), (7, 0.749983425)], [(4, 1), (5, 1), (6, 1), (7, 1)], [], [], [], []] */
		let bonds = SubtensorModule::get_bonds( netuid );
		assert_eq!(bonds[0][7], 10951);
		assert_eq!(bonds[1][7], 24609);
		assert_eq!(bonds[2][7], 49150);
		assert_eq!(bonds[3][7], 65535);

        next_block();
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  current_block: 6
			B: [[(4, 12601), (5, 12601), (6, 12601), (7, 10951)], [(4, 28319), (5, 28319), (6, 28319), (7, 24609)], [(4, 49149), (5, 49149), (6, 49149), (7, 49150)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (outdatedmask): [[(4, 12601), (5, 12601), (6, 12601), (7, 10951)], [(4, 28319), (5, 28319), (6, 28319), (7, 24609)], [(4, 49149), (5, 49149), (6, 49149), (7, 49150)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (mask+norm): [[(4, 0.0809812085), (5, 0.0809812085), (6, 0.0809812085), (7, 0.0728876167)], [(4, 0.181994036), (5, 0.181994036), (6, 0.181994036), (7, 0.163792472)], [(4, 0.3158594894), (5, 0.3158594894), (6, 0.3158594894), (7, 0.3271323503)], [(4, 0.4211652656), (5, 0.4211652656), (6, 0.4211652656), (7, 0.4361875602)], [], [], [], []]
			ΔB: [[], [], [(7, 0.1200025633)], [(7, 0.1600034179)], [], [], [], []]
			ΔB (norm): [[], [], [(7, 0.4285714284)], [(7, 0.5714285714)], [], [], [], []]
			emaB: [[(4, 0.0809812082), (5, 0.0809812082), (6, 0.0809812082), (7, 0.0655988548)], [(4, 0.181994036), (5, 0.181994036), (6, 0.181994036), (7, 0.1474132247)], [(4, 0.3158594896), (5, 0.3158594896), (6, 0.3158594896), (7, 0.3372762585)], [(4, 0.4211652658), (5, 0.4211652658), (6, 0.4211652658), (7, 0.4497116616)], [], [], [], []]
			D: [0.0655988548, 0.1474132247, 0.3372762585, 0.4497116616, 0, 0, 0, 0]
			nE: [0.0327994274, 0.0737066122, 0.1686381293, 0.2248558307, 0, 0, 0, 0.5]
			E: [32799427, 73706612, 168638129, 224855830, 0, 0, 0, 500000000]
			P: [0.0327994274, 0.0737066122, 0.1686381293, 0.2248558307, 0, 0, 0, 0.5]
			emaB: [[(4, 0.1922789337), (5, 0.1922789337), (6, 0.1922789337), (7, 0.1458686984)], [(4, 0.4321202405), (5, 0.4321202405), (6, 0.4321202405), (7, 0.3277949789)], [(4, 0.749965667), (5, 0.749965667), (6, 0.749965667), (7, 0.74998335)], [(4, 1), (5, 1), (6, 1), (7, 1)], [], [], [], []] */
		let bonds = SubtensorModule::get_bonds( netuid );
		assert_eq!(bonds[0][7], 9559);
		assert_eq!(bonds[1][7], 21482);
		assert_eq!(bonds[2][7], 49150);
		assert_eq!(bonds[3][7], 65535);

        next_block();
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  current_block: 7
			B: [[(4, 12600), (5, 12600), (6, 12600), (7, 9559)], [(4, 28318), (5, 28318), (6, 28318), (7, 21482)], [(4, 49148), (5, 49148), (6, 49148), (7, 49150)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (outdatedmask): [[(4, 12600), (5, 12600), (6, 12600), (7, 9559)], [(4, 28318), (5, 28318), (6, 28318), (7, 21482)], [(4, 49148), (5, 49148), (6, 49148), (7, 49150)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (mask+norm): [[(4, 0.0809763432), (5, 0.0809763432), (6, 0.0809763432), (7, 0.065595707)], [(4, 0.1819911182), (5, 0.1819911182), (6, 0.1819911182), (7, 0.1474136391)], [(4, 0.3158591525), (5, 0.3158591525), (6, 0.3158591525), (7, 0.337276807)], [(4, 0.4211733856), (5, 0.4211733856), (6, 0.4211733856), (7, 0.4497138464)], [], [], [], []]
			ΔB: [[], [], [(7, 0.1200025633)], [(7, 0.1600034179)], [], [], [], []]
			ΔB (norm): [[], [], [(7, 0.4285714284)], [(7, 0.5714285714)], [], [], [], []]
			emaB: [[(4, 0.080976343), (5, 0.080976343), (6, 0.080976343), (7, 0.0590361361)], [(4, 0.181991118), (5, 0.181991118), (6, 0.181991118), (7, 0.1326722752)], [(4, 0.3158591525), (5, 0.3158591525), (6, 0.3158591525), (7, 0.3464062694)], [(4, 0.4211733858), (5, 0.4211733858), (6, 0.4211733858), (7, 0.4618853189)], [], [], [], []]
			D: [0.0590361361, 0.1326722752, 0.3464062694, 0.4618853189, 0, 0, 0, 0]
			nE: [0.029518068, 0.0663361375, 0.1732031347, 0.2309426593, 0, 0, 0, 0.5]
			E: [29518068, 66336137, 173203134, 230942659, 0, 0, 0, 500000000]
			P: [0.029518068, 0.0663361375, 0.1732031347, 0.2309426593, 0, 0, 0, 0.5]
			emaB: [[(4, 0.192263675), (5, 0.192263675), (6, 0.192263675), (7, 0.1278155716)], [(4, 0.4321049813), (5, 0.4321049813), (6, 0.4321049813), (7, 0.2872407278)], [(4, 0.7499504078), (5, 0.7499504078), (6, 0.7499504078), (7, 0.7499832863)], [(4, 1), (5, 1), (6, 1), (7, 1)], [], [], [], []] */
		let bonds = SubtensorModule::get_bonds( netuid );
		assert_eq!(bonds[0][7], 8376);
		assert_eq!(bonds[1][7], 18824);
		assert_eq!(bonds[2][7], 49150);
		assert_eq!(bonds[3][7], 65535);

		next_block();
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  current_block: 8
			B: [[(4, 12599), (5, 12599), (6, 12599), (7, 8376)], [(4, 28317), (5, 28317), (6, 28317), (7, 18824)], [(4, 49147), (5, 49147), (6, 49147), (7, 49150)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (outdatedmask): [[(4, 12599), (5, 12599), (6, 12599), (7, 8376)], [(4, 28317), (5, 28317), (6, 28317), (7, 18824)], [(4, 49147), (5, 49147), (6, 49147), (7, 49150)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
			B (mask+norm): [[(4, 0.0809714776), (5, 0.0809714776), (6, 0.0809714776), (7, 0.0590337245)], [(4, 0.1819882002), (5, 0.1819882002), (6, 0.1819882002), (7, 0.1326708249)], [(4, 0.3158588156), (5, 0.3158588156), (6, 0.3158588156), (7, 0.3464073015)], [(4, 0.421181506), (5, 0.421181506), (6, 0.421181506), (7, 0.4618881487)], [], [], [], []]
			ΔB: [[], [], [(7, 0.1200025633)], [(7, 0.1600034179)], [], [], [], []]
			ΔB (norm): [[], [], [(7, 0.4285714284)], [(7, 0.5714285714)], [], [], [], []]
			emaB: [[(4, 0.0809714776), (5, 0.0809714776), (6, 0.0809714776), (7, 0.053130352)], [(4, 0.1819882002), (5, 0.1819882002), (6, 0.1819882002), (7, 0.1194037423)], [(4, 0.3158588156), (5, 0.3158588156), (6, 0.3158588156), (7, 0.3546237142)], [(4, 0.4211815062), (5, 0.4211815062), (6, 0.4211815062), (7, 0.472842191)], [], [], [], []]
			D: [0.053130352, 0.1194037423, 0.3546237142, 0.472842191, 0, 0, 0, 0]
			nE: [0.026565176, 0.0597018711, 0.177311857, 0.2364210954, 0, 0, 0, 0.5]
			E: [26565175, 59701871, 177311856, 236421095, 0, 0, 0, 500000000]
			P: [0.026565176, 0.0597018711, 0.177311857, 0.2364210954, 0, 0, 0, 0.5]
			emaB: [[(4, 0.1922484161), (5, 0.1922484161), (6, 0.1922484161), (7, 0.1123638137)], [(4, 0.4320897225), (5, 0.4320897225), (6, 0.4320897225), (7, 0.2525234516)], [(4, 0.7499351487), (5, 0.7499351487), (6, 0.7499351487), (7, 0.7499832308)], [(4, 1), (5, 1), (6, 1), (7, 1)], [], [], [], []] */
	});
}

#[test]
fn test_bonds_with_liquid_alpha() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 8;
        let netuid: u16 = 1;
        let tempo: u16 = u16::MAX - 1; // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
        let max_stake: u64 = 4;
        let stakes: Vec<u64> = vec![1, 2, 3, 4, 0, 0, 0, 0];
        let block_number = System::block_number();
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        SubtensorModule::set_max_registrations_per_block(netuid, n);
        SubtensorModule::set_target_registrations_per_interval(netuid, n);
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_min_allowed_weights(netuid, 1);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);

        // Register validators and servers
        for key in 0..n as u64 {
            SubtensorModule::add_balance_to_coldkey_account(&U256::from(key), max_stake);
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                key * 1_000_000,
                &U256::from(key),
            );
            assert_ok!(SubtensorModule::register(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(key),
                U256::from(key)
            ));
            SubtensorModule::increase_stake_on_coldkey_hotkey_account(
                &U256::from(key),
                &U256::from(key),
                stakes[key as usize],
            );
        }

        // Initilize with first epoch
        SubtensorModule::epoch(netuid, 1_000_000_000);
        next_block();

        // Set weights
        for uid in 0..(n / 2) {
            SubtensorModule::set_validator_permit_for_uid(netuid, uid, true);
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![u16::MAX / 4, u16::MAX / 2, (u16::MAX / 4) * 3, u16::MAX],
                0
            ));
        }

        // Enable Liquid Alpha
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        // Run epoch with Liquid Alpha
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }

        // Check bonds and emissions
        let bonds = SubtensorModule::get_bonds(netuid);

        /*  n: 8
            current_block: 2; activity_cutoff: 5000;
            Last update: [1, 1, 1, 1, 0, 0, 0, 0]
            activity_cutoff: 5000
            Last update: [2, 2, 2, 2, 1, 1, 1, 1]
            Inactive: [false, false, false, false, false, false, false, false]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
            Stake: [1, 2, 3, 4, 0, 0, 0, 0]
            Normalised Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.099997558, 0.2000012202, 0.2999926745, 0.4000085443]
            Consensus: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            Weights: [[(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Validator Trust: [0.9999999995, 0.9999999995, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0.099997558, 0.2000012202, 0.2999926745, 0.4000085443]
            T: [0, 0, 0, 0, 1, 1, 1, 1]
            Incentive (=Rank): [0, 0, 0, 0, 0.0999975582, 0.2000012207,            0, 0.0999975582, 0.2000012207, 0.2999926752, 0.4000085455]
            B: [[], [], [], [], [], [], [], []]
            B (outdatedmask): [[], [], [], [], [], [], [], []]
            B (mask+norm): [[], [], [], [], [], [], [], []]
            ΔB: [[(4, 0.0099997558), (5, 0.020000122), (6, 0.0299992673), (7, 0.0400008543)], [(4, 0.0199995115), (5, 0.040000244), (6, 0.0599985349), (7, 0.0800017088)], [(4, 0.0299992673), (5, 0.060000366), (6, 0.0899978024), (7, 0.1200025633)], [(4, 0.0399990233), (5, 0.080000488), (6, 0.11999707), (7, 0.1600034179)], [], [], [], []]
            ΔB (norm): [[(4, 0.0999999996), (5, 0.0999999999), (6, 0.0999999994), (7, 0.0999999996)], [(4, 0.1999999995), (5, 0.2), (6, 0.1999999997), (7, 0.1999999997)], [(4, 0.299999999), (5, 0.2999999998), (6, 0.3), (7, 0.3)], [(4, 0.4000000013), (5, 0.4), (6, 0.4000000004), (7, 0.4000000001)], [], [], [], []]
            Exponential Moving Average Bonds Liquid Alpha: [[(4, 0.0499983232), (5, 0.0899999999), (6, 0.0899999994), (7, 0.0899999996)], [(4, 0.0999966469), (5, 0.18), (6, 0.1799999997), (7, 0.1799999997)], [(4, 0.1499949703), (5, 0.2699999998), (6, 0.2699999998), (7, 0.2699999998)], [(4, 0.199993295), (5, 0.3599999999), (6, 0.36), (7, 0.3599999999)], [], [], [], []]
            Exponential Moving Average Bonds: [[(4, 0.0999999992), (5, 0.0999999999), (6, 0.0999999994), (7, 0.0999999996)], [(4, 0.1999999995), (5, 0.2), (6, 0.1999999997), (7, 0.1999999997)], [(4, 0.2999999993), (5, 0.2999999998), (6, 0.3), (7, 0.3)], [(4, 0.4000000015), (5, 0.4), (6, 0.4000000004), (7, 0.4000000001)], [], [], [], []]
            Dividends: [0.0999999994, 0.1999999997, 0.3, 0.4000000006, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0.049998779, 0.1000006103, 0.1499963375, 0.2000042726]
            Server Emission: [0, 0, 0, 0, 49998779, 100000610, 149996337, 200004272]
            Normalized Validator Emission: [0.0499999996, 0.0999999999, 0.15, 0.2000000002, 0, 0, 0, 0]
            Validator Emission: [49999999, 99999999, 149999999, 200000000, 0, 0, 0, 0]
            Normalized Combined Emission: [0.0499999996, 0.0999999999, 0.15, 0.2000000002, 0.049998779, 0.1000006103, 0.1499963375, 0.2000042726]
            Combined Emission: [49999999, 99999999, 149999999, 200000000, 49998779, 100000610, 149996337, 200004272]
            Pruning Scores: [0.0499999996, 0.0999999999, 0.15, 0.2000000002, 0.049998779, 0.1000006103, 0.1499963375, 0.2000042726]
        */

        // Expected bonds calculations
        // For uid 0:
        // Initial weights: [0.25, 0.5, 0.75, 1.0]
        // Active stake: [1, 2, 3, 4]
        // ΔB = W◦S = [0.25*1, 0.5*2, 0.75*3, 1.0*4] = [0.25, 1.0, 2.25, 4.0]
        // Normalize ΔB: [0.25/7.5, 1.0/7.5, 2.25/7.5, 4.0/7.5] = [0.0333, 0.1333, 0.3, 0.5333]
        // Final bonds for netuid: [16383, 32767, 49151, 65535]

        assert_eq!(bonds[0][4], 16383); // Note: Calculated as explained above
        assert_eq!(bonds[1][4], 32767); // Note: Calculated as explained above
        assert_eq!(bonds[2][4], 49151); // Note: Calculated as explained above
        assert_eq!(bonds[3][4], 65535); // Note: Calculated as explained above

        // === Set self-weight only on val1
        let uid = 0;
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid)),
            netuid,
            vec![uid],
            vec![u16::MAX],
            0
        ));
        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }

        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][4], 2862);
        assert_eq!(bonds[1][4], 32767);
        assert_eq!(bonds[2][4], 49151);
        assert_eq!(bonds[3][4], 65535);

        // === Set self-weight only on val2
        let uid = 1;
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid)),
            netuid,
            vec![uid],
            vec![u16::MAX],
            0
        ));
        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        let bonds = SubtensorModule::get_bonds(netuid);

        /*  n: 8
            current_block: 4; activity_cutoff: 5000;
            Last update: [2, 3, 2, 2, 1, 1, 1, 1]
            Inactive: [false, false, false, false, false, false, false, false]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
            Stake: [1, 2, 3, 4, 0, 0, 0, 0]
            Normalised Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 64
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            Consensus: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            Weights: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Validator Trust: [0, 0, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            T: [0, 0, 0, 0, 1, 1, 1, 1]
            Incentive (=Rank): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            B: [[(4, 7760), (5, 1489), (6, 1489), (7, 1489)], [(4, 32767), (5, 32767), (6, 32767), (7, 32767)], [(4, 49151), (5, 49151), (6, 49151), (7, 49151)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
            B (outdatedmask): [[(4, 7760), (5, 1489), (6, 1489), (7, 1489)], [(4, 32767), (5, 32767), (6, 32767), (7, 32767)], [(4, 49151), (5, 49151), (6, 49151), (7, 49151)], [(4, 65535), (5, 65535), (6, 65535), (7, 65535)], [], [], [], []]
            B (mask+norm): [[(4, 0.0499958121), (5, 0.00999718), (6, 0.00999718), (7, 0.00999718)], [(4, 0.211109894), (5, 0.2199983886), (6, 0.2199983886), (7, 0.2199983886)], [(4, 0.3166680625), (5, 0.3300009398), (6, 0.3300009398), (7, 0.3300009398)], [(4, 0.4222262308), (5, 0.4400034912), (6, 0.4400034912), (7, 0.4400034912)], [], [], [], []]
            ΔB: [[], [], [(4, 0.0299992673), (5, 0.060000366), (6, 0.0899978024), (7, 0.1200025633)], [(4, 0.0399990233), (5, 0.080000488), (6, 0.11999707), (7, 0.1600034179)], [], [], [], []]
            ΔB (norm): [[], [], [(4, 0.428571427), (5, 0.4285714284), (6, 0.4285714284), (7, 0.4285714284)], [(4, 0.5714285728), (5, 0.5714285714), (6, 0.5714285714), (7, 0.5714285714)], [], [], [], []]
            Exponential Moving Average Bonds Liquid Alpha: [[(4, 0.024998744), (5, 0.000999718), (6, 0.000999718), (7, 0.000999718)], [(4, 0.105558486), (5, 0.0219998388), (6, 0.0219998388), (7, 0.0219998388)], [(4, 0.3726178685), (5, 0.4187143792), (6, 0.4187143792), (7, 0.4187143792)], [(4, 0.4968249004), (5, 0.5582860631), (6, 0.5582860631), (7, 0.5582860631)], [], [], [], []]
            Exponential Moving Average Bonds: [[(4, 0.024998744), (5, 0.000999718), (6, 0.000999718), (7, 0.000999718)], [(4, 0.105558486), (5, 0.0219998388), (6, 0.0219998388), (7, 0.0219998388)], [(4, 0.3726178687), (5, 0.4187143794), (6, 0.4187143794), (7, 0.4187143794)], [(4, 0.4968249009), (5, 0.5582860636), (6, 0.5582860636), (7, 0.5582860636)], [], [], [], []]
            Dividends: [0.0033995616, 0.030355499, 0.4141048414, 0.5521400978, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Server Emission: [0, 0, 0, 0, 49998779, 100000610, 149996337, 200004272]
            Normalized Validator Emission: [0.0016997808, 0.0151777493, 0.2070524206, 0.2760700488, 0, 0, 0, 0]
            Validator Emission: [1699780, 15177749, 207052420, 276070048, 0, 0, 0, 0]
            Normalized Combined Emission: [0.0016997808, 0.0151777493, 0.2070524206, 0.2760700488, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Combined Emission: [1699780, 15177749, 207052420, 276070048, 49998779, 100000610, 149996337, 200004272]
            Pruning Scores: [0.0016997808, 0.0151777493, 0.2070524206, 0.2760700488, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
        */

        assert_eq!(bonds[0][4], 435);
        assert_eq!(bonds[1][4], 4985);
        assert_eq!(bonds[2][4], 49151);
        assert_eq!(bonds[3][4], 65535);
    });
}

#[test]
fn test_set_alpha_disabled() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey: U256 = U256::from(1);
        let coldkey: U256 = U256::from(1 + 456);
        let signer = <<Test as Config>::RuntimeOrigin>::signed(coldkey);

        // Enable Liquid Alpha and setup
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        migration::migrate_create_root_network::<Test>();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_000);
        assert_ok!(SubtensorModule::root_register(signer.clone(), hotkey,));
        assert_ok!(SubtensorModule::add_stake(signer.clone(), hotkey, 1000));
        // Only owner can set alpha values
        assert_ok!(SubtensorModule::register_network(signer.clone()));

        // Explicitly set to false
        SubtensorModule::set_liquid_alpha_enabled(netuid, false);
        assert_err!(
            SubtensorModule::do_set_alpha_values(signer.clone(), netuid, 12_u16, u16::MAX),
            Error::<Test>::LiquidAlphaDisabled
        );

        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            12_u16,
            u16::MAX
        ));
    });
}

// Test that epoch masks out inactive stake of validators with outdated weights beyond activity cutoff.
#[test]
fn test_active_stake() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        let sparse: bool = true;
        let n: u16 = 4;
        let netuid: u16 = 1;
        let tempo: u16 = u16::MAX - 1; // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
        let block_number: u64 = System::block_number();
        let stake: u64 = 1;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
        SubtensorModule::set_max_registrations_per_block(netuid, n);
        SubtensorModule::set_target_registrations_per_interval(netuid, n);
        SubtensorModule::set_min_allowed_weights(netuid, 0);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);

        // === Register [validator1, validator2, server1, server2]
        for key in 0..n as u64 {
            SubtensorModule::add_balance_to_coldkey_account(&U256::from(key), stake);
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                key * 1_000_000,
                &U256::from(key),
            );
            assert_ok!(SubtensorModule::register(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(key),
                U256::from(key)
            ));
            SubtensorModule::increase_stake_on_coldkey_hotkey_account(
                &U256::from(key),
                &U256::from(key),
                stake,
            );
        }
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

        // === Issue validator permits
        SubtensorModule::set_max_allowed_validators(netuid, n);
        assert_eq!(SubtensorModule::get_max_allowed_validators(netuid), n);
        SubtensorModule::epoch(netuid, 1_000_000_000); // run first epoch to set allowed validators
        next_block(); // run to next block to ensure weights are set on nodes after their registration block

        // === Set weights [val1->srv1: 0.5, val1->srv2: 0.5, val2->srv1: 0.5, val2->srv2: 0.5]
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![u16::MAX / (n / 2); (n / 2) as usize],
                0
            ));
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        let bonds = SubtensorModule::get_bonds(netuid);
        for uid in 0..n {
            // log::info!("\n{uid}" );
            // uid_stats(netuid, uid);
            // log::info!("bonds: {:?}", bonds[uid as usize]);
            if uid < n / 2 {
                assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 32767);
                // Note D = floor(0.5 * 65_535)
            }
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, uid),
                250000000
            ); // Note E = 0.5 / (n/2) * 1_000_000_000 = 250_000_000
        }
        for bond in bonds.iter().take((n / 2) as usize) {
            // for on_validator in 0..(n / 2) as usize {
            for i in bond.iter().take((n / 2) as usize) {
                assert_eq!(*i, 0);
            }
            for i in bond.iter().take(n as usize).skip((n / 2) as usize) {
                assert_eq!(*i, I32F32::from_num(65_535)); // floor(0.5*(2^16-1))/(2^16-1), then max-upscale to 65_535
            }
        }
        let activity_cutoff: u64 = SubtensorModule::get_activity_cutoff(netuid) as u64;
        run_to_block(activity_cutoff + 2); // run to block where validator (uid 0, 1) weights become outdated

        // === Update uid 0 weights
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(0)),
            netuid,
            ((n / 2)..n).collect(),
            vec![u16::MAX / (n / 2); (n / 2) as usize],
            0
        ));
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  current_block: 5002; activity_cutoff: 5000
        Last update: [5002, 1, 0, 0]; Inactive: [false, true, true, true]; Block at registration: [0, 0, 0, 0]
        S: [0.25, 0.25, 0.25, 0.25]; S (mask): [0.25, 0, 0, 0]; S (mask+norm): [1, 0, 0, 0]
        validator_permits: [true, true, true, true]; max_allowed_validators: 4; new_validator_permits: [true, true, true, true]
        W: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
        W (permit): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
        W (permit+diag): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
        W (permit+diag+outdate): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
        W (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
        R: [0, 0, 0.5, 0.5]
        W (threshold): [[(2, 1), (3, 1)], [(2, 1), (3, 1)], [], []]
        T: [0, 0, 1, 1]
        C: [0.006693358, 0.006693358, 0.9933076561, 0.9933076561]
        I: [0, 0, 0.5, 0.5]
        B: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
        B (outdatedmask): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
        B (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
        ΔB: [[(2, 0.5), (3, 0.5)], [(2, 0), (3, 0)], [], []]
        ΔB (norm): [[(2, 1), (3, 1)], [(2, 0), (3, 0)], [], []]
        emaB: [[(2, 0.55), (3, 0.55)], [(2, 0.45), (3, 0.45)], [], []]
        emaB (max-upscale): [[(2, 1), (3, 1)], [(2, 1), (3, 1)], [], []]
        D: [0.55, 0.4499999997, 0, 0]
        nE: [0.275, 0.2249999999, 0.25, 0.25]
        E: [274999999, 224999999, 250000000, 250000000]
        P: [0.275, 0.2249999999, 0.25, 0.25]
        P (u16): [65535, 53619, 59577, 59577] */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 0), 36044); // Note D = floor((0.5 * 0.9 + 0.1) * 65_535)
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 0), 274999999); // Note E = 0.5 * 0.55 * 1_000_000_000 = 275_000_000 (discrepancy)
        for server in ((n / 2) as usize)..n as usize {
            assert_eq!(bonds[0][server], I32F32::from_num(65_535)); // floor(0.55*(2^16-1))/(2^16-1), then max-upscale
        }
        for validator in 1..(n / 2) {
            assert_eq!(
                SubtensorModule::get_dividends_for_uid(netuid, validator),
                29490
            ); // Note D = floor((0.5 * 0.9) * 65_535)
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, validator),
                224999999
            ); // Note E = 0.5 * 0.45 * 1_000_000_000 = 225_000_000 (discrepancy)
            for server in ((n / 2) as usize)..n as usize {
                assert_eq!(bonds[validator as usize][server], I32F32::from_num(53619));
                // floor(0.45*(2^16-1))/(2^16-1), then max-upscale
            }
        }

        // === Update uid 1 weights as well
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(1)),
            netuid,
            ((n / 2)..n).collect(),
            vec![u16::MAX / (n / 2); (n / 2) as usize],
            0
        ));
        run_to_block(activity_cutoff + 3); // run to block where validator (uid 0, 1) weights become outdated
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  current_block: 5003; activity_cutoff: 5000
        Last update: [5002, 5002, 0, 0]; Inactive: [false, false, true, true]; Block at registration: [0, 0, 0, 0]
        S: [0.25, 0.25, 0.25, 0.25]; S (mask): [0.25, 0.25, 0, 0]; S (mask+norm): [0.5, 0.5, 0, 0]
        validator_permits: [true, true, true, true]; max_allowed_validators: 4; new_validator_permits: [true, true, true, true]
        W: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
        W (permit): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
        W (permit+diag): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
        W (permit+diag+outdate): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
        W (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
        R: [0, 0, 0.5, 0.5]
        W (threshold): [[(2, 1), (3, 1)], [(2, 1), (3, 1)], [], []]
        T: [0, 0, 1, 1]
        C: [0.006693358, 0.006693358, 0.9933076561, 0.9933076561]
        I: [0, 0, 0.5, 0.5]
        B: [[(2, 65535), (3, 65535)], [(2, 53619), (3, 53619)], [], []]
        B (outdatedmask): [[(2, 65535), (3, 65535)], [(2, 53619), (3, 53619)], [], []]
        B (mask+norm): [[(2, 0.5500025176), (3, 0.5500025176)], [(2, 0.4499974821), (3, 0.4499974821)], [], []]
        ΔB: [[(2, 0.25), (3, 0.25)], [(2, 0.25), (3, 0.25)], [], []]
        ΔB (norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
        emaB: [[(2, 0.545002266), (3, 0.545002266)], [(2, 0.4549977337), (3, 0.4549977337)], [], []]
        emaB (max-upscale): [[(2, 1), (3, 1)], [(2, 0.8348547556), (3, 0.8348547556)], [], []]
        D: [0.545002266, 0.4549977337, 0, 0]
        nE: [0.272501133, 0.2274988669, 0.25, 0.25]
        E: [272501132, 227498866, 250000000, 250000000]
        P: [0.272501133, 0.2274988669, 0.25, 0.25]
        P (u16): [65535, 54711, 60123, 60123] */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 0), 35716); // Note D = floor((0.55 * 0.9 + 0.5 * 0.1) * 65_535)
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 0), 272501132); // Note E = 0.5 * (0.55 * 0.9 + 0.5 * 0.1) * 1_000_000_000 = 272_500_000 (discrepancy)
        for server in ((n / 2) as usize)..n as usize {
            assert_eq!(bonds[0][server], I32F32::from_num(65_535)); // floor((0.55 * 0.9 + 0.5 * 0.1)*(2^16-1))/(2^16-1), then max-upscale
        }
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 1), 29818); // Note D = floor((0.45 * 0.9 + 0.5 * 0.1) * 65_535)
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 1), 227498866); // Note E = 0.5 * (0.45 * 0.9 + 0.5 * 0.1) * 1_000_000_000 = 227_500_000 (discrepancy)
        for server in ((n / 2) as usize)..n as usize {
            assert_eq!(bonds[1][server], I32F32::from_num(54712)); // floor((0.45 * 0.9 + 0.5 * 0.1)/(0.55 * 0.9 + 0.5 * 0.1)*(2^16-1))
        }
    });
}

// Test that epoch masks out outdated weights and bonds of validators on deregistered servers.
#[test]
fn test_outdated_weights() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 4;
        let netuid: u16 = 1;
        let tempo: u16 = u16::MAX - 1; // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
        let mut block_number: u64 = System::block_number();
        let stake: u64 = 1;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, n);
        SubtensorModule::set_target_registrations_per_interval(netuid, n);
        SubtensorModule::set_min_allowed_weights(netuid, 0);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 0);

        // === Register [validator1, validator2, server1, server2]
        for key in 0..n as u64 {
            SubtensorModule::add_balance_to_coldkey_account(&U256::from(key), stake);
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                key * 1_000_000,
                &U256::from(key),
            );
            assert_ok!(SubtensorModule::register(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(key),
                U256::from(key)
            ));
            SubtensorModule::increase_stake_on_coldkey_hotkey_account(
                &U256::from(key),
                &U256::from(key),
                stake,
            );
        }
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 4);

        // === Issue validator permits
        SubtensorModule::set_max_allowed_validators(netuid, n);
        assert_eq!(SubtensorModule::get_max_allowed_validators(netuid), n);
        SubtensorModule::epoch(netuid, 1_000_000_000); // run first epoch to set allowed validators
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 4);
        block_number = next_block(); // run to next block to ensure weights are set on nodes after their registration block
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 0);

        // === Set weights [val1->srv1: 2/3, val1->srv2: 1/3, val2->srv1: 2/3, val2->srv2: 1/3, srv1->srv1: 1, srv2->srv2: 1]
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![2 * (u16::MAX / 3), u16::MAX / 3],
                0
            ));
        }
        for uid in ((n / 2) as u64)..n as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                vec![uid as u16],
                vec![u16::MAX],
                0
            )); // server self-weight
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  current_block: 1; activity_cutoff: 5000
        Last update: [1, 1, 1, 1]; Inactive: [false, false, false, false]; Block at registration: [0, 0, 0, 0]
        S: [0.25, 0.25, 0.25, 0.25]; S (mask): [0.25, 0.25, 0.25, 0.25]; S (mask+norm): [0.25, 0.25, 0.25, 0.25]
        validator_permits: [true, true, true, true]; max_allowed_validators: 4; new_validator_permits: [true, true, true, true]
        W: [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
        W (permit): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
        W (permit+diag): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [], []]
        W (permit+diag+outdate): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [], []]
        W (mask+norm): [[(2, 0.6666632756), (3, 0.3333367242)], [(2, 0.6666632756), (3, 0.3333367242)], [], []]
        R (before): [0, 0, 0.3333316376, 0.166668362]
        C: [0, 0, 0.6666632756, 0.3333367242]
        W: [[(2, 0.6666632756), (3, 0.3333367242)], [(2, 0.6666632756), (3, 0.3333367242)], [], []]
        Tv: [0.9999999998, 0.9999999998, 0, 0]
        R (after): [0, 0, 0.3333316376, 0.166668362]
        T: [0, 0, 1, 1]
        I (=R): [0, 0, 0.6666632756, 0.3333367242]
        B: [[], [], [], []]
        B (outdatedmask): [[], [], [], []]
        B (mask+norm): [[], [], [], []]
        ΔB: [[(2, 0.1666658188), (3, 0.083334181)], [(2, 0.1666658188), (3, 0.083334181)], [], []]
        ΔB (norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
        emaB: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
        D: [0.5, 0.5, 0, 0]
        nE: [0.25, 0.25, 0.3333316378, 0.166668362]
        E: [250000000, 250000000, 333331637, 166668361]
        P: [0.25, 0.25, 0.3333316378, 0.166668362]
        P (u16): [49151, 49151, 65535, 32767] */

        // === Dereg server2 at uid3 (least emission) + register new key over uid3
        let new_key: u64 = n as u64; // register a new key while at max capacity, which means the least incentive uid will be deregistered
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &U256::from(new_key),
        );
        assert_eq!(System::block_number(), block_number);
        assert_eq!(SubtensorModule::get_max_registrations_per_block(netuid), n);
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 0);
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(U256::from(new_key)),
            netuid,
            block_number,
            nonce,
            work,
            U256::from(new_key),
            U256::from(new_key)
        ));
        let deregistered_uid: u16 = n - 1; // since uid=n-1 only recieved 1/3 of weight, it will get pruned first
        assert_eq!(
            U256::from(new_key),
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, deregistered_uid)
                .expect("Not registered")
        );
        next_block(); // run to next block to outdate weights and bonds set on deregistered uid

        // === Update weights from only uid=0
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(0)),
            netuid,
            ((n / 2)..n).collect(),
            vec![2 * (u16::MAX / 3), u16::MAX / 3],
            0
        ));
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  current_block: 2; activity_cutoff: 5000
        Last update: [2, 1, 1, 1]; Inactive: [false, false, false, false]; Block at registration: [0, 0, 0, 1]
        S: [0.3333333333, 0.3333333333, 0.3333333333, 0]
        S (mask): [0.3333333333, 0.3333333333, 0.3333333333, 0]
        S (mask+norm): [0.3333333333, 0.3333333333, 0.3333333333, 0]
        validator_permits: [true, true, true, false]; max_allowed_validators: 4; new_validator_permits: [true, true, true, true]
        W: [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
        W (permit): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
        W (permit+diag): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [], []]
        W (permit+diag+outdate): [[(2, 65535), (3, 32768)], [(2, 65535)], [], []]
        W (mask+norm): [[(2, 0.6666632756), (3, 0.3333367242)], [(2, 1)], [], []]
        R (before): [0, 0, 0.5555544249, 0.1111122412]
        C: [0, 0, 0.6666632756, 0]
        W: [[(2, 0.6666632756)], [(2, 0.6666632756)], [], []]
        Tv: [0.6666632756, 0.6666632756, 0, 0]
        R (after): [0, 0, 0.4444421832, 0]
        T: [0, 0, 0.799997558, 0]
        I (=R): [0, 0, 1, 0]
        B: [[(2, 65535), (3, 65535)], [(2, 65535), (3, 65535)], [], []]
        B (outdatedmask): [[(2, 65535), (3, 65535)], [(2, 65535)], [], []]
        B (mask+norm): [[(2, 0.5), (3, 1)], [(2, 0.5)], [], []]
        ΔB: [[(2, 0.2222210916)], [(2, 0.2222210916)], [], []]
        ΔB (norm): [[(2, 0.5)], [(2, 0.5)], [], []]
        emaB: [[(2, 0.5), (3, 1)], [(2, 0.5)], [], []]
        emaB (max-upscale): [[(2, 1), (3, 1)], [(2, 1)], [], []]
        D: [0.5, 0.5, 0, 0]
        nE: [0.25, 0.25, 0.5, 0]
        E: [250000000, 250000000, 500000000, 0]
        P: [0.25, 0.25, 0.5, 0]
        P (u16): [32767, 32767, 65535, 0] */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 0), 32767); // Note D = floor(0.5 * 65_535)
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 0), 250000000); // Note E = 0.5 * 0.5 * 1_000_000_000 = 249311245
        assert_eq!(bonds[0][2], I32F32::from_num(65_535)); // floor(0.5*(2^16-1))/(2^16-1), then max-upscale
        assert_eq!(bonds[0][3], I32F32::from_num(65_535)); // only uid0 has updated weights for new reg
    });
}

// Test the zero emission handling and fallback under zero effective weight conditions, to ensure non-zero effective emission.
#[test]
fn test_zero_weights() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 2;
        let netuid: u16 = 1;
        let tempo: u16 = u16::MAX - 1; // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
        let mut block_number: u64 = 0;
        let stake: u64 = 1;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, n);
        SubtensorModule::set_target_registrations_per_interval(netuid, n);
        SubtensorModule::set_min_allowed_weights(netuid, 0);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);

        // === Register [validator, server]
        for key in 0..n as u64 {
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                key * 1_000_000,
                &U256::from(key),
            );
            assert_ok!(SubtensorModule::register(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(key),
                U256::from(key)
            ));
        }
        for validator in 0..(n / 2) as u64 {
            SubtensorModule::add_balance_to_coldkey_account(&U256::from(validator), stake);
            SubtensorModule::increase_stake_on_coldkey_hotkey_account(
                &U256::from(validator),
                &U256::from(validator),
                stake,
            );
        }
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

        // === No weights
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*	current_block: 0; activity_cutoff: 5000; Last update: [0, 0]; Inactive: [false, false]
        S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 0]
        W: [[], []]; W (diagmask): [[], []]; W (diag+outdatemask): [[], []]; W (mask+norm): [[], []]
        R: [0, 0]; W (threshold): [[], []]; T: [0, 0]; C: [0.006693358, 0.006693358]; I: [0, 0]
        B: [[], []]; B (outdatedmask): [[], []]; B (mask+norm): [[], []];
        ΔB: [[], []]; ΔB (norm): [[], []]; emaB: [[], []]; D: [0, 0]
        E: [1000000000, 0]; P: [1, 0] */
        for validator in 0..(n / 2) {
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, validator),
                1000000000
            ); // Note E = 1 * 1_000_000_000
        }
        for server in (n / 2)..n {
            assert_eq!(SubtensorModule::get_emission_for_uid(netuid, server), 0);
            // no stake
        }
        run_to_block(1);
        block_number += 1; // run to next block to ensure weights are set on nodes after their registration block

        // === Self-weights only: set weights [srv->srv: 1]
        for uid in ((n / 2) as u64)..n as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                vec![uid as u16],
                vec![u16::MAX],
                0
            )); // server self-weight
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*	current_block: 1; activity_cutoff: 5000; Last update: [0, 1]; Inactive: [false, false]
        S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 0]
        W: [[], [(1, 1)]]
        W (diagmask): [[], []]; W (diag+outdatemask): [[], []]; W (mask+norm): [[], []]
        R: [0, 0]; W (threshold): [[], []]; T: [0, 0]; C: [0.006693358, 0.006693358]; I: [0, 0]
        B: [[], []]: B (outdatedmask): [[], []]; B (mask+norm): [[], []]
        ΔB: [[], []]; ΔB (norm): [[], []]; emaB: [[], []]; D: [0, 0]
        E: [1000000000, 0]; P: [1, 0] */
        for validator in 0..(n / 2) {
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, validator),
                1000000000
            ); // Note E = 1 * 1_000_000_000
        }
        for server in (n / 2)..n {
            assert_eq!(SubtensorModule::get_emission_for_uid(netuid, server), 0);
            // no stake
        }
        run_to_block(2);
        block_number += 1;

        // === Set weights [val->srv: 1/(n/2)]
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![u16::MAX / (n / 2); (n / 2) as usize],
                0
            ));
        }

        // === Outdate weights by reregistering servers
        for new_key in n..n + (n / 2) {
            // register a new key while at max capacity, which means the least emission uid will be deregistered
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                new_key as u64 * 1_000_000,
                &(U256::from(new_key)),
            );
            assert_ok!(SubtensorModule::register(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(new_key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(new_key),
                U256::from(new_key)
            ));
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*	current_block: 2; activity_cutoff: 5000; Last update: [2, 1]; Inactive: [false, false];
        S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 2];
        W: [[(1, 1)], []]; W (diagmask): [[(1, 1)], []]; W (diag+outdatemask): [[], []]; W (mask+norm): [[], []];
        R: [0, 0]; W (threshold): [[], []]; T: [0, 0]; C: [0.006693358, 0.006693358]; I: [0, 0];
        B: [[], []]; B (outdatedmask): [[], []]; B (mask+norm): [[], []];
        ΔB: [[], []]; ΔB (norm): [[], []]; emaB: [[], []]; D: [0, 0];
        E: [1000000000, 0]; P: [1, 0] */
        for validator in 0..(n / 2) {
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, validator),
                1000000000
            ); // Note E = 1 * 1_000_000_000
        }
        for server in (n / 2)..n {
            assert_eq!(SubtensorModule::get_emission_for_uid(netuid, server), 0);
            // no stake
        }
        run_to_block(3);

        // === Set new weights [val->srv: 1/(n/2)] to check that updated weights would produce non-zero incentive
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![u16::MAX / (n / 2); (n / 2) as usize],
                0
            ));
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*	current_block: 3; activity_cutoff: 5000; Last update: [3, 1]; Inactive: [false, false];
        S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 2];
        W: [[(1, 1)], []]; W (diagmask): [[(1, 1)], []]; W (diag+outdatemask): [[(1, 1)], []]; W (mask+norm): [[(1, 1)], []];
        R: [0, 1]; W (threshold): [[(1, 1)], []]; T: [0, 1]; C: [0.006693358, 0.9933076561]; I: [0, 1];
        B: [[], []]; B (outdatedmask): [[], []]; B (mask+norm): [[], []];
        ΔB: [[(1, 1)], []]; ΔB (norm): [[(1, 1)], []]; emaB: [[(1, 1)], []]; D: [1, 0]; emaB (max-upscale): [[(1, 1)], []]
        E: [500000000, 500000000]; P: [0.5, 0.5] */
        for validator in 0..n {
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, validator),
                1000000000 / (n as u64)
            ); // Note E = 1/2 * 1_000_000_000
        }
    });
}

// Test that epoch assigns validator permits to highest stake uids, varies uid interleaving and stake values.
#[test]
#[cfg(not(tarpaulin))]
fn test_validator_permits() {
    let netuid: u16 = 1;
    let tempo: u16 = u16::MAX - 1; // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
    for interleave in 0..3 {
        for (network_n, validators_n) in [(2, 1), (4, 2), (8, 4)] {
            for assignment in 0..=1 {
                let (validators, servers) =
                    distribute_nodes(validators_n as usize, network_n, interleave as usize);
                let correct: bool = true;
                let mut stake: Vec<u64> = vec![0; network_n];
                for validator in &validators {
                    stake[*validator as usize] = match assignment {
                        1 => *validator as u64 + network_n as u64,
                        _ => 1,
                    };
                }
                for server in &servers {
                    stake[*server as usize] = match assignment {
                        1 => *server as u64,
                        _ => 0,
                    };
                }
                new_test_ext(1).execute_with(|| {
                    let block_number: u64 = 0;
                    add_network(netuid, tempo, 0);
                    SubtensorModule::set_max_allowed_uids(netuid, network_n as u16);
                    assert_eq!(
                        SubtensorModule::get_max_allowed_uids(netuid),
                        network_n as u16
                    );
                    SubtensorModule::set_max_registrations_per_block(netuid, network_n as u16);
                    SubtensorModule::set_target_registrations_per_interval(
                        netuid,
                        network_n as u16,
                    );

                    // === Register [validator1, validator2, server1, server2]
                    for key in 0..network_n as u64 {
                        SubtensorModule::add_balance_to_coldkey_account(
                            &U256::from(key),
                            stake[key as usize],
                        );
                        let (nonce, work): (u64, Vec<u8>) =
                            SubtensorModule::create_work_for_block_number(
                                netuid,
                                block_number,
                                key * 1_000_000,
                                &U256::from(key),
                            );
                        assert_ok!(SubtensorModule::register(
                            <<Test as Config>::RuntimeOrigin>::signed(U256::from(key)),
                            netuid,
                            block_number,
                            nonce,
                            work,
                            U256::from(key),
                            U256::from(key)
                        ));
                        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
                            &U256::from(key),
                            &U256::from(key),
                            stake[key as usize],
                        );
                    }
                    assert_eq!(SubtensorModule::get_subnetwork_n(netuid), network_n as u16);

                    // === Issue validator permits
                    SubtensorModule::set_max_allowed_validators(netuid, validators_n as u16);
                    assert_eq!(
                        SubtensorModule::get_max_allowed_validators(netuid),
                        validators_n as u16
                    );
                    SubtensorModule::epoch(netuid, 1_000_000_000); // run first epoch to set allowed validators
                    for validator in &validators {
                        assert_eq!(
                            correct,
                            SubtensorModule::get_validator_permit_for_uid(netuid, *validator)
                        );
                    }
                    for server in &servers {
                        assert_eq!(
                            !correct,
                            SubtensorModule::get_validator_permit_for_uid(netuid, *server)
                        );
                    }

                    // === Increase server stake above validators
                    for server in &servers {
                        SubtensorModule::add_balance_to_coldkey_account(
                            &(U256::from(*server as u64)),
                            2 * network_n as u64,
                        );
                        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
                            &(U256::from(*server as u64)),
                            &(U256::from(*server as u64)),
                            2 * network_n as u64,
                        );
                    }

                    // === Update validator permits
                    run_to_block(1);
                    SubtensorModule::epoch(netuid, 1_000_000_000);

                    // === Check that servers now own permits instead of the validator uids
                    for validator in &validators {
                        assert_eq!(
                            !correct,
                            SubtensorModule::get_validator_permit_for_uid(netuid, *validator)
                        );
                    }
                    for server in &servers {
                        assert_eq!(
                            correct,
                            SubtensorModule::get_validator_permit_for_uid(netuid, *server)
                        );
                    }
                });
            }
        }
    }
}

#[test]
fn test_compute_alpha_values() {
    // Define the consensus values.
    let consensus = vec![
        I32F32::from_num(0.1),
        I32F32::from_num(0.5),
        I32F32::from_num(0.9),
    ];
    // Define the logistic function parameters 'a' and 'b'.
    let a = I32F32::from_num(1.0);
    let b = I32F32::from_num(0.0);

    // Compute the alpha values using the function.
    let alpha = SubtensorModule::compute_alpha_values(&consensus, a, b);

    // Ensure the length of the alpha vector matches the consensus vector.
    assert_eq!(alpha.len(), consensus.len());

    // Manually compute the expected alpha values for each consensus value.
    // The logistic function is: 1 / (1 + exp(b - a * c))
    // where c is the consensus value.

    // For consensus[0] = 0.1:
    // exp_val = exp(0.0 - 1.0 * 0.1) = exp(-0.1)
    // alpha[0] = 1 / (1 + exp(-0.1)) ~ 0.9048374180359595
    let exp_val_0 = I32F32::from_num(0.9048374180359595);
    let expected_alpha_0 =
        I32F32::from_num(1.0).saturating_div(I32F32::from_num(1.0).saturating_add(exp_val_0));

    // For consensus[1] = 0.5:
    // exp_val = exp(0.0 - 1.0 * 0.5) = exp(-0.5)
    // alpha[1] = 1 / (1 + exp(-0.5)) ~ 0.6065306597126334
    let exp_val_1 = I32F32::from_num(0.6065306597126334);
    let expected_alpha_1 =
        I32F32::from_num(1.0).saturating_div(I32F32::from_num(1.0).saturating_add(exp_val_1));

    // For consensus[2] = 0.9:
    // exp_val = exp(0.0 - 1.0 * 0.9) = exp(-0.9)
    // alpha[2] = 1 / (1 + exp(-0.9)) ~ 0.4065696597405991
    let exp_val_2 = I32F32::from_num(0.4065696597405991);
    let expected_alpha_2 =
        I32F32::from_num(1.0).saturating_div(I32F32::from_num(1.0).saturating_add(exp_val_2));

    // Define an epsilon for approximate equality checks.
    let epsilon = I32F32::from_num(1e-6);

    // Assert that the computed alpha values match the expected values within the epsilon.
    assert_approx_eq(alpha[0], expected_alpha_0, epsilon);
    assert_approx_eq(alpha[1], expected_alpha_1, epsilon);
    assert_approx_eq(alpha[2], expected_alpha_2, epsilon);
}

#[test]
fn test_compute_alpha_values_256_miners() {
    // Define the consensus values for 256 miners.
    let consensus: Vec<I32F32> = (0..256)
        .map(|i| I32F32::from_num(i as f32 / 255.0))
        .collect();
    // Define the logistic function parameters 'a' and 'b'.
    let a = I32F32::from_num(1.0);
    let b = I32F32::from_num(0.0);

    // Compute the alpha values using the function.
    let alpha = SubtensorModule::compute_alpha_values(&consensus, a, b);

    // Ensure the length of the alpha vector matches the consensus vector.
    assert_eq!(alpha.len(), consensus.len());

    // Define an epsilon for approximate equality checks.
    let epsilon = I32F32::from_num(1e-6);

    for (i, &c) in consensus.iter().enumerate() {
        // Use saturating subtraction and multiplication
        let exponent = b.saturating_sub(a.saturating_mul(c));

        // Use safe_exp instead of exp
        let exp_val = safe_exp(exponent);

        // Use saturating addition and division
        let expected_alpha =
            I32F32::from_num(1.0).saturating_div(I32F32::from_num(1.0).saturating_add(exp_val));

        // Assert that the computed alpha values match the expected values within the epsilon.
        assert_approx_eq(alpha[i], expected_alpha, epsilon);
    }
}

#[test]
fn test_clamp_alpha_values() {
    // Define the alpha values.
    let alpha = vec![
        I32F32::from_num(0.1),
        I32F32::from_num(0.5),
        I32F32::from_num(0.9),
    ];
    // Define the high and low clamping values.
    let alpha_high = I32F32::from_num(0.8);
    let alpha_low = I32F32::from_num(0.2);

    // Compute the clamped alpha values using the function.
    let clamped_alpha = SubtensorModule::clamp_alpha_values(alpha.clone(), alpha_high, alpha_low);

    // Ensure the length of the clamped alpha vector matches the original alpha vector.
    assert_eq!(clamped_alpha.len(), alpha.len());

    // Manually compute the expected clamped alpha values for each alpha value.
    // The clamping logic is: max(alpha_low, min(alpha_high, a))

    // For alpha[0] = 0.1:
    // clamped_a = max(0.2, min(0.8, 0.1)) = max(0.2, 0.1) = 0.2
    let expected_clamped_alpha_0 = I32F32::from_num(0.2);

    // For alpha[1] = 0.5:
    // clamped_a = max(0.2, min(0.8, 0.5)) = max(0.2, 0.5) = 0.5
    let expected_clamped_alpha_1 = I32F32::from_num(0.5);

    // For alpha[2] = 0.9:
    // clamped_a = max(0.2, min(0.8, 0.9)) = max(0.2, 0.8) = 0.8
    let expected_clamped_alpha_2 = I32F32::from_num(0.8);

    // Assert that the computed clamped alpha values match the expected values.
    assert_eq!(clamped_alpha[0], expected_clamped_alpha_0);
    assert_eq!(clamped_alpha[1], expected_clamped_alpha_1);
    assert_eq!(clamped_alpha[2], expected_clamped_alpha_2);
}

#[test]
fn test_calculate_logistic_params() {
    // Define test inputs
    let alpha_high = I32F32::from_num(0.9);
    let alpha_low = I32F32::from_num(0.1);
    let consensus_high = I32F32::from_num(0.8);
    let consensus_low = I32F32::from_num(0.2);

    // Expected values
    // a = (ln((1 / alpha_high - 1)) - ln((1 / alpha_low - 1))) / (consensus_low - consensus_high)
    //   = (ln((1 / 0.9 - 1)) - ln((1 / 0.1 - 1))) / (0.2 - 0.8)
    //   = (ln(0.1111) - ln(9)) / -0.6
    //   = (-2.1972 - 2.1972) / -0.6
    //   = -4.3944 / -0.6
    //   = 7.324
    let expected_a = I32F32::from_num(7.324);

    // b = ln((1 / alpha_low - 1)) + a * consensus_low
    //   = ln((1 / 0.1 - 1)) + 7.324 * 0.2
    //   = ln(9) + 1.4648
    //   = 2.1972 + 1.4648
    //   = 3.662
    let expected_b = I32F32::from_num(3.662);

    // Call the function
    let (a, b) = SubtensorModule::calculate_logistic_params(
        alpha_high,
        alpha_low,
        consensus_high,
        consensus_low,
    );

    // Assert the results
    assert!(
        (a - expected_a).abs() < I32F32::from_num(0.001),
        "Expected a: {:?}, got: {:?}",
        expected_a,
        a
    );
    assert!(
        (b - expected_b).abs() < I32F32::from_num(0.001),
        "Expected b: {:?}, got: {:?}",
        expected_b,
        b
    );
}

#[test]
fn test_calculate_logistic_params_edge_cases() {
    // Edge Case 1: Alpha values at their boundaries (0 and 1)
    let alpha_high = I32F32::from_num(1.0);
    let alpha_low = I32F32::from_num(0.0);
    let consensus_high = I32F32::from_num(0.8);
    let consensus_low = I32F32::from_num(0.2);

    // Call the function
    let (a, b) = SubtensorModule::calculate_logistic_params(
        alpha_high,
        alpha_low,
        consensus_high,
        consensus_low,
    );

    // Assert the results
    assert_eq!(a, I32F32::from_num(0.0), "Expected a to be 0, got: {:?}", a);
    assert_eq!(b, I32F32::from_num(0.0), "Expected b to be 0, got: {:?}", b);

    // Edge Case 2: Consensus values at their boundaries (0 and 1)
    let alpha_high = I32F32::from_num(0.9);
    let alpha_low = I32F32::from_num(0.1);
    let consensus_high = I32F32::from_num(1.0);
    let consensus_low = I32F32::from_num(0.0);

    // Call the function
    let (a, b) = SubtensorModule::calculate_logistic_params(
        alpha_high,
        alpha_low,
        consensus_high,
        consensus_low,
    );

    // Expected values
    // a = (ln((1 / 0.9 - 1)) - ln((1 / 0.1 - 1))) / (0.0 - 1.0)
    //   = (ln(0.1111) - ln(9)) / -1.0
    //   = (-2.1972 - 2.1972) / -1.0
    //   = -4.3944 / -1.0
    //   = 4.3944
    let expected_a = I32F32::from_num(4.3944);

    // b = ln((1 / 0.1 - 1)) + a * 0.0
    //   = ln(9) + 0
    //   = 2.1972
    let expected_b = I32F32::from_num(2.1972);

    // Assert the results
    assert!(
        (a - expected_a).abs() < I32F32::from_num(0.001),
        "Expected a: {:?}, got: {:?}",
        expected_a,
        a
    );
    assert!(
        (b - expected_b).abs() < I32F32::from_num(0.001),
        "Expected b: {:?}, got: {:?}",
        expected_b,
        b
    );

    // Edge Case 3: Alpha values being equal
    let alpha_high = I32F32::from_num(0.5);
    let alpha_low = I32F32::from_num(0.5);
    let consensus_high = I32F32::from_num(0.8);
    let consensus_low = I32F32::from_num(0.2);

    // Call the function
    let (a, b) = SubtensorModule::calculate_logistic_params(
        alpha_high,
        alpha_low,
        consensus_high,
        consensus_low,
    );

    // Assert the results
    assert_eq!(a, I32F32::from_num(0.0), "Expected a to be 0, got: {:?}", a);
    assert_eq!(b, I32F32::from_num(0.0), "Expected b to be 0, got: {:?}", b);

    // Edge Case 4: Consensus values being equal
    let alpha_high = I32F32::from_num(0.9);
    let alpha_low = I32F32::from_num(0.1);
    let consensus_high = I32F32::from_num(0.5);
    let consensus_low = I32F32::from_num(0.5);

    // Call the function
    let (a, b) = SubtensorModule::calculate_logistic_params(
        alpha_high,
        alpha_low,
        consensus_high,
        consensus_low,
    );

    // Assert the results
    assert_eq!(a, I32F32::from_num(0.0), "Expected a to be 0, got: {:?}", a);
    assert_eq!(b, I32F32::from_num(0.0), "Expected b to be 0, got: {:?}", b);
}

#[test]
fn test_compute_ema_bonds_with_liquid_alpha_sparse() {
    // Define test inputs
    let bonds_delta = vec![
        vec![(0, I32F32::from_num(0.1)), (1, I32F32::from_num(0.2))],
        vec![(0, I32F32::from_num(0.3)), (1, I32F32::from_num(0.4))],
    ];
    let bonds = vec![
        vec![(0, I32F32::from_num(0.5)), (1, I32F32::from_num(0.6))],
        vec![(0, I32F32::from_num(0.7)), (1, I32F32::from_num(0.8))],
    ];
    let alpha = vec![I32F32::from_num(0.9), I32F32::from_num(0.8)];

    // Expected values
    // EMA calculation for each bond:
    // EMA = alpha * bond_delta + (1 - alpha) * bond
    // For bond (0, 0):
    // EMA = 0.9 * 0.1 + (1 - 0.9) * 0.5 = 0.09 + 0.05 = 0.14
    // For bond (0, 1):
    // EMA = 0.8 * 0.2 + (1 - 0.8) * 0.6 = 0.16 + 0.12 = 0.28
    // For bond (1, 0):
    // EMA = 0.9 * 0.3 + (1 - 0.9) * 0.7 = 0.27 + 0.07 = 0.34
    // For bond (1, 1):
    // EMA = 0.8 * 0.4 + (1 - 0.8) * 0.8 = 0.32 + 0.16 = 0.48
    let expected_ema_bonds = vec![
        vec![(0, I32F32::from_num(0.14)), (1, I32F32::from_num(0.28))],
        vec![(0, I32F32::from_num(0.34)), (1, I32F32::from_num(0.48))],
    ];

    // Call the function
    let ema_bonds =
        SubtensorModule::compute_ema_bonds_with_liquid_alpha_sparse(&bonds_delta, &bonds, alpha);

    // Assert the results with an epsilon for approximate equality
    let epsilon = I32F32::from_num(1e-6);
    assert_approx_eq_vec_of_vec(&ema_bonds, &expected_ema_bonds, epsilon);
}

#[test]
fn test_compute_ema_bonds_with_liquid_alpha_sparse_empty() {
    // Test with empty inputs
    let bonds_delta: Vec<Vec<(u16, I32F32)>> = vec![];
    let bonds: Vec<Vec<(u16, I32F32)>> = vec![];
    let alpha: Vec<I32F32> = vec![];

    // Expected values: Empty Vec
    let expected_ema_bonds: Vec<Vec<(u16, I32F32)>> = vec![];

    // Call the function
    let ema_bonds =
        SubtensorModule::compute_ema_bonds_with_liquid_alpha_sparse(&bonds_delta, &bonds, alpha);

    // Assert the results
    assert_eq!(
        ema_bonds, expected_ema_bonds,
        "Expected EMA bonds: {:?}, got: {:?}",
        expected_ema_bonds, ema_bonds
    );
}

#[test]
fn test_get_set_alpha() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let alpha_low: u16 = 12_u16;
        let alpha_high: u16 = u16::MAX - 10;

        let hotkey: U256 = U256::from(1);
        let coldkey: U256 = U256::from(1 + 456);
        let signer = <<Test as Config>::RuntimeOrigin>::signed(coldkey);

        // Enable Liquid Alpha and setup
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        migration::migrate_create_root_network::<Test>();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_000);
        assert_ok!(SubtensorModule::root_register(signer.clone(), hotkey,));
        assert_ok!(SubtensorModule::add_stake(signer.clone(), hotkey, 1000));

        // Should fail as signer does not own the subnet
        assert_err!(
            SubtensorModule::do_set_alpha_values(signer.clone(), netuid, alpha_low, alpha_high),
            DispatchError::BadOrigin
        );

        assert_ok!(SubtensorModule::register_network(signer.clone()));

        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));
        let (grabbed_alpha_low, grabbed_alpha_high): (u16, u16) =
            SubtensorModule::get_alpha_values(netuid);

        log::info!(
            "alpha_low: {:?} alpha_high: {:?}",
            grabbed_alpha_low,
            grabbed_alpha_high
        );
        assert_eq!(grabbed_alpha_low, alpha_low);
        assert_eq!(grabbed_alpha_high, alpha_high);

        // Convert the u16 values to decimal values
        fn unnormalize_u16_to_float(normalized_value: u16) -> f32 {
            const MAX_U16: u16 = 65535;
            normalized_value as f32 / MAX_U16 as f32
        }

        let alpha_low_decimal = unnormalize_u16_to_float(alpha_low);
        let alpha_high_decimal = unnormalize_u16_to_float(alpha_high);

        let (alpha_low_32, alpha_high_32) = SubtensorModule::get_alpha_values_32(netuid);

        let tolerance: f32 = 1e-6; // 0.000001

        // Check if the values are equal to the sixth decimal
        assert!(
            (alpha_low_32.to_num::<f32>() - alpha_low_decimal).abs() < tolerance,
            "alpha_low mismatch: {} != {}",
            alpha_low_32.to_num::<f32>(),
            alpha_low_decimal
        );
        assert!(
            (alpha_high_32.to_num::<f32>() - alpha_high_decimal).abs() < tolerance,
            "alpha_high mismatch: {} != {}",
            alpha_high_32.to_num::<f32>(),
            alpha_high_decimal
        );

        // 1. Liquid alpha disabled
        SubtensorModule::set_liquid_alpha_enabled(netuid, false);
        assert_err!(
            SubtensorModule::do_set_alpha_values(signer.clone(), netuid, alpha_low, alpha_high),
            Error::<Test>::LiquidAlphaDisabled
        );
        // Correct scenario after error
        SubtensorModule::set_liquid_alpha_enabled(netuid, true); // Re-enable for further tests
        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        // 2. Alpha high too low
        let alpha_high_too_low = (u16::MAX as u32 * 4 / 5) as u16 - 1; // One less than the minimum acceptable value
        assert_err!(
            SubtensorModule::do_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low,
                alpha_high_too_low
            ),
            Error::<Test>::AlphaHighTooLow
        );
        // Correct scenario after error
        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        // 3. Alpha low too low or too high
        let alpha_low_too_low = 0_u16;
        assert_err!(
            SubtensorModule::do_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low_too_low,
                alpha_high
            ),
            Error::<Test>::AlphaLowOutOfRange
        );
        // Correct scenario after error
        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        let alpha_low_too_high = (u16::MAX as u32 * 4 / 5) as u16 + 1; // One more than the maximum acceptable value
        assert_err!(
            SubtensorModule::do_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low_too_high,
                alpha_high
            ),
            Error::<Test>::AlphaLowOutOfRange
        );
        // Correct scenario after error
        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));
    });
}

// // Map the retention graph for consensus guarantees with an single epoch on a graph with 512 nodes, of which the first 64 are validators, the graph is split into a major and minor set, each setting specific weight on itself and the complement on the other.
// //
// // ```import torch
// // import matplotlib.pyplot as plt
// // from matplotlib.pyplot import cm
// // %matplotlib inline
// //
// // with open('finney_consensus_0.4.txt') as f:  # test output saved to finney_consensus.txt
// //     retention_map = eval(f.read())
// //
// // major_ratios = {}
// // avg_weight_devs = {}
// // for major_stake, major_weight, minor_weight, avg_weight_dev, major_ratio in retention_map:
// //     major_stake = f'{major_stake:.2f}'
// //     maj, min = int(round(50 * major_weight)), int(round(50 * minor_weight))
// //     avg_weight_devs.setdefault(major_stake, torch.zeros((51, 51)))
// //     avg_weight_devs[major_stake][maj][min] = avg_weight_dev
// //     major_ratios.setdefault(major_stake, torch.zeros((51, 51)))
// //     major_ratios[major_stake][maj][min] = major_ratio
// //
// // _x = torch.linspace(0, 1, 51); _y = torch.linspace(0, 1, 51)
// // x, y = torch.meshgrid(_x, _y, indexing='ij')
// //
// // fig = plt.figure(figsize=(6, 6), dpi=70); ax = fig.gca()
// // ax.set_xticks(torch.arange(0, 1, 0.05)); ax.set_yticks(torch.arange(0, 1., 0.05))
// // ax.set_xticklabels([f'{_:.2f}'[1:] for _ in torch.arange(0, 1., 0.05)])
// // plt.grid(); plt.rc('grid', linestyle="dotted", color=[0.85, 0.85, 0.85])
// //
// // isolate = ['0.60']; stakes = [0.51, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 0.99]
// // colors = cm.viridis(torch.linspace(0, 1, len(stakes) + 1))
// // for i, stake in enumerate(stakes):
// //     contours = plt.contour(x, y, major_ratios[f'{stake:.2f}'], levels=[0., stake], colors=[colors[i + 1]])
// //     if f'{stake:.2f}' in isolate:
// //         contours.collections[1].set_linewidth(3)
// //     plt.clabel(contours, inline=True, fontsize=10)
// //
// // plt.title(f'Major emission [$stake_{{maj}}=emission_{{maj}}$ retention lines]')
// // plt.ylabel('Minor self-weight'); plt.xlabel('Major self-weight'); plt.show()
// // ```
// // #[test]
// fn _map_consensus_guarantees() {
//     let netuid: u16 = 1;
//     let network_n: u16 = 512;
//     let validators_n: u16 = 64;
//     let epochs: u16 = 1;
//     let interleave = 0;
//     let weight_stddev: I32F32 = fixed(0.4);
//     println!("[");
//     for _major_stake in vec![0.51, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 0.99] {
//         let major_stake: I32F32 = I32F32::from_num(_major_stake);
//         for _major_weight in 0..51 {
//             let major_weight: I32F32 = I32F32::from_num(50 - _major_weight) / I32F32::from_num(50);
//             for _minor_weight in 0..51 {
//                 let minor_weight: I32F32 =
//                     I32F32::from_num(50 - _minor_weight) / I32F32::from_num(50);
//                 let (
//                     validators,
//                     servers,
//                     major_validators,
//                     minor_validators,
//                     major_servers,
//                     minor_servers,
//                     stake,
//                     weights,
//                     avg_weight_dev,
//                 ) = split_graph(
//                     major_stake,
//                     major_weight,
//                     minor_weight,
//                     weight_stddev,
//                     validators_n as usize,
//                     network_n as usize,
//                     interleave as usize,
//                 );

//                 new_test_ext(1).execute_with(|| {
// 					init_run_epochs(netuid, network_n, &validators, &servers, epochs, 1, true, &stake, true, &weights, true, false, 0, true);

// 					let mut major_emission: I64F64 = I64F64::from_num(0);
// 					let mut minor_emission: I64F64 = I64F64::from_num(0);
// 					for set in vec![major_validators, major_servers] {
// 						for uid in set {
// 							major_emission += I64F64::from_num(SubtensorModule::get_emission_for_uid( netuid, uid ));
// 						}
// 					}
// 					for set in vec![minor_validators, minor_servers] {
// 						for uid in set {
// 							minor_emission += I64F64::from_num(SubtensorModule::get_emission_for_uid( netuid, uid ));
// 						}
// 					}
// 					let major_ratio: I32F32 = I32F32::from_num(major_emission / (major_emission + minor_emission));
// 					println!("[{major_stake}, {major_weight:.2}, {minor_weight:.2}, {avg_weight_dev:.3}, {major_ratio:.3}], ");
// 				});
//             }
//         }
//     }
//     println!("]");
// }

/// Helpers

/// Asserts that two I32F32 values are approximately equal within a given epsilon.
///
/// # Arguments
/// * `left` - The first value to compare.
/// * `right` - The second value to compare.
/// * `epsilon` - The maximum allowed difference between the two values.
fn assert_approx_eq(left: I32F32, right: I32F32, epsilon: I32F32) {
    if (left - right).abs() > epsilon {
        panic!(
            "assertion failed: `(left ≈ right)`\n  left: `{:?}`,\n right: `{:?}`,\n epsilon: `{:?}`",
            left, right, epsilon
        );
    }
}

/// Helper function to assert approximate equality of two vectors of vectors of tuples.
fn assert_approx_eq_vec_of_vec(
    left: &[Vec<(u16, I32F32)>],
    right: &[Vec<(u16, I32F32)>],
    epsilon: I32F32,
) {
    assert_eq!(left.len(), right.len(), "Vectors have different lengths");
    for (left_row, right_row) in left.iter().zip(right.iter()) {
        assert_eq!(
            left_row.len(),
            right_row.len(),
            "Rows have different lengths"
        );
        for ((left_idx, left_val), (right_idx, right_val)) in left_row.iter().zip(right_row.iter())
        {
            assert_eq!(left_idx, right_idx, "Indices are different");
            assert!(
                (left_val - right_val).abs() < epsilon,
                "Values are different: left = {:?}, right = {:?}, epsilon = {:?}",
                left_val,
                right_val,
                epsilon
            );
        }
    }
}
