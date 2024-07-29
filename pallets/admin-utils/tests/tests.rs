use frame_support::sp_runtime::DispatchError;
use frame_support::{
    assert_err, assert_ok,
    dispatch::{DispatchClass, GetDispatchInfo, Pays},
};
use frame_system::Config;
use pallet_admin_utils::Error;
use pallet_subtensor::Error as SubtensorError;
use pallet_subtensor::{migration, Event};
use sp_core::U256;

mod mock;
use mock::*;

#[test]
fn test_sudo_set_default_take() {
    new_test_ext().execute_with(|| {
        let to_be_set: u16 = 10;
        let init_value: u16 = SubtensorModule::get_default_take();
        assert_eq!(
            AdminUtils::sudo_set_default_take(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_default_take(), init_value);
        assert_ok!(AdminUtils::sudo_set_default_take(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_default_take(), to_be_set);
    });
}

#[test]
fn test_sudo_set_serving_rate_limit() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 3;
        let to_be_set: u64 = 10;
        let init_value: u64 = SubtensorModule::get_serving_rate_limit(netuid);
        assert_eq!(
            AdminUtils::sudo_set_serving_rate_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_serving_rate_limit(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_serving_rate_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_serving_rate_limit(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_min_difficulty() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_min_difficulty(netuid);
        assert_eq!(
            AdminUtils::sudo_set_min_difficulty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_min_difficulty(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_min_difficulty(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_min_difficulty(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_min_difficulty(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_max_difficulty() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_max_difficulty(netuid);
        assert_eq!(
            AdminUtils::sudo_set_max_difficulty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_max_difficulty(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_difficulty(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_max_difficulty(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_max_difficulty(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_weights_version_key() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_weights_version_key(netuid);
        assert_eq!(
            AdminUtils::sudo_set_weights_version_key(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_weights_version_key(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_weights_version_key(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_weights_version_key(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_weights_version_key(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_weights_set_rate_limit() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_weights_set_rate_limit(netuid);
        assert_eq!(
            AdminUtils::sudo_set_weights_set_rate_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_weights_set_rate_limit(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_weights_set_rate_limit(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_weights_set_rate_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_weights_set_rate_limit(netuid),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_adjustment_interval() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_adjustment_interval(netuid);
        assert_eq!(
            AdminUtils::sudo_set_adjustment_interval(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_adjustment_interval(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_adjustment_interval(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_adjustment_interval(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_adjustment_interval(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_adjustment_alpha() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_adjustment_alpha(netuid);
        assert_eq!(
            AdminUtils::sudo_set_adjustment_alpha(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_adjustment_alpha(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_adjustment_alpha(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_adjustment_alpha(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_adjustment_alpha(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_subnet_owner_cut() {
    new_test_ext().execute_with(|| {
        let to_be_set: u16 = 10;
        let init_value: u16 = SubtensorModule::get_subnet_owner_cut();
        assert_eq!(
            AdminUtils::sudo_set_subnet_owner_cut(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_subnet_owner_cut(), init_value);
        assert_ok!(AdminUtils::sudo_set_subnet_owner_cut(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_subnet_owner_cut(), to_be_set);
    });
}

#[test]
fn test_sudo_set_max_weight_limit() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_max_weight_limit(netuid);
        assert_eq!(
            AdminUtils::sudo_set_max_weight_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_max_weight_limit(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_weight_limit(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_max_weight_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_max_weight_limit(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_issuance() {
    new_test_ext().execute_with(|| {
        let to_be_set: u64 = 10;
        assert_eq!(
            AdminUtils::sudo_set_total_issuance(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_ok!(AdminUtils::sudo_set_total_issuance(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_total_issuance(), to_be_set);
    });
}

#[test]
fn test_sudo_set_immunity_period() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_immunity_period(netuid);
        assert_eq!(
            AdminUtils::sudo_set_immunity_period(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_immunity_period(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_immunity_period(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_immunity_period(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_immunity_period(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_min_allowed_weights() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_min_allowed_weights(netuid);
        assert_eq!(
            AdminUtils::sudo_set_min_allowed_weights(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_min_allowed_weights(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_min_allowed_weights(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_min_allowed_weights(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_min_allowed_weights(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_max_allowed_uids() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_max_allowed_uids(netuid);
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_and_decrease_max_allowed_uids() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_max_allowed_uids(netuid);
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_ok!(AdminUtils::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set - 1
        ));
    });
}

#[test]
fn test_sudo_set_kappa() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_kappa(netuid);
        assert_eq!(
            AdminUtils::sudo_set_kappa(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_kappa(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_kappa(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_kappa(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_kappa(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_rho() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_rho(netuid);
        assert_eq!(
            AdminUtils::sudo_set_rho(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_rho(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_rho(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_rho(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_rho(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_activity_cutoff() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_activity_cutoff(netuid);
        assert_eq!(
            AdminUtils::sudo_set_activity_cutoff(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_activity_cutoff(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_activity_cutoff(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_activity_cutoff(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_activity_cutoff(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_target_registrations_per_interval() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_target_registrations_per_interval(netuid);
        assert_eq!(
            AdminUtils::sudo_set_target_registrations_per_interval(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_target_registrations_per_interval(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_target_registrations_per_interval(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_target_registrations_per_interval(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_target_registrations_per_interval(netuid),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_difficulty() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_difficulty_as_u64(netuid);
        assert_eq!(
            AdminUtils::sudo_set_difficulty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_difficulty(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_difficulty(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_max_allowed_validators() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_max_allowed_validators(netuid);
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_validators(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_validators(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_max_allowed_validators(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_max_allowed_validators(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_max_allowed_validators(netuid),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_weights_min_stake() {
    new_test_ext().execute_with(|| {
        let to_be_set: u64 = 10;
        let init_value: u64 = SubtensorModule::get_weights_min_stake();
        assert_eq!(
            AdminUtils::sudo_set_weights_min_stake(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_weights_min_stake(), init_value);
        assert_ok!(AdminUtils::sudo_set_weights_min_stake(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_weights_min_stake(), to_be_set);
    });
}

#[test]
fn test_sudo_set_bonds_moving_average() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_bonds_moving_average(netuid);
        assert_eq!(
            AdminUtils::sudo_set_bonds_moving_average(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_bonds_moving_average(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_bonds_moving_average(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_bonds_moving_average(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_bonds_moving_average(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_rao_recycled() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_rao_recycled(netuid);

        // Need to run from genesis block
        run_to_block(1);

        assert_eq!(
            AdminUtils::sudo_set_rao_recycled(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_rao_recycled(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_rao_recycled(netuid), init_value);

        // Verify no events emitted matching the expected event
        assert_eq!(
            System::events()
                .iter()
                .filter(|r| r.event
                    == RuntimeEvent::SubtensorModule(Event::RAORecycledForRegistrationSet(
                        netuid, to_be_set
                    )))
                .count(),
            0
        );

        assert_ok!(AdminUtils::sudo_set_rao_recycled(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_rao_recycled(netuid), to_be_set);

        // Verify event emitted with correct values
        assert_eq!(
            System::events()
                .last()
                .unwrap_or_else(|| panic!(
                    "Expected there to be events: {:?}",
                    System::events().to_vec()
                ))
                .event,
            RuntimeEvent::SubtensorModule(Event::RAORecycledForRegistrationSet(netuid, to_be_set))
        );
    });
}

#[test]
fn test_sudo_set_subnet_limit() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);

        let init_value: u16 = SubtensorModule::get_max_subnets();
        assert_eq!(
            AdminUtils::sudo_set_subnet_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_max_subnets(), init_value);
        assert_ok!(AdminUtils::sudo_set_subnet_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_max_subnets(), to_be_set);
    });
}

#[test]
fn test_sudo_set_network_lock_reduction_interval() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 7200;
        add_network(netuid, 10);

        let init_value: u64 = SubtensorModule::get_lock_reduction_interval();
        assert_eq!(
            AdminUtils::sudo_set_lock_reduction_interval(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_lock_reduction_interval(), init_value);
        assert_ok!(AdminUtils::sudo_set_lock_reduction_interval(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_lock_reduction_interval(), to_be_set);
    });
}

#[test]
fn test_sudo_set_network_pow_registration_allowed() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: bool = true;
        add_network(netuid, 10);

        let init_value: bool = SubtensorModule::get_network_pow_registration_allowed(netuid);
        assert_eq!(
            AdminUtils::sudo_set_network_pow_registration_allowed(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            SubtensorModule::get_network_pow_registration_allowed(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_network_pow_registration_allowed(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_network_pow_registration_allowed(netuid),
            to_be_set
        );
    });
}

mod sudo_set_nominator_min_required_stake {
    use super::*;

    #[test]
    fn can_only_be_called_by_admin() {
        new_test_ext().execute_with(|| {
            let to_be_set: u64 = SubtensorModule::get_nominator_min_required_stake() + 5_u64;
            assert_eq!(
                AdminUtils::sudo_set_nominator_min_required_stake(
                    <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                    to_be_set
                ),
                Err(DispatchError::BadOrigin)
            );
        });
    }

    #[test]
    fn sets_a_lower_value() {
        new_test_ext().execute_with(|| {
            assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
                <<Test as Config>::RuntimeOrigin>::root(),
                10u64
            ));
            assert_eq!(SubtensorModule::get_nominator_min_required_stake(), 10u64);

            assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
                <<Test as Config>::RuntimeOrigin>::root(),
                5u64
            ));
            assert_eq!(SubtensorModule::get_nominator_min_required_stake(), 5u64);
        });
    }

    #[test]
    fn sets_a_higher_value() {
        new_test_ext().execute_with(|| {
            let to_be_set: u64 = SubtensorModule::get_nominator_min_required_stake() + 5_u64;
            assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
                <<Test as Config>::RuntimeOrigin>::root(),
                to_be_set
            ));
            assert_eq!(
                SubtensorModule::get_nominator_min_required_stake(),
                to_be_set
            );
        });
    }

    #[test]
    fn clears_staker_nominations_below_min() {
        new_test_ext().execute_with(|| {
            System::set_block_number(1);

            // Create accounts.
            let netuid = 1;
            let hot1 = U256::from(1);
            let hot2 = U256::from(2);
            let cold1 = U256::from(3);
            let cold2 = U256::from(4);

            SubtensorModule::set_target_stakes_per_interval(10);
            // Register network.
            add_network(netuid, 0);

            // Register hot1.
            register_ok_neuron(netuid, hot1, cold1, 0);
            assert_ok!(SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(cold1),
                hot1,
                u16::MAX / 10
            ));
            assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot1), cold1);

            // Register hot2.
            register_ok_neuron(netuid, hot2, cold2, 0);
            assert_ok!(SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(cold2),
                hot2,
                u16::MAX / 10
            ));
            assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot2), cold2);

            // Add stake cold1 --> hot1 (non delegation.)
            SubtensorModule::add_balance_to_coldkey_account(&cold1, 5);
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold1),
                hot1,
                1
            ));
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot1),
                1
            );
            assert_eq!(Balances::free_balance(cold1), 4);

            // Add stake cold2 --> hot1 (is delegation.)
            SubtensorModule::add_balance_to_coldkey_account(&cold2, 5);
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold2),
                hot1,
                1
            ));
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot1),
                1
            );
            assert_eq!(Balances::free_balance(cold2), 4);

            // Add stake cold1 --> hot2 (non delegation.)
            SubtensorModule::add_balance_to_coldkey_account(&cold1, 5);
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold1),
                hot2,
                1
            ));
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot2),
                1
            );
            assert_eq!(Balances::free_balance(cold1), 8);

            // Add stake cold2 --> hot2 (is delegation.)
            SubtensorModule::add_balance_to_coldkey_account(&cold2, 5);
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold2),
                hot2,
                1
            ));
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot2),
                1
            );
            assert_eq!(Balances::free_balance(cold2), 8);

            // Set min stake to 0 (noop)
            assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
                <<Test as Config>::RuntimeOrigin>::root(),
                0u64
            ));
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot1),
                1
            );
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot2),
                1
            );
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot1),
                1
            );
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot2),
                1
            );

            // Set min nomination to 10: should clear (cold2, hot1) and (cold1, hot2).
            assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
                <<Test as Config>::RuntimeOrigin>::root(),
                10u64
            ));
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot1),
                1
            );
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot2),
                0
            );
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot1),
                0
            );
            assert_eq!(
                SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot2),
                1
            );

            // Balances have been added back into accounts.
            assert_eq!(Balances::free_balance(cold1), 9);
            assert_eq!(Balances::free_balance(cold2), 9);
        });
    }
}

#[test]
fn test_sudo_set_tx_delegate_take_rate_limit() {
    new_test_ext().execute_with(|| {
        let to_be_set: u64 = 10;
        let init_value: u64 = SubtensorModule::get_tx_delegate_take_rate_limit();
        assert_eq!(
            AdminUtils::sudo_set_tx_delegate_take_rate_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            SubtensorModule::get_tx_delegate_take_rate_limit(),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_tx_delegate_take_rate_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_tx_delegate_take_rate_limit(),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_min_delegate_take() {
    new_test_ext().execute_with(|| {
        let to_be_set = u16::MAX / 100;
        let init_value = SubtensorModule::get_min_delegate_take();
        assert_eq!(
            AdminUtils::sudo_set_min_delegate_take(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_min_delegate_take(), init_value);
        assert_ok!(AdminUtils::sudo_set_min_delegate_take(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_min_delegate_take(), to_be_set);
    });
}

#[test]
fn test_sudo_set_weight_commit_interval() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 10);

        let to_be_set = 55;
        let init_value = SubtensorModule::get_commit_reveal_weights_interval(netuid);

        assert_ok!(AdminUtils::sudo_set_commit_reveal_weights_interval(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));

        assert!(init_value != to_be_set);
        assert_eq!(
            SubtensorModule::get_commit_reveal_weights_interval(netuid),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_commit_reveal_weights_enabled() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 10);

        let to_be_set: bool = true;
        let init_value: bool = SubtensorModule::get_commit_reveal_weights_enabled(netuid);

        assert_ok!(AdminUtils::sudo_set_commit_reveal_weights_enabled(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));

        assert!(init_value != to_be_set);
        assert_eq!(
            SubtensorModule::get_commit_reveal_weights_enabled(netuid),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_target_stakes_per_interval() {
    new_test_ext().execute_with(|| {
        let to_be_set = 100;
        let init_value = SubtensorModule::get_target_stakes_per_interval();
        assert_eq!(
            AdminUtils::sudo_set_target_stakes_per_interval(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            SubtensorModule::get_target_stakes_per_interval(),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_target_stakes_per_interval(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_target_stakes_per_interval(), to_be_set);
    });
}

#[test]
fn test_sudo_set_liquid_alpha_enabled() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let enabled: bool = true;
        assert_eq!(!enabled, SubtensorModule::get_liquid_alpha_enabled(netuid));

        assert_ok!(AdminUtils::sudo_set_liquid_alpha_enabled(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            enabled
        ));

        assert_eq!(enabled, SubtensorModule::get_liquid_alpha_enabled(netuid));
    });
}

#[test]
fn test_set_alpha_values_dispatch_info_ok() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let alpha_low: u16 = 12_u16;
        let alpha_high: u16 = u16::MAX - 10;
        let call = RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_alpha_values {
            netuid,
            alpha_low,
            alpha_high,
        });

        let dispatch_info = call.get_dispatch_info();

        assert_eq!(dispatch_info.class, DispatchClass::Operational);
        assert_eq!(dispatch_info.pays_fee, Pays::No);
    });
}

#[test]
fn test_sudo_get_set_alpha() {
    new_test_ext().execute_with(|| {
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
            AdminUtils::sudo_set_alpha_values(signer.clone(), netuid, alpha_low, alpha_high),
            DispatchError::BadOrigin
        );

        assert_ok!(SubtensorModule::register_network(signer.clone()));

        assert_ok!(AdminUtils::sudo_set_alpha_values(
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
            AdminUtils::sudo_set_alpha_values(signer.clone(), netuid, alpha_low, alpha_high),
            SubtensorError::<Test>::LiquidAlphaDisabled
        );
        // Correct scenario after error
        SubtensorModule::set_liquid_alpha_enabled(netuid, true); // Re-enable for further tests
        assert_ok!(AdminUtils::sudo_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        // 2. Alpha high too low
        let alpha_high_too_low = (u16::MAX as u32 * 4 / 5) as u16 - 1; // One less than the minimum acceptable value
        assert_err!(
            AdminUtils::sudo_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low,
                alpha_high_too_low
            ),
            SubtensorError::<Test>::AlphaHighTooLow
        );
        // Correct scenario after error
        assert_ok!(AdminUtils::sudo_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        // 3. Alpha low too low or too high
        let alpha_low_too_low = 0_u16;
        assert_err!(
            AdminUtils::sudo_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low_too_low,
                alpha_high
            ),
            SubtensorError::<Test>::AlphaLowOutOfRange
        );
        // Correct scenario after error
        assert_ok!(AdminUtils::sudo_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        let alpha_low_too_high = (u16::MAX as u32 * 4 / 5) as u16 + 1; // One more than the maximum acceptable value
        assert_err!(
            AdminUtils::sudo_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low_too_high,
                alpha_high
            ),
            SubtensorError::<Test>::AlphaLowOutOfRange
        );
        // Correct scenario after error
        assert_ok!(AdminUtils::sudo_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));
    });
}
