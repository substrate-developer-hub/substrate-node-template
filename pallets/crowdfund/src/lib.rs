#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[derive(Encode, Decode, Default, PartialEq, Eq)]
#[cfg_attr(feaure = "std", derive(Debug))]
pub struct FundInfo<AccountId, Balance, BlockNumber> { 
	//	Receive the funds if the funding round is successful 
	beneficiary: AccountId, 
	// deposited funds 
	deposit: Balance,
	// amount raised by the fund
	raised: Balance,
	// blocknumber that the funding stopped
	end: BlockNumber,
	// TARGET price 
	goal: Balance,

}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, ensure, storage::child,
		traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons}, 
	sp_runtime::{traits::{AccountIdConversion, ArithmeticERror, Saturating, Zero, Hash, AtLeast32BitUnsigned}, ModuleId}

	};
	use sp_std::prelude::*;
	use frame_system::{pallet_prelude::*, ensure_signed};

	use super::*;

	//	Unique identifier for module => used to generate new accounts(fund) for each token
	const PALLET_ID: ModuleId = ModuleId(*b"ex/cfund");

	//	Placeholder we use to implement traits and methods 
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//	This will represent the currency denominated during the crowdfund 
		type Currency: ReservableCurrency<Self::AccountId>;
		//	Get MinContribution: Minimum amount that may be contributed to the crowdfund  
		type MinContribution = Get<BalanceOf<Self>>;
		//	The amount to be held by the owner of a crowdfund 
		type SubmissionDeposit: Get<BalanceOf<Self>>;
		//	The amoutn of time in blocks after an unsuccessful crowdfund ending during which contributors are able 
		//	withdraw their funds. After this period, their funds are lost 
		type RetirementPeriod: Get<Self::BlockNumber>;
	}
	//	Indentify a fund 
	pub type FundIndex = u32;
	//	Type to call Account Id
	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	// 	Currency -> AccountId -> Balance
	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	//	AccountId, BalanceOf, Blocknumber
	type FundInfoOf<T> = FundInfo<AcountIdOf<T>, BalanceOf<T>, BlockNumber>; 
	//	Type to call Blocknumber 
	type BlockNumber<T> = <T as frame_system::Config>::BlockNumber;
	
	//	Store Fund Info using Fund Index 
	#[pallet::storage]
	#[pallet::getter(fn fund)]
	pub type Fund<T> = StorageMap<
		_, 
		Blake2_128Concat,
		FundIndex,
		FundInfoOf<T>,
		OptionQuery,
		>;

	//	Storing FundIndex Values
	#[pallet::storage]
	#[pallet::getter(fn fund_count)]
	pub type FundCount<T> = StorageValue<
		_, 
		FundIndex, 
		ValueQuery
		>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(BalanceOf<T> = "Balance", AccountIdOf<T> = "Account_Id", BlockNumber<T> = "BlockNumber")]
	pub enum Event<T: Config> {
		Created(FundIndex, BlockNumber),
		Contributed(AccountIdOf<T>, FundIndex, BalanceOf<T>, BlockNumber),
		Withdrew(AccountIdOf<T>, FundIndex, BalanceOf<T>, BlockNumber),
		Retiring(FundIndex, BlockNumber),
		Dissolved(AccountIdOf<T>, FundIndex, BlockNumber),
		Dispensed(AccountIdOf<T>, FundIndex, BlockNumber),
	}

	#[pallet::error]
	pub enum Error<T> {
		EndTooEarly, 
		ContributionTooSmall,
		InvalidIndex, 
		ContributionPeriodOver, 
		FundStillActive,
		NoContributionAvailable,
		FundNotRetired,
		Unsuccessful,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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
		pub fn create(
			origin: OriginFor<T>, 
			beneficiary: AccountIdOf<T>, 
			goal: BalanceOf<T>,
			end: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;	

			// Get blocknumber now 
			let now = <frame_system::Pallet<T>>::block_number();
				ensure!(now < end, Error::<T>::EndTooEarly);
			// Get Deposit
			let deposit = T::SubmissionDeposit::get();
			// Internal transfer of deposit to beneficiary 
			let imb = T::Currency::withdraw(
				&sender, 
				&deposit, 
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?;
			// Generate Fund Index 
			FundCount::<T>::try_mutate(|next_id| -> DispatchResult { 
				let current_id = *next_id;
				*next_id = next_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

				FundCount::<T>::insert(&current_id);
				//	resolve_creating: only accepts a NegativeImbalance and returns nothing on success
				//	No fees are paid here if we need to generate this new account; that's why we don't just transfer 
				T::Currency::resolve_creating(&Self::fund_account_id(index), imb);
				
				//	update fund info 
				<Funds<T>>::insert(index, FundInfo { 
					beneficiary, 
					deposit, 
					raised: Zero::zero(), 
					end, 
					goal,
				});

				Ok(())
			});
			// emit event 
			Self::deposit_event(Event::Created(current_id, now));
			//	DispatchResultWithPostInfo
			Ok(().into())
		}
		#[pallet::weight(10_000, DispatchClass::Operational)]
		pub fn contribute(
			origin: OriginFor<T>, 
			index: FundIndex,
			value: BalanceOf<T>,
			) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?;
			
			//	Ensure contributed value is greater than the MinimumContribution
			ensure!(value >= MinContribution::get(), Error::<T>::ContributionTooSmall);
			
			//	Get the fund index stored in the storagemap
			let mut fund = Self::fund(index).ok_or(Error::<T>::InvalidIndex);

			//	Make sure the crowdfund hasnt ended 
			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(fund.end > now, Error::<T>::ContributionPeriodOver);

			// Transfer deposit to storage map 
			T::Currency::transfer(
				&sender,
				&Self::fund_account_id(index),
				value,
				ExistenceRequirement::AllowDeath
			)?;

			//	Increase fund value 
			fund.raised += value;
			
			// update Fund Storagemap = [FundIndex, FundInfo]
			Fund::<T>::insert(index, &fund);

			//	Get the users contribtion to the fund fund 
			let balance = Self::contribution_get(index, &who);
			//	Add the users inputted value
			let balance = balance.saturating_add(value);

			//	COntribution put into the specified fund index 
			Self::contribution_put(index, &who, &balance);

			//	Declare event 
			Self::deposit_event(Event::Contributed(sender, index, balance, now));

			Ok(().into())
		}
		#[pallet::weight(10_000, DispatchClass::Normal)]
		fn withdraw(
			origin: OriginFor<T>,
			index: FundIndex,
		) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?;

			//	Find the fund index from the storage 
			let fund = Self::fund(index).ok_or(Error::<T>::InvalidIndex);
			
			//	Check if the fund is still active before withdrawing 
			let now = <frame_system::Pallet<T>>::block_number();
			
			//	You can only withdraw as soon as the crowd funding has ended 
			ensure!(fund.end < now, Error::<T>::FundStillActive);

			//	Get the users contribution to the crowd fund [index, ]
			let balance = Self::contribution_get(index, &sender);
			ensure!(balance > Zero::zero(), Error::<T>::NoContributionAvailable);

			//	Transfer sender's contribution without charging any transaction fees from the sender's balance 
			//	Resolve_into_existing: Accepts a NegativeImbalance and returns nothing on success 
			let _ = T::Currency::resolve_into_existing(&sender, 
				//	Self::NegativeImbalanacne -> Token required
				T::Currency::withdraw(
					&Self::fund_account_id(index),
					balance,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
			)?);

			//	Call function to remove contribution associated with the user and fund index 
			Self::contribution_kill(index, &sender);

			//	Take away the senders contribution to the crowdfund 
			fund.raised = fund.raised.saturating_sub(balance);
			
			//	Store this new balance to the storage 
			<Funds<T>>::insert(index, &fund);
			
			//	emit event 
			Self::deposit_event(Event::Withdrew(sender, fund, balance, now));
			Ok(().into())

		}
		//	Dissolve an entire crowdfund after its retirement period has expired
		//	Anyone can call this function, and they they incentivised to do because they inherit the deposit
		#[pallet::weight(10_000, DispatchClass::Normal, Pays::No)]
		fn dissolve(
			origin: OriginFor<T>, 
			index: FundIndex
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let fund = Self::fund(index).ok_or(Error::<T>::InvalidIndex);
			let now = <frame_system::Pallet<T>>::block_number();

			//	Check that enough time has passed to remove form storage 
			ensure!(now >= fund.end + T::RetirementPeriod::get(), Error::<T>::FundNotRetired);

			//	Access the fund accunt id 
			let account = Self::fund_account_id(index);

			//	Dissolver receives the deposit and any remaining funds
			//	Sender inherits the deposit and doesn't receive any transaction fee 
			let _ = T::Currency::resolve_creating(&sender, T::Currency::withdraw(
				//	Withdraw from fund account id 
				&account,
				//	Withdraw all funds inside the fund account 
				fund.deposit + fund.raised,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,  
			)?);

			//	Update the new balance of this fund account in the storage map 
			//	<Funds<T>>::insert(index, &fund);

			//	Remove the fundIndex from the stprage 
			<Fund<T>>::remove(index);

			//	Kill storage for this particular fund 
			Self::crowdfund_kill(index);
			
			//	emit event 
			Self::deposit_event(Event::Dissoved(sender, index, now));
			//	Return actual weight consumed 
			Ok(Some(10_000).into())
		}
		
		//	Dispense a payment to the beneficiary of a successful crowdfund 
		//	the beneficiary receives the contributed funds and the caller receives the 
		//	deposit as a reward to incentivise clearing settled crowfunds out of storage 

		//	Dispatch class is set to Normal and the user does not pay weigh 
		#[pallet::weight(10_000, DipatchClass::Normal, Pays::No)]
		pub fn dispense(
			origin: OriginFor<T>,
			index: FundIndex
		) -> DispatchResultWithPostInfo {
			let sender = ensure_origin(index);

			let account = Self::fund_account_id(index);
			let fund = Self::fund(index).ok_or(Error::<T>::InvalidIndex);
			let now = <frame_system::Pallet<T>>::block_number();

			let _ = T::Currency::resolve_creating(&sender, T::Currency::withdraw( 
				todo!
			));
			

			Ok(().into())
		}

	}

	impl<T: Config> Pallet<T> { 
		//	The Account ID of the fund pot 
		//	We create a fund for each token, which is ultimately controlled and managed by our pallet 
		pub fn fund_account_id(index: FundIndex) -> T::AccountId { 
			PALLET_ID.into_sub_account(index);
			//	WE use Into_sub_account(index): we use the unique module id we create 
			//	to generate any numbetr of unique Accountids whcih represents the funds for each of the tokens
		}
		//	Function to find the id associated with the fund id (child trie)
		//	Each fund stores information about it ***contributors and their ***contributions in a child trie 
		
		//	This helper function calculates the id of the associate child trie 
		pub fn id_from_index(index: FundIndex) -> child::ChildInfo { 
			let mut buf = Vec::new();

			//	Append and elements of b"crowdfind" into the vector 
			buf.extend_from_slice(b"crowdfind");
	
			//	create a memory representation of account id (FUndIndex) then append this in the bug Vec 
			//	0x0000000 --> [0x12, 0x90, .. etc]
			buf.extend_from_slice(&index.to_le_bytes()[..]);

			//	store the following mem representaion + byte representaion crowdfind as a ** hash ** into a child trie 
			//https://docs.substrate.io/rustdocs/latest/frame_support/storage/child/enum.ChildInfo.html
			child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
		}

		//	Record a contribution in the associated child trie 
		pub fn contribution_put(who: &T::AccountId, index: FundIndex, balance: &BalanceOf<T>) { 
			//	Childinfo type: fund's Account Id 
			let id = Self::id_from_index(index);

			//	Turn 'who' into a slice and using this as a key to the contribution to the child trie 
			//	id -> access child trie root 
			//	[id, key, value] => get child root and access key to value db
			who.using_encoded(|key| child::put(&id, key, &balance));
		}
		//	Remove contribution in the associated child trie 
		pub fn contribution_kill(index: FundIndex, who: &T::AccountId) { 
			//
			let id = Self::id_from_index(index);	

			//	No need to check if the key has no value in it since the function is only triggered if the preceding fucntion is maintained properly
			who.using_encoded(|key| child::kill(&index, key));
		}
		//	Lookup a contribution in the associated child trie
		pub fn contribution_get(index: FundIndex, who: &T::AccountId) -> BalanceOf<T> { 
			//	Fund's Account Id 
			let id = Self::id_from_index(index);
			
			//	the Access child-trie using the fund's id and user's account id => return value as BalanceOf<T> (by defualt)
			//	get_or_default: return the value of the item in storage under key or the type's default if there is no explicit entry 
			who.using_encoded(|key| child::get_or_default::<BalanceOf<T>>(&id, key))
		}
		//	Remove the entire record of contributions in the associated child trie in a single storage write 
		pub fn crowdfund_kill(index: FundIndex) { 
			let id = Self::id_from_index(index);
			
			//	Some(Option<u32>) => Deletes all keys from the overlay and upto limit keys from the backend (u32)
			//	If set to None, No limit is applied
			child::kill_storage(&id, None)
		}

	
	}
}

