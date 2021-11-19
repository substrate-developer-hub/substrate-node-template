#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::{prelude::*, if_std};
use sp_runtime::{
	RuntimeDebug, 
	print,
	traits::{AtLeast32BitUnsigned, Zero, Saturating, CheckedAdd, CheckedSub, Printable},
};

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct MetaData<AccountId, Balance> { 

	issuance: Balance, 

	minter: AccountId,

	burner: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, transactional};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//	the type used to store balances 
		type Balance: Balance + Parameter + AtLeast32BitUnsigned + Default + Copy;
		//	the minimum balance necessary for an account to exist
		type MinBalance: Get<Self::Balance>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn metadata_store)]
	pub type MetaDataStore<T: Config> = StorageValue<
		_, 
		MetaData<T::AccountId, T::Balance>, 
		ValueQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn account)]
	pub type Accounts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		T::Balance,
		ValueQuery, 
	>;

	//	Genesis COnfig 
	//	The value of this attribute will be used as the initial value of the storage item in your chain's genesis block
	//	The config extension takes a parameter that will determine the name of the attribute on the GenesisConfig data type - this parameter is optional if the get extension is provided 

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub admin: T::AccountId,
		//	DOn't forget to update the chain_spec 
	}
	
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> { 
		fn default() -> Self { 
			Self { 
				admin: Default::default()
			}
		}
	
	}
	//	This allows you to define how genesis_configuration is built within the pallet itself 
	//	This will set the initial value of a storage item 
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig { 
		fn build(&self) { 
			MetaDataStore::<T>::put(
				MetaData {
					issuance: Zero::zero(),
					minter: self.admin.clone(),
					burned: self.admin.clone(),
				}
			);
		}
	}
	//	TODO Set up Blockchain Hooks 
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> { 
		fn on_initialize(_n: T::BlockNumber) -> Weight { 
			let mut meta = MetaDataStore::<T>::get();

			let value: T::Balance = 50u8;
			meta.issuance = meta.issuance.saturating_add(value);


			//	 \[Minter, Balance]\
			Accounts::<T>::mutate(&meta.minter, |bal| { 
				*bal = bal.saturating_add(value)
			});
			0
		}
	}


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(
		T::AccountId = "AccountId",
		T::Balance = "Balance")]
	pub enum Event<T: Config> {
		Created(T::AccountId),
		//	[beneficiary, amount minted]
		Killed(T::AccountId),

		Minted(T::AccountId, T::Balance),
		
		Burned(T::AccountId, T::Balance),
		
		Transferred(T::AccountId, T::AccountId, T::Balance),
		//	[from, to, balance]
	}

	#[pallet::error]
	pub enum Error<T> {
		BelowMinBalance,
		//	An account would go below the minimum balance if the operation were executed 
		NoPermission,
		//	The origin account does not have the required permission for the operation 
		Overflow,
		//	An operation would lead to overflow 
		Underflow,
		//	An operation would lead to underflow 
		CannotBurnEmpty,
		//	Cannot burn the balance of a non-existent account 
		InsufficientBalance 
		//	Not enough balance in the sender's account for the transfer 
	}
	impl<T: Config> Printable for Error<T> { 
		fn print(&self) { 
			match self { 
				Error::Overflow => "Value Exceed and Overflowed".print(),
				Error::NoPermission => "You Have No Permission to Access".print(),
				Error::Underflow =>  "Underflowed".print(),
				Error::CannotBurnEmpty => "You Have to include an Amount here before burnign Duh!".print(),
				Error::InsufficientBalance => "You're broke? Lol Tryagain next time!".print(),
				_ => "Invalid Error Case".print(),
			}
		}

	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
		// 	// Check that the extrinsic was signed and get the signer.
		// 	// This function will return an error if the extrinsic is not signed.
		// 	// https://substrate.dev/docs/en/knowledgebase/runtime/origin
		// 	let who = ensure_signed(origin)?;

		// 	// Update storage.
		// 	<Something<T>>::put(something);

		// 	// Emit an event.
		// 	Self::deposit_event(Event::SomethingStored(something, who));
		// 	// Return a successful DispatchResultWithPostInfo
		// 	Ok(())
		// }
		#[pallet::weight(1_000)]
		pub fn mint(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let meta = Self::metadata_store();
			//	Ensure the caller is that the minter is also the sender for the metadata struct
			ensure!(sender == meta.minter, Error::<T>::NoPermission);
			ensure!(amount >= T::MinBalance::get(), Error::<T>::BelowMinBalance);

			//	Add the Amount to issuance 
			meta.issuance = meta.issuance.checked_add(&amount).ok_or(Error::<T>::Overflow)?;

			//	Update the new issuance to MetaData Store
			MetaDataStore::<T>::put(meta);

			if Self::increase_balance(&beneficiary, amount) { 
				Self::deposit_event(Event::Created(beneficiary.clone()))
			}
			Self::deposit_event(Event::Minted(beneficiary, amount));
		
			Ok(().into())
		}
		
		#[pallet::weight(1_000)]
		#[transactional]
		pub(super) fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId, 
			#[pallet::compact] amount: T::Balance
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			//	Mutate the storage map for the sender account  
			Accounts::<T>::try_mutate(&sender, |bal| -> DispatchResult { 
				let new_bal = *bal;
				*bal = bal.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance)?;
				ensure!(new_bal >= T::MinBalance::get(), Error::<T>::BelowMinBalance);
				Ok(())
			})?;
			Accounts::<T>::try_mutate(&to, |bal| -> DisptachResult { 
				let new_bal = *bal;
				ensure!(new_bal >= T::MinBalance::get(), Error::<T>::BelowMinBalance);
				*bal = bal.saturating_add(amount);
				Ok(())
			})?;
			Self::deposit_event(Event::Transferred(sender, to, amount));

			Ok(().into())
		}
		#[pallet::weight(1_000)]
		pub fn burn(
			origin: OriginFor<T>,
			burned: T::AccountId,
			#[pallet::compact] amount: T::Balance, 
			allow_killing: bool,
		) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?;
			print("After the user has signed in");
			let mut meta = Self::metadata_store();
			ensure!(sender == meta.burner, Error::<T>::NoPermission);

			let balance = Accounts::<T>::get(&burned);
			//	get: load the value associated with the key(burned)
			
			ensure!(balance > Zero::zero(), Error::<T>::CannotBurnEmpty);

			let new_balance = balance.saturating_sub(amount);
			//	Take away the amount to be 'burned' to the burned account

			print("Gathering burning Amount!")
			let burn_amount = if new_balance < T::MinBalance::get() { 
				ensure!(allow_killing, Error::<T>::BelowMinBalance);
				
				let burn_amount = balance;
				//	Set the burn_amount to the balanace of the sender
				ensure!(meta.issuance.checked_sub(&burn_amount).is_some(), Error::<T>::Underflow);
				//	Check if the sender has enough to burn in their account 
				Accounts::<T>::remove(&burned);
				//	Remove Key for Burned Account 
				Self::deposit_event(Event::<T>::Killed(burned.clone()));
				//	Emit event for killing the burned account 
				burn_amount
			} else { 
				let burn_amount = amount;
				ensure!(meta.issuance.checked_sub(&burn_amount).is_some(), Error::<T>::Underflow);
				//	If success, store this new value to the associated key in the storage map 
				Accounts::<T>::insert(&burned, new_balance);
				burn_amount
			};

			if_std! {
				println!("This is the amount to be burnt! {:#?}", burn_amount);
				println!("The caller of this logic is {:#?}", sender);
			}

			meta.issuance = meta.issuance.saturating_sub(&burn_amount);

			//	Update the value stored under this key 
			MetaDataStore::<T>::put(meta);
			
			//	Emit event 
			Self::deposit_event(Event::<T>::Burned(&burned, burn_amount));

			Ok(().into())
		}
	} 
}
//	Internal functions of the pallet 
impl<T: Config> Pallet<T> {
	//	This function should increase the balance of an account stored on chain 
	fn increase_balance(acc: &T::AccountId, amount: T::Balance) -> bool { 

		Accounts::<T>::mutate(&acc, |bal| { 
			let created = *bal == &Zero::zero();
			//	This will add the amount to the account balance stored in our chain 
			*bal = bal.saturating_add(amount);	
			created
		})
	}
}
