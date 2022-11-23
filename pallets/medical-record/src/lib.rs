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
	pub trait Config: frame_system::Config + pallet_medical_encryption::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MaxRecordContentLength: Get<u32>;
		type SignatureLength: Get<u32>;
		type MaxRecordLength: Get<u32>;
	}

	#[derive(Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
	pub enum UserType {
		Patient,
		Doctor,
	}

	type RecordId = u32;
	type RecordContent<T> = BoundedVec<u8, <T as Config>::MaxRecordContentLength>;
	type Signature<T> = BoundedVec<u8, <T as Config>::SignatureLength>;

	#[derive(Decode, Encode, Debug, Eq, PartialEq, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub enum Record<T: Config> {
		VerifiedRecord(RecordId, T::AccountId, RecordContent<T>),
		UnverifiedRecord(RecordId, T::AccountId, RecordContent<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn records)]
	pub type Records<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		UserType,
		BoundedVec<Record<T>, T::MaxRecordLength>,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AccountCreated(T::AccountId, UserType),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AccountNotFound,
		AccountAlreadyExist,
		InvalidArgument,
		ExceedsMaxRecordLength,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_account(origin: OriginFor<T>, user_type: UserType) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match Self::records(&who, &user_type) {
				Some(_) => Err(Error::<T>::AccountAlreadyExist.into()),
				None => {
					<Records<T>>::insert(
						who.clone(),
						user_type.clone(),
						BoundedVec::with_bounded_capacity(T::MaxRecordLength::get() as usize),
					);
					Self::deposit_event(Event::AccountCreated(who, user_type));
					Ok(())
				},
			}
		}

		#[pallet::weight(10_000)]
		pub fn patient_adds_record(
			origin: OriginFor<T>,
			record_content: RecordContent<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let add_record = |mb_record: &mut Option<BoundedVec<Record<_>, _>>| match mb_record {
				None => Err(Error::<T>::AccountNotFound),
				Some(patient_records) => {
					let record_id = patient_records.len() as u32 + 1;
					patient_records
						.try_push(Record::<T>::UnverifiedRecord(
							record_id,
							who.clone(),
							record_content,
						))
						.map_err(|_| Error::<T>::ExceedsMaxRecordLength)
				},
			};

			<Records<T>>::mutate(&who, &UserType::Patient, add_record)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn doctor_adds_record(
			origin: OriginFor<T>,
			patient_account_id: T::AccountId,
			record_content: RecordContent<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				<Records<T>>::contains_key(&who, &UserType::Doctor),
				Error::<T>::AccountNotFound
			);
			let add_record = |mb_record: &mut Option<BoundedVec<Record<_>, _>>| match mb_record {
				None => Err(Error::<T>::AccountNotFound),
				Some(patient_records) => {
					let record_id = patient_records.len() as u32 + 1;
					patient_records
						.try_push(Record::<T>::VerifiedRecord(
							record_id,
							who.clone(),
							record_content,
						))
						.map_err(|_| Error::<T>::ExceedsMaxRecordLength)
				},
			};

			<Records<T>>::mutate(&patient_account_id, &UserType::Patient, add_record)?;

			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn doctor_verifies_record(
			origin: OriginFor<T>,
			patient_account_id: T::AccountId,
			record_id: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(
				<Records<T>>::contains_key(&who, &UserType::Doctor),
				Error::<T>::AccountNotFound
			);
			let patient_records = <Records<T>>::get(&patient_account_id, &UserType::Patient)
				.ok_or(Error::<T>::AccountNotFound)?;
			ensure!((record_id as usize) < patient_records.len(), Error::<T>::InvalidArgument);
			let old_unverified_record = &patient_records[(record_id as usize)];
			if let Record::UnverifiedRecord(_, _, content_to_verify) = old_unverified_record {
				Self::doctor_adds_record(
					origin,
					patient_account_id,
					content_to_verify.clone(),
				)?;
			}

			Ok(())
		}
	}
}
