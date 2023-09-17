#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::{pallet_prelude::*, sp_runtime::traits::BlockNumberProvider};
    use frame_system::pallet_prelude::*;
    use frame_support::traits::{Currency, Randomness};

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: Currency<Self::AccountId>;
        type CollectionRandomness: Randomness<Self::Hash, Self::BlockNumber>;
        type BlockNumber: BlockNumberProvider;
    
        #[pallet::constant]
        type MaximumOwned: Get<u32>;
    }
}