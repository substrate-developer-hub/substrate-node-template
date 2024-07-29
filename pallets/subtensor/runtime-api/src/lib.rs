#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use alloc::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// src/neuron_info.rs, src/subnet_info.rs, and src/delegate_info.rs
sp_api::decl_runtime_apis! {
    pub trait DelegateInfoRuntimeApi {
        fn get_delegates() -> Vec<u8>;
        fn get_delegate( delegate_account_vec: Vec<u8> ) -> Vec<u8>;
        fn get_delegated( delegatee_account_vec: Vec<u8> ) -> Vec<u8>;
    }

    pub trait NeuronInfoRuntimeApi {
        fn get_neurons(netuid: u16) -> Vec<u8>;
        fn get_neuron(netuid: u16, uid: u16) -> Vec<u8>;
        fn get_neurons_lite(netuid: u16) -> Vec<u8>;
        fn get_neuron_lite(netuid: u16, uid: u16) -> Vec<u8>;
    }

    pub trait SubnetInfoRuntimeApi {
        fn get_subnet_info(netuid: u16) -> Vec<u8>;
        fn get_subnets_info() -> Vec<u8>;
        fn get_subnet_hyperparams(netuid: u16) -> Vec<u8>;
    }

    pub trait StakeInfoRuntimeApi {
        fn get_stake_info_for_coldkey( coldkey_account_vec: Vec<u8> ) -> Vec<u8>;
        fn get_stake_info_for_coldkeys( coldkey_account_vecs: Vec<Vec<u8>> ) -> Vec<u8>;
    }

    pub trait SubnetRegistrationRuntimeApi {
        fn get_network_registration_cost() -> u64;
    }
}
