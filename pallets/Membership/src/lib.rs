#![cfg_attr(not(feature = "std"), no_std)]

//!	Objective:
//! Demonstrate how to implement a storage set on top of a vector, and explores
//! the performance of the implemenetation 

use account_set::AccountSet;
use sp_std::collections::btree_set::BtreeSet;
pub use pallet::*;
use sp_std::{Vec, default};

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
	use sp_runtime::print;
	use sp_std::prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}
	//	Maximum number of members
	//	When membership reaches this number, no new members may join in 
	pub const MAX_MEMBERS: usize = 16;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	#[pallet::getter(fn member)]
	pub type Member<T> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MemberAdded(T::AccountId),
		MemberRemoved(T::AccountId),
		MemberSwapped(T::AccountId)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AlreadyMember,
		NotMember,
		MembershipLimited
	}
	
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn add_member(
			origin: OriginFor<T>
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			//	Load the values in the provided storage 
			let mut members = Member::<T>::get();
			//	Check if there is enough member space 
			ensure!(members.len() < MAX_MEMBERS, Error::<T>::MembershipLimited)?;
			
			//	**Note** There is not insert() method found in StorageValue
			//	Check if the sender is already a member, if not send an error 
			// match members.binary_search(&sender) { 
		    //	Binary search will return the index of this value 
			// 	Err(index) => { 
			// 		members.insert(index, sender.clone());
			// 		Member::<T>::put(new_member);
			// 		Self::deposit_event(Event::MemberAdded(new_member));
			// 	},
			// Binary Search will return the index of where we found the value
			// 	Ok(_) => {
			// 		Err(Error::<T>::AlreadyMember.into())
			// 	} 
			// }
			//	Binary tree will return the index of the existing member 

			let member_location = members
				.binary_search(&sender)
				//	New member means the sender is not in our Database-> therefore return Error 
				//	Discard any ok result
				.err()
				//	If err, convert result into Option
				//	If ok, means the member is already inside the DB
				.ok_or(Error::<T>::AlreadyMember.into());
				//	If Some Value is found, return an error 
			
			members.insert(member_location, sender.clone());
			Member::<T>::put(member);
			//	Store the value under this account 
			Self::deposit_event(Event::MemberAdded(sender));


			Ok(().into())
		}
		pub fn remove_member(
			origin: OriginFor<T>
		) -> DispatchResultWithPostInfo {
			let already_member = ensure_signed(origin)?;

			//	Load the values inside of this storage 
			let mut members = Member::<T>::get();
			
			//	If err, the member is not found inside the database
			// if let Err(index) = members.binary_search(&already_member) { 
			// 	Err(Error::<T>::NotMember);
			// } else {
			// 	//	member ==> index inside the database  
			// 	members.remove(index);
			// 	//	Store the value under this account as the key for this vlaue 
			// 	Member::<T>::put(member);
			// 	Self::deposit_event(Event::MemberRemoved(already_member));
			// }

			let member_location = members.binary_search(&already_member)
				.ok()
				//	Discard any Err
				//	Return Option (Result)
				.ok_or(Error::<T>::NotMember);
				//	If Some return ok 
				// If  None return Err
			members.remove(member_location);
			//	Remove the value from this index
			Self::deposit_event(Event::<T>::MemberRemoved);
			Ok(().into())
		}
		pub fn swap_member(
			origin: OriginFor<T>,
			new_member: T::AccountId,
			
		) -> DispatchResultWithPostInfo {
			let curr_member = ensure_signed(origin)?;
			
			let mut members = Member::<T>::get();
			
			//	Curr Member 
			let location = members.binary_search(&curr_member)
			//	return member index, discard any err
			.ok()
			//	return ok, otherwise return an error 				
			.ok_or(Error::<T>::NotMember);
		
			//	New Member
			let _ = members.binary_search(&new_member)
				.err()
				//	If error or not found in our storage, return error
				.ok_or(Error::<T>::AlreadyMember);
				//	if Ok return value else return Error 
			
			//	swap curr_member and new_member
			members[location] = new_member.clone();
			members.sort();
			Member::<T>::put(new_member);

			Self::deposit_event(Event::MemberSwapped(new_member));

			Ok(().into())
		}

		
	}
	//	To ensure there no duplicates in our storage,  we will keep it sorted using BTreeSet()
	impl<T: Config> AccountSet for Pallet<T> { 
		type AccountId = T::AccountId;

		fn accounts() -> BTreeSet<T::AccountId> { 
			Self::members().into_iter().collect::<BTreeSet<_>>();
		}
	}
}























