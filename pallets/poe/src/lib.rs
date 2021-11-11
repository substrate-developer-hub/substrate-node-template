#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	
	#[pallet::storage]
	pub type Proofs<T> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u32>,
		(T::AccountId, T::BlockNumber),
		ValueQuery
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// parameters. [something, who]
		ClaimedCreated(T::AccountId, Vec<u32>),
		ClaimedRevoked(T::AccountId, Vec<u32>)
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyClaimed,
		NoSuchProof,
		NotProofOwner,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_claim(
			origin: OriginFor<T>,
			proofs: Vec<u32>
		) -> DispatchResultPostInfo {

			let who = ensure_signed(origin)?;

			// proofs 
			//	Get blocknumber 
			ensure!(Proof::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);
			let current_block = <frame_system::Pallet<T>>::block_number();

			// Update storage.
			Proofs::<T>::insert(&proof, (&sender, current_block));

			// Emit an event.
			Self::deposit_event(Event::ClaimedCreated(who, proof));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn revoke_claim(
			origin: OriginFor<T>, 
			proofs: Vec<u32>
		) -> DispatchResultPostInfo {
			let sender = ensure_signed(origin)?;
			
			ensure!(Proof::<T>::contains_key(&proofs), Error::<T>::NoSuchProof);
			let (owner, _) = Proofs::<T>::get(&proof); // load the value associated with the account id 
			Proof::<T>::remove(&proofs);

			Self::deposit_event(Event::ClaimedRevoked(sender, proofs ))
			Ok(().into())
		}
	}
}
