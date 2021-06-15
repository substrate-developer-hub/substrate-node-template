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
        TokenCreated(Token)
    }

    #[pallet::error]
    pub enum Error<T> {
        TokenAlreadyExist
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn token_by_rpc)]
    pub(super) type TokenByRPC<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Token>;

    #[pallet::storage]
    #[pallet::getter(fn tokens)]
    pub(super) type Tokens<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<Token>>;

    #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub fn add_token(
            origin: OriginFor<T>, 
            token_id: Vec<u8>,
            token_name: Vec<u8>,
            token_decimal: Vec<u8>,
            token_address_format: Vec<u8>,
            token_rpc_address: Vec<u8>
        ) -> DispatchResult {
            let _creator = ensure_signed(origin)?;
            let new_token = Token {
                token_id: token_id.clone(), 
                token_name: token_name.clone(), 
                token_decimal: token_decimal.clone(), 
                token_address_format: token_address_format.clone(), 
                token_rpc_address: token_rpc_address.clone()
            };

            let mut flag = true;

            match TokenByRPC::<T>::get(token_rpc_address.clone()) {
                None => {
                    flag = false
                },
                Some(_) => {}
            }

            ensure!(flag == false, Error::<T>::TokenAlreadyExist);

            let mut tokens = Self::tokens("tokens".as_bytes()).unwrap_or(Vec::new());

            tokens.push(new_token.clone());

            Tokens::<T>::insert("tokens".as_bytes(), tokens);

            TokenByRPC::<T>::insert(token_rpc_address.clone(), new_token.clone());

            Self::deposit_event(Event::TokenCreated(new_token));

            Ok(().into())
        }
    }

    #[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
    pub struct Token {
        token_id: Vec<u8>,
        token_name: Vec<u8>,
        token_decimal: Vec<u8>,
        token_address_format: Vec<u8>,
        token_rpc_address: Vec<u8>,
    }
}
