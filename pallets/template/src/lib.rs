#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub use pallet::*;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
 use super::*;
 use frame_support::pallet_prelude::*;
 use frame_system::pallet_prelude::*;

 #[pallet::pallet]
 pub struct Pallet<T>(_);

 /// Configure the pallet by specifying the parameters and types on which it depends.
#[pallet::config]
pub trait Config: frame_system::Config {
 /// Because this pallet emits events, it depends on the runtime's definition of an event.
 type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
 type WeightInfo: WeightInfo;
}  

 // Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
 /// Event emitted when a claim has been created.
 ClaimCreated { who: T::AccountId, claim: T::Hash },
 /// Event emitted when a claim is revoked by the owner.
 ClaimRevoked { who: T::AccountId, claim: T::Hash },
}

#[pallet::error]
pub enum Error<T> {
 /// The claim already exists.
 AlreadyClaimed,
 /// The claim does not exist, so it cannot be revoked.
 NoSuchClaim,
 /// The claim is owned by another account, so caller can't revoke it.
 NotClaimOwner,
}

#[pallet::storage]
pub(super) type Claims<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, BlockNumberFor<T>)>;
 
// Dispatchable functions allow users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
#[pallet::call]
impl<T: Config> Pallet<T> {
 #[pallet::weight({0})]
 #[pallet::call_index(1)]
 pub fn create_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
   // Check that the extrinsic was signed and get the signer.
   // This function will return an error if the extrinsic is not signed.
   let sender = ensure_signed(origin)?;

   // Verify that the specified claim has not already been stored.
   ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);

   // Get the block number from the FRAME System pallet.
   let current_block = <frame_system::Pallet<T>>::block_number();

   // Store the claim with the sender and block number.
   Claims::<T>::insert(&claim, (&sender, current_block));

   // Emit an event that the claim was created.
   Self::deposit_event(Event::ClaimCreated { who: sender, claim });

   Ok(())
 }

 #[pallet::weight({0})]
 #[pallet::call_index(2)]
 pub fn revoke_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
   // Check that the extrinsic was signed and get the signer.
   // This function will return an error if the extrinsic is not signed.
   let sender = ensure_signed(origin)?;

   // Get owner of the claim, if none return an error.
   let (owner, _) = Claims::<T>::get(&claim).ok_or(Error::<T>::NoSuchClaim)?;

   // Verify that sender of the current call is the claim owner.
   ensure!(sender == owner, Error::<T>::NotClaimOwner);

   // Remove claim from storage.
   Claims::<T>::remove(&claim);

   // Emit an event that the claim was erased.
   Self::deposit_event(Event::ClaimRevoked { who: sender, claim });
   Ok(())
 }
}
}