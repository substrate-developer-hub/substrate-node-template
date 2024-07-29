use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::storage::IterableStorageMap;
use frame_support::IterableStorageDoubleMap;
use substrate_fixed::types::U64F64;
extern crate alloc;
use codec::Compact;
use sp_core::hexdisplay::AsBytesRef;

#[freeze_struct("5752e4c650a83e0d")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct DelegateInfo<T: Config> {
    delegate_ss58: T::AccountId,
    take: Compact<u16>,
    nominators: Vec<(T::AccountId, Compact<u64>)>, // map of nominator_ss58 to stake amount
    owner_ss58: T::AccountId,
    registrations: Vec<Compact<u16>>, // Vec of netuid this delegate is registered on
    validator_permits: Vec<Compact<u16>>, // Vec of netuid this delegate has validator permit on
    return_per_1000: Compact<u64>, // Delegators current daily return per 1000 TAO staked minus take fee
    total_daily_return: Compact<u64>, // Delegators current daily return
}

impl<T: Config> Pallet<T> {
    fn get_delegate_by_existing_account(delegate: AccountIdOf<T>) -> DelegateInfo<T> {
        let mut nominators = Vec::<(T::AccountId, Compact<u64>)>::new();

        for (nominator, stake) in
            <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix(
                delegate.clone(),
            )
        {
            if stake == 0 {
                continue;
            }
            // Only add nominators with stake
            nominators.push((nominator.clone(), stake.into()));
        }

        let registrations = Self::get_registered_networks_for_hotkey(&delegate.clone());
        let mut validator_permits = Vec::<Compact<u16>>::new();
        let mut emissions_per_day: U64F64 = U64F64::from_num(0);

        for netuid in registrations.iter() {
            if let Ok(uid) = Self::get_uid_for_net_and_hotkey(*netuid, &delegate.clone()) {
                let validator_permit = Self::get_validator_permit_for_uid(*netuid, uid);
                if validator_permit {
                    validator_permits.push((*netuid).into());
                }

                let emission: U64F64 = Self::get_emission_for_uid(*netuid, uid).into();
                let tempo: U64F64 = Self::get_tempo(*netuid).into();
                if tempo > U64F64::from_num(0) {
                    let epochs_per_day: U64F64 = U64F64::from_num(7200).saturating_div(tempo);
                    emissions_per_day =
                        emissions_per_day.saturating_add(emission.saturating_mul(epochs_per_day));
                }
            }
        }

        let owner = Self::get_owning_coldkey_for_hotkey(&delegate.clone());
        let take: Compact<u16> = <Delegates<T>>::get(delegate.clone()).into();

        let total_stake: U64F64 = Self::get_total_stake_for_hotkey(&delegate.clone()).into();

        let return_per_1000: U64F64 = if total_stake > U64F64::from_num(0) {
            emissions_per_day
                .saturating_mul(U64F64::from_num(0.82))
                .saturating_div(total_stake.saturating_div(U64F64::from_num(1000)))
        } else {
            U64F64::from_num(0)
        };

        DelegateInfo {
            delegate_ss58: delegate.clone(),
            take,
            nominators,
            owner_ss58: owner.clone(),
            registrations: registrations.iter().map(|x| x.into()).collect(),
            validator_permits,
            return_per_1000: U64F64::to_num::<u64>(return_per_1000).into(),
            total_daily_return: U64F64::to_num::<u64>(emissions_per_day).into(),
        }
    }

    pub fn get_delegate(delegate_account_vec: Vec<u8>) -> Option<DelegateInfo<T>> {
        if delegate_account_vec.len() != 32 {
            return None;
        }

        let delegate: AccountIdOf<T> =
            T::AccountId::decode(&mut delegate_account_vec.as_bytes_ref()).ok()?;
        // Check delegate exists
        if !<Delegates<T>>::contains_key(delegate.clone()) {
            return None;
        }

        let delegate_info = Self::get_delegate_by_existing_account(delegate.clone());
        Some(delegate_info)
    }

    /// get all delegates info from storage
    ///
    pub fn get_delegates() -> Vec<DelegateInfo<T>> {
        let mut delegates = Vec::<DelegateInfo<T>>::new();
        for delegate in <Delegates<T> as IterableStorageMap<T::AccountId, u16>>::iter_keys() {
            let delegate_info = Self::get_delegate_by_existing_account(delegate.clone());
            delegates.push(delegate_info);
        }

        delegates
    }

    /// get all delegate info and staked token amount for a given delegatee account
    ///
    pub fn get_delegated(delegatee_account_vec: Vec<u8>) -> Vec<(DelegateInfo<T>, Compact<u64>)> {
        let Ok(delegatee) = T::AccountId::decode(&mut delegatee_account_vec.as_bytes_ref()) else {
            return Vec::new(); // No delegates for invalid account
        };

        let mut delegates: Vec<(DelegateInfo<T>, Compact<u64>)> = Vec::new();
        for delegate in <Delegates<T> as IterableStorageMap<T::AccountId, u16>>::iter_keys() {
            let staked_to_this_delegatee =
                Self::get_stake_for_coldkey_and_hotkey(&delegatee.clone(), &delegate.clone());
            if staked_to_this_delegatee == 0 {
                continue; // No stake to this delegate
            }
            // Staked to this delegate, so add to list
            let delegate_info = Self::get_delegate_by_existing_account(delegate.clone());
            delegates.push((delegate_info, staked_to_this_delegatee.into()));
        }

        delegates
    }

    pub fn get_total_delegated_stake(coldkey: &T::AccountId) -> u64 {
        let mut total_delegated = 0u64;

        // Get all hotkeys associated with this coldkey
        let hotkeys = StakingHotkeys::<T>::get(coldkey);

        for hotkey in hotkeys {
            let owner = Owner::<T>::get(&hotkey);

            for (delegator, stake) in Stake::<T>::iter_prefix(&hotkey) {
                if delegator != owner {
                    total_delegated = total_delegated.saturating_add(stake);
                }
            }
        }

        log::info!(
            "Total delegated stake for coldkey {:?}: {}",
            coldkey,
            total_delegated
        );
        total_delegated
    }

    // Helper function to get total delegated stake for a hotkey
    pub fn get_total_hotkey_delegated_stake(hotkey: &T::AccountId) -> u64 {
        let mut total_delegated = 0u64;

        // Iterate through all delegators for this hotkey
        for (delegator, stake) in Stake::<T>::iter_prefix(hotkey) {
            if delegator != Self::get_coldkey_for_hotkey(hotkey) {
                total_delegated = total_delegated.saturating_add(stake);
            }
        }

        total_delegated
    }

    // Helper function to get the coldkey associated with a hotkey
    pub fn get_coldkey_for_hotkey(hotkey: &T::AccountId) -> T::AccountId {
        Owner::<T>::get(hotkey)
    }
}
