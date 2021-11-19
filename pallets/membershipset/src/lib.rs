#![cfg_attr(not(feature = "std"), no_std)]

//! WE do not use the set for anything in this pallet; we simply maintain its membership
//! We will privide dispatchable dalls to add and remove members, ensuring that the number of members nver exceeds a hard-coded maximum 

use account_set::AccountSet;
use frame_support::storage::IterableStorageMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_runtime::ArithmeticError;
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
	use frame_system::{pallet_prelude::*, ensure_signed};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

	}
	//	The maximum number of members f
	pub const MAX_MEMBERS: u32 = 16; 

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	#[pallet::getter(fn member_count)]
	pub type MemberCount<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn member)]
	pub type Member<T> = StorageMap<_, Blake2_128Concat, T::AccountId, (), ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MemberAdded(T::AccountId),
		MemberRemoved(T::AccountId),
	}

	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyMember,
		NotMember,
		MembershipLimitReached,
		Underflow,

	}
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn add_member(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			//	Load the values inside the Storage Value 
			let member_counts = MemberCount::<T>::get();
			//	Check if there is enough space 
			ensure!(member_counts <= MAX_MEMBERS, Error::<T>::MembershipLimitReached)?;
			//	Prevent member duplication
			ensure!(!Member::<T>::contains_key(&sender), Error::<T>::NotMember)?;
			//	Insert Member Account ID into Storage
			Member::<T>::insert(sender);
			//	Increase Member count 
			if Self::increase_member().ok() { 
				Self::deposit_event(Event::<T>::MemberAdded(sender.clone()))
			}
			
			Ok(().into())
		}
		#[pallet::weight(10_000)]
		pub fn remove_member(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			let curr_member = ensure_signed(origin)?;
			let member_counts = MemberCount::<T>::get();
			//	Ensure Member has a key inside the storage map
			ensure!(!Member::<T>::contains_key(&curr_memner), Error::<T>::NotMember)?;  
			//	Check if there is enough Members to deduct 
			ensure!(member_counts < MAX_MEMBERS, Error::<T>::MembershipLimitReached);
			//	Remove key value from storage: Member
			Member::<T>::remove(&sender);
			//	Decrease member from MemberCount<T>
			if Self::decrease_member().is_ok() { 
				Self::deposit_event(Event::MemberRemoved(sender.clone()))
			}
			Ok(().into())

		}
	}
}
impl<T: Config> Pallet<T> { 
	//	Increase member count while checkign for Arithmetic Error 
	fn increase_member() -> Result<u32, DispatchError> { 
		MemberCount::<T>::try_mutate(|count| -> Result<u32, DispatchError> { 
			let curr_count = *count;
			*count = count.checked_add(1).ok_or(ArithmeticError::Overflow);
			Ok(curr_count)
		})
	}
	fn decrease_member() -> Result<u32, DispatchError> { 
		MemberCount::<T>::try_mutate(|count| -> Result<u32, DisptachError> { 
			let curr_count = *count;
			*count = count.checked_sub(1).ok_or(Error::<T>::Underflow);
			Ok(curr_count)
		})
	}
}

//	Structure Storage into Sorted Sets 
impl<T: Config> AccountSet for Pallet<T> { 
	type AccountId = T::AccountId;
	/// IterableStorageMap: a strongly typed map in storage whose keys and values can be iterated over 
	fn accounts() -> BTreeSet<T::AccountId> { 
		<Member<T> as IterableStorageMap<T::AccountId, ()>>::iter()
		//	DO something (key, value)
		.map(|(acc, _)| acc)
		//	Collect elements into BTreeSet
		.collect::<BTreeSet<_>>()
	}
}

























