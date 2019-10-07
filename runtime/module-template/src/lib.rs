//! A runtime module template with necessary imports

#![cfg_attr(not(feature = "std"), no_std)]

use support::{decl_module, decl_storage, decl_event, ensure, StorageMap};
use rstd::vec::Vec;
use system::ensure_signed;

pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event! {
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        /// Event emitted when a proof has been claimed.
        ClaimCreated(AccountId, Vec<u8>),
        /// Event emitted when a claim is revoked by the owner.
        ClaimRevoked(AccountId, Vec<u8>),
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as PoeStorage {
        /// The storage item for our proofs.
        /// It maps a proof to the user who made the claim and when they made it.
        Proofs: map Vec<u8> => (T::AccountId, T::BlockNumber);
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // A default function for depositing events
        fn deposit_event() = default;
        /// Allow a user to claim ownership of an unclaimed proof
        fn create_claim(origin, proof: Vec<u8>) {
            // Verify that the incoming transaction is signed and store who the
            // caller of this function is.
            let sender = ensure_signed(origin)?;
            // Verify that the specified proof has not been claimed yet or error with the message
            ensure!(!Proofs::<T>::exists(&proof), "This proof has already been claimed.");
            // Call the `system` runtime module to get the current block number
            let current_block = <system::Module<T>>::block_number();
            // Store the proof with the sender and the current block number
            Proofs::<T>::insert(&proof, (sender.clone(), current_block));
            // Emit an event that the claim was created
            Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
        }
        /// Allow the owner to revoke their claim
        fn revoke_claim(origin, proof: Vec<u8>) {
            // Determine who is calling the function
            let sender = ensure_signed(origin)?;
            // Verify that the specified proof has been claimed
            ensure!(Proofs::<T>::exists(&proof), "This proof has not been stored yet.");
            // Get owner of the claim
            let (owner, _) = Proofs::<T>::get(&proof);
            // Verify that sender of the current call is the claim owner
            ensure!(sender == owner, "You must own this claim to revoke it.");
            // Remove claim from storage
            Proofs::<T>::remove(&proof);
            // Emit an event that the claim was erased
            Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
        }
    }
}
