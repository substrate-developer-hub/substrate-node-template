// Tests to be written here
use super::*;
use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use codec::{Decode, Encode};
use sp_core::{sr25519, Blake2Hasher, Pair, Public, H256};
use sp_runtime::traits::IdentifyAccount;

// reproduce with "substratee_worker dump_ra"
const TEST1_CERT: &[u8] =
	include_bytes!("../host-calls/test/test_ra_cert_MRSIGNER1_MRENCLAVE1.der");
const TEST2_CERT: &[u8] =
	include_bytes!("../host-calls/test/test_ra_cert_MRSIGNER2_MRENCLAVE2.der");
const TEST3_CERT: &[u8] =
	include_bytes!("../host-calls/test/test_ra_cert_MRSIGNER3_MRENCLAVE2.der");
const TEST1_SIGNER_ATTN: &[u8] =
	include_bytes!("../host-calls/test/test_ra_signer_attn_MRSIGNER1_MRENCLAVE1.bin");
const TEST2_SIGNER_ATTN: &[u8] =
	include_bytes!("../host-calls/test/test_ra_signer_attn_MRSIGNER2_MRENCLAVE2.bin");
const TEST3_SIGNER_ATTN: &[u8] =
	include_bytes!("../host-calls/test/test_ra_signer_attn_MRSIGNER3_MRENCLAVE2.bin");
// reproduce with "substratee_worker getsignkey"
const TEST1_SIGNER_PUB: &[u8] =
	include_bytes!("../host-calls/test/test_ra_signer_pubkey_MRSIGNER1_MRENCLAVE1.bin");
const TEST2_SIGNER_PUB: &[u8] =
	include_bytes!("../host-calls/test/test_ra_signer_pubkey_MRSIGNER2_MRENCLAVE2.bin");
const TEST3_SIGNER_PUB: &[u8] =
	include_bytes!("../host-calls/test/test_ra_signer_pubkey_MRSIGNER3_MRENCLAVE2.bin");

// reproduce with "make mrenclave" in worker repo root
const TEST1_MRENCLAVE: [u8; 32] = [
	62, 252, 187, 232, 60, 135, 108, 204, 87, 78, 35, 169, 241, 237, 106, 217, 251, 241, 99,
	189, 138, 157, 86, 136, 77, 91, 93, 23, 192, 104, 140, 167,
];
const TEST2_MRENCLAVE: [u8; 32] = [
	4, 190, 230, 132, 211, 129, 59, 237, 101, 78, 55, 174, 144, 177, 91, 134, 1, 240, 27, 174,
	81, 139, 8, 22, 32, 241, 228, 103, 189, 43, 44, 102,
];
const TEST3_MRENCLAVE: [u8; 32] = [
	4, 190, 230, 132, 211, 129, 59, 237, 101, 78, 55, 174, 144, 177, 91, 134, 1, 240, 27, 174,
	81, 139, 8, 22, 32, 241, 228, 103, 189, 43, 44, 102,
];
// unix epoch. must be later than this
const TEST1_TIMESTAMP: i64 = 1580587262i64;
const TEST2_TIMESTAMP: i64 = 1581259412i64;
const TEST3_TIMESTAMP: i64 = 1581259975i64;

//    const WASM_CODE: &'static [u8] = include_bytes!("../wasm/target/wasm32-unknown-unknown/release/substratee_node_runtime_wasm.compact.wasm");
//const CERT: &[u8] = b"0\x82\x0c\x8c0\x82\x0c2\xa0\x03\x02\x01\x02\x02\x01\x010\n\x06\x08*\x86H\xce=\x04\x03\x020\x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0\x1e\x17\r190617124609Z\x17\r190915124609Z0\x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0Y0\x13\x06\x07*\x86H\xce=\x02\x01\x06\x08*\x86H\xce=\x03\x01\x07\x03B\0\x04RT\x16\x16 \xef_\xd8\xe7\xc3\xb7\x03\x1d\xd6:\x1fF\xe3\xf2b!\xa9/\x8b\xd4\x82\x8f\xd1\xff[\x9c\x97\xbc\xf27\xb8,L\x8a\x01\xb0r;;\xa9\x83\xdc\x86\x9f\x1d%y\xf4;I\xe4Y\xc80'$K[\xd6\xa3\x82\x0bw0\x82\x0bs0\x82\x0bo\x06\t`\x86H\x01\x86\xf8B\x01\r\x04\x82\x0b`{\"id\":\"117077750682263877593646412006783680848\",\"timestamp\":\"2019-06-17T12:46:04.002066\",\"version\":3,\"isvEnclaveQuoteStatus\":\"GROUP_OUT_OF_DATE\",\"platformInfoBlob\":\"1502006504000900000909020401800000000000000000000008000009000000020000000000000B401A355B313FC939B4F48A54349C914A32A3AE2C4871BFABF22E960C55635869FC66293A3D9B2D58ED96CA620B65D669A444C80291314EF691E896F664317CF80C\",\"isvEnclaveQuoteBody\":\"AgAAAEALAAAIAAcAAAAAAOE6wgoHKsZsnVWSrsWX9kky0kWt9K4xcan0fQ996Ct+CAj//wGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABwAAAAAAAAAHAAAAAAAAAFJJYIbPVot9NzRCjW2z9+k+9K8BsHQKzVMEHOR14hNbAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACD1xnnferKFHD2uvYqTXdDA8iZ22kCD5xw7h38CMfOngAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSVBYWIO9f2OfDtwMd1jofRuPyYiGpL4vUgo/R/1ucl7zyN7gsTIoBsHI7O6mD3IafHSV59DtJ5FnIMCckS1vW\"}|EbPFH/ThUaS/dMZoDKC5EgmdUXUORFtQzF49Umi1P55oeESreJaUvmA0sg/ATSTn5t2e+e6ZoBQIUbLHjcWLMLzK4pJJUeHhok7EfVgoQ378i+eGR9v7ICNDGX7a1rroOe0s1OKxwo/0hid2KWvtAUBvf1BDkqlHy025IOiXWhXFLkb/qQwUZDWzrV4dooMfX5hfqJPi1q9s18SsdLPmhrGBheh9keazeCR9hiLhRO9TbnVgR9zJk43SPXW+pHkbNigW+2STpVAi5ugWaSwBOdK11ZjaEU1paVIpxQnlW1D6dj1Zc3LibMH+ly9ZGrbYtuJks4eRnjPhroPXxlJWpQ==|MIIEoTCCAwmgAwIBAgIJANEHdl0yo7CWMA0GCSqGSIb3DQEBCwUAMH4xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwLU2FudGEgQ2xhcmExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQDDCdJbnRlbCBTR1ggQXR0ZXN0YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwHhcNMTYxMTIyMDkzNjU4WhcNMjYxMTIwMDkzNjU4WjB7MQswCQYDVQQGEwJVUzELMAkGA1UECAwCQ0ExFDASBgNVBAcMC1NhbnRhIENsYXJhMRowGAYDVQQKDBFJbnRlbCBDb3Jwb3JhdGlvbjEtMCsGA1UEAwwkSW50ZWwgU0dYIEF0dGVzdGF0aW9uIFJlcG9ydCBTaWduaW5nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAqXot4OZuphR8nudFrAFiaGxxkgma/Es/BA+tbeCTUR106AL1ENcWA4FX3K+E9BBL0/7X5rj5nIgX/R/1ubhkKWw9gfqPG3KeAtIdcv/uTO1yXv50vqaPvE1CRChvzdS/ZEBqQ5oVvLTPZ3VEicQjlytKgN9cLnxbwtuvLUK7eyRPfJW/ksddOzP8VBBniolYnRCD2jrMRZ8nBM2ZWYwnXnwYeOAHV+W9tOhAImwRwKF/95yAsVwd21ryHMJBcGH70qLagZ7Ttyt++qO/6+KAXJuKwZqjRlEtSEz8gZQeFfVYgcwSfo96oSMAzVr7V0L6HSDLRnpb6xxmbPdqNol4tQIDAQABo4GkMIGhMB8GA1UdIwQYMBaAFHhDe3amfrzQr35CN+s1fDuHAVE8MA4GA1UdDwEB/wQEAwIGwDAMBgNVHRMBAf8EAjAAMGAGA1UdHwRZMFcwVaBToFGGT2h0dHA6Ly90cnVzdGVkc2VydmljZXMuaW50ZWwuY29tL2NvbnRlbnQvQ1JML1NHWC9BdHRlc3RhdGlvblJlcG9ydFNpZ25pbmdDQS5jcmwwDQYJKoZIhvcNAQELBQADggGBAGcIthtcK9IVRz4rRq+ZKE+7k50/OxUsmW8aavOzKb0iCx07YQ9rzi5nU73tME2yGRLzhSViFs/LpFa9lpQL6JL1aQwmDR74TxYGBAIi5f4I5TJoCCEqRHz91kpG6Uvyn2tLmnIdJbPE4vYvWLrtXXfFBSSPD4Afn7+3/XUggAlc7oCTizOfbbtOFlYA4g5KcYgS1J2ZAeMQqbUdZseZCcaZZZn65tdqee8UXZlDvx0+NdO0LR+5pFy+juM0wWbu59MvzcmTXbjsi7HY6zd53Yq5K244fwFHRQ8eOB0IWB+4PfM7FeAApZvlfqlKOlLcZL2uyVmzRkyR5yW72uo9mehX44CiPJ2fse9Y6eQtcfEhMPkmHXI01sN+KwPbpA39+xOsStjhP9N1Y1a2tQAVo+yVgLgV2Hws73Fc0o3wC78qPEA+v2aRs/Be3ZFDgDyghc/1fgU+7C+P6kbqd4poyb6IW8KCJbxfMJvkordNOgOUUxndPHEi/tb/U7uLjLOgPA==0\n\x06\x08*\x86H\xce=\x04\x03\x02\x03H\00E\x02!\0\xae6\x06\t@Sy\x8f\x8ec\x9d\xdci^Ex*\x92}\xdcG\x15A\x97\xd7\xd7\xd1\xccx\xe0\x1e\x08\x02 \x15Q\xa0BT\xde'~\xec\xbd\x027\xd3\xd8\x83\xf7\xe6Z\xc5H\xb4D\xf7\xe2\r\xa7\xe4^f\x10\x85p";
const URL: &[u8] = &[
	119, 115, 58, 47, 47, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 57, 57, 57, 49,
];


fn get_signer1() -> (AccountId, [u32; 16]) {
	let signer_attn: [u32; 16] = Decode::decode(&mut TEST1_SIGNER_ATTN).unwrap();
	let mut pubkey = [0u8; 32];
	pubkey.copy_from_slice(&TEST1_SIGNER_PUB[..32]);
	let signer: AccountId =
		AccountPublic::from(sr25519::Public::decode(&mut &TEST1_SIGNER_PUB[..]).unwrap())
			.into_account();

	(signer, signer_attn)
}

fn get_signer2() -> (AccountId, [u32; 16]) {
	let signer_attn: [u32; 16] = Decode::decode(&mut TEST2_SIGNER_ATTN).unwrap();
	let mut pubkey = [0u8; 32];
	pubkey.copy_from_slice(&TEST2_SIGNER_PUB[..32]);
	let signer: AccountId =
		AccountPublic::from(sr25519::Public::decode(&mut &TEST2_SIGNER_PUB[..]).unwrap())
			.into_account();

	(signer, signer_attn)
}

fn get_signer3() -> (AccountId, [u32; 16]) {
	let signer_attn: [u32; 16] = Decode::decode(&mut TEST3_SIGNER_ATTN).unwrap();
	let mut pubkey = [0u8; 32];
	pubkey.copy_from_slice(&TEST3_SIGNER_PUB[..32]);
	let signer: AccountId =
		AccountPublic::from(sr25519::Public::decode(&mut &TEST3_SIGNER_PUB[..]).unwrap())
			.into_account();

	(signer, signer_attn)
}

fn list_enclaves() -> Vec<(u64, Enclave<AccountId, Vec<u8>>)> {
	<EnclaveRegistry<TestRuntime>>::iter()
		.collect::<Vec<(u64, Enclave<AccountId, Vec<u8>>)>>()
}

#[test]
fn add_enclave_works() {
	new_test_ext().execute_with(|| {
		let (signer, signer_attn) = get_signer1();
		assert_ok!(Registry::register_enclave(
			Origin::signed(signer),
			TEST1_CERT.to_vec(),
			signer_attn,
			URL.to_vec()
		));
		assert_eq!(Registry::enclave_count(), 1);
	})
}

#[test]
fn add_and_remove_enclave_works() {
	new_test_ext().execute_with(|| {
		let (signer, signer_attn) = get_signer1();
		assert_ok!(Registry::register_enclave(
			Origin::signed(signer.clone()),
			TEST1_CERT.to_vec(),
			signer_attn,
			URL.to_vec()
		));
		assert_eq!(Registry::enclave_count(), 1);
		assert_ok!(Registry::unregister_enclave(Origin::signed(signer)));
		assert_eq!(Registry::enclave_count(), 0);
		assert_eq!(list_enclaves(), vec![])
	})
}

#[test]
fn list_enclaves_works() {
	new_test_ext().execute_with(|| {
		let (signer, signer_attn) = get_signer1();
		let e_1: Enclave<AccountId, Vec<u8>> = Enclave {
			pubkey: signer.clone(),
			mr_enclave: TEST1_MRENCLAVE,
			timestamp: TEST1_TIMESTAMP,
			url: URL.to_vec(),
		};
		assert_ok!(Registry::register_enclave(
			Origin::signed(signer.clone()),
			TEST1_CERT.to_vec(),
			signer_attn,
			URL.to_vec()
		));
		assert_eq!(Registry::enclave_count(), 1);
		let enclaves = list_enclaves();
		assert_eq!(enclaves[0].1.pubkey, signer)
	})
}

#[test]
fn remove_middle_enclave_works() {
	new_test_ext().execute_with(|| {
		let (signer1, signer_attn1) = get_signer1();
		let (signer2, signer_attn2) = get_signer2();
		let (signer3, signer_attn3) = get_signer3();

		// add enclave 1
		let e_1: Enclave<AccountId, Vec<u8>> = Enclave {
			pubkey: signer1.clone(),
			mr_enclave: TEST1_MRENCLAVE,
			timestamp: TEST1_TIMESTAMP,
			url: URL.to_vec(),
		};

		let e_2: Enclave<AccountId, Vec<u8>> = Enclave {
			pubkey: signer2.clone(),
			mr_enclave: TEST2_MRENCLAVE,
			timestamp: TEST2_TIMESTAMP,
			url: URL.to_vec(),
		};

		let e_3: Enclave<AccountId, Vec<u8>> = Enclave {
			pubkey: signer3.clone(),
			mr_enclave: TEST3_MRENCLAVE,
			timestamp: TEST3_TIMESTAMP,
			url: URL.to_vec(),
		};

		assert_ok!(Registry::register_enclave(
			Origin::signed(signer1.clone()),
			TEST1_CERT.to_vec(),
			signer_attn1,
			URL.to_vec()
		));
		assert_eq!(Registry::enclave_count(), 1);
		assert_eq!(list_enclaves(), vec![(1, e_1.clone())]);

		// add enclave 2
		assert_ok!(Registry::register_enclave(
			Origin::signed(signer2.clone()),
			TEST2_CERT.to_vec(),
			signer_attn2,
			URL.to_vec()
		));
		assert_eq!(Registry::enclave_count(), 2);
		let enclaves = list_enclaves();
		assert!(enclaves.contains(&(1, e_1.clone())));
		assert!(enclaves.contains(&(2, e_2.clone())));
		
		// add enclave 3
		assert_ok!(Registry::register_enclave(
			Origin::signed(signer3.clone()),
			TEST3_CERT.to_vec(),
			signer_attn3,
			URL.to_vec()
		));
		assert_eq!(Registry::enclave_count(), 3);
		let enclaves = list_enclaves();
		assert!(enclaves.contains(&(1, e_1.clone())));
		assert!(enclaves.contains(&(2, e_2.clone())));
		assert!(enclaves.contains(&(3, e_3.clone())));

		// remove enclave 2
		assert_ok!(Registry::unregister_enclave(Origin::signed(signer2)));
		assert_eq!(Registry::enclave_count(), 2);
		let enclaves = list_enclaves();
		assert!(enclaves.contains(&(1, e_1.clone())));
		assert!(enclaves.contains(&(2, e_3.clone())));
	})
}

#[test]
fn register_invalid_enclave_fails() {
	new_test_ext().execute_with(|| {
		let (signer, signer_attn) = get_signer1();
		assert!(
			Registry::register_enclave(
				Origin::signed(signer),
				Vec::new(),
				[0u32; 16],
				URL.to_vec()
			)
			.is_err(),
			URL.to_vec()
		);
	})
}

#[test]
fn update_enclave_url_works() {
	new_test_ext().execute_with(|| {
		let (signer, signer_attn) = get_signer1();
		let url2 = "my fancy url".as_bytes();
		let e_1: Enclave<AccountId, Vec<u8>> = Enclave {
			pubkey: signer.clone(),
			mr_enclave: TEST1_MRENCLAVE,
			timestamp: TEST1_TIMESTAMP,
			url: url2.to_vec(),
		};

		assert_ok!(Registry::register_enclave(
			Origin::signed(signer.clone()),
			TEST1_CERT.to_vec(),
			signer_attn,
			URL.to_vec()
		));
		assert_eq!(Registry::enclave(1).url, URL.to_vec());

		assert_ok!(Registry::register_enclave(
			Origin::signed(signer.clone()),
			TEST1_CERT.to_vec(),
			signer_attn,
			url2.to_vec()
		));
		assert_eq!(Registry::enclave(1).url, url2.to_vec());
		let enclaves = list_enclaves();
		assert_eq!(enclaves[0].1.pubkey, signer)
	})
}

#[test]
fn update_ipfs_hash_works() {
	new_test_ext().execute_with(|| {
		let ipfs_hash = "QmYY9U7sQzBYe79tVfiMyJ4prEJoJRWCD8t85j9qjssS9y";
		let shard = H256::default();
		let request_hash = vec![];
		let (signer, signer_attn) = get_signer1();

		assert_ok!(Registry::register_enclave(
			Origin::signed(signer.clone()),
			TEST1_CERT.to_vec(),
			signer_attn,
			URL.to_vec()
		));
		assert_eq!(Registry::enclave_count(), 1);
		assert_ok!(Registry::confirm_call(
			Origin::signed(signer.clone()),
			shard.clone(),
			request_hash.clone(),
			ipfs_hash.as_bytes().to_vec()
		));
		assert_eq!(
			str::from_utf8(&Registry::latest_ipfs_hash(shard.clone())).unwrap(),
			ipfs_hash
		);
		assert_eq!(Registry::worker_for_shard(shard.clone()), 1u64);

		let expected_event = TestEvent::registry(RawEvent::UpdatedIpfsHash(
			shard.clone(),
			1,
			ipfs_hash.as_bytes().to_vec(),
		));
		assert!(System::events().iter().any(|a| a.event == expected_event));

		let expected_event =
			TestEvent::registry(RawEvent::CallConfirmed(signer.clone(), request_hash));
		assert!(System::events().iter().any(|a| a.event == expected_event));
	})
}

#[test]
fn ipfs_update_from_unregistered_enclave_fails() {
	new_test_ext().execute_with(|| {
		let ipfs_hash = "QmYY9U7sQzBYe79tVfiMyJ4prEJoJRWCD8t85j9qjssS9y";
		let (signer, signer_attn) = get_signer1();
		assert!(Registry::confirm_call(
			Origin::signed(signer),
			H256::default(),
			vec![],
			ipfs_hash.as_bytes().to_vec()
		)
		.is_err());
	})
}

#[test]
fn call_worker_works() {
	new_test_ext().execute_with(|| {
		let req = Request {
			shard: ShardIdentifier::default(),
			cyphertext: vec![0u8, 1, 2, 3, 4],
		};
		let (signer, signer_attn) = get_signer1();
		assert!(Registry::call_worker(Origin::signed(signer), req.clone()).is_ok());
		let expected_event = TestEvent::registry(RawEvent::Forwarded(req));
		assert!(System::events().iter().any(|a| a.event == expected_event));
	})
}


