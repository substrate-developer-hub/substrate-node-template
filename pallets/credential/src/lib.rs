#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use sp_std::prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        sp_runtime::traits::Hash
    };
    
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewCredentialCreated(T::AccountId, Credential<T::Hash, T::AccountId>),
        CredentialRemoved(T::AccountId),
        UserCredentialsUpdated(T::AccountId)
    }

    #[pallet::error]
    pub enum Error<T> {
        CredentialAlreadyExist,
        MaximumCredentialExceeded,
        CredentialNotBelong,
        CredentialNotExist
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type HashOf<T> = <T as frame_system::Config>::Hash;

    #[pallet::storage]
    #[pallet::getter(fn credential_by_owner)]
    pub(super) type Credentials<T: Config> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Vec<Credential<HashOf<T>, AccountIdOf<T>>>>;

    #[pallet::storage]
    #[pallet::getter(fn credential_by_id)]
    pub(super) type CredentialById<T: Config> = StorageMap<_, Blake2_128Concat, HashOf<T>, Credential<HashOf<T>, AccountIdOf<T>>>;

    #[pallet::storage] 
	#[pallet::getter(fn credential_by_people)]
    pub(super) type CredentialByPeople<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Credential<HashOf<T>, AccountIdOf<T>>>;

    #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub fn mint_credential(origin: OriginFor<T>, people_id: Vec<u8>, platform: Vec<u8>) -> DispatchResult {
            let creator = ensure_signed(origin)?;

            match CredentialByPeople::<T>::get(people_id.clone()) {
                None => {},
                Some(credential) => {
                    ensure!(credential.owner_id == creator, Error::<T>::CredentialAlreadyExist);
                }
            }

            let mut credentials = Self::credential_by_owner(&creator).unwrap_or(Vec::new());

            ensure!(credentials.len() < 3, Error::<T>::MaximumCredentialExceeded);

            let mut flag = false;

            for credential in &credentials {
                if credential.platform == platform {
                    flag = true;
                    break;
                }

                if credential.people_id == people_id {
                    flag = true;
                    break;
                }
            }

            ensure!(flag == false, Error::<T>::CredentialAlreadyExist);


            let new_credential = Self::create_credential(&creator, &people_id, &platform);

            credentials.push(new_credential.clone());

            Credentials::<T>::insert(&creator, &credentials);

            Self::deposit_event(Event::NewCredentialCreated(creator, new_credential));

            Ok(().into())
        }

        #[pallet::weight(1_000)]
        pub fn remove_credential(origin: OriginFor<T>, id: HashOf<T>) -> DispatchResult {
            let destroyer = ensure_signed(origin)?;

            match CredentialById::<T>::get(&id) {
                None => {
                    ensure!(!true, Error::<T>::CredentialNotExist);
                },
                Some(credential) => {
                    ensure!(credential.owner_id == destroyer, Error::<T>::CredentialNotBelong);
                    
                    CredentialById::<T>::remove(&id);
                    CredentialByPeople::<T>::remove(&credential.people_id);

                    Self::deposit_event(Event::CredentialRemoved(destroyer.clone()));
                }
            }

            match Credentials::<T>::get(&destroyer) {
                None => {},
                Some(credentials) => {
                    let mut new_credentials: Vec<Credential<HashOf<T>, AccountIdOf<T>>> = Vec::new();

                    for credential in &credentials {
                        if credential.id != id {
                            new_credentials.push(credential.clone());
                        }
                    }

                    Credentials::<T>::insert(&destroyer, &new_credentials);

                    Self::deposit_event(Event::UserCredentialsUpdated(destroyer.clone()))
                }
            }

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn create_credential(
            owner_id: &T::AccountId,
            people_id: &Vec<u8>,
            platform: &Vec<u8>
        ) -> Credential<HashOf<T>, AccountIdOf<T>> {
            let credentials = Self::credential_by_owner(owner_id).unwrap_or(Vec::new());
            let count: u64 = credentials.len() as u64;
            let credential_id = Self::generate_id(owner_id, people_id, platform, count);

            let new_credential = Credential::new(credential_id.clone(), owner_id.clone(), people_id.clone(), platform.clone());

            if count <= 3 {
                CredentialById::<T>::insert(&credential_id, &new_credential);
                CredentialByPeople::<T>::insert(&people_id, &new_credential);
            }

            new_credential
        }

        pub fn generate_id(
            owner_id: &T::AccountId, 
            people_id: &Vec<u8>, 
            platform: &Vec<u8>, 
            count: u64
        ) -> HashOf<T> {
            let mut owner_id_byte = owner_id.encode();
			let mut people_id_byte = people_id.encode();
			let mut platform_byte = platform.encode();
			let mut count_byte = count.encode();

            owner_id_byte.append(&mut people_id_byte);
			owner_id_byte.append(&mut platform_byte);
			owner_id_byte.append(&mut count_byte);

			let seed = &owner_id_byte;
			T::Hashing::hash(seed)
        }
    }

    #[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
    pub struct Credential<Hash, AccountId> {
        id: Hash,
        owner_id: AccountId,
        people_id: Vec<u8>,
        platform: Vec<u8>
    }

    impl <Hash, AccountId> Credential <Hash, AccountId> {
        pub fn new(
            id: Hash,
            owner_id: AccountId,
            people_id: Vec<u8>,
            platform: Vec<u8>
        ) -> Self {
            Self {id, owner_id, people_id, platform}
        }
    }
}
