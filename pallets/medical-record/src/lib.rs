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
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MaxRecordContentLength: Get<u32>;
		type SignatureLength: Get<u32>;
		type MaxRecordLength: Get<u32>;
	}

	type RecordId = u32;
	type RecordContent<T> = BoundedVec<u8, <T as Config>::MaxRecordContentLength>;
	type Signature<T> = BoundedVec<u8, <T as Config>::SignatureLength>;

	#[derive(Decode, Encode, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub enum Record<T: Config> {
		VerifiedRecord(RecordId, T::AccountId, RecordContent<T>, Signature<T>),
		UnverifiedRecord(RecordId, T::AccountId, RecordContent<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn patient_records)]
	pub type PatientRecords<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<Record<T>, T::MaxRecordLength>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PatientAccountCreated(T::AccountId),
		DoctorAccountCreated(T::AccountId),
		PatientRecordsUpdated(u32, T::AccountId),
		DoctorRecordsUpdated(u32, T::AccountId),
		PatientRecordVerified(u32),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AccountNotFound,
		AccountAlreadyExist,
		InvalidRecordId,
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
		#[pallet::weight(10_000)]
		pub fn create_patient_account(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match Self::patient_records(&who) {
				Some(_) => Err(Error::<T>::AccountAlreadyExist.into()),
				None => {
					<PatientRecords<T>>::insert(
						who.clone(),
						BoundedVec::with_bounded_capacity(T::MaxRecordLength::get() as usize),
					);
					Self::deposit_event(Event::PatientAccountCreated(who));
					Ok(())
				},
			}
		}
	}
}
