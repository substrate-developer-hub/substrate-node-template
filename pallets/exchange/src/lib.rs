#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	
	use frame_support::{dispatch::DispatchResultWithPostInfo, ensure, pallet_prelude::*, 
		traits::Parameter};
	use frame_system::{pallet_prelude::*, ensure_signed};
	use sp_runtime::traits::AtLeast32BitUnsigned;
	use sp_runtime::traits::Saturating;



	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_balance)]
	pub type BalanceToAccount<T> = StorageMap<
		_, 
		Blake2_128Concat,
		T::AccountId,	
		T::Balance,
		ValueQuery
		>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [Account Id]
		MintedNewSupply(T::AccountId),
		/// [From, to, balance]
		Transferred(T::AccountId, T::AccountId, T::Balance),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		//	Account not found 
		InvalidAccountId,
		// Balanace cant be zero
		BalanceZero,
		//	Not enough funds to transfer 
		BalanceLow,
		SameAddress,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer(
			origin: OriginFor<T>, 
			to: T::AccountId,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?;
			//	Balances of acocunts
			let sender_balance = Self::get_balance(&origin); // return value of account 
			let target_balance = Self::get_balance(&to); // return value of account 

			ensure!(origin != to, Error::<T>::SameAddress)?;
			ensure!(sender_balance >= amount, Error::<T>::BalanceLow)?;
			ensure!(sender_balance.is_some(), Error::<T>::BalanceZero)?;


			// Update balances of sender 
			let sender_update = sender_balance.saturating_sub(amount);
			let target_update = target_balance.saturating_add(amount);
			// Update storages
			//	insert: store a value to be associated with the given key from the map
			<BalanceToAccount<T>>::insert(&sender, sender_update);
			<BalanceToAccount<T>>::insert(&sender, target_balance);

			// Emit an event.
			Self::deposit_event(Event::Transferred(&sender, to, amount));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn mint(
			origin: OriginFor<T>, 
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?;

			// Update: Sender's Balance 
			<BalanceToAccount<T>>::insert(&sender, amount);
			Self::deposit_event(Event::MintedNewSupply(&sender));

			Ok(().into())
			
		}
	}
}
