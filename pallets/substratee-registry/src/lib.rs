/*
    Copyright 2019 Supercomputing Systems AG

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

*/
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use host_calls::runtime_interfaces::verify_ra_report;
use host_calls::SgxReport;
use sp_core::H256;
use sp_std::prelude::*;
use sp_std::str;
use sp_io::misc::print_utf8;
use frame_support::{decl_event, decl_module, decl_storage, decl_error, 
    dispatch::DispatchResult, ensure};
use frame_system::{self as system, ensure_signed};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

const MAX_RA_REPORT_LEN: usize = 4096;
const MAX_URL_LEN: usize = 256;

#[derive(Encode, Decode, Default, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Enclave<PubKey, Url> {
    pub pubkey: PubKey, // FIXME: this is redundant information
    pub mr_enclave: [u8; 32],
    pub timestamp: i64, // unix epoch
    pub url: Url,       // utf8 encoded url
}

pub type ShardIdentifier = H256;

#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct Request {
    pub shard: ShardIdentifier,
    pub cyphertext: Vec<u8>,
}

decl_event!(
	pub enum Event<T>
	where
		<T as system::Trait>::AccountId,
	{
		AddedEnclave(AccountId, Vec<u8>),
		RemovedEnclave(AccountId),
		UpdatedIpfsHash(ShardIdentifier, u64, Vec<u8>),
		Forwarded(Request),
		CallConfirmed(AccountId, Vec<u8>),
	}
);

decl_storage! {
    trait Store for Module<T: Trait> as substraTEERegistry {
        // Simple lists are not supported in runtime modules as theoretically O(n)
        // operations can be executed while only being charged O(1), see substrate
        // Kitties tutorial Chapter 2, Tracking all Kitties.

        // watch out: we start indexing with 1 instead of zero in order to
        // avoid ambiguity between Null and 0
        pub EnclaveRegistry get(enclave): map hasher(blake2_128_concat) u64 => Enclave<T::AccountId, Vec<u8>>;
        pub EnclaveCount get(enclave_count): u64;
        pub EnclaveIndex get(enclave_index): map hasher(blake2_128_concat) T::AccountId => u64;
        pub LatestIpfsHash get(latest_ipfs_hash) : map hasher(blake2_128_concat) ShardIdentifier => Vec<u8>;
        // enclave index of the worker that recently committed an update
        pub WorkerForShard get(worker_for_shard) : map hasher(blake2_128_concat) ShardIdentifier => u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        fn deposit_event() = default;
        
        // the substraTEE-worker wants to register his enclave
        #[weight = frame_support::weights::SimpleDispatchInfo::default()]
        pub fn register_enclave(origin, ra_report: Vec<u8>, ra_signer_attn: [u32; 16], worker_url: Vec<u8>) -> DispatchResult {
            print_utf8(b"substraTEE_registry: called into runtime call register_enclave()");
            let sender = ensure_signed(origin)?;
            ensure!(ra_report.len() <= MAX_RA_REPORT_LEN, "RA report too long");
            ensure!(worker_url.len() <= MAX_URL_LEN, "URL too long");
            print_utf8(b"substraTEE_registry: parameter lenght ok");
            match verify_ra_report(&ra_report, &ra_signer_attn.to_vec(), &sender.encode()) {
                Some(rep) => {
                    print_utf8(b"substraTEE_registry: host_call successful");
                    let report = SgxReport::decode(&mut &rep[..]).unwrap();
                    let enclave_signer = match T::AccountId::decode(&mut &report.pubkey[..]) {
                        Ok(signer) => signer,
                        Err(_) => return Err(<Error<T>>::EnclaveSignerDecodeError.into())
                    };
                    print_utf8(b"substraTEE_registry: decoded signer");
                    // this is actually already implicitly tested by verify_ra_report
                    ensure!(sender == enclave_signer,
                        "extrinsic must be signed by attested enclave key");
                    print_utf8(b"substraTEE_registry: signer is a match");
                    // TODO: activate state checks as soon as we've fixed our setup
//                    ensure!((report.status == SgxStatus::Ok) | (report.status == SgxStatus::ConfigurationNeeded),
//                        "RA status is insufficient");
//                    print_utf8(b"substraTEE_registry: status is acceptable");
                    Self::register_verified_enclave(&sender, &report, worker_url.clone())?;
                    Self::deposit_event(RawEvent::AddedEnclave(sender, worker_url));
                    print_utf8(b"substraTEE_registry: enclave registered");
                    Ok(())

                }
                None => Err(<Error<T>>::RemoteAttestationVerificationFailed.into())
            }
        }
        // TODO: we can't expect a dead enclave to unregister itself
        // alternative: allow anyone to unregister an enclave that hasn't recently supplied a RA
        // such a call should be feeless if successful
        #[weight = frame_support::weights::SimpleDispatchInfo::default()]
        pub fn unregister_enclave(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::remove_enclave(&sender)?;
            Self::deposit_event(RawEvent::RemovedEnclave(sender));
            Ok(())
        }

        #[weight = frame_support::weights::SimpleDispatchInfo::default()]
        pub fn call_worker(origin, request: Request) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            Self::deposit_event(RawEvent::Forwarded(request));
            Ok(())
        }

        // the substraTEE-worker calls this function for every processed call to confirm a state update
        #[weight = frame_support::weights::SimpleDispatchInfo::default()]
        pub fn confirm_call(origin, shard: ShardIdentifier, call_hash: Vec<u8>, ipfs_hash: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<EnclaveIndex<T>>::contains_key(&sender),
            "[SubstraTEERegistry]: IPFS state update requested by enclave that is not registered");
            let sender_index = Self::enclave_index(&sender);
            <LatestIpfsHash>::insert(shard, ipfs_hash.clone());
            <WorkerForShard>::insert(shard, sender_index);

            Self::deposit_event(RawEvent::CallConfirmed(sender, call_hash));
            Self::deposit_event(RawEvent::UpdatedIpfsHash(shard, sender_index, ipfs_hash));
            Ok(())
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        // failed to decode enclave signer
        EnclaveSignerDecodeError,
        // Verifying RA report failed
        RemoteAttestationVerificationFailed
    }
}

impl<T: Trait> Module<T> {
    fn register_verified_enclave(
        sender: &T::AccountId,
        report: &SgxReport,
        url: Vec<u8>,
    ) -> DispatchResult {
        let enclave = Enclave {
            pubkey: sender.clone(),
            mr_enclave: report.mr_enclave,
            timestamp: report.timestamp,
            url,
        };
        let enclave_idx = if <EnclaveIndex<T>>::contains_key(sender) {
            print_utf8(b"Updating already registered enclave");
            <EnclaveIndex<T>>::get(sender)
        } else {
            let enclaves_count = Self::enclave_count()
                .checked_add(1)
                .ok_or("[SubstraTEERegistry]: Overflow adding new enclave to registry")?;
            <EnclaveIndex<T>>::insert(sender, enclaves_count);
            <EnclaveCount>::put(enclaves_count);
            enclaves_count
        };

        <EnclaveRegistry<T>>::insert(enclave_idx, &enclave);
        Ok(())
    }

    fn remove_enclave(sender: &T::AccountId) -> DispatchResult {
        ensure!(
            <EnclaveIndex<T>>::contains_key(sender),
            "[SubstraTEERegistry]: Trying to remove an enclave that doesn't exist."
        );
        let index_to_remove = <EnclaveIndex<T>>::take(sender);

        let enclaves_count = Self::enclave_count();
        let new_enclaves_count = enclaves_count
            .checked_sub(1)
            .ok_or("[SubstraTEERegistry]: Underflow removing an enclave from the registry")?;

        Self::swap_and_pop(index_to_remove, new_enclaves_count + 1)?;
        <EnclaveCount>::put(new_enclaves_count);

        Ok(())
    }

    /// Our list implementation would introduce holes in out list if if we try to remove elements from the middle.
    /// As the order of the enclave entries is not important, we use the swap an pop method to remove elements from
    /// the registry.
    fn swap_and_pop(index_to_remove: u64, new_enclaves_count: u64) -> DispatchResult {
        if index_to_remove != new_enclaves_count {
            let last_enclave = <EnclaveRegistry<T>>::get(&new_enclaves_count);
            <EnclaveRegistry<T>>::insert(index_to_remove, &last_enclave);
            <EnclaveIndex<T>>::insert(last_enclave.pubkey, index_to_remove);
        }

        <EnclaveRegistry<T>>::remove(new_enclaves_count);

        Ok(())
    }
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

