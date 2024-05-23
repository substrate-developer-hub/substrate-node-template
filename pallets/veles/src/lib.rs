#![cfg_attr(not(feature = "std"), no_std)]

pub use codec::{Decode, Encode};
pub use common::BoundedString;
pub use frame_support::pallet_prelude::Get;
pub use pallet::*;
pub use sp_core::H256;
pub use sp_std::collections::btree_set::BTreeSet;

/// Global data structures
// Project Validator / Project Owner data structure
#[derive(Encode, Decode, Default, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
#[scale_info(skip_type_params(IPFSLength))]
pub struct PVoPOInfo<MomentOf, IPFSLength: Get<u32>> {
	// IPFS link to PV/PO documentation
	documentation_ipfs: BoundedString<IPFSLength>,
	// Penalty level
	penalty_level: u8,
	// Penalty timeout
	penalty_timeout: MomentOf,
}

// Carbon Footprint account data structure
#[derive(Encode, Decode, Default, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
#[scale_info(skip_type_params(IPFSLength))]
pub struct CFAInfo<MomentOf, IPFSLength: Get<u32>> {
	// IPFS link to CFA documentation
	documentation_ipfs: BoundedString<IPFSLength>,
	// Carbon credit balance
	carbon_credit_balance: i128,
	// Creation date
	creation_date: MomentOf,
}

// Carbon Footprint report data structure
#[derive(Encode, Decode, Default, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
#[scale_info(skip_type_params(IPFSLength))]
pub struct CFReportInfo<AccountIdOf, MomentOf> {
	// Account
	account_id: AccountIdOf,
	// Creation date
	creation_date: MomentOf,
	// Carbon deficit (aka Carbon footprint)
	carbon_deficit: i128,
	// Votes for
	votes_for: BTreeSet<AccountIdOf>,
	// Votes against
	votes_against: BTreeSet<AccountIdOf>,
}

// Project Proposal info structure
#[derive(Encode, Decode, Default, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PProposalInfo<AccountIdOf, MomentOf> {
	 // Project Owner
	 project_owner: AccountIdOf,
     // Creation date
	 creation_date: MomentOf,
	 // Project hash 
	 project_hash: H256,
	 // Votes for
	 votes_for: BTreeSet<AccountIdOf>,
	 // Votes against
	 votes_against: BTreeSet<AccountIdOf>,
}

// Penalty level structure for carbon footprint
#[derive(Encode, Decode, Default, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PenaltyLevelConfig {
	pub level: u8,
	pub base: i32, // Balance
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Time;
	use frame_system::pallet_prelude::*;
	// use hex_literal::hex;
	use sp_std::collections::btree_set::BTreeSet;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Pallet configuration
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type IPFSLength: Get<u32>;
		type CarboCreditDecimal: Get<u8>;
		type Time: Time;

		#[pallet::constant]
		type PenaltyLevelsConfiguration: Get<[PenaltyLevelConfig; 5]>;
	}

	/// Pallet types and constants
	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type MomentOf<T> = <<T as Config>::Time as Time>::Moment;
	type IPFSHash = H256;

	/// Helper functions
	// Default authority accounts
	#[pallet::type_value]
	pub fn DefaultForAuthorityAccounts<T: Config>() -> BTreeSet<AccountIdOf<T>> {
		let set: BTreeSet<AccountIdOf<T>> = BTreeSet::<AccountIdOf<T>>::new();

		// let bytes_1 = hex!("fed77d0df3f5068d8a875e5ae7c3248ba7c602439623cab507206af8e50edd4b");
		// let account_1 = AccountIdOf::<T>::decode(&mut &bytes_1[..]).unwrap();
		// set.insert(account_1);

		set
	}

	// Default trade accounts
	#[pallet::type_value]
	pub fn DefaultForTraderAccounts<T: Config>() -> BTreeSet<AccountIdOf<T>> {
		let set: BTreeSet<AccountIdOf<T>> = BTreeSet::<AccountIdOf<T>>::new();

		set
	}

	/// Pallet storages
	// Authority accounts
	#[pallet::storage]
	#[pallet::getter(fn authority_accounts)]
	pub type AuthorityAccounts<T: Config> =
		StorageValue<_, BTreeSet<AccountIdOf<T>>, ValueQuery, DefaultForAuthorityAccounts<T>>;

	// Carbon Footprint accounts
	#[pallet::storage]
	#[pallet::getter(fn carbon_footprint_accounts)]
	pub(super) type CarbonFootprintAccounts<T: Config> = StorageMap<
		_,
		Identity,
		AccountIdOf<T>,
		CFAInfo<MomentOf<T>, T::IPFSLength>,
		OptionQuery,
	>;

	// Trader accounts
	#[pallet::storage]
	#[pallet::getter(fn trader_accounts)]
	pub type TraderAccounts<T: Config> =
		StorageValue<_, BTreeSet<AccountIdOf<T>>, ValueQuery, DefaultForTraderAccounts<T>>;

	// Project Validator accounts
	#[pallet::storage]
	#[pallet::getter(fn project_validators)]
	pub(super) type ProjectValidators<T: Config> = StorageMap<
		_,
		Identity,
		AccountIdOf<T>,
		PVoPOInfo<MomentOf<T>, T::IPFSLength>,
		OptionQuery,
	>;

	// Project Owner accounts
	#[pallet::storage]
	#[pallet::getter(fn project_owners)]
	pub(super) type ProjectOwners<T: Config> = StorageMap<
		_,
		Identity,
		AccountIdOf<T>,
		PVoPOInfo<MomentOf<T>, T::IPFSLength>,
		OptionQuery,
	>;

	// Penalty timeouts
	#[pallet::storage]
	#[pallet::getter(fn penalty_timeouts)]
	pub(super) type PenaltyTimeouts<T: Config> =
		StorageMap<_, Identity, MomentOf<T>, BTreeSet<AccountIdOf<T>>, OptionQuery>;

	// Carbon deficit reports
	#[pallet::storage]
	#[pallet::getter(fn carbon_deficit_reports)]
	pub(super) type CarbonDeficitReports<T: Config> = StorageMap<
		_,
		Identity,
		IPFSHash,
		CFReportInfo<AccountIdOf<T>, MomentOf<T>>,
		OptionQuery,
	>;

	// Projects proposals
	#[pallet::storage]
    #[pallet::getter(fn project_proposals)]
    pub(super) type ProjectProposals<T: Config> = StorageMap<
        _,
        Identity,
		IPFSHash, 
		PProposalInfo<AccountIdOf<T>, MomentOf<T>>,
        OptionQuery,
    >;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Successful Vote
		SuccessfulVote(AccountIdOf<T>, IPFSHash),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Report not found
		ReportNotFound,
		/// Not Authorized
		NotAuthorized,
		/// Vote already submitted
		VoteAlreadySubmitted,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Vote for/against Carbon Deficit Reports
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn cdr_vote(
			origin: OriginFor<T>,
			ipfs: IPFSHash,
			vote: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if caller is Project Validator account
			ensure!(ProjectValidators::<T>::contains_key(who.clone()), Error::<T>::NotAuthorized);

			// Check if report exists
			ensure!(CarbonDeficitReports::<T>::contains_key(ipfs), Error::<T>::ReportNotFound);

			// Get report info
			let report = CarbonDeficitReports::<T>::get(ipfs);

			// If report_info exists submit vote
			if report.is_some() {
				let mut report_info = report.unwrap();
				// Check if vote already exists
				ensure!(
					!report_info.votes_for.contains(&who)
						&& !report_info.votes_against.contains(&who),
					Error::<T>::VoteAlreadySubmitted
				);

				if vote {
					report_info.votes_for.insert(who.clone());
				} else {
					report_info.votes_against.insert(who.clone());
				};

				// Write to a storage
				CarbonDeficitReports::<T>::insert(ipfs, report_info);

				Self::deposit_event(Event::SuccessfulVote(who.clone(), ipfs));
			} else {
				return Err(Error::<T>::ReportNotFound.into());
			}

			Ok(().into())
		}
	}
}
