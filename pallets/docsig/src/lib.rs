#![cfg_attr(not(feature = "std"), no_std)]

/// Pallet to manage the state of the docsig
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
	// Document data storage
	#[pallet::storage]
	#[pallet::getter(fn get_document)]
	pub(super) type Documents<T: Config> = StorageDoubleMap< _,Blake2_128Concat, T::AccountId,Blake2_128Concat, u32,Vec<u8>,ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_signature)]
	pub(super) type Signatures<T: Config> = StorageDoubleMap< _,Blake2_128Concat, T::AccountId,Blake2_128Concat, u32,Vec<u8>,ValueQuery>;

	// Events definitions
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>  {
		/// Event documentation should end with an array that provides descriptive names for event
		DocumentCreated(T::AccountId, u32,Vec<u8>),       // New document has been created
        DocumentDestroyed(T::AccountId,u32),              // Document destroyed
        DocumentSigned(T::AccountId,u32,Vec<u8>),         // Document signed
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
        /// Document address is too long, cannot be more than 128
        DocumentTooLong,
        /// Document address is too short, cannot be less than 32
        DocumentTooShort,
        /// Id cannot be zero
        IdCannotBeZero,
        /// Document not found on the blockchain
        DocumentNotFound,
        /// Document already present on the blockchain
        DocumentAlreadyPresent,
        /// Document has been already signed from the sender
        DocumentAlreadySigned,
        ///  hash is too short
        HashTooShort,
        ///  hash is too long
        HashTooLong,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	#[pallet::call]
	impl<T: Config > Pallet<T> {
		
		  /// Create a new document to be signed
		  #[pallet::call_index(1)]
		  #[pallet::weight(T::WeightInfo::cause_error())]
		  pub fn new_document(origin:OriginFor<T>, id: u32,document: Vec<u8>) -> DispatchResult {
			  // check the request is signed
			  let sender = ensure_signed(origin)?;
			  //check document length
		          ensure!(document.len() >= 32, Error::<T>::DocumentTooShort);
		          ensure!(document.len() <= 128, Error::<T>::DocumentTooLong);
		          ensure!(id>0,Error::<T>::IdCannotBeZero);
		          ensure!(!Documents::<T>::contains_key(&sender,&id),Error::<T>::DocumentAlreadyPresent);
			  // Insert new Document
			  Documents::<T>::insert(sender.clone(),id.clone(),document.clone());
			  // Generate event
			  Self::deposit_event(Event::DocumentCreated(sender,id,document));
			  // Return a successful DispatchResult
			  Ok(())
		  }
		  /// Destroy a Document
		  #[pallet::call_index(2)]
		  #[pallet::weight(T::WeightInfo::cause_error())]
		  pub fn destroy_document(origin:OriginFor<T>,id:u32) -> DispatchResult {
			  // check the request is signed
			  let sender = ensure_signed(origin)?;
			  // verify the document exists
			  ensure!(Documents::<T>::contains_key(&sender,&id)==true, Error::<T>::DocumentNotFound);
			  // Remove Document 
			  Documents::<T>::take(sender.clone(),id.clone());
			  // Generate event
			  //it can leave orphans, anyway it's a decision of the super user
			  Self::deposit_event(Event::DocumentDestroyed(sender,id));
			  // Return a successful DispatchResult
			  Ok(())
		  }
          #[pallet::call_index(3)]
		  #[pallet::weight(T::WeightInfo::cause_error())]
		  pub fn sign_document(origin:OriginFor<T>, id: u32,hash: Vec<u8>) -> DispatchResult {
			  // check the request is signed
			  let sender = ensure_signed(origin)?;
			  //check  hash length
		          ensure!(hash.len() < 128, Error::<T>::HashTooLong);
	                  ensure!(hash.len() > 2, Error::<T>::HashTooShort);
		          ensure!(id>0,Error::<T>::IdCannotBeZero);
	                  ensure!(!Signatures::<T>::contains_key(&sender,&id),Error::<T>::DocumentAlreadySigned);
			  // Insert Signature
			  Signatures::<T>::insert(sender.clone(),id.clone(),hash.clone());
			  // Generate event
			  Self::deposit_event(Event::DocumentSigned(sender,id,hash));
			  // Return a successful DispatchResult
			  Ok(())
		  }

	}
	
}

