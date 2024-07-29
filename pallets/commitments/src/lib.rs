#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
#[cfg(test)]
mod tests;

pub mod types;
pub mod weights;

pub use pallet::*;
use subtensor_macros::freeze_struct;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::traits::Currency;
use sp_runtime::{traits::Zero, Saturating};
use sp_std::boxed::Box;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
#[deny(missing_docs)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
    use frame_system::pallet_prelude::{BlockNumberFor, *};

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency type that will be used to place deposits on neurons
        type Currency: ReservableCurrency<Self::AccountId> + Send + Sync;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Interface to access-limit metadata commitments
        type CanCommit: CanCommit<Self::AccountId>;

        /// The maximum number of additional fields that can be added to a commitment
        #[pallet::constant]
        type MaxFields: Get<u32>;

        /// The amount held on deposit for a registered identity
        #[pallet::constant]
        type InitialDeposit: Get<BalanceOf<Self>>;

        /// The amount held on deposit per additional field for a registered identity.
        #[pallet::constant]
        type FieldDeposit: Get<BalanceOf<Self>>;

        /// The rate limit for commitments
        #[pallet::constant]
        type RateLimit: Get<BlockNumberFor<Self>>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A commitment was set
        Commitment {
            /// The netuid of the commitment
            netuid: u16,
            /// The account
            who: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account passed too many additional fields to their commitment
        TooManyFieldsInCommitmentInfo,
        /// Account is not allow to make commitments to the chain
        AccountNotAllowedCommit,
        /// Account is trying to commit data too fast, rate limit exceeded
        CommitmentSetRateLimitExceeded,
    }

    /// Identity data by account
    #[pallet::storage]
    #[pallet::getter(fn commitment_of)]
    pub(super) type CommitmentOf<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Twox64Concat,
        T::AccountId,
        Registration<BalanceOf<T>, T::MaxFields, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn last_commitment)]
    pub(super) type LastCommitment<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Twox64Concat,
        T::AccountId,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the commitment for a given netuid
        #[pallet::call_index(0)]
        #[pallet::weight((
			T::WeightInfo::set_commitment(),
			DispatchClass::Operational,
			Pays::No
		))]
        pub fn set_commitment(
            origin: OriginFor<T>,
            netuid: u16,
            info: Box<CommitmentInfo<T::MaxFields>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::CanCommit::can_commit(netuid, &who),
                Error::<T>::AccountNotAllowedCommit
            );

            let extra_fields = info.fields.len() as u32;
            ensure!(
                extra_fields <= T::MaxFields::get(),
                Error::<T>::TooManyFieldsInCommitmentInfo
            );

            let cur_block = <frame_system::Pallet<T>>::block_number();
            if let Some(last_commit) = <LastCommitment<T>>::get(netuid, &who) {
                ensure!(
                    cur_block >= last_commit.saturating_add(T::RateLimit::get()),
                    Error::<T>::CommitmentSetRateLimitExceeded
                );
            }

            let fd = <BalanceOf<T>>::from(extra_fields).saturating_mul(T::FieldDeposit::get());
            let mut id = match <CommitmentOf<T>>::get(netuid, &who) {
                Some(mut id) => {
                    id.info = *info;
                    id.block = cur_block;
                    id
                }
                None => Registration {
                    info: *info,
                    block: cur_block,
                    deposit: Zero::zero(),
                },
            };

            let old_deposit = id.deposit;
            id.deposit = T::InitialDeposit::get().saturating_add(fd);
            if id.deposit > old_deposit {
                T::Currency::reserve(&who, id.deposit.saturating_sub(old_deposit))?;
            }
            if old_deposit > id.deposit {
                let err_amount =
                    T::Currency::unreserve(&who, old_deposit.saturating_sub(id.deposit));
                debug_assert!(err_amount.is_zero());
            }

            <CommitmentOf<T>>::insert(netuid, &who, id);
            <LastCommitment<T>>::insert(netuid, &who, cur_block);
            Self::deposit_event(Event::Commitment { netuid, who });

            Ok(())
        }
    }
}

// Interfaces to interact with other pallets
pub trait CanCommit<AccountId> {
    fn can_commit(netuid: u16, who: &AccountId) -> bool;
}

impl<A> CanCommit<A> for () {
    fn can_commit(_: u16, _: &A) -> bool {
        false
    }
}

/************************************************************
    CallType definition
************************************************************/
#[derive(Debug, PartialEq, Default)]
pub enum CallType {
    SetCommitment,
    #[default]
    Other,
}

use {
    frame_support::{
        dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
        pallet_prelude::{Decode, Encode, PhantomData, TypeInfo},
        traits::IsSubType,
    },
    sp_runtime::{
        traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension},
        transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
    },
};

#[freeze_struct("6a00398e14a8a984")]
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub struct CommitmentsSignedExtension<T: Config + Send + Sync + TypeInfo>(pub PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> Default for CommitmentsSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Config + Send + Sync + TypeInfo> CommitmentsSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn get_priority_vanilla() -> u64 {
        // Return high priority so that every extrinsic except set_weights function will
        // have a higher priority than the set_weights call
        u64::MAX
    }
}

impl<T: Config + Send + Sync + TypeInfo> sp_std::fmt::Debug for CommitmentsSignedExtension<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "SignedExtension")
    }
}

impl<T: Config + Send + Sync + TypeInfo> SignedExtension for CommitmentsSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    const IDENTIFIER: &'static str = "CommitmentsSignedExtension";

    type AccountId = T::AccountId;
    type Call = T::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = (CallType, u64, Self::AccountId);

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        _: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        call.is_sub_type();
        Ok(ValidTransaction {
            priority: Self::get_priority_vanilla(),
            ..Default::default()
        })
    }

    // NOTE: Add later when we put in a pre and post dispatch step.
    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        match call.is_sub_type() {
            Some(Call::set_commitment { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::SetCommitment, transaction_fee, who.clone()))
            }
            _ => {
                let transaction_fee = 0;
                Ok((CallType::Other, transaction_fee, who.clone()))
            }
        }
    }

    fn post_dispatch(
        _maybe_pre: Option<Self::Pre>,
        _info: &DispatchInfoOf<Self::Call>,
        _post_info: &PostDispatchInfoOf<Self::Call>,
        _len: usize,
        _result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        Ok(())
    }
}
