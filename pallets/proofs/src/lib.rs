#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Get,
    };
    use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

	#[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        #[pallet::constant]
        type MaxClaimLength: Get<u32>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
		// OptionQuery,
	>;


    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, BoundedVec<u8, T::MaxClaimLength>),
        ClaimRevoked(T::AccountId, BoundedVec<u8, T::MaxClaimLength>),
        ClaimTransferred(T::AccountId, T::AccountId, BoundedVec<u8, T::MaxClaimLength>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofAlreadyExist,
        NoSuchProof,
        NotProofOwner,
        ProofTooLong,
    }

	// const MAX_CLAIM_LENGTH: usize = 1024; 

	// #[derive(Clone, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug )]
	// pub struct ProofData(pub Vec<u8>);

	// pub type ProofData = BoundedVec<u8, T::MaxClaimLength>; // 使用适当的长度作为泛型参数

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_claim(
            origin: OriginFor<T>,
            proof: BoundedVec<u8, T::MaxClaimLength>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                !Proofs::<T>::contains_key(&proof),
                Error::<T>::ProofAlreadyExist
            );
            ensure!(
                proof.len() <= T::MaxClaimLength::get() as usize,
                Error::<T>::ProofTooLong
            );

            let current_block = <frame_system::Pallet<T>>::block_number();
            Proofs::<T>::insert(&proof, (sender.clone(), current_block));

            Self::deposit_event(Event::ClaimCreated(sender, proof));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn revoke_claim(
            origin: OriginFor<T>,
            proof: BoundedVec<u8, T::MaxClaimLength>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            let (owner, _) = if let Some(proof_data) = Proofs::<T>::get(&proof) {
                proof_data
            } else {
                Err(Error::<T>::NoSuchProof)?
            };
            ensure!(sender == owner, Error::<T>::NotProofOwner);

            Proofs::<T>::remove(&proof);

            Self::deposit_event(Event::ClaimRevoked(sender, proof));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer_claim(
            origin: OriginFor<T>,
            proof: BoundedVec<u8, T::MaxClaimLength>,
            recipient: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            let (owner, block_number) = Proofs::<T>::get(&proof)
                .ok_or(Error::<T>::NoSuchProof)?;

            ensure!(sender == owner, Error::<T>::NotProofOwner);

            Proofs::<T>::insert(&proof, (recipient.clone(), block_number));

            Self::deposit_event(Event::ClaimTransferred(sender, recipient, proof));

            Ok(().into())
        }
    }
}
