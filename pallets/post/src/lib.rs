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
        PostCreated(T::AccountId, Post<AccountIdOf<T>>),
    }

    #[pallet::error]
    pub enum Error<T> {
        SomethingError
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type BalanceOf = Balance;

    #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::storage]
    #[pallet::getter(fn post_by_people)]
    pub(super) type PostsByPeople<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<Post<AccountIdOf<T>>>>;

    #[pallet::storage]
    #[pallet::getter(fn post_by_id)]
    pub(super) type PostById<T: Config> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Post<AccountIdOf<T>>>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn add_post(
            origin: OriginFor<T>, 
            people_id: Vec<u8>, 
            platform: Vec<u8>, 
            token_id: Vec<u8>, 
            balance: u32
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;            
            let created_post = Self::create_post(
                &creator, 
                &people_id, 
                &platform, 
                &balance, 
                &token_id
            );

            Self::create_people_post(&created_post);

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn create_post(
            post_id: &T::AccountId,
            people_id: &Vec<u8>,
            platform: &Vec<u8>,
            balance: &u32,
            token_id: &Vec<u8>,
        ) -> Post<AccountIdOf<T>> {
            let new_post: Post<AccountIdOf<T>>;
            
            match PostById::<T>::get(&post_id) {
                None => {
                    new_post = Post::new(
                        post_id.clone(),
                        people_id.clone(), 
                        platform.clone(), 
                        token_id.clone(),
                        balance.clone()
                    );
                },
                Some(post) => {
                    let current_balance: u32;
                    if  post.token_id == token_id.clone() {
                        let previous_balance = post.balance.balance;
                        current_balance = balance.clone() + previous_balance;
                    } else {
                        current_balance = post.balance.balance;
                    }

                    new_post = Post::new(
                        post_id.clone(),
                        post.people_id,
                        post.platform,
                        token_id.clone(),
                        current_balance
                    );
                }
            }

            PostById::<T>::insert(post_id, &new_post);

            Self::deposit_event(Event::PostCreated(post_id.clone(), new_post.clone()));

            new_post
        }

        pub fn create_people_post(post: &Post<AccountIdOf<T>>) {
            let mut new_posts: Vec<Post<AccountIdOf<T>>> = Vec::new(); 
            let posts = Self::post_by_people(&post.people_id).unwrap_or(Vec::new());
            
            for pst in posts {
                if pst.post_id == post.post_id {
                    new_posts.push(post.clone());
                } else {
                    new_posts.push(pst);
                }
            }

            PostsByPeople::<T>::insert(&post.people_id, new_posts);
        }
    }

    #[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
    pub struct Post<AccountId> {
        post_id: AccountId,
        people_id: Vec<u8>,
        platform: Vec<u8>,
        token_id: Vec<u8>,
        balance: Balance
    }

    #[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
    pub struct Balance {
        balance: u32
    }

    impl <AccountId>Post<AccountId> {
        pub fn new(
            post_id: AccountId,
            people_id: Vec<u8>,
            platform: Vec<u8>,
            token_id: Vec<u8>,
            balance: u32
        ) -> Self {
            Self {post_id, people_id, platform, token_id, balance: Balance {
                balance
            }}
        }
    }

}