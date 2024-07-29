use frame_support::{assert_ok, traits::Currency};
use frame_system::Config;
use sp_core::U256;
mod mock;
use mock::*;

#[test]
fn test_batch_txs() {
    let alice = U256::from(0);
    let bob = U256::from(1);
    let charlie = U256::from(2);
    let initial_balances = vec![
        (alice, 8_000_000_000),
        (bob, 1_000_000_000),
        (charlie, 1_000_000_000),
    ];
    test_ext_with_balances(initial_balances).execute_with(|| {
        assert_ok!(Utility::batch(
            <<Test as Config>::RuntimeOrigin>::signed(alice),
            vec![
                RuntimeCall::Balances(BalanceCall::transfer_allow_death {
                    dest: bob,
                    value: 1_000_000_000
                }),
                RuntimeCall::Balances(BalanceCall::transfer_allow_death {
                    dest: charlie,
                    value: 1_000_000_000
                })
            ]
        ));
        assert_eq!(Balances::total_balance(&alice), 6_000_000_000);
        assert_eq!(Balances::total_balance(&bob), 2_000_000_000);
        assert_eq!(Balances::total_balance(&charlie), 2_000_000_000);
    });
}
