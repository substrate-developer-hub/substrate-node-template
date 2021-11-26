//! Double Map Example with remove_prefix
//! `double_map` maps two keys to a single value.
//! the first key might be a group identifier
//! the second key might be a unique identifier
//! `remove_prefix` enables clean removal of all values with the group identifier

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
pub use pallet::*;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	pub type GroupIndex = u32; // this is Encode (which is necessary for double_map)

	#[pallet::storage]
	#[pallet::getter(fn member_score)]
	pub(super) type MemberScore<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		GroupIndex,
		Blake2_128Concat,
		T::AccountId,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn group_membership)]
	pub(super) type GroupMembership<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, GroupIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_members)]
	pub(super) type AllMembers<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New member for `AllMembers` group
		NewMember(T::AccountId),
		/// Put member score (id, index, score)
		MemberJoinsGroup(T::AccountId, GroupIndex, u32),
		/// Remove a single member with AccountId
		RemoveMember(T::AccountId),
		/// Remove all members with GroupId
		RemoveGroup(GroupIndex),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Join the `AllMembers` vec before joining a group
		#[pallet::weight(10_000)]
		pub fn join_all_members(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let new_member = ensure_signed(origin)?;
			ensure!(
				!Self::is_member(&new_member),
				"already a member, can't join"
			);
			<AllMembers<T>>::append(&new_member);

			Self::deposit_event(Event::NewMember(new_member));
			Ok(().into())
		}

		/// Put MemberScore (for testing purposes)
		#[pallet::weight(10_000)]
		pub fn join_a_group(
			origin: OriginFor<T>,
			index: GroupIndex,
			score: u32,
		) -> DispatchResultWithPostInfo {
			let member = ensure_signed(origin)?;
			ensure!(Self::is_member(&member), "not a member, can't remove");
			<MemberScore<T>>::insert(&index, &member, score);
			<GroupMembership<T>>::insert(&member, &index);

			Self::deposit_event(Event::MemberJoinsGroup(member, index, score));
			Ok(().into())
		}

		/// Remove a member
		#[pallet::weight(10_000)]
		pub fn remove_member(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let member_to_remove = ensure_signed(origin)?;
			ensure!(
				Self::is_member(&member_to_remove),
				"not a member, can't remove"
			);
			let group_id = <GroupMembership<T>>::take(member_to_remove.clone());
			<MemberScore<T>>::remove(&group_id, &member_to_remove);

			Self::deposit_event(Event::RemoveMember(member_to_remove));
			Ok(().into())
		}

		/// Remove group score
		#[pallet::weight(10_000)]
		pub fn remove_group_score(
			origin: OriginFor<T>,
			group: GroupIndex,
		) -> DispatchResultWithPostInfo {
			let member = ensure_signed(origin)?;

			let group_id = <GroupMembership<T>>::get(member);
			// check that the member is in the group
			ensure!(
				group_id == group,
				"member isn't in the group, can't remove it"
			);

			// remove all group members from MemberScore at once
			<MemberScore<T>>::remove_prefix(&group_id);

			Self::deposit_event(Event::RemoveGroup(group_id));
			Ok(().into())
		}
	}
	//	Helper Functions
	impl<T: Config> Pallet<T> { 
		fn is_member(acc: &T::AccountId) -> bool { 
			//	Load values on this storage
			let members = AllMembers::<T>::get();
			//	Check if 'acc' exists on loaded values above
			members.contains(acc)
		}
	}
	//	Impl Storage Functions
	impl<T: Config> MembershipScore<T> { 
		fn add_member(who: &T::AccountId, index: GroupIndex, score: u32) { 
			MembershipScore::<T>::insert(group_index, member, score)
		}
		fn remove_score(index: GroupIndex) { 
			Membership::<T>::remove_prefix(&index)
		}
	}
}

