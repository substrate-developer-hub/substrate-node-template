#![cfg_attr(not(feature = "std"), no_std)]

/// Pallet to manage the state of the docusign
pub use pallet::*;
pub use core::str;
pub use core::str::FromStr;
pub use scale_info::prelude::vec::Vec;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
	}

	// we use a safe crypto hashing by blake2_128
	// Passport data storage, you can have multiple state if the image stored is bigger >100K
	// the UI has to assemble the different parts to render properly the image
	#[pallet::storage]
	#[pallet::getter(fn get_passportbyaccount)]
	pub(super) type Passports<T: Config> = StorageDoubleMap< _,Blake2_128Concat, T::AccountId,Blake2_128Concat, u32,Vec<u8>,ValueQuery>;
	
	/*
	// This is storage to search by passport number and obtain the account linked, the passport data is on the Passports storage
	#[pallet::storage]
	#[pallet::getter(fn get_passportbynumber)]
	pub(super) type PassportNumbers<T: Config> = StorageDoubleMap< _,Blake2_128Concat, u32,Blake2_128Concat, u32,T::AccountId,ValueQuery>;
    */
	// The administrators account allowed to create new passports one single account is allowed (+ sudo)
	// a type of access is in the mapping for future usage, the passport writing access level is 1
	#[pallet::storage]
	#[pallet::getter(fn get_adminaccount)]
	pub(super) type Administrators<T: Config> = StorageDoubleMap< _,Blake2_128Concat, T::AccountId,Blake2_128Concat, u32,Vec<u8>,ValueQuery>;
	
    
	// Events definitions
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>  {
		/// Event documentation should end with an array that provides descriptive names for event
		PassportCreated(T::AccountId, u32),       // New passport has been created
        PassportDestroyed(T::AccountId,u32),              // Passport destroyed
		NewAdministrator(T::AccountId, u32,Vec<u8>), // new administrator
		DestroyedAdministrator(T::AccountId, u32),   // destroyed administrator
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
        /// Passport is too long, cannot be more than 100000 bytes
        PassportTooLong,
        /// Document address is too short, cannot be less than 1000 bytes
        PassportTooShort,
        /// Id cannot be zero
        IdCannotBeZero,
        /// Document already present on the blockchain
        PassportAlreadyPresent,
		///the signer is not the administrator account neither the superuser
		SignerHasNoAccess,
		/// Passport has not been found on chain
		PassportNotFound,
		/// Account already configured as administrator
		AdministratorAlreadyPresent,
		/// Note is too long max 128 chars
		NoteTooLong,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	#[pallet::call]
	impl<T: Config > Pallet<T> {
		
		  /// Create a new passport
		  #[pallet::call_index(1)]
		  #[pallet::weight(T::WeightInfo::cause_error())]
		  pub fn new_passport(origin:OriginFor<T>, account: T::AccountId,id: u32,passport: Vec<u8>) -> DispatchResult {
			  // check the request is signed
			  let sender = ensure_signed(origin)?;
			  // check for a valid administrator account
			  ensure!(Administrators::<T>::contains_key(&sender,1),Error::<T>::SignerHasNoAccess);
			  //check passport length
              ensure!(passport.len() < 32, Error::<T>::PassportTooShort);
			  ensure!(passport.len() > 100000, Error::<T>::PassportTooLong);
              ensure!(id>0,Error::<T>::IdCannotBeZero);
              ensure!(!Passports::<T>::contains_key(&account,&id),Error::<T>::PassportAlreadyPresent);
			  // Insert new passport
			  Passports::<T>::insert(account.clone(),id.clone(),passport);
			  // Generate event
			  Self::deposit_event(Event::PassportCreated(account,id));
			  // Return a successful DispatchResult
			  Ok(())
		  }
		  /// Destroy a Passport
		  #[pallet::call_index(2)]
		  #[pallet::weight(T::WeightInfo::cause_error())]
		  pub fn destroy_passport(origin:OriginFor<T>,account: T::AccountId,id:u32) -> DispatchResult {
			  // check the request is signed
			  let sender = ensure_signed(origin)?;
			  // check for a valid administrator account as signer
			  ensure!(Administrators::<T>::contains_key(&sender,1),Error::<T>::SignerHasNoAccess);
			  // verify the passport exists
			  ensure!(Passports::<T>::contains_key(&account,&id)==true, Error::<T>::PassportNotFound);
			  // Remove Document 
			  Passports::<T>::take(account.clone(),id.clone());
			  // Generate event
			  Self::deposit_event(Event::PassportDestroyed(account,id));
			  // Return a successful DispatchResult
			  Ok(())
		  }
		  /// Add Admin Account
		  #[pallet::call_index(3)]
		  #[pallet::weight(T::WeightInfo::cause_error())]
		  pub fn new_admin(origin:OriginFor<T>,account: T::AccountId,id:u32,note:Vec<u8>) -> DispatchResult {
			  // check the request is signed from root
			  let _sender = ensure_root(origin)?;
			  // check the same account is not already poresent for the same level
			  ensure!(!Administrators::<T>::contains_key(account.clone(),id.clone()),Error::<T>::AdministratorAlreadyPresent);
			  // check size of note field
			  ensure!(note.len() < 128, Error::<T>::NoteTooLong);
			  // add administrator
			  Administrators::<T>::insert(account.clone(),id.clone(),note.clone());
			  // Generate event
			  Self::deposit_event(Event::NewAdministrator(account,id,note));
			  // Return a successful DispatchResult
			  Ok(())
		  }
		  /// Destroy/remove Admin Account
		  #[pallet::call_index(4)]
		  #[pallet::weight(T::WeightInfo::cause_error())]
		  pub fn destroy_admin(origin:OriginFor<T>,account: T::AccountId,id:u32) -> DispatchResult {
			  // check the request is signed from root
			  let _sender = ensure_root(origin)?;
			  // check is present for the same level
			  ensure!(Administrators::<T>::contains_key(account.clone(),id.clone()),Error::<T>::AdministratorAlreadyPresent);
			  // add administrator
			  Administrators::<T>::take(account.clone(),id.clone());
			  // Generate event
			  Self::deposit_event(Event::DestroyedAdministrator(account,id));
			  // Return a successful DispatchResult
			  Ok(())
		  }
	}
	
}

