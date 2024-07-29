//! Benchmarking setup
#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::arithmetic_side_effects)]
use super::*;

#[allow(unused)]
use crate::Pallet as AdminUtils;
use frame_benchmarking::v1::account;
use frame_benchmarking::v2::*;
use frame_support::BoundedVec;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn swap_authorities(a: Linear<0, 32>) {
        let mut value: BoundedVec<
            <T as pallet::Config>::AuthorityId,
            <T as pallet::Config>::MaxAuthorities,
        > = BoundedVec::new();

        for idx in 1..=a {
            let authority: <T as pallet::Config>::AuthorityId = account("Authority", idx, 0u32);
            let result = value.try_push(authority.clone());
            if result.is_err() {
                // Handle the error, perhaps by breaking the loop or logging an error message
            }
        }

        #[extrinsic_call]
        _(RawOrigin::Root, value);
    }

    #[benchmark]
    fn sudo_set_default_take() {
        #[extrinsic_call]
		_(RawOrigin::Root, 100u16/*default_take*/)/*sudo_set_default_take*/;
    }

    #[benchmark]
    fn sudo_set_serving_rate_limit() {
        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 100u64/*serving_rate_limit*/)/*sudo_set_serving_rate_limit*/;
    }

    #[benchmark]
    fn sudo_set_max_difficulty() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 10000u64/*max_difficulty*/)/*sudo_set_max_difficulty*/;
    }

    #[benchmark]
    fn sudo_set_min_difficulty() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 1000u64/*min_difficulty*/)/*sudo_set_min_difficulty*/;
    }

    #[benchmark]
    fn sudo_set_weights_set_rate_limit() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 3u64/*rate_limit*/)/*sudo_set_weights_set_rate_limit*/;
    }

    #[benchmark]
    fn sudo_set_weights_version_key() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 1u64/*version_key*/)/*sudo_set_weights_version_key*/;
    }

    #[benchmark]
    fn sudo_set_bonds_moving_average() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 100u64/*bonds_moving_average*/)/*sudo_set_bonds_moving_average*/;
    }

    #[benchmark]
    fn sudo_set_max_allowed_validators() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 10u16/*max_allowed_validators*/)/*sudo_set_max_allowed_validators*/;
    }

    #[benchmark]
    fn sudo_set_difficulty() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 1200000u64/*difficulty*/)/*sudo_set_difficulty*/;
    }

    #[benchmark]
    fn sudo_set_adjustment_interval() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 12u16/*adjustment_interval*/)/*sudo_set_adjustment_interval*/;
    }

    #[benchmark]
    fn sudo_set_target_registrations_per_interval() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 300u16/*target_registrations*/)/*sudo_set_target_registrations_per_interval*/;
    }

    #[benchmark]
    fn sudo_set_activity_cutoff() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 300u16/*activity_cutoff*/)/*sudo_set_activity_cutoff*/;
    }

    #[benchmark]
    fn sudo_set_rho() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 300u16/*rho*/)/*sudo_set_rho*/;
    }

    #[benchmark]
    fn sudo_set_kappa() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*sudo_tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 3u16/*kappa*/)/*set_kappa*/;
    }

    #[benchmark]
    fn sudo_set_max_allowed_uids() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 4097u16/*max_allowed_uids*/)/*sudo_set_max_allowed_uids*/;
    }

    #[benchmark]
    fn sudo_set_min_allowed_weights() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 10u16/*max_allowed_uids*/)/*sudo_set_min_allowed_weights*/;
    }

    #[benchmark]
    fn sudo_set_immunity_period() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 100u16/*immunity_period*/)/*sudo_set_immunity_period*/;
    }

    #[benchmark]
    fn sudo_set_max_weight_limit() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 100u16/*max_weight_limit*/)/*sudo_set_max_weight_limit*/;
    }

    #[benchmark]
    fn sudo_set_max_registrations_per_block() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 100u16/*max_registrations*/)/*sudo_set_max_registrations_per_block*/;
    }

    #[benchmark]
    fn sudo_set_max_burn() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 10u64/*max_burn*/)/*sudo_set_max_burn*/;
    }

    #[benchmark]
    fn sudo_set_min_burn() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 10u64/*min_burn*/)/*sudo_set_min_burn*/;
    }

    #[benchmark]
    fn sudo_set_network_registration_allowed() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, true/*registration_allowed*/)/*sudo_set_network_registration_allowed*/;
    }

    /*
        benchmark_sudo_set_tempo {
        let netuid: u16 = 1;
        let tempo_default: u16 = 1; <------- unused?
        let tempo: u16 = 15;
        let modality: u16 = 0;

        T::Subtensor::init_new_network(netuid, tempo);

    }: sudo_set_tempo(RawOrigin::<AccountIdOf<T>>::Root, netuid, tempo)
    */
    #[benchmark]
    fn sudo_set_tempo() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 1u16/*tempo*/)/*sudo_set_tempo*/;
    }

    #[benchmark]
    fn sudo_set_commit_reveal_weights_interval() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*sudo_tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, 3u64/*interval*/)/*set_commit_reveal_weights_interval()*/;
    }

    #[benchmark]
    fn sudo_set_commit_reveal_weights_enabled() {
        T::Subtensor::init_new_network(1u16 /*netuid*/, 1u16 /*sudo_tempo*/);

        #[extrinsic_call]
		_(RawOrigin::Root, 1u16/*netuid*/, true/*enabled*/)/*set_commit_reveal_weights_enabled*/;
    }

    //impl_benchmark_test_suite!(AdminUtils, crate::mock::new_test_ext(), crate::mock::Test);
}
