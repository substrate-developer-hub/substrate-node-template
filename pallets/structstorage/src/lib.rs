#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>


pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct InnerThing<Hash, Balance> { 
	pub number: u32,
	pub hash: Hash,
	pub balance: Balance,
}
#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct SuperThing<Hash, Balance> { 
	pub super_thing: u32,
	pub inner_thing: InnerThing<Hash, Balance>,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo,
		 pallet_prelude::*, codec::{Encode, Decode}
		};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}
	pub type InnerThingOf<T> = 
		InnerThing<<T as frame_system::Config>::Hash, <T as pallet_balances::Config>::Balance>;
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn innerthing)]
	pub type InnerThingByNumbers<T> = StorageMap<
		_,
		Blake2_128Concat,  
		u32, 
		InnerThingOf<T>, // -> Access to InnerThing Struct 
		ValueQuery,
	>;
	#[pallet::storage]
	#[pallet::getter(fn superthing)]
	pub type SuperThingByNumbers<T> = Storage<
		_, 
		Blake2_128Concat,
		u32, 
		SuperThing<T::Hash, T::Balance>,
		ValueQuery
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewInnerThing(u32, T::Hash, T::Balance),
		NewSuperThingByExistingInner(u32, u32, T::Hash, T::Balance),
		NewSuperThingByNewInner(u32, u32, T::Hash, T::Balance),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn insert_inner_thing(
			origin: OriginFor<T>,
			inner_number: u32,
			hash: T::Hash,
			balance: T::Balance, 
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?:

			//	Create metadata 
			let metadata = InnerThing { 
				inner_thing,
				hash,
				balance,
			};
			//	Insert into on-chain storage 
			InnerThingByNumbers::<T>::insert(inner_thing, metadata);
			Self::deposit_event(Event::NewInnerThing(inner_thing, hash, balance));
			Ok(().into())
		}
		//	Insert superthing with existing inner inside the storage map 
		#[pallet::weight(10_000)]
		pub fn insert_super_thing_with_existing_inner(
			origin: OriginFor<T>,
			super_number: u32,
			inner_number: u32,

		) -> DispatchResultWithPostInfo { 
			let sender = ensure_signed(origin)?;
			//	Get InnerThing from onchain storage
			let inner_thing = InnerThingByNumbers::<T>::get(inner_thing);
			
			let super_thing = SuperThing { 
				super_number, 
				inner_thing.clone()
			}
			// 	Insert InnerThing to SuperThing 
			SuperThingByNumbers::<T>::insert(super_number, super_thing);
			// Insert into StorageMap	
			Self::deposit_event(Event::NewSuperThingByExistingInner(
				super_inner, 
				inner_thing.number,
				inner_thing.hash,
				inner_thing.balance,
			));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn insert_super_thing_with_new_inner(
			origin: OriginFor<T>,
			super_number: u32,
			inner_number: u32,
			hash: T::Hash,
			balance: T::Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			//	create a new inner
			let inner_thing = InnerThing { 
				inner_number, 
				hash, 
				balance,
			};
			InnerThingByNumbers::<T>::insert(inner_number, inner_thing); 
			Self::deposit_event(Event::NewInnerThing(inner_number, hash, number));
			let super_thing = SuperThing { 
				super_number,
				inner_thing.clone()
			};
			SuperThingByNumbers::<T>::insert(super_number, super_thing);
			Self::deposit_event(Event::NewSuperThingByNewInner(
				super_number, 
				inner_number, 
				hash, 
				balance
			));
			Ok(().into())
		}
	}
}