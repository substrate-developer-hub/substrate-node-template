use super::*;
use crate::math::*;
use frame_support::IterableStorageDoubleMap;
use sp_std::vec;
use substrate_fixed::types::{I32F32, I64F64, I96F32};

impl<T: Config> Pallet<T> {
    /// Calculates reward consensus and returns the emissions for uids/hotkeys in a given `netuid`.
    /// (Dense version used only for testing purposes.)
    #[allow(clippy::indexing_slicing)]
    pub fn epoch_dense(netuid: u16, rao_emission: u64) -> Vec<(T::AccountId, u64, u64)> {
        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n(netuid);
        log::trace!("n:\n{:?}\n", n);

        // ======================
        // == Active & updated ==
        // ======================

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::trace!("current_block:\n{:?}\n", current_block);

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff(netuid) as u64;
        log::trace!("activity_cutoff:\n{:?}\n", activity_cutoff);

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update(netuid);
        log::trace!("Last update:\n{:?}\n", &last_update);

        // Inactive mask.
        let inactive: Vec<bool> = last_update
            .iter()
            .map(|updated| updated.saturating_add(activity_cutoff) < current_block)
            .collect();
        log::trace!("Inactive:\n{:?}\n", inactive.clone());

        // Logical negation of inactive.
        let active: Vec<bool> = inactive.iter().map(|&b| !b).collect();

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration(netuid);
        log::trace!("Block at registration:\n{:?}\n", &block_at_registration);

        // Outdated matrix, updated_ij=True if i has last updated (weights) after j has last registered.
        let outdated: Vec<Vec<bool>> = last_update
            .iter()
            .map(|updated| {
                block_at_registration
                    .iter()
                    .map(|registered| updated <= registered)
                    .collect()
            })
            .collect();
        log::trace!("Outdated:\n{:?}\n", &outdated);

        // ===========
        // == Stake ==
        // ===========

        let hotkeys: Vec<(u16, T::AccountId)> =
            <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(netuid)
                .collect();
        log::trace!("hotkeys: {:?}", &hotkeys);

        // Access network stake as normalized vector.
        let mut stake_64: Vec<I64F64> = vec![I64F64::from_num(0.0); n as usize];
        for (uid_i, hotkey) in &hotkeys {
            stake_64[*uid_i as usize] = I64F64::from_num(Self::get_total_stake_for_hotkey(hotkey));
        }
        inplace_normalize_64(&mut stake_64);
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32(stake_64);
        log::trace!("S:\n{:?}\n", &stake);

        // =======================
        // == Validator permits ==
        // =======================

        // Get validator permits.
        let validator_permits: Vec<bool> = Self::get_validator_permit(netuid);
        log::trace!("validator_permits: {:?}", validator_permits);

        // Logical negation of validator_permits.
        let validator_forbids: Vec<bool> = validator_permits.iter().map(|&b| !b).collect();

        // Get max allowed validators.
        let max_allowed_validators: u16 = Self::get_max_allowed_validators(netuid);
        log::trace!("max_allowed_validators: {:?}", max_allowed_validators);

        // Get new validator permits.
        let new_validator_permits: Vec<bool> = is_topk(&stake, max_allowed_validators as usize);
        log::trace!("new_validator_permits: {:?}", new_validator_permits);

        // ==================
        // == Active Stake ==
        // ==================

        let mut active_stake: Vec<I32F32> = stake.clone();

        // Remove inactive stake.
        inplace_mask_vector(&inactive, &mut active_stake);

        // Remove non-validator stake.
        inplace_mask_vector(&validator_forbids, &mut active_stake);

        // Normalize active stake.
        inplace_normalize(&mut active_stake);
        log::trace!("S:\n{:?}\n", &active_stake);

        // =============
        // == Weights ==
        // =============

        // Access network weights row unnormalized.
        let mut weights: Vec<Vec<I32F32>> = Self::get_weights(netuid);
        log::trace!("W:\n{:?}\n", &weights);

        // Mask weights that are not from permitted validators.
        inplace_mask_rows(&validator_forbids, &mut weights);
        log::trace!("W (permit): {:?}", &weights);

        // Remove self-weight by masking diagonal.
        inplace_mask_diag(&mut weights);
        log::trace!("W (permit+diag):\n{:?}\n", &weights);

        // Mask outdated weights: remove weights referring to deregistered neurons.
        inplace_mask_matrix(&outdated, &mut weights);
        log::trace!("W (permit+diag+outdate):\n{:?}\n", &weights);

        // Normalize remaining weights.
        inplace_row_normalize(&mut weights);
        log::trace!("W (mask+norm):\n{:?}\n", &weights);

        // ================================
        // == Consensus, Validator Trust ==
        // ================================

        // Compute preranks: r_j = SUM(i) w_ij * s_i
        let preranks: Vec<I32F32> = matmul(&weights, &active_stake);

        // Clip weights at majority consensus
        let kappa: I32F32 = Self::get_float_kappa(netuid); // consensus majority ratio, e.g. 51%.
        let consensus: Vec<I32F32> = weighted_median_col(&active_stake, &weights, kappa);
        inplace_col_clip(&mut weights, &consensus);
        let validator_trust: Vec<I32F32> = row_sum(&weights);

        // ====================================
        // == Ranks, Server Trust, Incentive ==
        // ====================================

        // Compute ranks: r_j = SUM(i) w_ij * s_i
        let mut ranks: Vec<I32F32> = matmul(&weights, &active_stake);

        // Compute server trust: ratio of rank after vs. rank before.
        let trust: Vec<I32F32> = vecdiv(&ranks, &preranks);

        inplace_normalize(&mut ranks);
        let incentive: Vec<I32F32> = ranks.clone();
        log::trace!("I:\n{:?}\n", &incentive);

        // =========================
        // == Bonds and Dividends ==
        // =========================

        // Access network bonds.
        let mut bonds: Vec<Vec<I32F32>> = Self::get_bonds(netuid);
        inplace_mask_matrix(&outdated, &mut bonds); // mask outdated bonds
        inplace_col_normalize(&mut bonds); // sum_i b_ij = 1
        log::trace!("B:\n{:?}\n", &bonds);

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<I32F32>> = row_hadamard(&weights, &active_stake); // ΔB = W◦S
        inplace_col_normalize(&mut bonds_delta); // sum_i b_ij = 1
        log::trace!("ΔB:\n{:?}\n", &bonds_delta);
        // Compute the Exponential Moving Average (EMA) of bonds.
        let mut ema_bonds = Self::compute_ema_bonds(netuid, consensus.clone(), bonds_delta, bonds);
        inplace_col_normalize(&mut ema_bonds); // sum_i b_ij = 1
        log::trace!("emaB:\n{:?}\n", &ema_bonds);

        // Compute dividends: d_i = SUM(j) b_ij * inc_j
        let mut dividends: Vec<I32F32> = matmul_transpose(&ema_bonds, &incentive);
        inplace_normalize(&mut dividends);
        log::trace!("D:\n{:?}\n", &dividends);

        // =================================
        // == Emission and Pruning scores ==
        // =================================

        // Compute emission scores.

        // Compute normalized emission scores. range: I32F32(0, 1)
        // Compute normalized emission scores. range: I32F32(0, 1)
        let combined_emission: Vec<I32F32> = incentive
            .iter()
            .zip(dividends.clone())
            .map(|(ii, di)| ii.saturating_add(di))
            .collect();
        let emission_sum: I32F32 = combined_emission.iter().sum();

        let mut normalized_server_emission: Vec<I32F32> = incentive.clone(); // Servers get incentive.
        let mut normalized_validator_emission: Vec<I32F32> = dividends.clone(); // Validators get dividends.
        let mut normalized_combined_emission: Vec<I32F32> = combined_emission.clone();
        // Normalize on the sum of incentive + dividends.
        inplace_normalize_using_sum(&mut normalized_server_emission, emission_sum);
        inplace_normalize_using_sum(&mut normalized_validator_emission, emission_sum);
        inplace_normalize(&mut normalized_combined_emission);

        // If emission is zero, replace emission with normalized stake.
        if emission_sum == I32F32::from(0) {
            // no weights set | outdated weights | self_weights
            if is_zero(&active_stake) {
                // no active stake
                normalized_validator_emission.clone_from(&stake); // do not mask inactive, assumes stake is normalized
                normalized_combined_emission.clone_from(&stake);
            } else {
                normalized_validator_emission.clone_from(&active_stake); // emission proportional to inactive-masked normalized stake
                normalized_combined_emission.clone_from(&active_stake);
            }
        }

        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num(rao_emission);

        let server_emission: Vec<I96F32> = normalized_server_emission
            .iter()
            .map(|se: &I32F32| I96F32::from_num(*se).saturating_mul(float_rao_emission))
            .collect();
        let server_emission: Vec<u64> = server_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        let validator_emission: Vec<I96F32> = normalized_validator_emission
            .iter()
            .map(|ve: &I32F32| I96F32::from_num(*ve).saturating_mul(float_rao_emission))
            .collect();
        let validator_emission: Vec<u64> = validator_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        // Used only to track combined emission in the storage.
        let combined_emission: Vec<I96F32> = normalized_combined_emission
            .iter()
            .map(|ce: &I32F32| I96F32::from_num(*ce).saturating_mul(float_rao_emission))
            .collect();
        let combined_emission: Vec<u64> = combined_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        log::trace!("nSE: {:?}", &normalized_server_emission);
        log::trace!("SE: {:?}", &server_emission);
        log::trace!("nVE: {:?}", &normalized_validator_emission);
        log::trace!("VE: {:?}", &validator_emission);
        log::trace!("nCE: {:?}", &normalized_combined_emission);
        log::trace!("CE: {:?}", &combined_emission);

        // Set pruning scores using combined emission scores.
        let pruning_scores: Vec<I32F32> = normalized_combined_emission.clone();
        log::trace!("P: {:?}", &pruning_scores);

        // ===================
        // == Value storage ==
        // ===================
        let cloned_emission: Vec<u64> = combined_emission.clone();
        let cloned_ranks: Vec<u16> = ranks
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_trust: Vec<u16> = trust
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_consensus: Vec<u16> = consensus
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_incentive: Vec<u16> = incentive
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_dividends: Vec<u16> = dividends
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_pruning_scores: Vec<u16> = vec_max_upscale_to_u16(&pruning_scores);
        let cloned_validator_trust: Vec<u16> = validator_trust
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        Active::<T>::insert(netuid, active.clone());
        Emission::<T>::insert(netuid, cloned_emission);
        Rank::<T>::insert(netuid, cloned_ranks);
        Trust::<T>::insert(netuid, cloned_trust);
        Consensus::<T>::insert(netuid, cloned_consensus);
        Incentive::<T>::insert(netuid, cloned_incentive);
        Dividends::<T>::insert(netuid, cloned_dividends);
        PruningScores::<T>::insert(netuid, cloned_pruning_scores);
        ValidatorTrust::<T>::insert(netuid, cloned_validator_trust);
        ValidatorPermit::<T>::insert(netuid, new_validator_permits.clone());

        // Column max-upscale EMA bonds for storage: max_i w_ij = 1.
        inplace_col_max_upscale(&mut ema_bonds);
        new_validator_permits
            .iter()
            .zip(validator_permits)
            .zip(ema_bonds)
            .enumerate()
            .for_each(|(i, ((new_permit, validator_permit), ema_bond))| {
                // Set bonds only if uid retains validator permit, otherwise clear bonds.
                if *new_permit {
                    let new_bonds_row: Vec<(u16, u16)> = (0..n)
                        .zip(vec_fixed_proportions_to_u16(ema_bond.clone()))
                        .collect();
                    Bonds::<T>::insert(netuid, i as u16, new_bonds_row);
                } else if validator_permit {
                    // Only overwrite the intersection.
                    let new_empty_bonds_row: Vec<(u16, u16)> = vec![];
                    Bonds::<T>::insert(netuid, i as u16, new_empty_bonds_row);
                }
            });

        hotkeys
            .into_iter()
            .map(|(uid_i, hotkey)| {
                (
                    hotkey,
                    server_emission[uid_i as usize],
                    validator_emission[uid_i as usize],
                )
            })
            .collect()
    }

    /// Calculates reward consensus values, then updates rank, trust, consensus, incentive, dividend, pruning_score, emission and bonds, and
    /// returns the emissions for uids/hotkeys in a given `netuid`.
    ///
    /// # Args:
    ///  * 'netuid': ( u16 ):
    ///     - The network to distribute the emission onto.
    ///
    ///  * 'rao_emission': ( u64 ):
    ///     - The total emission for the epoch.
    ///
    ///  * 'debug' ( bool ):
    ///     - Print debugging outputs.
    ///
    #[allow(clippy::indexing_slicing)]
    pub fn epoch(netuid: u16, rao_emission: u64) -> Vec<(T::AccountId, u64, u64)> {
        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n(netuid);
        log::trace!("Number of Neurons in Network: {:?}", n);

        // ======================
        // == Active & updated ==
        // ======================

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::trace!("current_block: {:?}", current_block);

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff(netuid) as u64;
        log::trace!("activity_cutoff: {:?}", activity_cutoff);

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update(netuid);
        log::trace!("Last update: {:?}", &last_update);

        // Inactive mask.
        let inactive: Vec<bool> = last_update
            .iter()
            .map(|updated| updated.saturating_add(activity_cutoff) < current_block)
            .collect();
        log::trace!("Inactive: {:?}", inactive.clone());

        // Logical negation of inactive.
        let active: Vec<bool> = inactive.iter().map(|&b| !b).collect();

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration(netuid);
        log::trace!("Block at registration: {:?}", &block_at_registration);

        // ===========
        // == Stake ==
        // ===========

        let hotkeys: Vec<(u16, T::AccountId)> =
            <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(netuid)
                .collect();
        log::trace!("hotkeys: {:?}", &hotkeys);

        // Access network stake as normalized vector.
        let mut stake_64: Vec<I64F64> = vec![I64F64::from_num(0.0); n as usize];
        for (uid_i, hotkey) in &hotkeys {
            stake_64[*uid_i as usize] = I64F64::from_num(Self::get_total_stake_for_hotkey(hotkey));
        }
        log::trace!("Stake : {:?}", &stake_64);
        inplace_normalize_64(&mut stake_64);
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32(stake_64);
        // range: I32F32(0, 1)
        log::trace!("Normalised Stake: {:?}", &stake);

        // =======================
        // == Validator permits ==
        // =======================

        // Get current validator permits.
        let validator_permits: Vec<bool> = Self::get_validator_permit(netuid);
        log::trace!("validator_permits: {:?}", validator_permits);

        // Logical negation of validator_permits.
        let validator_forbids: Vec<bool> = validator_permits.iter().map(|&b| !b).collect();

        // Get max allowed validators.
        let max_allowed_validators: u16 = Self::get_max_allowed_validators(netuid);
        log::trace!("max_allowed_validators: {:?}", max_allowed_validators);

        // Get new validator permits.
        let new_validator_permits: Vec<bool> = is_topk(&stake, max_allowed_validators as usize);
        log::trace!("new_validator_permits: {:?}", new_validator_permits);

        // ==================
        // == Active Stake ==
        // ==================

        let mut active_stake: Vec<I32F32> = stake.clone();

        // Remove inactive stake.
        inplace_mask_vector(&inactive, &mut active_stake);

        // Remove non-validator stake.
        inplace_mask_vector(&validator_forbids, &mut active_stake);

        // Normalize active stake.
        inplace_normalize(&mut active_stake);
        log::trace!("Active Stake:\n{:?}\n", &active_stake);

        // =============
        // == Weights ==
        // =============

        // Access network weights row unnormalized.
        let mut weights: Vec<Vec<(u16, I32F32)>> = Self::get_weights_sparse(netuid);
        log::trace!("Weights: {:?}", &weights);

        // Mask weights that are not from permitted validators.
        weights = mask_rows_sparse(&validator_forbids, &weights);
        log::trace!("Weights (permit): {:?}", &weights);

        // Remove self-weight by masking diagonal.
        weights = mask_diag_sparse(&weights);
        log::trace!("Weights (permit+diag): {:?}", &weights);

        // Remove weights referring to deregistered neurons.
        weights = vec_mask_sparse_matrix(
            &weights,
            &last_update,
            &block_at_registration,
            &|updated, registered| updated <= registered,
        );
        log::trace!("Weights (permit+diag+outdate): {:?}", &weights);

        // Normalize remaining weights.
        inplace_row_normalize_sparse(&mut weights);
        log::trace!("Weights (mask+norm): {:?}", &weights);

        // ================================
        // == Consensus, Validator Trust ==
        // ================================

        // Compute preranks: r_j = SUM(i) w_ij * s_i
        let preranks: Vec<I32F32> = matmul_sparse(&weights, &active_stake, n);
        log::trace!("Ranks (before): {:?}", &preranks);

        // Clip weights at majority consensus
        let kappa: I32F32 = Self::get_float_kappa(netuid); // consensus majority ratio, e.g. 51%.
        let consensus: Vec<I32F32> = weighted_median_col_sparse(&active_stake, &weights, n, kappa);
        log::trace!("Consensus: {:?}", &consensus);

        weights = col_clip_sparse(&weights, &consensus);
        log::trace!("Weights: {:?}", &weights);

        let validator_trust: Vec<I32F32> = row_sum_sparse(&weights);
        log::trace!("Validator Trust: {:?}", &validator_trust);

        // =============================
        // == Ranks, Trust, Incentive ==
        // =============================

        // Compute ranks: r_j = SUM(i) w_ij * s_i.
        let mut ranks: Vec<I32F32> = matmul_sparse(&weights, &active_stake, n);
        log::trace!("Ranks (after): {:?}", &ranks);

        // Compute server trust: ratio of rank after vs. rank before.
        let trust: Vec<I32F32> = vecdiv(&ranks, &preranks); // range: I32F32(0, 1)
        log::trace!("T: {:?}", &trust);

        inplace_normalize(&mut ranks); // range: I32F32(0, 1)
        let incentive: Vec<I32F32> = ranks.clone();
        log::trace!("Incentive (=Rank): {:?}", &incentive);

        // =========================
        // == Bonds and Dividends ==
        // =========================

        // Access network bonds.
        let mut bonds: Vec<Vec<(u16, I32F32)>> = Self::get_bonds_sparse(netuid);
        log::trace!("B: {:?}", &bonds);

        // Remove bonds referring to deregistered neurons.
        bonds = vec_mask_sparse_matrix(
            &bonds,
            &last_update,
            &block_at_registration,
            &|updated, registered| updated <= registered,
        );
        log::trace!("B (outdatedmask): {:?}", &bonds);

        // Normalize remaining bonds: sum_i b_ij = 1.
        inplace_col_normalize_sparse(&mut bonds, n);
        log::trace!("B (mask+norm): {:?}", &bonds);

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<(u16, I32F32)>> = row_hadamard_sparse(&weights, &active_stake); // ΔB = W◦S (outdated W masked)
        log::trace!("ΔB: {:?}", &bonds_delta);

        // Normalize bonds delta.
        inplace_col_normalize_sparse(&mut bonds_delta, n); // sum_i b_ij = 1
        log::trace!("ΔB (norm): {:?}", &bonds_delta);

        // Compute the Exponential Moving Average (EMA) of bonds.
        let mut ema_bonds =
            Self::compute_ema_bonds_sparse(netuid, consensus.clone(), bonds_delta, bonds);
        // Normalize EMA bonds.
        inplace_col_normalize_sparse(&mut ema_bonds, n); // sum_i b_ij = 1
        log::trace!("Exponential Moving Average Bonds: {:?}", &ema_bonds);

        // Compute dividends: d_i = SUM(j) b_ij * inc_j.
        // range: I32F32(0, 1)
        let mut dividends: Vec<I32F32> = matmul_transpose_sparse(&ema_bonds, &incentive);
        inplace_normalize(&mut dividends);
        log::trace!("Dividends: {:?}", &dividends);

        // =================================
        // == Emission and Pruning scores ==
        // =================================

        // Compute normalized emission scores. range: I32F32(0, 1)
        let combined_emission: Vec<I32F32> = incentive
            .iter()
            .zip(dividends.clone())
            .map(|(ii, di)| ii.saturating_add(di))
            .collect();
        let emission_sum: I32F32 = combined_emission.iter().sum();

        let mut normalized_server_emission: Vec<I32F32> = incentive.clone(); // Servers get incentive.
        let mut normalized_validator_emission: Vec<I32F32> = dividends.clone(); // Validators get dividends.
        let mut normalized_combined_emission: Vec<I32F32> = combined_emission.clone();
        // Normalize on the sum of incentive + dividends.
        inplace_normalize_using_sum(&mut normalized_server_emission, emission_sum);
        inplace_normalize_using_sum(&mut normalized_validator_emission, emission_sum);
        inplace_normalize(&mut normalized_combined_emission);

        // If emission is zero, replace emission with normalized stake.
        if emission_sum == I32F32::from(0) {
            // no weights set | outdated weights | self_weights
            if is_zero(&active_stake) {
                // no active stake
                normalized_validator_emission.clone_from(&stake); // do not mask inactive, assumes stake is normalized
                normalized_combined_emission.clone_from(&stake);
            } else {
                normalized_validator_emission.clone_from(&active_stake); // emission proportional to inactive-masked normalized stake
                normalized_combined_emission.clone_from(&active_stake);
            }
        }

        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num(rao_emission);

        let server_emission: Vec<I96F32> = normalized_server_emission
            .iter()
            .map(|se: &I32F32| I96F32::from_num(*se).saturating_mul(float_rao_emission))
            .collect();
        let server_emission: Vec<u64> = server_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        let validator_emission: Vec<I96F32> = normalized_validator_emission
            .iter()
            .map(|ve: &I32F32| I96F32::from_num(*ve).saturating_mul(float_rao_emission))
            .collect();
        let validator_emission: Vec<u64> = validator_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        // Only used to track emission in storage.
        let combined_emission: Vec<I96F32> = normalized_combined_emission
            .iter()
            .map(|ce: &I32F32| I96F32::from_num(*ce).saturating_mul(float_rao_emission))
            .collect();
        let combined_emission: Vec<u64> = combined_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        log::trace!(
            "Normalized Server Emission: {:?}",
            &normalized_server_emission
        );
        log::trace!("Server Emission: {:?}", &server_emission);
        log::trace!(
            "Normalized Validator Emission: {:?}",
            &normalized_validator_emission
        );
        log::trace!("Validator Emission: {:?}", &validator_emission);
        log::trace!(
            "Normalized Combined Emission: {:?}",
            &normalized_combined_emission
        );
        log::trace!("Combined Emission: {:?}", &combined_emission);

        // Set pruning scores using combined emission scores.
        let pruning_scores: Vec<I32F32> = normalized_combined_emission.clone();
        log::trace!("Pruning Scores: {:?}", &pruning_scores);

        // ===================
        // == Value storage ==
        // ===================
        let cloned_emission: Vec<u64> = combined_emission.clone();
        let cloned_ranks: Vec<u16> = ranks
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_trust: Vec<u16> = trust
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_consensus: Vec<u16> = consensus
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_incentive: Vec<u16> = incentive
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_dividends: Vec<u16> = dividends
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_pruning_scores: Vec<u16> = vec_max_upscale_to_u16(&pruning_scores);
        let cloned_validator_trust: Vec<u16> = validator_trust
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        Active::<T>::insert(netuid, active.clone());
        Emission::<T>::insert(netuid, cloned_emission);
        Rank::<T>::insert(netuid, cloned_ranks);
        Trust::<T>::insert(netuid, cloned_trust);
        Consensus::<T>::insert(netuid, cloned_consensus);
        Incentive::<T>::insert(netuid, cloned_incentive);
        Dividends::<T>::insert(netuid, cloned_dividends);
        PruningScores::<T>::insert(netuid, cloned_pruning_scores);
        ValidatorTrust::<T>::insert(netuid, cloned_validator_trust);
        ValidatorPermit::<T>::insert(netuid, new_validator_permits.clone());

        // Column max-upscale EMA bonds for storage: max_i w_ij = 1.
        inplace_col_max_upscale_sparse(&mut ema_bonds, n);
        new_validator_permits
            .iter()
            .zip(validator_permits)
            .zip(ema_bonds)
            .enumerate()
            .for_each(|(i, ((new_permit, validator_permit), ema_bond))| {
                // Set bonds only if uid retains validator permit, otherwise clear bonds.
                if *new_permit {
                    let new_bonds_row: Vec<(u16, u16)> = ema_bond
                        .iter()
                        .map(|(j, value)| (*j, fixed_proportion_to_u16(*value)))
                        .collect();
                    Bonds::<T>::insert(netuid, i as u16, new_bonds_row);
                } else if validator_permit {
                    // Only overwrite the intersection.
                    let new_empty_bonds_row: Vec<(u16, u16)> = vec![];
                    Bonds::<T>::insert(netuid, i as u16, new_empty_bonds_row);
                }
            });

        // Emission tuples ( hotkeys, server_emission, validator_emission )
        hotkeys
            .into_iter()
            .map(|(uid_i, hotkey)| {
                (
                    hotkey,
                    server_emission[uid_i as usize],
                    validator_emission[uid_i as usize],
                )
            })
            .collect()
    }

    pub fn get_float_rho(netuid: u16) -> I32F32 {
        I32F32::from_num(Self::get_rho(netuid))
    }
    pub fn get_float_kappa(netuid: u16) -> I32F32 {
        I32F32::from_num(Self::get_kappa(netuid)).saturating_div(I32F32::from_num(u16::MAX))
    }

    pub fn get_normalized_stake(netuid: u16) -> Vec<I32F32> {
        let n = Self::get_subnetwork_n(netuid);
        let mut stake_64: Vec<I64F64> = (0..n)
            .map(|neuron_uid| {
                I64F64::from_num(Self::get_stake_for_uid_and_subnetwork(netuid, neuron_uid))
            })
            .collect();
        inplace_normalize_64(&mut stake_64);
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32(stake_64);
        stake
    }

    pub fn get_block_at_registration(netuid: u16) -> Vec<u64> {
        let n = Self::get_subnetwork_n(netuid);
        let block_at_registration: Vec<u64> = (0..n)
            .map(|neuron_uid| {
                if Keys::<T>::contains_key(netuid, neuron_uid) {
                    Self::get_neuron_block_at_registration(netuid, neuron_uid)
                } else {
                    0
                }
            })
            .collect();
        block_at_registration
    }

    /// Output unnormalized sparse weights, input weights are assumed to be row max-upscaled in u16.
    pub fn get_weights_sparse(netuid: u16) -> Vec<Vec<(u16, I32F32)>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut weights: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
        for (uid_i, weights_i) in
            <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
                .filter(|(uid_i, _)| *uid_i < n as u16)
        {
            for (uid_j, weight_ij) in weights_i.iter().filter(|(uid_j, _)| *uid_j < n as u16) {
                weights
                    .get_mut(uid_i as usize)
                    .expect("uid_i is filtered to be less than n; qed")
                    .push((*uid_j, I32F32::from_num(*weight_ij)));
            }
        }
        weights
    }

    /// Output unnormalized weights in [n, n] matrix, input weights are assumed to be row max-upscaled in u16.
    pub fn get_weights(netuid: u16) -> Vec<Vec<I32F32>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut weights: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0.0); n]; n];
        for (uid_i, weights_vec) in
            <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
                .filter(|(uid_i, _)| *uid_i < n as u16)
        {
            for (uid_j, weight_ij) in weights_vec
                .into_iter()
                .filter(|(uid_j, _)| *uid_j < n as u16)
            {
                *weights
                    .get_mut(uid_i as usize)
                    .expect("uid_i is filtered to be less than n; qed")
                    .get_mut(uid_j as usize)
                    .expect("uid_j is filtered to be less than n; qed") =
                    I32F32::from_num(weight_ij);
            }
        }
        weights
    }

    /// Output unnormalized sparse bonds, input bonds are assumed to be column max-upscaled in u16.
    pub fn get_bonds_sparse(netuid: u16) -> Vec<Vec<(u16, I32F32)>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut bonds: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
        for (uid_i, bonds_vec) in
            <Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
                .filter(|(uid_i, _)| *uid_i < n as u16)
        {
            for (uid_j, bonds_ij) in bonds_vec {
                bonds
                    .get_mut(uid_i as usize)
                    .expect("uid_i is filtered to be less than n; qed")
                    .push((uid_j, I32F32::from_num(bonds_ij)));
            }
        }
        bonds
    }

    /// Output unnormalized bonds in [n, n] matrix, input bonds are assumed to be column max-upscaled in u16.
    pub fn get_bonds(netuid: u16) -> Vec<Vec<I32F32>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut bonds: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0.0); n]; n];
        for (uid_i, bonds_vec) in
            <Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
                .filter(|(uid_i, _)| *uid_i < n as u16)
        {
            for (uid_j, bonds_ij) in bonds_vec.into_iter().filter(|(uid_j, _)| *uid_j < n as u16) {
                *bonds
                    .get_mut(uid_i as usize)
                    .expect("uid_i has been filtered to be less than n; qed")
                    .get_mut(uid_j as usize)
                    .expect("uid_j has been filtered to be less than n; qed") =
                    I32F32::from_num(bonds_ij);
            }
        }
        bonds
    }

    /// Calculate the logistic function parameters 'a' and 'b' based on alpha and consensus values.
    ///
    /// # Args:
    /// * `alpha_high` - The high alpha value.
    /// * `alpha_low` - The low alpha value.
    /// * `consensus_high` - The high consensus value.
    /// * `consensus_low` - The low consensus value.
    ///
    /// # Returns:
    /// A tuple containing the slope 'a' and intercept 'b' for the logistic function.
    pub fn calculate_logistic_params(
        alpha_high: I32F32,
        alpha_low: I32F32,
        consensus_high: I32F32,
        consensus_low: I32F32,
    ) -> (I32F32, I32F32) {
        log::trace!("alpha_high: {:?}", alpha_high);
        log::trace!("alpha_low: {:?}", alpha_low);
        log::trace!("consensus_high: {:?}", consensus_high);
        log::trace!("consensus_low: {:?}", consensus_low);
        // Check for division by zero
        // extra caution to ensure we never divide by zero
        if consensus_high <= consensus_low || alpha_low == 0 || alpha_high == 0 {
            // Return 0 for both 'a' and 'b' when consensus values are equal
            return (I32F32::from_num(0.0), I32F32::from_num(0.0));
        }

        // Calculate the slope 'a' of the logistic function.
        // a = (ln((1 / alpha_high - 1)) - ln((1 / alpha_low - 1))) / (consensus_low - consensus_high)
        let a = (safe_ln(
            (I32F32::from_num(1.0).saturating_div(alpha_high))
                .saturating_sub(I32F32::from_num(1.0)),
        )
        .saturating_sub(safe_ln(
            (I32F32::from_num(1.0).saturating_div(alpha_low)).saturating_sub(I32F32::from_num(1.0)),
        )))
        .saturating_div(consensus_low.saturating_sub(consensus_high));
        log::trace!("a: {:?}", a);

        // Calculate the intercept 'b' of the logistic function.
        // b = ln((1 / alpha_low - 1)) + a * consensus_low
        let b = safe_ln(
            (I32F32::from_num(1.0).saturating_div(alpha_low)).saturating_sub(I32F32::from_num(1.0)),
        )
        .saturating_add(a.saturating_mul(consensus_low));
        log::trace!("b: {:?}", b);

        // Return the calculated slope 'a' and intercept 'b'.
        (a, b)
    }

    /// Compute the alpha values using the logistic function parameters 'a' and 'b'.
    ///
    /// # Args:
    /// * `consensus` - A vector of consensus values.
    /// * `a` - The slope of the logistic function.
    /// * `b` - The intercept of the logistic function.
    ///
    /// # Returns:
    /// A vector of computed alpha values.
    pub fn compute_alpha_values(consensus: &[I32F32], a: I32F32, b: I32F32) -> Vec<I32F32> {
        // Compute the alpha values for each consensus value.
        let alpha: Vec<I32F32> = consensus
            .iter()
            .map(|c| {
                // Calculate the exponent value for the logistic function.
                // exp_val = exp(b - a * c)
                let exp_val = safe_exp(b.saturating_sub(a.saturating_mul(*c)));

                // Compute the alpha value using the logistic function formula.
                // alpha = 1 / (1 + exp_val)
                I32F32::from_num(1.0).saturating_div(I32F32::from_num(1.0).saturating_add(exp_val))
            })
            .collect();

        // Log the computed alpha values for debugging purposes.
        log::trace!("alpha: {:?}", alpha);

        // Return the computed alpha values.
        alpha
    }

    /// Clamp the alpha values between alpha_high and alpha_low.
    ///
    /// # Args:
    /// * `alpha` - A vector of alpha values.
    /// * `alpha_high` - The high alpha value.
    /// * `alpha_low` - The low alpha value.
    ///
    /// # Returns:
    /// A vector of clamped alpha values.
    pub fn clamp_alpha_values(
        alpha: Vec<I32F32>,
        alpha_high: I32F32,
        alpha_low: I32F32,
    ) -> Vec<I32F32> {
        let clamped_alpha: Vec<I32F32> = alpha
            .iter()
            .map(|a| {
                // First, clamp the value to ensure it does not exceed the upper bound (alpha_high).
                // If 'a' is greater than 'alpha_high', it will be set to 'alpha_high'.
                // If 'a' is less than or equal to 'alpha_high', it remains unchanged.
                let clamped_a = a
                    .min(&alpha_high)
                    // Next, clamp the value to ensure it does not go below the lower bound (alpha_low).
                    // If the value (after the first clamping) is less than 'alpha_low', it will be set to 'alpha_low'.
                    // If the value is greater than or equal to 'alpha_low', it remains unchanged.
                    .max(&alpha_low);
                // Return the clamped value.
                *clamped_a
            })
            .collect();
        log::trace!("alpha_clamped: {:?}", clamped_alpha);
        clamped_alpha
    }

    /// Compute the Exponential Moving Average (EMA) of bonds using the clamped alpha values for a sparse matrix.
    ///
    /// # Args:
    /// * `bonds_delta` - A vector of bond deltas.
    /// * `bonds` - A vector of bonds.
    /// * `alpha` - A vector of clamped alpha values.
    ///
    /// # Returns:
    /// A vector of EMA bonds.
    pub fn compute_ema_bonds_with_liquid_alpha_sparse(
        bonds_delta: &[Vec<(u16, I32F32)>],
        bonds: &[Vec<(u16, I32F32)>],
        alpha: Vec<I32F32>,
    ) -> Vec<Vec<(u16, I32F32)>> {
        // Compute the Exponential Moving Average (EMA) of bonds using the provided clamped alpha values.
        let ema_bonds = mat_ema_alpha_vec_sparse(bonds_delta, bonds, &alpha);

        // Log the computed EMA bonds for debugging purposes.
        log::trace!(
            "Exponential Moving Average Bonds Liquid Alpha: {:?}",
            ema_bonds
        );

        // Return the computed EMA bonds.
        ema_bonds
    }

    /// Compute the Exponential Moving Average (EMA) of bonds using the clamped alpha values.
    ///
    /// # Args:
    /// * `bonds_delta` - A vector of bond deltas.
    /// * `bonds` - A vector of bonds.
    /// * `alpha` - A vector of clamped alpha values.
    ///
    /// # Returns:
    /// A vector of EMA bonds.
    pub fn compute_ema_bonds_with_liquid_alpha(
        bonds_delta: &[Vec<I32F32>],
        bonds: &[Vec<I32F32>],
        alpha: Vec<I32F32>,
    ) -> Vec<Vec<I32F32>> {
        // Compute the Exponential Moving Average (EMA) of bonds using the provided clamped alpha values.
        let ema_bonds = mat_ema_alpha_vec(bonds_delta, bonds, &alpha);

        // Log the computed EMA bonds for debugging purposes.
        log::trace!(
            "Exponential Moving Average Bonds Liquid Alpha: {:?}",
            ema_bonds
        );

        // Return the computed EMA bonds.
        ema_bonds
    }

    /// Compute the Exponential Moving Average (EMA) of bonds using a normal alpha value for a sparse matrix.
    ///
    /// # Args:
    /// * `bonds_delta` - A vector of bond deltas.
    /// * `bonds` - A vector of bonds.
    /// * `netuid` - The network ID.
    ///
    /// # Returns:
    /// A vector of EMA bonds.
    pub fn compute_ema_bonds_normal_sparse(
        bonds_delta: &[Vec<(u16, I32F32)>],
        bonds: &[Vec<(u16, I32F32)>],
        netuid: u16,
    ) -> Vec<Vec<(u16, I32F32)>> {
        // Retrieve the bonds moving average for the given network ID and scale it down.
        let bonds_moving_average: I64F64 = I64F64::from_num(Self::get_bonds_moving_average(netuid))
            .saturating_div(I64F64::from_num(1_000_000));

        // Calculate the alpha value for the EMA calculation.
        // Alpha is derived by subtracting the scaled bonds moving average from 1.
        let alpha: I32F32 =
            I32F32::from_num(1).saturating_sub(I32F32::from_num(bonds_moving_average));

        // Compute the Exponential Moving Average (EMA) of bonds using the calculated alpha value.
        let ema_bonds = mat_ema_sparse(bonds_delta, bonds, alpha);

        // Log the computed EMA bonds for debugging purposes.
        log::trace!("Exponential Moving Average Bonds Normal: {:?}", ema_bonds);

        // Return the computed EMA bonds.
        ema_bonds
    }

    /// Compute the Exponential Moving Average (EMA) of bonds using a normal alpha value.
    ///
    /// # Args:
    /// * `bonds_delta` - A vector of bond deltas.
    /// * `bonds` - A vector of bonds.
    /// * `netuid` - The network ID.
    ///
    /// # Returns:
    /// A vector of EMA bonds.
    pub fn compute_ema_bonds_normal(
        bonds_delta: &[Vec<I32F32>],
        bonds: &[Vec<I32F32>],
        netuid: u16,
    ) -> Vec<Vec<I32F32>> {
        // Retrieve the bonds moving average for the given network ID and scale it down.
        let bonds_moving_average: I64F64 = I64F64::from_num(Self::get_bonds_moving_average(netuid))
            .saturating_div(I64F64::from_num(1_000_000));

        // Calculate the alpha value for the EMA calculation.
        // Alpha is derived by subtracting the scaled bonds moving average from 1.
        let alpha: I32F32 =
            I32F32::from_num(1).saturating_sub(I32F32::from_num(bonds_moving_average));

        // Compute the Exponential Moving Average (EMA) of bonds using the calculated alpha value.
        let ema_bonds = mat_ema(bonds_delta, bonds, alpha);

        // Log the computed EMA bonds for debugging purposes.
        log::trace!("Exponential Moving Average Bonds Normal: {:?}", ema_bonds);

        // Return the computed EMA bonds.
        ema_bonds
    }

    /// Compute the Exponential Moving Average (EMA) of bonds based on the Liquid Alpha setting for a sparse matrix.
    ///
    /// # Args:
    /// * `netuid` - The network ID.
    /// * `consensus` - A vector of consensus values.
    /// * `bonds_delta` - A vector of bond deltas.
    /// * `bonds` - A vector of bonds.
    ///
    /// # Returns:
    /// A vector of EMA bonds.
    pub fn compute_ema_bonds_sparse(
        netuid: u16,
        consensus: Vec<I32F32>,
        bonds_delta: Vec<Vec<(u16, I32F32)>>,
        bonds: Vec<Vec<(u16, I32F32)>>,
    ) -> Vec<Vec<(u16, I32F32)>> {
        // Check if Liquid Alpha is enabled, consensus is not empty, and contains non-zero values.
        // This way we avoid the quantil function panic.
        if LiquidAlphaOn::<T>::get(netuid)
            && !consensus.is_empty()
            && consensus.iter().any(|&c| c != I32F32::from_num(0))
        {
            // Calculate the 75th percentile (high) and 25th percentile (low) of the consensus values.
            let consensus_high = quantile(&consensus, 0.75);
            let consensus_low = quantile(&consensus, 0.25);
            // Further check if the high and low consensus values meet the required conditions.
            if (consensus_high > consensus_low) || consensus_high != 0 || consensus_low < 0 {
                // if (consensus_high > consensus_low) || consensus_high != 0) || consensus_low != 0 {
                // if (consensus_high > consensus_low) || consensus_low != 0 {
                log::trace!("Using Liquid Alpha");

                // Get the high and low alpha values for the network.
                let (alpha_low, alpha_high): (I32F32, I32F32) = Self::get_alpha_values_32(netuid);
                log::trace!("alpha_low: {:?} alpha_high: {:?}", alpha_low, alpha_high);

                // Calculate the logistic function parameters 'a' and 'b' based on alpha and consensus values.
                let (a, b) = Self::calculate_logistic_params(
                    alpha_high,
                    alpha_low,
                    consensus_high,
                    consensus_low,
                );

                // Compute the alpha values using the logistic function parameters.
                let alpha = Self::compute_alpha_values(&consensus, a, b);

                // Clamp the alpha values between alpha_high and alpha_low.
                let clamped_alpha = Self::clamp_alpha_values(alpha, alpha_high, alpha_low);

                // Compute the Exponential Moving Average (EMA) of bonds using the clamped alpha values.
                Self::compute_ema_bonds_with_liquid_alpha_sparse(
                    &bonds_delta,
                    &bonds,
                    clamped_alpha,
                )
            } else {
                log::trace!("Using Bonds Moving Average");

                // Compute the EMA of bonds using a normal alpha value.
                Self::compute_ema_bonds_normal_sparse(&bonds_delta, &bonds, netuid)
            }
        } else {
            log::trace!("Using Bonds Moving Average");

            // Compute the EMA of bonds using a normal alpha value.
            Self::compute_ema_bonds_normal_sparse(&bonds_delta, &bonds, netuid)
        }
    }

    /// Compute the Exponential Moving Average (EMA) of bonds based on the Liquid Alpha setting.
    ///
    /// # Args:
    /// * `netuid` - The network ID.
    /// * `consensus` - A vector of consensus values.
    /// * `bonds_delta` - A vector of bond deltas.
    /// * `bonds` - A vector of bonds.
    ///
    /// # Returns:
    /// A vector of EMA bonds.
    pub fn compute_ema_bonds(
        netuid: u16,
        consensus: Vec<I32F32>,
        bonds_delta: Vec<Vec<I32F32>>,
        bonds: Vec<Vec<I32F32>>,
    ) -> Vec<Vec<I32F32>> {
        // Check if Liquid Alpha is enabled, consensus is not empty, and contains non-zero values.
        if LiquidAlphaOn::<T>::get(netuid)
            && !consensus.is_empty()
            && consensus.iter().any(|&c| c != I32F32::from_num(0))
        {
            // Calculate the 75th percentile (high) and 25th percentile (low) of the consensus values.
            let consensus_high = quantile(&consensus, 0.75);
            let consensus_low = quantile(&consensus, 0.25);

            // Further check if the high and low consensus values meet the required conditions.
            if (consensus_high > consensus_low) || consensus_high != 0 || consensus_low < 0 {
                log::trace!("Using Liquid Alpha");

                // Get the high and low alpha values for the network.
                let (alpha_low, alpha_high): (I32F32, I32F32) = Self::get_alpha_values_32(netuid);
                log::trace!("alpha_low: {:?} alpha_high: {:?}", alpha_low, alpha_high);

                // Calculate the logistic function parameters 'a' and 'b' based on alpha and consensus values.
                let (a, b) = Self::calculate_logistic_params(
                    alpha_high,
                    alpha_low,
                    consensus_high,
                    consensus_low,
                );

                // Compute the alpha values using the logistic function parameters.
                let alpha = Self::compute_alpha_values(&consensus, a, b);

                // Clamp the alpha values between alpha_high and alpha_low.
                let clamped_alpha = Self::clamp_alpha_values(alpha, alpha_high, alpha_low);

                // Compute the Exponential Moving Average (EMA) of bonds using the clamped alpha values.
                Self::compute_ema_bonds_with_liquid_alpha(&bonds_delta, &bonds, clamped_alpha)
            } else {
                log::trace!("Using Bonds Moving Average");

                // Compute the EMA of bonds using a normal alpha value.
                Self::compute_ema_bonds_normal(&bonds_delta, &bonds, netuid)
            }
        } else {
            log::trace!("Using Bonds Moving Average");

            // Compute the EMA of bonds using a normal alpha value.
            Self::compute_ema_bonds_normal(&bonds_delta, &bonds, netuid)
        }
    }

    pub fn do_set_alpha_values(
        origin: T::RuntimeOrigin,
        netuid: u16,
        alpha_low: u16,
        alpha_high: u16,
    ) -> Result<(), DispatchError> {
        // --- 1. Ensure the function caller is a signed user.
        ensure_signed(origin.clone())?;

        // --- 2. Ensure the function caller is the subnet owner or root.
        Self::ensure_subnet_owner_or_root(origin, netuid)?;

        // --- 3. Ensure liquid alpha is enabled
        ensure!(
            Self::get_liquid_alpha_enabled(netuid),
            Error::<T>::LiquidAlphaDisabled
        );

        let max_u16: u32 = u16::MAX as u32; // 65535
        let min_alpha_high: u16 = (max_u16.saturating_mul(4).saturating_div(5)) as u16; // 52428

        // --- 4. Ensure alpha high is greater than the minimum
        ensure!(alpha_high >= min_alpha_high, Error::<T>::AlphaHighTooLow);

        // -- 5. Ensure alpha low is within range
        ensure!(
            alpha_low > 0 && alpha_low < min_alpha_high,
            Error::<T>::AlphaLowOutOfRange
        );

        AlphaValues::<T>::insert(netuid, (alpha_low, alpha_high));

        log::info!(
            "AlphaValuesSet( netuid: {:?}, AlphaLow: {:?}, AlphaHigh: {:?} ) ",
            netuid,
            alpha_low,
            alpha_high,
        );
        Ok(())
    }
}
