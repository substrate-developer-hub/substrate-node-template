#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	#[cfg(test)]
	use std::{println as info, println as warn};

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	#[cfg(not(test))]
	use log::{info, warn};

	#[cfg(feature = "std")]
	#[pallet::config]
	pub trait Config: frame_system::Config {
		// /// Because this pallet emits events, it depends on the runtime's definition of an event.
		// type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn stored_value)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type StoredValue<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn init_storage)]
	pub type InitVal<T> = StorageValue<_, u32>;

	// Our pallet's genesis configuration
	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub init_val: u32,
	}

	// // Required to implement default for GenesisConfig
	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> GenesisConfig {
			GenesisConfig { init_val: 0 }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			// When building a kitty from genesis config, we require the DNA and Gender to be
			// supplied
			// for (account, dna, gender) in &self.kitties {
			// 	assert!(Pallet::<T>::mint(account, *dna, *gender).is_ok());
			// }
			// assert!(Pallet::<T>::<InitVal<T>>::put(self.initVal))
			assert!(Pallet::<T>::set_init_val(self.init_val).is_ok());
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn output_something(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			info!("called by {:?}", who);
			Ok(())
		}

		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[pallet::weight(10_000)]
		pub fn simple(origin: OriginFor<T>, val: u32) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			<StoredValue<T>>::put(val);
			Ok(())
		}
	}

	// non callable function
	impl<T: Config> Pallet<T> {
		pub fn set_init_val(val: u32) -> Result<bool, DispatchError> {
			<InitVal<T>>::put(val);
			Ok(true)
		}
	}
}
