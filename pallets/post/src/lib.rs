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
		CreateNewPost(T::AccountId, Post<AccountIdOf<T>>),
    }

    #[pallet::error]
    pub enum Error<T> {
        PostNotBelongToYou,
        BalanceNotBelongToYou
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    pub type FreeBalanceOf = FreeBalance;
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    #[pallet::storage]
    #[pallet::getter(fn post_by_people)]
    pub(super) type PostByPeople<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<Post<AccountIdOf<T>>>>;

    #[pallet::storage] 
	#[pallet::getter(fn post_by_id)]
    pub(super) type PostById<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Post<AccountIdOf<T>>>;

    #[pallet::storage]
    #[pallet::getter(fn person_balance)]
    pub(super) type PersonBalance<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Person<AccountIdOf<T>>>;

    #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn insert_post(
            origin: OriginFor<T>, 
            post_id: Vec<u8>,
            people_id: Vec<u8>,
            platform: Vec<u8>,
            token_id: Vec<u8>,
            free: u32 
        ) -> DispatchResult {
			let creator = ensure_signed(origin)?;

            match Self::create_post(
                &creator, 
                &post_id, 
                &people_id, 
                &platform, 
                &token_id, 
                free
            ) {
                None => {
                    let flag = true;
                    ensure!(flag == false, Error::<T>::PostNotBelongToYou);
                }
                Some(new_post) => {
                    let new_people_post = Self::create_people_post(people_id.clone(), new_post.clone());
                    <PostById<T>>::insert(post_id.clone(), new_post.clone());
                    <PostByPeople<T>>::insert(people_id.clone(), new_people_post.clone());

                    
			        Self::deposit_event(Event::CreateNewPost(creator.clone(), new_post.clone()));
                }
            }

			Ok(())
		}

        #[pallet::weight(10_000)]
        pub fn insert_balance(
            origin: OriginFor<T>,
            people_id: Vec<u8>, 
            post_id: Vec<u8>,
            platform: Vec<u8>,
            token_id: Vec<u8>,
            free: u32, 
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;

            match Self::create_post(
                &creator, 
                &post_id, 
                &people_id, 
                &platform, 
                &token_id, 
                free
            ) {
                None => {ensure!(false, Error::<T>::PostNotBelongToYou)}
                Some(new_post) => {
                    <PostById<T>>::insert(post_id.clone(), new_post.clone());

			        Self::deposit_event(Event::CreateNewPost(creator.clone(), new_post.clone()));
                }
            }
            
            let person_balance: Person<AccountIdOf<T>>;

            match PersonBalance::<T>::get(people_id.clone()) {
                None => {
                    let mut balances:Vec<FreeBalance> = Vec::new();

                    let free_balance = FreeBalance {
                        free: free,
                        token_id: token_id.clone()
                    };

                    balances.push(free_balance);

                    person_balance = Person {
                        owner_id: creator.clone(),
                        people_id: people_id.clone(),
                        platform: platform.clone(),
                        balances
                    };
                },
                Some(person) => {
                    ensure!(person.people_id == people_id.clone(), Error::<T>::BalanceNotBelongToYou);
                    ensure!(person.platform == platform.clone(), Error::<T>::BalanceNotBelongToYou);

                    let mut filter_balance: Vec<FreeBalance> = person.balances
                        .iter()
                        .filter(|balance| balance.token_id != token_id.clone())
                        .cloned()
                        .collect(); 

                    let balance = Self::recalculate_balance(person.balances.clone(), token_id.clone(), free);
                    
                    filter_balance.push(balance);

                    person_balance = Person {
                        owner_id: creator.clone(),
                        people_id: people_id.clone(),
                        platform: platform.clone(),
                        balances: filter_balance
                    }
                }
            }

            <PersonBalance<T>>::insert(people_id, person_balance);
            
            Ok(())
        }
    }

    impl <T: Config> Pallet <T> {
        pub fn create_post(
            creator: &T::AccountId, 
            post_id: &Vec<u8>,
            people_id: &Vec<u8>,
            platform: &Vec<u8>,
            token_id: &Vec<u8>,
            free: u32 
        ) -> Option<Post<AccountIdOf<T>>> {
            let new_post: Post<AccountIdOf<T>>;

            match PostById::<T>::get(post_id.clone()) {
                None => {
                    let mut balances:Vec<FreeBalance> = Vec::new();

                    let free_balance = FreeBalance {
                        free: free,
                        token_id: token_id.clone()
                    };

                    balances.push(free_balance);

                    new_post = Post {
                        owner_id: creator.clone(),
                        post_id: post_id.clone(),
                        people_id: people_id.clone(),
                        platform: platform.clone(),
                        balances
                    };
                },
                Some(post) => {
                    if post.people_id != people_id.clone() {
                        return None;
                    }

                    if post.platform != platform.clone() {
                        return None;
                    }

                    let mut filter_balance: Vec<FreeBalance> = post.balances
                        .iter()
                        .filter(|balance| balance.token_id != token_id.clone())
                        .cloned()
                        .collect(); 

                    let balance = Self::recalculate_balance(post.balances.clone(), token_id.clone(), free);
                    
                    filter_balance.push(balance);

                    new_post = Post {
                        owner_id: creator.clone(),
                        post_id: post_id.clone(),
                        people_id: people_id.clone(),
                        platform: platform.clone(),
                        balances: filter_balance
                    }
                }
            }

            Some(new_post)
        }

        pub fn create_people_post(
            people_id: Vec<u8>, 
            new_post: Post<AccountIdOf<T>>
        ) -> Vec<Post<AccountIdOf<T>>> {
            let mut filter_posts: Vec<Post<AccountIdOf<T>>> = Vec::new();

            match PostByPeople::<T>::get(people_id.clone()) {
                None => {
                    filter_posts.push(new_post);
                },
                Some(posts) => {
                    filter_posts = posts
                        .iter()
                        .filter(|post| post.post_id != new_post.post_id.clone())
                        .cloned()
                        .collect();

                    filter_posts.push(new_post);
                }
            }

            filter_posts
        }

        pub fn recalculate_balance(balances: Vec<FreeBalance>, token_id: Vec<u8>, free: u32) -> FreeBalance {
            let mut free_balance = FreeBalance {
                token_id: token_id.clone(),
                free: free
            };

            for (_, e) in balances.iter().enumerate() {
                if e.token_id == token_id.clone() {
                    free_balance = FreeBalance {
                        token_id: token_id.clone(),
                        free: e.free + free
                    };
                    
                    break;
                }
            }

            free_balance
        }
    }

    #[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
    pub struct Person<AccountId> {
        owner_id: AccountId,
        people_id: Vec<u8>,
        platform: Vec<u8>,
        balances: Vec<FreeBalance>
    }

    #[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
    pub struct FreeBalance {
        free: u32,
        token_id: Vec<u8>
    }

    #[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
    pub struct Post <AccountId> {
        owner_id: AccountId,
        post_id: Vec<u8>,
        people_id: Vec<u8>,
        platform: Vec<u8>,
        balances: Vec<FreeBalance>
    }
}
