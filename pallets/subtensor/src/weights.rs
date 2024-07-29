use super::*;
use crate::math::*;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};
use sp_std::vec;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for committing weight hashes.
    ///
    /// # Args:
    /// * `origin`: (`<T as frame_system::Config>::RuntimeOrigin`):
    ///   - The signature of the committing hotkey.
    ///
    /// * `netuid` (`u16`):
    ///   - The u16 network identifier.
    ///
    /// * `commit_hash` (`H256`):
    ///   - The hash representing the committed weights.
    ///
    /// # Raises:
    /// * `WeightsCommitNotAllowed`:
    ///   - Attempting to commit when it is not allowed.
    ///
    pub fn do_commit_weights(
        origin: T::RuntimeOrigin,
        netuid: u16,
        commit_hash: H256,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        log::info!("do_commit_weights( hotkey:{:?} netuid:{:?})", who, netuid);

        ensure!(
            Self::get_commit_reveal_weights_enabled(netuid),
            Error::<T>::CommitRevealDisabled
        );

        ensure!(
            Self::can_commit(netuid, &who),
            Error::<T>::WeightsCommitNotAllowed
        );

        WeightCommits::<T>::insert(
            netuid,
            &who,
            (commit_hash, Self::get_current_block_as_u64()),
        );
        Ok(())
    }

    /// ---- The implementation for revealing committed weights.
    ///
    /// # Args:
    /// * `origin`: (`<T as frame_system::Config>::RuntimeOrigin`):
    ///   - The signature of the revealing hotkey.
    ///
    /// * `netuid` (`u16`):
    ///   - The u16 network identifier.
    ///
    /// * `uids` (`Vec<u16>`):
    ///   - The uids for the weights being revealed.
    ///
    /// * `values` (`Vec<u16>`):
    ///   - The values of the weights being revealed.
    ///
    /// * `salt` (`Vec<u8>`):
    ///   - The values of the weights being revealed.
    ///
    /// * `version_key` (`u64`):
    ///   - The network version key.
    ///
    /// # Raises:
    /// * `NoWeightsCommitFound`:
    ///   - Attempting to reveal weights without an existing commit.
    ///
    /// * `InvalidRevealCommitHashNotMatchTempo`:
    ///   - Attempting to reveal weights outside the valid tempo.
    ///
    /// * `InvalidRevealCommitHashNotMatch`:
    ///   - The revealed hash does not match the committed hash.
    ///
    pub fn do_reveal_weights(
        origin: T::RuntimeOrigin,
        netuid: u16,
        uids: Vec<u16>,
        values: Vec<u16>,
        salt: Vec<u16>,
        version_key: u64,
    ) -> DispatchResult {
        let who = ensure_signed(origin.clone())?;

        log::info!("do_reveal_weights( hotkey:{:?} netuid:{:?})", who, netuid);

        ensure!(
            Self::get_commit_reveal_weights_enabled(netuid),
            Error::<T>::CommitRevealDisabled
        );

        WeightCommits::<T>::try_mutate_exists(netuid, &who, |maybe_commit| -> DispatchResult {
            let (commit_hash, commit_block) = maybe_commit
                .as_ref()
                .ok_or(Error::<T>::NoWeightsCommitFound)?;

            ensure!(
                Self::is_reveal_block_range(netuid, *commit_block),
                Error::<T>::InvalidRevealCommitTempo
            );

            let provided_hash: H256 = BlakeTwo256::hash_of(&(
                who.clone(),
                netuid,
                uids.clone(),
                values.clone(),
                salt.clone(),
                version_key,
            ));
            ensure!(
                provided_hash == *commit_hash,
                Error::<T>::InvalidRevealCommitHashNotMatch
            );

            Self::do_set_weights(origin, netuid, uids, values, version_key)
        })
    }

    /// ---- The implementation for the extrinsic set_weights.
    ///
    /// # Args:
    ///  * 'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///    - The signature of the calling hotkey.
    ///
    ///  * 'netuid' (u16):
    ///    - The u16 network identifier.
    ///
    ///  * 'uids' ( Vec<u16> ):
    ///    - The uids of the weights to be set on the chain.
    ///
    ///  * 'values' ( Vec<u16> ):
    ///    - The values of the weights to set on the chain.
    ///
    ///  * 'version_key' ( u64 ):
    ///    - The network version key.
    ///
    /// # Event:
    ///  * WeightsSet;
    ///    - On successfully setting the weights on chain.
    ///
    /// # Raises:
    ///  * 'SubNetworkDoesNotExist':
    ///    - Attempting to set weights on a non-existent network.
    ///
    ///  * 'NotRegistered':
    ///    - Attempting to set weights from a non registered account.
    ///
    ///  * 'IncorrectWeightVersionKey':
    ///    - Attempting to set weights without having an up-to-date version_key.
    ///
    ///  * 'SettingWeightsTooFast':
    ///    - Attempting to set weights faster than the weights_set_rate_limit.
    ///
    ///  * 'NeuronNoValidatorPermit':
    ///    - Attempting to set non-self weights without a validator permit.
    ///
    ///  * 'WeightVecNotEqualSize':
    ///    - Attempting to set weights with uids not of same length.
    ///
    ///  * 'DuplicateUids':
    ///    - Attempting to set weights with duplicate uids.
    ///
    /// * 'UidsLengthExceedUidsInSubNet':
    ///    - Attempting to set weights above the max allowed uids.
    ///
    /// * 'UidVecContainInvalidOne':
    ///    - Attempting to set weights with invalid uids.
    ///
    /// * 'WeightVecLengthIsLow':
    ///    - Attempting to set weights with fewer weights than min.
    ///
    /// * 'MaxWeightExceeded':
    ///    - Attempting to set weights with max value exceeding limit.
    ///
    pub fn do_set_weights(
        origin: T::RuntimeOrigin,
        netuid: u16,
        uids: Vec<u16>,
        values: Vec<u16>,
        version_key: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. Check the caller's signature. This is the hotkey of a registered account.
        let hotkey = ensure_signed(origin)?;
        log::info!(
            "do_set_weights( origin:{:?} netuid:{:?}, uids:{:?}, values:{:?})",
            hotkey,
            netuid,
            uids,
            values
        );

        // --- Check that the netuid is not the root network.
        ensure!(
            netuid != Self::get_root_netuid(),
            Error::<T>::CanNotSetRootNetworkWeights
        );

        // --- 2. Check that the length of uid list and value list are equal for this network.
        ensure!(
            Self::uids_match_values(&uids, &values),
            Error::<T>::WeightVecNotEqualSize
        );

        // --- 3. Check to see if this is a valid network.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 4. Check to see if the number of uids is within the max allowed uids for this network.
        ensure!(
            Self::check_len_uids_within_allowed(netuid, &uids),
            Error::<T>::UidsLengthExceedUidsInSubNet
        );

        // --- 5. Check to see if the hotkey is registered to the passed network.
        ensure!(
            Self::is_hotkey_registered_on_network(netuid, &hotkey),
            Error::<T>::HotKeyNotRegisteredInSubNet
        );

        // --- 6. Check to see if the hotkey has enought stake to set weights.
        ensure!(
            Self::get_total_stake_for_hotkey(&hotkey) >= Self::get_weights_min_stake(),
            Error::<T>::NotEnoughStakeToSetWeights
        );

        // --- 7. Ensure version_key is up-to-date.
        ensure!(
            Self::check_version_key(netuid, version_key),
            Error::<T>::IncorrectWeightVersionKey
        );

        // --- 9. Ensure the uid is not setting weights faster than the weights_set_rate_limit.
        let neuron_uid = Self::get_uid_for_net_and_hotkey(netuid, &hotkey)?;
        let current_block: u64 = Self::get_current_block_as_u64();
        ensure!(
            Self::check_rate_limit(netuid, neuron_uid, current_block),
            Error::<T>::SettingWeightsTooFast
        );

        // --- 10. Check that the neuron uid is an allowed validator permitted to set non-self weights.
        ensure!(
            Self::check_validator_permit(netuid, neuron_uid, &uids, &values),
            Error::<T>::NeuronNoValidatorPermit
        );

        // --- 11. Ensure the passed uids contain no duplicates.
        ensure!(!Self::has_duplicate_uids(&uids), Error::<T>::DuplicateUids);

        // --- 12. Ensure that the passed uids are valid for the network.
        ensure!(
            !Self::contains_invalid_uids(netuid, &uids),
            Error::<T>::UidVecContainInvalidOne
        );

        // --- 13. Ensure that the weights have the required length.
        ensure!(
            Self::check_length(netuid, neuron_uid, &uids, &values),
            Error::<T>::WeightVecLengthIsLow
        );

        // --- 14. Max-upscale the weights.
        let max_upscaled_weights: Vec<u16> = vec_u16_max_upscale_to_u16(&values);

        // --- 15. Ensure the weights are max weight limited
        ensure!(
            Self::max_weight_limited(netuid, neuron_uid, &uids, &max_upscaled_weights),
            Error::<T>::MaxWeightExceeded
        );

        // --- 16. Zip weights for sinking to storage map.
        let mut zipped_weights: Vec<(u16, u16)> = vec![];
        for (uid, val) in uids.iter().zip(max_upscaled_weights.iter()) {
            zipped_weights.push((*uid, *val))
        }

        // --- 17. Set weights under netuid, uid double map entry.
        Weights::<T>::insert(netuid, neuron_uid, zipped_weights);

        // --- 18. Set the activity for the weights on this network.
        Self::set_last_update_for_uid(netuid, neuron_uid, current_block);

        // --- 19. Emit the tracking event.
        log::info!(
            "WeightsSet( netuid:{:?}, neuron_uid:{:?} )",
            netuid,
            neuron_uid
        );
        Self::deposit_event(Event::WeightsSet(netuid, neuron_uid));

        // --- 20. Return ok.
        Ok(())
    }

    // ==========================
    // ==== Helper functions ====
    // ==========================

    /// Returns true if version_key is up-to-date.
    ///
    pub fn check_version_key(netuid: u16, version_key: u64) -> bool {
        let network_version_key: u64 = WeightsVersionKey::<T>::get(netuid);
        log::info!(
            "check_version_key( network_version_key:{:?}, version_key:{:?} )",
            network_version_key,
            version_key
        );
        network_version_key == 0 || version_key >= network_version_key
    }

    /// Checks if the neuron has set weights within the weights_set_rate_limit.
    ///
    pub fn check_rate_limit(netuid: u16, neuron_uid: u16, current_block: u64) -> bool {
        if Self::is_uid_exist_on_network(netuid, neuron_uid) {
            // --- 1. Ensure that the diff between current and last_set weights is greater than limit.
            let last_set_weights: u64 = Self::get_last_update_for_uid(netuid, neuron_uid);
            if last_set_weights == 0 {
                return true;
            } // (Storage default) Never set weights.
            return current_block.saturating_sub(last_set_weights)
                >= Self::get_weights_set_rate_limit(netuid);
        }
        // --- 3. Non registered peers cant pass.
        false
    }

    /// Checks for any invalid uids on this network.
    pub fn contains_invalid_uids(netuid: u16, uids: &[u16]) -> bool {
        for uid in uids {
            if !Self::is_uid_exist_on_network(netuid, *uid) {
                log::debug!(
                    "contains_invalid_uids( netuid:{:?}, uid:{:?} does not exist on network. )",
                    netuid,
                    uids
                );
                return true;
            }
        }
        false
    }

    /// Returns true if the passed uids have the same length of the passed values.
    pub fn uids_match_values(uids: &[u16], values: &[u16]) -> bool {
        uids.len() == values.len()
    }

    /// Returns true if the items contain duplicates.
    pub fn has_duplicate_uids(items: &[u16]) -> bool {
        let mut parsed: Vec<u16> = Vec::new();
        for item in items {
            if parsed.contains(item) {
                return true;
            }
            parsed.push(*item);
        }
        false
    }

    /// Returns True if setting self-weight or has validator permit.
    pub fn check_validator_permit(netuid: u16, uid: u16, uids: &[u16], weights: &[u16]) -> bool {
        // Check self weight. Allowed to set single value for self weight.
        if Self::is_self_weight(uid, uids, weights) {
            return true;
        }
        // Check if uid has validator permit.
        Self::get_validator_permit_for_uid(netuid, uid)
    }

    /// Returns True if the uids and weights are have a valid length for uid on network.
    pub fn check_length(netuid: u16, uid: u16, uids: &[u16], weights: &[u16]) -> bool {
        let subnet_n: usize = Self::get_subnetwork_n(netuid) as usize;
        let min_allowed_length: usize = Self::get_min_allowed_weights(netuid) as usize;
        let min_allowed: usize = {
            if subnet_n < min_allowed_length {
                subnet_n
            } else {
                min_allowed_length
            }
        };

        // Check self weight. Allowed to set single value for self weight.
        // Or check that this is the root netuid.
        if netuid != Self::get_root_netuid() && Self::is_self_weight(uid, uids, weights) {
            return true;
        }
        // Check if number of weights exceeds min.
        if weights.len() >= min_allowed {
            return true;
        }
        // To few weights.
        false
    }

    #[allow(clippy::arithmetic_side_effects)]
    /// Returns normalized the passed positive integer weights so that they sum to u16 max value.
    pub fn normalize_weights(mut weights: Vec<u16>) -> Vec<u16> {
        let sum: u64 = weights.iter().map(|x| *x as u64).sum();
        if sum == 0 {
            return weights;
        }
        weights.iter_mut().for_each(|x| {
            *x = (*x as u64)
                .saturating_mul(u16::MAX as u64)
                .saturating_div(sum) as u16;
        });
        weights
    }

    /// Returns False if the weights exceed the max_weight_limit for this network.
    pub fn max_weight_limited(netuid: u16, uid: u16, uids: &[u16], weights: &[u16]) -> bool {
        // Allow self weights to exceed max weight limit.
        if Self::is_self_weight(uid, uids, weights) {
            return true;
        }

        // If the max weight limit it u16 max, return true.
        let max_weight_limit: u16 = Self::get_max_weight_limit(netuid);
        if max_weight_limit == u16::MAX {
            return true;
        }

        // Check if the weights max value is less than or equal to the limit.
        check_vec_max_limited(weights, max_weight_limit)
    }

    /// Returns true if the uids and weights correspond to a self weight on the uid.
    pub fn is_self_weight(uid: u16, uids: &[u16], weights: &[u16]) -> bool {
        if weights.len() != 1 {
            return false;
        }
        let Some(first_uid) = uids.first() else {
            return false;
        };
        if uid != *first_uid {
            return false;
        }
        true
    }

    /// Returns False is the number of uids exceeds the allowed number of uids for this network.
    pub fn check_len_uids_within_allowed(netuid: u16, uids: &[u16]) -> bool {
        let subnetwork_n: u16 = Self::get_subnetwork_n(netuid);
        // we should expect at most subnetwork_n uids.
        uids.len() <= subnetwork_n as usize
    }

    #[allow(clippy::arithmetic_side_effects)]
    pub fn can_commit(netuid: u16, who: &T::AccountId) -> bool {
        if let Some((_hash, commit_block)) = WeightCommits::<T>::get(netuid, who) {
            let interval: u64 = Self::get_commit_reveal_weights_interval(netuid);
            if interval == 0 {
                return true; //prevent division by 0
            }

            let current_block: u64 = Self::get_current_block_as_u64();
            let interval_start: u64 = current_block.saturating_sub(current_block % interval);
            let last_commit_interval_start: u64 =
                commit_block.saturating_sub(commit_block % interval);

            // Allow commit if we're within the interval bounds
            if current_block <= interval_start.saturating_add(interval)
                && interval_start > last_commit_interval_start
            {
                return true;
            }

            false
        } else {
            true
        }
    }

    #[allow(clippy::arithmetic_side_effects)]
    pub fn is_reveal_block_range(netuid: u16, commit_block: u64) -> bool {
        let interval: u64 = Self::get_commit_reveal_weights_interval(netuid);
        if interval == 0 {
            return true; //prevent division by 0
        }

        let commit_interval_start: u64 = commit_block.saturating_sub(commit_block % interval); // Find the start of the interval in which the commit occurred
        let reveal_interval_start: u64 = commit_interval_start.saturating_add(interval); // Start of the next interval after the commit interval
        let current_block: u64 = Self::get_current_block_as_u64();

        // Allow reveal if the current block is within the interval following the commit's interval
        if current_block >= reveal_interval_start
            && current_block < reveal_interval_start.saturating_add(interval)
        {
            return true;
        }

        false
    }
}
