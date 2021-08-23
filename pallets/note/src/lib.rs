//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{inherent::Vec, pallet_prelude::*, Parameter};
use frame_system::ensure_signed;
pub use pallet::*;
use sp_runtime::{
    traits::{CheckedAdd, One},
    ArithmeticError,
};
use sp_std::result::Result;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Note(pub Vec<u8>);

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_system::pallet_prelude::OriginFor;
    use sp_runtime::traits::AtLeast32BitUnsigned;
    use sp_runtime::traits::Bounded;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type NoteIndex: Parameter + AtLeast32BitUnsigned + Bounded + Default + Copy;
    }

    /// Stores all the notes. Key is (T::AccountId, T::NoteIndex).
    #[pallet::storage]
    #[pallet::getter(fn notes)]
    pub type Notes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::NoteIndex,
        Note,
        OptionQuery,
    >;

    /// Stores the next note Id.
    #[pallet::storage]
    #[pallet::getter(fn next_note_id)]
    pub type NextNoteId<T: Config> = StorageValue<_, T::NoteIndex, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::NoteIndex = "NoteIndex")]
    /// All events that can be emitted by Pallet function.
    pub enum Event<T: Config> {
        /// A note is created. \[owner, note_id, note\]
        NoteCreated(T::AccountId, T::NoteIndex, Note),
        /// A note is transfer. \[owner, receiver, note_id\]
        NoteTransferred(T::AccountId, T::AccountId, T::NoteIndex),
        /// A note is burned. \[owner, note_id\]
        NoteBurned(T::AccountId, T::NoteIndex),
    }

    #[pallet::error]
    /// All errors that can be returned by the Pallet function.
    pub enum Error<T> {
        /// The NoteId is invalid and/or doesn't exist.
        InvalidNoteId,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    /// Contains all user-facing functions.
    impl<T: Config> Pallet<T> {
        /// Create a new note
        #[pallet::weight(1000)]
        pub fn create(origin: OriginFor<T>, ipfs_cid: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let current_id = Self::get_next_note_id()?;
            // Create new note
            let note = Note(ipfs_cid);
            // Insert note into Notes data storage
            Notes::<T>::insert(&sender, current_id, &note);
            // Emit event
            Self::deposit_event(Event::NoteCreated(sender, current_id, note));
            // Return success
            Ok(())
        }

        #[pallet::weight(1000)]
        /// Burn a note
        pub fn burn(origin: OriginFor<T>, note_id: T::NoteIndex) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Notes::<T>::try_mutate_exists(sender.clone(), note_id, |note| -> DispatchResult {
                // Test the user owns this note
                let _n = note.take().ok_or(Error::<T>::InvalidNoteId)?;
                let s = sender.clone();
                // Remove note from Notes data structure
                Notes::<T>::remove(sender, note_id);
                // Emit event
                Self::deposit_event(Event::NoteBurned(s, note_id));
                // Return success
                Ok(())
            })
        }

        #[pallet::weight(1000)]
        /// Transfer note to new owner
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            note_id: T::NoteIndex,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            Notes::<T>::try_mutate_exists(sender.clone(), note_id, |note| -> DispatchResult {
                if sender == to {
                    ensure!(note.is_some(), Error::<T>::InvalidNoteId);
                    return Ok(());
                }
                let note = note.take().ok_or(Error::<T>::InvalidNoteId)?;
                Notes::<T>::insert(&to, note_id, note);
                Self::deposit_event(Event::NoteTransferred(sender, to, note_id));
                Ok(())
            })
        }
    }
}

impl<T: Config> Pallet<T> {
    fn get_next_note_id() -> Result<T::NoteIndex, DispatchError> {
        NextNoteId::<T>::try_mutate(|next_id| -> Result<T::NoteIndex, DispatchError> {
            let current_id = *next_id;
            *next_id = next_id
                .checked_add(&One::one())
                .ok_or(ArithmeticError::Overflow)?;
            Ok(current_id)
        })
    }
}
