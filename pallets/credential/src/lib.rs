#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use sp_std::prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*
    };
    
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewCredentialCreated(T::AccountId, Credential<T::AccountId>),
        CredentialRemoved(T::AccountId),
        UserCredentialsUpdated(T::AccountId)
    }

    #[pallet::error]
    pub enum Error<T> {
        CredentialAlreadyExist,
        CredentialAlreadyBelong,
        MaximumCredentialExceeded,
        CredentialNotBelong,
        CredentialNotExist,
        PlatformNotExist
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    #[pallet::storage]
    #[pallet::getter(fn credential_by_owner)]
    pub(super) type Credentials<T: Config> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Vec<Credential<AccountIdOf<T>>>>;

    #[pallet::storage] 
	#[pallet::getter(fn credential_by_people)]
    pub(super) type CredentialByPeople<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Credential<AccountIdOf<T>>>;

    #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn add_credential(
            origin: OriginFor<T>, 
            people_id: Vec<u8>, 
            platform: Vec<u8>
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;
            let platforms: Vec<Vec<u8>> = vec![
                "twitter".as_bytes().to_vec(),
                "facebook".as_bytes().to_vec(),
                "reddit".as_bytes().to_vec()
            ];

            let mut flag = false;

            for (_, e) in platforms.iter().enumerate() {
                if e == &platform {
                    flag = true;
                    break;
                }
            }

            ensure!(flag, Error::<T>::PlatformNotExist);
                        
            match CredentialByPeople::<T>::get(people_id.clone()) {
                None => {},
                Some(credential) => {
                    ensure!(credential.owner_id == creator, Error::<T>::CredentialAlreadyBelong);

                    ensure!(credential.owner_id != creator, Error::<T>::CredentialAlreadyExist);
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

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_credential(origin: OriginFor<T>, people_id: Vec<u8>) -> DispatchResult {
            let destroyer = ensure_signed(origin)?;

            match CredentialByPeople::<T>::get(&people_id) {
                None => {
                    ensure!(!true, Error::<T>::CredentialNotExist);
                },
                Some(credential) => {
                    ensure!(credential.owner_id == destroyer, Error::<T>::CredentialNotBelong);
                    
                    CredentialByPeople::<T>::remove(&credential.people_id);

                    Self::deposit_event(Event::CredentialRemoved(destroyer.clone()));
                }
            }

            match Credentials::<T>::get(&destroyer) {
                None => {},
                Some(credentials) => {
                    let mut new_credentials: Vec<Credential<AccountIdOf<T>>> = Vec::new();

                    for credential in &credentials {
                        if credential.people_id != people_id {
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
        ) -> Credential<AccountIdOf<T>> {

            let new_credential = Credential::new(owner_id.clone(), people_id.clone(), platform.clone());

            CredentialByPeople::<T>::insert(&people_id, &new_credential);
            
            new_credential
        }
    }

    #[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
    pub struct Credential<AccountId> {
        owner_id: AccountId,
        people_id: Vec<u8>,
        platform: Vec<u8>
    }

    impl <AccountId> Credential <AccountId> {
        pub fn new(
            owner_id: AccountId,
            people_id: Vec<u8>,
            platform: Vec<u8>
        ) -> Self {
            Self {owner_id, people_id, platform}
        }
    }
}
