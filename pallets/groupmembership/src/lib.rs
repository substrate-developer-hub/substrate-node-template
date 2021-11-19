#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::Printable;
use sp_runtime::print;
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
	
	pub type GroupIndex = u32;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type AllMembers<T> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn member_score)]
	pub type MembershipScore<T> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		GroupIndex,
		Blake2_128Concat,
		T::AccountId, 
		u32,
		ValueQuery,
	>;
	
	#[pallet::storage]
	#[pallet::getter(fn membership)]
	pub type GroupMembership<T> = StorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId, 
		GroupIndex, 
		ValueQuery
	>;
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewMember(T::AccountId),
		MembersJoinGroup(T::AccountId, GroupIndex, u32),
		RemoveMember(T::AccountId),
		RemoveGroup(GroupIndex),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		/// Errors should have helpful documentation associated with them.
		NotMember,
		MemberIsNotFound,
	}
	impl<T: Config> Printable for Error<T> { 
		fn print(&self) { 
			match self { 
				Error::NotMember => "Not a member, can't remove!".print(),
				Error::MemberIsNotFound => "Member is not found in this group!".print(),
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
		pub fn add_members(
			origin: OriginFor<T>
		) -> DispatchResultWithPostInfo {
			let new_member = ensure_signed(origin)?;
			//	ensure the member is unique
			ensure!(!Self::is_member(&new_member), Error::<T>::NotMember); 
			//	Add member into AllMembers storagevalue  	
			AllMembers::<T>::append(&new_member);
			//	Emit event New Member
			Self::deposit_event(Event::NewMember(new_member));
			Ok(().into())
		}
		#[pallet::weight(10_000)]
		pub fn join_a_group(
			origin: OriginFor<T>,
			group_index: GroupIndex,
			score: u32, 
		) -> DispatchResultWithPostInfo {
			let member = ensure_signed(origin)?;
			//	Ensure the member is found in the DB of ALlMembers 
			ensure!(Self::is_member(&member), Error::<T>::NotMember)?;
			//	Insert MembershipScore into StorageDouble
			MembershipScore::<T>::insert(&group_index, &member, score);
			//	Insert member into Storage GroupMembership
			GroupMembership::<T>::insert(&member, &group_index);

			Self::deposit_event(Event::MembersJoinGroup(member, group_index, score))
			Ok(().into())
		}
		#[pallet::weight(10_000)]
		pub fn remove_member(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			let curr_member = ensure_signed(origin)?;
			//	Ensure the member exists
			ensure!(Self::is_member(&curr_member), Error::<T>::NotMember);
			//	Remove where the member can be found in a group: Get the group index  
			let assigned_id = GroupMembership::<T>::take(curr_member.clone());
			MembershipScore::<T>::remove(&assigned_id, &curr_member);

			Self::deposit_event(Event::RemoveMember(curr_member));
			Ok(().into())
		}
		#[pallet_weight(10_000)]
		pub fn remove_group_score(
			origin: OriginFor<T>,
			group_index: GroupIndex,
		) -> DispatchResultWithPostInfo {
			let member = ensure_signed(origin)?;

			//	Check if the member is in a group first 
			let group_id = GroupMembership::<T>::get(member);
			ensure!(
				group_id == group, 
				Error::<T>::MemberIsNotFound
			);
			MembershipScore::<T>::remove_prefix(&group_index);
			//	Remove all values under this first key 
			Self::deposit_event(Event::RemoveGroup(group_id));
			Ok(().into())
		}
	}
}
impl<T: Config> Pallet<T> { 
	fn is_member(acc: &T::AccountId) -> bool { 
		//	Load values on this storage
		let members = AllMembers::<T>::get();
		//	Check if 'acc' exists on loaded values above
		members.contains(acc)
	}
}