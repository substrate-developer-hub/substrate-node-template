//! Benchmarking setup
#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::arithmetic_side_effects)]
use super::*;

#[allow(unused)]
use crate::Pallet as Commitments;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_std::vec;

use sp_runtime::traits::Bounded;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

// This creates an `IdentityInfo` object with `num_fields` extra fields.
// All data is pre-populated with some arbitrary bytes.
fn create_identity_info<T: Config>(_num_fields: u32) -> CommitmentInfo<T::MaxFields> {
    let _data = Data::Raw(
        vec![0; 32]
            .try_into()
            .expect("vec length is less than 64; qed"),
    );

    CommitmentInfo {
        fields: Default::default(),
    }
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_commitment() {
        // The target user
        let netuid = 1;
        let caller: T::AccountId = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller.clone()),
            netuid,
            Box::new(create_identity_info::<T>(0)),
        );

        assert_last_event::<T>(
            Event::<T>::Commitment {
                netuid,
                who: caller,
            }
            .into(),
        );
    }

    //impl_benchmark_test_suite!(Commitments, crate::tests::new_test_ext(), crate::tests::Test);
}
