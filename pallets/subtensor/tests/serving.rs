use crate::mock::*;
mod mock;
use frame_support::{
    assert_ok,
    dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays},
};
use frame_system::Config;
use pallet_subtensor::Error;
use sp_core::U256;

mod test {
    use std::net::{Ipv4Addr, Ipv6Addr};

    // Generates an ipv6 address based on 8 ipv6 words and returns it as u128
    #[allow(clippy::too_many_arguments)]
    pub fn ipv6(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> u128 {
        Ipv6Addr::new(a, b, c, d, e, f, g, h).into()
    }

    // Generate an ipv4 address based on 4 bytes and returns the corresponding u128, so it can be fed
    // to the module::subscribe() function
    pub fn ipv4(a: u8, b: u8, c: u8, d: u8) -> u128 {
        let ipv4: Ipv4Addr = Ipv4Addr::new(a, b, c, d);
        let integer: u32 = ipv4.into();
        u128::from(integer)
    }
}

#[test]
fn test_serving_subscribe_ok_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let protocol: u8 = 0;
        let placeholder1: u8 = 0;
        let placeholder2: u8 = 0;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::serve_axon {
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2,
        });
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(246_000_000, 0),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
    });
}

#[test]
fn test_serving_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let modality: u16 = 0;
        let protocol: u8 = 0;
        let placeholder1: u8 = 0;
        let placeholder2: u8 = 0;
        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey_account_id, U256::from(66), 0);
        assert_ok!(SubtensorModule::serve_axon(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2
        ));
        let neuron = SubtensorModule::get_axon_info(netuid, &hotkey_account_id);
        assert_eq!(neuron.ip, ip);
        assert_eq!(neuron.version, version);
        assert_eq!(neuron.port, port);
        assert_eq!(neuron.ip_type, ip_type);
        assert_eq!(neuron.protocol, protocol);
        assert_eq!(neuron.placeholder1, placeholder1);
        assert_eq!(neuron.placeholder2, placeholder2);
    });
}

#[test]
fn test_serving_set_metadata_update() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let modality: u16 = 0;
        let protocol: u8 = 0;
        let placeholder1: u8 = 0;
        let placeholder2: u8 = 0;
        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey_account_id, U256::from(66), 0);
        assert_ok!(SubtensorModule::serve_axon(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2
        ));
        let neuron = SubtensorModule::get_axon_info(netuid, &hotkey_account_id);
        assert_eq!(neuron.ip, ip);
        assert_eq!(neuron.version, version);
        assert_eq!(neuron.port, port);
        assert_eq!(neuron.ip_type, ip_type);
        assert_eq!(neuron.protocol, protocol);
        assert_eq!(neuron.placeholder1, placeholder1);
        assert_eq!(neuron.placeholder2, placeholder2);
        let version2: u32 = version + 1;
        let ip2: u128 = ip + 1;
        let port2: u16 = port + 1;
        let ip_type2: u8 = 6;
        let protocol2: u8 = protocol + 1;
        let placeholder12: u8 = placeholder1 + 1;
        let placeholder22: u8 = placeholder2 + 1;
        assert_ok!(SubtensorModule::serve_axon(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version2,
            ip2,
            port2,
            ip_type2,
            protocol2,
            placeholder12,
            placeholder22
        ));
        let neuron = SubtensorModule::get_axon_info(netuid, &hotkey_account_id);
        assert_eq!(neuron.ip, ip2);
        assert_eq!(neuron.version, version2);
        assert_eq!(neuron.port, port2);
        assert_eq!(neuron.ip_type, ip_type2);
        assert_eq!(neuron.protocol, protocol2);
        assert_eq!(neuron.placeholder1, placeholder12);
        assert_eq!(neuron.placeholder2, placeholder22);
    });
}

#[test]
#[cfg(not(tarpaulin))]
fn test_axon_serving_rate_limit_exceeded() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let modality: u16 = 0;
        let protocol: u8 = 0;
        let placeholder1: u8 = 0;
        let placeholder2: u8 = 0;
        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey_account_id, U256::from(66), 0);
        run_to_block(1); // Go to block 1
                         // No issue on multiple
        assert_ok!(SubtensorModule::serve_axon(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2
        ));
        assert_ok!(SubtensorModule::serve_axon(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2
        ));
        assert_ok!(SubtensorModule::serve_axon(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2
        ));
        assert_ok!(SubtensorModule::serve_axon(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2
        ));
        SubtensorModule::set_serving_rate_limit(netuid, 2);
        run_to_block(2); // Go to block 2
                         // Needs to be 2 blocks apart, we are only 1 block apart
        assert_eq!(
            SubtensorModule::serve_axon(
                <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
                netuid,
                version,
                ip,
                port,
                ip_type,
                protocol,
                placeholder1,
                placeholder2
            ),
            Err(Error::<Test>::ServingRateLimitExceeded.into())
        );
    });
}

#[test]
fn test_axon_invalid_port() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 0;
        let ip_type: u8 = 4;
        let modality: u16 = 0;
        let protocol: u8 = 0;
        let placeholder1: u8 = 0;
        let placeholder2: u8 = 0;
        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey_account_id, U256::from(66), 0);
        run_to_block(1); // Go to block 1
        assert_eq!(
            SubtensorModule::serve_axon(
                <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
                netuid,
                version,
                ip,
                port,
                ip_type,
                protocol,
                placeholder1,
                placeholder2
            ),
            Err(Error::<Test>::InvalidPort.into())
        );
    });
}

#[test]
fn test_prometheus_serving_subscribe_ok_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::serve_prometheus {
            netuid,
            version,
            ip,
            port,
            ip_type,
        });
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(245_000_000, 0),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
    });
}

#[test]
fn test_prometheus_serving_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let modality: u16 = 0;
        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey_account_id, U256::from(66), 0);
        assert_ok!(SubtensorModule::serve_prometheus(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type
        ));
        let neuron = SubtensorModule::get_prometheus_info(netuid, &hotkey_account_id);
        assert_eq!(neuron.ip, ip);
        assert_eq!(neuron.version, version);
        assert_eq!(neuron.port, port);
        assert_eq!(neuron.ip_type, ip_type);
    });
}

#[test]
fn test_prometheus_serving_set_metadata_update() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let modality: u16 = 0;
        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey_account_id, U256::from(66), 0);
        assert_ok!(SubtensorModule::serve_prometheus(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type
        ));
        let neuron = SubtensorModule::get_prometheus_info(netuid, &hotkey_account_id);
        assert_eq!(neuron.ip, ip);
        assert_eq!(neuron.version, version);
        assert_eq!(neuron.port, port);
        assert_eq!(neuron.ip_type, ip_type);
        let version2: u32 = version + 1;
        let ip2: u128 = ip + 1;
        let port2: u16 = port + 1;
        let ip_type2: u8 = 6;
        assert_ok!(SubtensorModule::serve_prometheus(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version2,
            ip2,
            port2,
            ip_type2
        ));
        let neuron = SubtensorModule::get_prometheus_info(netuid, &hotkey_account_id);
        assert_eq!(neuron.ip, ip2);
        assert_eq!(neuron.version, version2);
        assert_eq!(neuron.port, port2);
        assert_eq!(neuron.ip_type, ip_type2);
    });
}

#[test]
#[cfg(not(tarpaulin))]
fn test_prometheus_serving_rate_limit_exceeded() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let modality: u16 = 0;
        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey_account_id, U256::from(66), 0);
        run_to_block(1); // Go to block 1
                         // No issue on multiple
        assert_ok!(SubtensorModule::serve_prometheus(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type
        ));
        assert_ok!(SubtensorModule::serve_prometheus(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type
        ));
        assert_ok!(SubtensorModule::serve_prometheus(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type
        ));
        assert_ok!(SubtensorModule::serve_prometheus(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            version,
            ip,
            port,
            ip_type
        ));
        SubtensorModule::set_serving_rate_limit(netuid, 1);
        // Same block, need 1 block to pass
        assert_eq!(
            SubtensorModule::serve_prometheus(
                <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
                netuid,
                version,
                ip,
                port,
                ip_type
            ),
            Err(Error::<Test>::ServingRateLimitExceeded.into())
        );
    });
}

#[test]
fn test_prometheus_invalid_port() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 0;
        let ip_type: u8 = 4;
        let modality: u16 = 0;
        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey_account_id, U256::from(66), 0);
        run_to_block(1); // Go to block 1
        assert_eq!(
            SubtensorModule::serve_prometheus(
                <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
                netuid,
                version,
                ip,
                port,
                ip_type
            ),
            Err(Error::<Test>::InvalidPort.into())
        );
    });
}

#[test]
fn test_serving_is_valid_ip_type_ok_ipv4() {
    new_test_ext(1).execute_with(|| {
        assert!(SubtensorModule::is_valid_ip_type(4));
    });
}

#[test]
fn test_serving_is_valid_ip_type_ok_ipv6() {
    new_test_ext(1).execute_with(|| {
        assert!(SubtensorModule::is_valid_ip_type(6));
    });
}

#[test]
fn test_serving_is_valid_ip_type_nok() {
    new_test_ext(1).execute_with(|| {
        assert!(!SubtensorModule::is_valid_ip_type(10));
    });
}

#[test]
fn test_serving_is_valid_ip_address_ipv4() {
    new_test_ext(1).execute_with(|| {
        assert!(SubtensorModule::is_valid_ip_address(
            4,
            test::ipv4(8, 8, 8, 8)
        ));
    });
}

#[test]
fn test_serving_is_valid_ip_address_ipv6() {
    new_test_ext(1).execute_with(|| {
        assert!(SubtensorModule::is_valid_ip_address(
            6,
            test::ipv6(1, 2, 3, 4, 5, 6, 7, 8)
        ));
        assert!(SubtensorModule::is_valid_ip_address(
            6,
            test::ipv6(1, 2, 3, 4, 5, 6, 7, 8)
        ));
    });
}

#[test]
fn test_serving_is_invalid_ipv4_address() {
    new_test_ext(1).execute_with(|| {
        assert!(!SubtensorModule::is_valid_ip_address(
            4,
            test::ipv4(0, 0, 0, 0)
        ));
        assert!(!SubtensorModule::is_valid_ip_address(
            4,
            test::ipv4(255, 255, 255, 255)
        ));
        assert!(!SubtensorModule::is_valid_ip_address(
            4,
            test::ipv4(127, 0, 0, 1)
        ));
        assert!(!SubtensorModule::is_valid_ip_address(
            4,
            test::ipv6(0xffff, 2, 3, 4, 5, 6, 7, 8)
        ));
    });
}

#[test]
fn test_serving_is_invalid_ipv6_address() {
    new_test_ext(1).execute_with(|| {
        assert!(!SubtensorModule::is_valid_ip_address(
            6,
            test::ipv6(0, 0, 0, 0, 0, 0, 0, 0)
        ));
        assert!(!SubtensorModule::is_valid_ip_address(
            4,
            test::ipv6(0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff)
        ));
    });
}
