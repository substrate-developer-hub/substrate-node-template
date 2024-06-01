#![cfg_attr(not(feature = "std"), no_std)]

pub use codec::{Decode, Encode};
pub use common::BoundedString;
pub use frame_support::pallet_prelude::Get;
pub use pallet::*;
pub use sp_core::{H256, blake2_256};
pub use sp_std::collections::btree_set::BTreeSet;

// This module contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
#[cfg(test)]
mod tests;

/// Global data structures
// Project Validator / Project Owner data structure
#[derive(Encode, Decode, Default, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
#[scale_info(skip_type_params(IPFSLength))]
pub struct PVoPOInfo<IPFSLength: Get<u32>, BlockNumber> {
	// IPFS link to PV/PO documentation
	documentation_ipfs: BoundedString<IPFSLength>,
	// Penalty level
	penalty_level: u8,
	// Penalty timeout
	penalty_timeout: BlockNumber,
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
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, scale_info::TypeInfo)]
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

// Projects info structure
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
#[scale_info(skip_type_params(IPFSLength))]
pub struct ProjectInfo<IPFSLength: Get<u32>, MomentOf, BlockNumber> {
	// IPFS link to CFA documentation
	documentation_ipfs: BoundedString<IPFSLength>,
	// Creation date
	creation_date: MomentOf,
	// Penalty level
	penalty_level: u8,
	// Penalty timeout
	penalty_timeout: BlockNumber,
}

// Penalty level structure for carbon footprint
#[derive(Encode, Decode, Default, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PenaltyLevelConfig {
	pub level: u8,
	pub base: i32, // Balance
}

// Vote type enum
#[derive(Encode, Decode, PartialEq, Eq, scale_info::TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum VoteType {
	CdrVote,
	ProposalVote,
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
	type BlockNumber<T> = BlockNumberFor<T>;

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
	pub(super) type CarbonFootprintAccounts<T: Config> =
		StorageMap<_, Identity, AccountIdOf<T>, CFAInfo<MomentOf<T>, T::IPFSLength>, OptionQuery>;

	// Trader accounts
	#[pallet::storage]
	#[pallet::getter(fn trader_accounts)]
	pub type TraderAccounts<T: Config> =
		StorageValue<_, BTreeSet<AccountIdOf<T>>, ValueQuery, DefaultForTraderAccounts<T>>;

	// Project Validator accounts
	#[pallet::storage]
	#[pallet::getter(fn project_validators)]
	pub(super) type ProjectValidators<T: Config> =
		StorageMap<_, Identity, AccountIdOf<T>, PVoPOInfo<T::IPFSLength, BlockNumber<T>>, OptionQuery>;

	// Project Owner accounts
	#[pallet::storage]
	#[pallet::getter(fn project_owners)]
	pub(super) type ProjectOwners<T: Config> =
		StorageMap<_, Identity, AccountIdOf<T>, PVoPOInfo<T::IPFSLength, BlockNumber<T>>, OptionQuery>;

	// Penalty timeouts
	#[pallet::storage]
	#[pallet::getter(fn penalty_timeouts)]
	pub(super) type PenaltyTimeouts<T: Config> =
		StorageMap<_, Identity, BlockNumber<T>, BTreeSet<AccountIdOf<T>>, OptionQuery>;

	// Carbon deficit reports
	#[pallet::storage]
	#[pallet::getter(fn carbon_deficit_reports)]
	pub(super) type CarbonDeficitReports<T: Config> =
		StorageMap<_, Identity, BoundedString<T::IPFSLength>, CFReportInfo<AccountIdOf<T>, MomentOf<T>>, OptionQuery>;

	// Projects proposals
	#[pallet::storage]
	#[pallet::getter(fn project_proposals)]
	pub(super) type ProjectProposals<T: Config> =
		StorageMap<_, Identity, BoundedString<T::IPFSLength>, PProposalInfo<AccountIdOf<T>, MomentOf<T>>, OptionQuery>;

	// Projects
	#[pallet::storage]
	#[pallet::getter(fn projects)]
	pub(super) type Projects<T: Config> =
		StorageMap<_, Identity, H256, ProjectInfo<T::IPFSLength, MomentOf<T>, BlockNumber<T>>, OptionQuery>;
 
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Successful Vote
		SuccessfulVote(AccountIdOf<T>, BoundedString<T::IPFSLength>),
		/// Successful Project Proposal
		ProjectProposalCreated(AccountIdOf<T>, BoundedString<T::IPFSLength>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Report not found
		ReportNotFound,
		/// Not Authorized
		NotAuthorized,
		/// Vote already submitted
		VoteAlreadySubmitted,
		/// Project proposal already exists
		ProjectProposalAlreadyExists,
		/// Project Proposal not found
		ProjectProposalNotFound,
		/// Wrong vote type
		WrongVoteType,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> { 
		// Vote for/against Carbon Deficit Reports or for/against project Proposals
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn cast_vote(
			origin: OriginFor<T>,
			vote_type: VoteType,
			ipfs: BoundedString<T::IPFSLength>,
			vote: bool,
		) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;

			// Check if caller is Project Validator account
			ensure!(ProjectValidators::<T>::contains_key(user.clone()), Error::<T>::NotAuthorized);

			match vote_type {
				VoteType::CdrVote => {
					// Get report info and return error if it does not exist
					let mut report =
						CarbonDeficitReports::<T>::get(ipfs.clone()).ok_or(Error::<T>::ReportNotFound)?;

					// Check if vote already exists
					ensure!(
						!report.votes_for.contains(&user) && !report.votes_against.contains(&user),
						Error::<T>::VoteAlreadySubmitted
					);

					if vote {
						report.votes_for.insert(user.clone());
					} else {
						report.votes_against.insert(user.clone());
					};

					CarbonDeficitReports::<T>::insert(ipfs.clone(), report);
				},
				VoteType::ProposalVote => {
					// Get report info or return error if it does not exist
					let mut report = ProjectProposals::<T>::get(ipfs.clone())
						.ok_or(Error::<T>::ProjectProposalNotFound)?;

					// Check if vote already exists
					ensure!(
						!report.votes_for.contains(&user) && !report.votes_against.contains(&user),
						Error::<T>::VoteAlreadySubmitted
					);

					if vote {
						report.votes_for.insert(user.clone());
					} else {
						report.votes_against.insert(user.clone());
					};

					ProjectProposals::<T>::insert(ipfs.clone(), report);
				}
			}

			Self::deposit_event(Event::SuccessfulVote(user.clone(), ipfs.clone()));

			Ok(().into())
		}

		// Propose project
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn propose_project(origin: OriginFor<T>, ipfs: BoundedString<T::IPFSLength>) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;

			// Check if caller is Project Owner account
			ensure!(ProjectOwners::<T>::contains_key(user.clone()), Error::<T>::NotAuthorized);

			// Ensure project does not exist
			ensure!(
				!ProjectProposals::<T>::contains_key(ipfs.clone()),
				Error::<T>::ProjectProposalAlreadyExists
			);

			// Get time
			let creation_date = T::Time::now();

			// Create project hash
			let nonce = frame_system::Pallet::<T>::account_nonce(&user);
            let encoded: [u8; 32] = (&user, nonce).using_encoded(blake2_256);
            let project_hash = H256::from(encoded);

			// Project Proposal info
			let project_proposal_info = PProposalInfo {
				project_owner: user.clone(),
				creation_date,
				project_hash,
				votes_for: BTreeSet::<AccountIdOf<T>>::new(),
				votes_against: BTreeSet::<AccountIdOf<T>>::new(),
			};

			// Write to a storage
			ProjectProposals::<T>::insert(ipfs.clone(), project_proposal_info);

			Self::deposit_event(Event::ProjectProposalCreated(user.clone(), ipfs));

			Ok(().into())
		}
	}
}
