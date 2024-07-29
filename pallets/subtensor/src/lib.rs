#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]
#![allow(clippy::too_many_arguments)]
// Edit this file to define custom logic or remove it if it is not needed.
// Learn more about FRAME and the core library of Substrate FRAME pallets:
// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use frame_system::{self as system, ensure_signed};

use frame_support::{
    dispatch::{self, DispatchInfo, DispatchResult, DispatchResultWithPostInfo, PostDispatchInfo},
    ensure,
    pallet_macros::import_section,
    traits::{tokens::fungible, IsSubType},
};

use codec::{Decode, Encode};
use frame_support::sp_runtime::transaction_validity::InvalidTransaction;
use frame_support::sp_runtime::transaction_validity::ValidTransaction;
use pallet_balances::Call as BalancesCall;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError},
    DispatchError,
};
use sp_std::marker::PhantomData;

// ============================
//	==== Benchmark Imports =====
// ============================
mod benchmarks;

// =========================
//	==== Pallet Imports =====
// =========================
mod block_step;
mod epoch;
mod errors;
mod events;
pub mod math;
mod registration;
mod root;
mod serving;
mod staking;
mod swap;
mod uids;
mod utils;
mod weights;

pub mod delegate_info;
pub mod neuron_info;
pub mod stake_info;
pub mod subnet_info;

// apparently this is stabilized since rust 1.36
extern crate alloc;
pub mod migration;

#[deny(missing_docs)]
#[import_section(errors::errors)]
#[import_section(events::events)]
#[frame_support::pallet]
pub mod pallet {

    use frame_support::{
        dispatch::GetDispatchInfo,
        pallet_prelude::{DispatchResult, StorageMap, ValueQuery, *},
        traits::{tokens::fungible, UnfilteredDispatchable},
    };
    use frame_system::pallet_prelude::*;
    use sp_core::H256;
    use sp_runtime::traits::TrailingZeroInput;
    use sp_std::vec;
    use sp_std::vec::Vec;

    use subtensor_macros::freeze_struct;

    #[cfg(not(feature = "std"))]
    use alloc::boxed::Box;
    #[cfg(feature = "std")]
    use sp_std::prelude::Box;

    /// Tracks version for migrations. Should be monotonic with respect to the
    /// order of migrations. (i.e. always increasing)
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

    /// Minimum balance required to perform a coldkey swap
    pub const MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP: u64 = 100_000_000; // 0.1 TAO in RAO

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// A sudo-able call.
        type SudoRuntimeCall: Parameter
            + UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo;

        /// Origin checking for council majority
        type CouncilOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        ///  Currency type that will be used to place deposits on neurons
        type Currency: fungible::Balanced<Self::AccountId, Balance = u64>
            + fungible::Mutate<Self::AccountId>;

        /// Senate members with members management functions.
        type SenateMembers: crate::MemberManagement<Self::AccountId>;

        /// Interface to allow other pallets to control who can register identities
        type TriumvirateInterface: crate::CollectiveInterface<Self::AccountId, Self::Hash, u32>;

        /// =================================
        /// ==== Initial Value Constants ====
        /// =================================

        /// Initial currency issuance.
        #[pallet::constant]
        type InitialIssuance: Get<u64>;
        /// Initial min allowed weights setting.
        #[pallet::constant]
        type InitialMinAllowedWeights: Get<u16>;
        /// Initial Emission Ratio.
        #[pallet::constant]
        type InitialEmissionValue: Get<u16>;
        /// Initial max weight limit.
        #[pallet::constant]
        type InitialMaxWeightsLimit: Get<u16>;
        /// Tempo for each network.
        #[pallet::constant]
        type InitialTempo: Get<u16>;
        /// Initial Difficulty.
        #[pallet::constant]
        type InitialDifficulty: Get<u64>;
        /// Initial Max Difficulty.
        #[pallet::constant]
        type InitialMaxDifficulty: Get<u64>;
        /// Initial Min Difficulty.
        #[pallet::constant]
        type InitialMinDifficulty: Get<u64>;
        /// Initial RAO Recycled.
        #[pallet::constant]
        type InitialRAORecycledForRegistration: Get<u64>;
        /// Initial Burn.
        #[pallet::constant]
        type InitialBurn: Get<u64>;
        /// Initial Max Burn.
        #[pallet::constant]
        type InitialMaxBurn: Get<u64>;
        /// Initial Min Burn.
        #[pallet::constant]
        type InitialMinBurn: Get<u64>;
        /// Initial adjustment interval.
        #[pallet::constant]
        type InitialAdjustmentInterval: Get<u16>;
        /// Initial bonds moving average.
        #[pallet::constant]
        type InitialBondsMovingAverage: Get<u64>;
        /// Initial target registrations per interval.
        #[pallet::constant]
        type InitialTargetRegistrationsPerInterval: Get<u16>;
        /// Rho constant.
        #[pallet::constant]
        type InitialRho: Get<u16>;
        /// Kappa constant.
        #[pallet::constant]
        type InitialKappa: Get<u16>;
        /// Max UID constant.
        #[pallet::constant]
        type InitialMaxAllowedUids: Get<u16>;
        /// Initial validator context pruning length.
        #[pallet::constant]
        type InitialValidatorPruneLen: Get<u64>;
        /// Initial scaling law power.
        #[pallet::constant]
        type InitialScalingLawPower: Get<u16>;
        /// Immunity Period Constant.
        #[pallet::constant]
        type InitialImmunityPeriod: Get<u16>;
        /// Activity constant.
        #[pallet::constant]
        type InitialActivityCutoff: Get<u16>;
        /// Initial max registrations per block.
        #[pallet::constant]
        type InitialMaxRegistrationsPerBlock: Get<u16>;
        /// Initial pruning score for each neuron.
        #[pallet::constant]
        type InitialPruningScore: Get<u16>;
        /// Initial maximum allowed validators per network.
        #[pallet::constant]
        type InitialMaxAllowedValidators: Get<u16>;
        /// Initial default delegation take.
        #[pallet::constant]
        type InitialDefaultTake: Get<u16>;
        /// Initial minimum delegation take.
        #[pallet::constant]
        type InitialMinTake: Get<u16>;
        /// Initial weights version key.
        #[pallet::constant]
        type InitialWeightsVersionKey: Get<u64>;
        /// Initial serving rate limit.
        #[pallet::constant]
        type InitialServingRateLimit: Get<u64>;
        /// Initial transaction rate limit.
        #[pallet::constant]
        type InitialTxRateLimit: Get<u64>;
        /// Initial delegate take transaction rate limit.
        #[pallet::constant]
        type InitialTxDelegateTakeRateLimit: Get<u64>;
        /// Initial percentage of total stake required to join senate.
        #[pallet::constant]
        type InitialSenateRequiredStakePercentage: Get<u64>;
        /// Initial adjustment alpha on burn and pow.
        #[pallet::constant]
        type InitialAdjustmentAlpha: Get<u64>;
        /// Initial network immunity period
        #[pallet::constant]
        type InitialNetworkImmunityPeriod: Get<u64>;
        /// Initial minimum allowed network UIDs
        #[pallet::constant]
        type InitialNetworkMinAllowedUids: Get<u16>;
        /// Initial network minimum burn cost
        #[pallet::constant]
        type InitialNetworkMinLockCost: Get<u64>;
        /// Initial network subnet cut.
        #[pallet::constant]
        type InitialSubnetOwnerCut: Get<u16>;
        /// Initial lock reduction interval.
        #[pallet::constant]
        type InitialNetworkLockReductionInterval: Get<u64>;
        /// Initial max allowed subnets
        #[pallet::constant]
        type InitialSubnetLimit: Get<u16>;
        /// Initial network creation rate limit
        #[pallet::constant]
        type InitialNetworkRateLimit: Get<u64>;
        /// Initial target stakes per interval issuance.
        #[pallet::constant]
        type InitialTargetStakesPerInterval: Get<u64>;
        /// Cost of swapping a hotkey.
        #[pallet::constant]
        type KeySwapCost: Get<u64>;
        /// The upper bound for the alpha parameter. Used for Liquid Alpha.
        #[pallet::constant]
        type AlphaHigh: Get<u16>;
        /// The lower bound for the alpha parameter. Used for Liquid Alpha.
        #[pallet::constant]
        type AlphaLow: Get<u16>;
        /// A flag to indicate if Liquid Alpha is enabled.
        #[pallet::constant]
        type LiquidAlphaOn: Get<bool>;
        /// The base difficulty for proof of work for coldkey swaps
        #[pallet::constant]
        type InitialBaseDifficulty: Get<u64>;
    }

    /// Alias for the account ID.
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    /// Senate requirements
    #[pallet::type_value]
    pub fn DefaultSenateRequiredStakePercentage<T: Config>() -> u64 {
        T::InitialSenateRequiredStakePercentage::get()
    }

    #[pallet::storage]
    pub(super) type SenateRequiredStakePercentage<T> =
        StorageValue<_, u64, ValueQuery, DefaultSenateRequiredStakePercentage<T>>;

    /// ============================
    /// ==== Staking + Accounts ====
    /// ============================

    /// Total Rao in circulation.
    #[pallet::type_value]
    pub fn TotalSupply<T: Config>() -> u64 {
        21_000_000_000_000_000 // Rao => 21_000_000 Tao
    }
    /// Default total stake.
    #[pallet::type_value]
    pub fn DefaultDefaultTake<T: Config>() -> u16 {
        T::InitialDefaultTake::get()
    }
    /// Default minimum take.
    #[pallet::type_value]
    pub fn DefaultMinTake<T: Config>() -> u16 {
        T::InitialMinTake::get()
    }
    /// Default account take.
    #[pallet::type_value]
    pub fn DefaultAccountTake<T: Config>() -> u64 {
        0
    }
    /// Default stakes per interval.
    #[pallet::type_value]
    pub fn DefaultStakesPerInterval<T: Config>() -> (u64, u64) {
        (0, 0)
    }
    /// Default emission per block.
    #[pallet::type_value]
    pub fn DefaultBlockEmission<T: Config>() -> u64 {
        1_000_000_000
    }
    /// Default allowed delegation.
    #[pallet::type_value]
    pub fn DefaultAllowsDelegation<T: Config>() -> bool {
        false
    }
    /// Default total issuance.
    #[pallet::type_value]
    pub fn DefaultTotalIssuance<T: Config>() -> u64 {
        T::InitialIssuance::get()
    }
    /// Default account, derived from zero trailing bytes.
    #[pallet::type_value]
    pub fn DefaultAccount<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut TrailingZeroInput::zeroes())
            .expect("trailing zeroes always produce a valid account ID; qed")
    }
    /// Default target stakes per interval.
    #[pallet::type_value]
    pub fn DefaultTargetStakesPerInterval<T: Config>() -> u64 {
        T::InitialTargetStakesPerInterval::get()
    }
    /// Default stake interval.
    #[pallet::type_value]
    pub fn DefaultStakeInterval<T: Config>() -> u64 {
        360
    }

    /// Default base difficulty for proof of work for coldkey swaps
    #[pallet::type_value]
    pub fn DefaultBaseDifficulty<T: Config>() -> u64 {
        T::InitialBaseDifficulty::get()
    }

    #[pallet::storage] // --- ITEM ( total_stake )
    pub type TotalStake<T> = StorageValue<_, u64, ValueQuery>;
    #[pallet::storage] // --- ITEM ( default_take )
    pub type MaxTake<T> = StorageValue<_, u16, ValueQuery, DefaultDefaultTake<T>>;
    #[pallet::storage] // --- ITEM ( min_take )
    pub type MinTake<T> = StorageValue<_, u16, ValueQuery, DefaultMinTake<T>>;
    #[pallet::storage] // --- ITEM ( global_block_emission )
    pub type BlockEmission<T> = StorageValue<_, u64, ValueQuery, DefaultBlockEmission<T>>;
    #[pallet::storage] // --- ITEM ( total_issuance )
    pub type TotalIssuance<T> = StorageValue<_, u64, ValueQuery, DefaultTotalIssuance<T>>;
    #[pallet::storage] // --- ITEM (target_stakes_per_interval)
    pub type TargetStakesPerInterval<T> =
        StorageValue<_, u64, ValueQuery, DefaultTargetStakesPerInterval<T>>;

    #[pallet::storage] // --- ITEM ( base_difficulty )
    pub type BaseDifficulty<T> = StorageValue<_, u64, ValueQuery, DefaultBaseDifficulty<T>>;
    #[pallet::storage] // --- ITEM (default_stake_interval)
    pub type StakeInterval<T> = StorageValue<_, u64, ValueQuery, DefaultStakeInterval<T>>;
    #[pallet::storage] // --- MAP ( hot ) --> stake | Returns the total amount of stake under a hotkey.
    pub type TotalHotkeyStake<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultAccountTake<T>>;
    #[pallet::storage] // --- MAP ( cold ) --> stake | Returns the total amount of stake under a coldkey.
    pub type TotalColdkeyStake<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultAccountTake<T>>;
    #[pallet::storage]
    ///  MAP (hot, cold) --> stake | Returns a tuple (u64: stakes, u64: block_number)
    pub type TotalHotkeyColdkeyStakesThisInterval<T: Config> = StorageDoubleMap<
        _,
        Identity,
        T::AccountId,
        Identity,
        T::AccountId,
        (u64, u64),
        ValueQuery,
        DefaultStakesPerInterval<T>,
    >;
    #[pallet::storage] // --- MAP ( hot ) --> cold | Returns the controlling coldkey for a hotkey.
    pub type Owner<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultAccount<T>>;
    #[pallet::storage] // --- MAP ( cold ) --> Vec<hot> | Returns the vector of hotkeys controlled by this coldkey.
    pub type OwnedHotkeys<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>, ValueQuery>;
    #[pallet::storage] // --- DMAP ( cold ) --> Vec<hot> | Maps coldkey to hotkeys that stake to it
    pub type StakingHotkeys<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>, ValueQuery>;
    #[pallet::storage] // --- MAP ( hot ) --> take | Returns the hotkey delegation take. And signals that this key is open for delegation.
    pub type Delegates<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u16, ValueQuery, DefaultDefaultTake<T>>;
    #[pallet::storage] // --- DMAP ( hot, cold ) --> stake | Returns the stake under a coldkey prefixed by hotkey.
    pub type Stake<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        T::AccountId,
        u64,
        ValueQuery,
        DefaultAccountTake<T>,
    >;

    #[pallet::type_value]
    /// Default value for hotkeys.
    pub fn EmptyAccounts<T: Config>() -> Vec<T::AccountId> {
        vec![]
    }
    #[pallet::type_value]
    /// Default arbitration period.
    /// This value represents the default arbitration period in blocks.
    /// The period is set to 18 hours, assuming a block time of 12 seconds.
    pub fn DefaultArbitrationPeriod<T: Config>() -> u64 {
        7200 * 3 // 3 days
    }
    #[pallet::storage] // ---- StorageItem Global Used Work.
    pub type ArbitrationPeriod<T: Config> =
        StorageValue<_, u64, ValueQuery, DefaultArbitrationPeriod<T>>;
    #[pallet::storage] // --- MAP ( cold ) --> Vec<wallet_to_drain_to> | Returns a list of keys to drain to, if there are two, we extend the period.
    pub type ColdkeySwapDestinations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Vec<T::AccountId>,
        ValueQuery,
        EmptyAccounts<T>,
    >;
    #[pallet::storage] // --- MAP ( cold ) --> u64 | Block when the coldkey will be arbitrated.
    pub type ColdkeyArbitrationBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;
    #[pallet::storage] // --- MAP ( u64 ) --> Vec<coldkeys_to_drain>  | Coldkeys to drain on the specific block.
    pub type ColdkeysToSwapAtBlock<T: Config> =
        StorageMap<_, Identity, u64, Vec<T::AccountId>, ValueQuery, EmptyAccounts<T>>;
    /// -- ITEM (switches liquid alpha on)
    #[pallet::type_value]
    pub fn DefaultLiquidAlpha<T: Config>() -> bool {
        false
    }
    #[pallet::storage] // --- MAP ( netuid ) --> Whether or not Liquid Alpha is enabled
    pub type LiquidAlphaOn<T> =
        StorageMap<_, Blake2_128Concat, u16, bool, ValueQuery, DefaultLiquidAlpha<T>>;

    /// =====================================
    /// ==== Difficulty / Registrations =====
    /// =====================================

    /// Default last adjustment block.
    #[pallet::type_value]
    pub fn DefaultLastAdjustmentBlock<T: Config>() -> u64 {
        0
    }
    /// Default registrations this block.
    #[pallet::type_value]
    pub fn DefaultRegistrationsThisBlock<T: Config>() -> u16 {
        0
    }
    /// Default burn token.
    #[pallet::type_value]
    pub fn DefaultBurn<T: Config>() -> u64 {
        T::InitialBurn::get()
    }
    /// Default min burn token.
    #[pallet::type_value]
    pub fn DefaultMinBurn<T: Config>() -> u64 {
        T::InitialMinBurn::get()
    }
    /// Default max burn token.
    #[pallet::type_value]
    pub fn DefaultMaxBurn<T: Config>() -> u64 {
        T::InitialMaxBurn::get()
    }
    /// Default difficulty value.
    #[pallet::type_value]
    pub fn DefaultDifficulty<T: Config>() -> u64 {
        T::InitialDifficulty::get()
    }
    /// Default min difficulty value.
    #[pallet::type_value]
    pub fn DefaultMinDifficulty<T: Config>() -> u64 {
        T::InitialMinDifficulty::get()
    }
    /// Default max difficulty value.
    #[pallet::type_value]
    pub fn DefaultMaxDifficulty<T: Config>() -> u64 {
        T::InitialMaxDifficulty::get()
    }
    /// Default max registrations per block.
    #[pallet::type_value]
    pub fn DefaultMaxRegistrationsPerBlock<T: Config>() -> u16 {
        T::InitialMaxRegistrationsPerBlock::get()
    }
    /// Default RAO recycled for registration.
    #[pallet::type_value]
    pub fn DefaultRAORecycledForRegistration<T: Config>() -> u64 {
        T::InitialRAORecycledForRegistration::get()
    }

    #[pallet::storage] // ---- StorageItem Global Used Work.
    pub type UsedWork<T: Config> = StorageMap<_, Identity, Vec<u8>, u64, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> Burn
    pub type Burn<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBurn<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> Difficulty
    pub type Difficulty<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultDifficulty<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> MinBurn
    pub type MinBurn<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMinBurn<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> MaxBurn
    pub type MaxBurn<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMaxBurn<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> MinDifficulty
    pub type MinDifficulty<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMinDifficulty<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> MaxDifficulty
    pub type MaxDifficulty<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMaxDifficulty<T>>;
    #[pallet::storage] // --- MAP ( netuid ) -->  Block at last adjustment.
    pub type LastAdjustmentBlock<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultLastAdjustmentBlock<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> Registrations of this Block.
    pub type RegistrationsThisBlock<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRegistrationsThisBlock<T>>;
    #[pallet::storage] // --- ITEM( global_max_registrations_per_block )
    pub type MaxRegistrationsPerBlock<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxRegistrationsPerBlock<T>>;
    #[pallet::storage] // --- MAP ( netuid, global_RAO_recycled_for_registration )
    pub type RAORecycledForRegistration<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultRAORecycledForRegistration<T>>;

    /// ==============================
    /// ==== Subnetworks Storage =====
    /// ==============================

    /// Default number of networks.
    #[pallet::type_value]
    pub fn DefaultN<T: Config>() -> u16 {
        0
    }
    /// Default value for modality.
    #[pallet::type_value]
    pub fn DefaultModality<T: Config>() -> u16 {
        0
    }
    /// Default value for hotkeys.
    #[pallet::type_value]
    pub fn DefaultHotkeys<T: Config>() -> Vec<u16> {
        vec![]
    }
    /// Default value if network is added.
    #[pallet::type_value]
    pub fn DefaultNeworksAdded<T: Config>() -> bool {
        false
    }
    /// Default value for network member.
    #[pallet::type_value]
    pub fn DefaultIsNetworkMember<T: Config>() -> bool {
        false
    }
    /// Default value for registration allowed.
    #[pallet::type_value]
    pub fn DefaultRegistrationAllowed<T: Config>() -> bool {
        false
    }
    /// Default value for network registered at.
    #[pallet::type_value]
    pub fn DefaultNetworkRegisteredAt<T: Config>() -> u64 {
        0
    }
    /// Default value for network immunity period.
    #[pallet::type_value]
    pub fn DefaultNetworkImmunityPeriod<T: Config>() -> u64 {
        T::InitialNetworkImmunityPeriod::get()
    }
    /// Default value for network last registered.
    #[pallet::type_value]
    pub fn DefaultNetworkLastRegistered<T: Config>() -> u64 {
        0
    }
    /// Default value for nominator min required stake.
    #[pallet::type_value]
    pub fn DefaultNominatorMinRequiredStake<T: Config>() -> u64 {
        0
    }
    /// Default value for network min allowed UIDs.
    #[pallet::type_value]
    pub fn DefaultNetworkMinAllowedUids<T: Config>() -> u16 {
        T::InitialNetworkMinAllowedUids::get()
    }
    /// Default value for network min lock cost.
    #[pallet::type_value]
    pub fn DefaultNetworkMinLockCost<T: Config>() -> u64 {
        T::InitialNetworkMinLockCost::get()
    }
    /// Default value for network lock reduction interval.
    #[pallet::type_value]
    pub fn DefaultNetworkLockReductionInterval<T: Config>() -> u64 {
        T::InitialNetworkLockReductionInterval::get()
    }
    /// Default value for subnet owner cut.
    #[pallet::type_value]
    pub fn DefaultSubnetOwnerCut<T: Config>() -> u16 {
        T::InitialSubnetOwnerCut::get()
    }
    /// Default value for subnet limit.
    #[pallet::type_value]
    pub fn DefaultSubnetLimit<T: Config>() -> u16 {
        T::InitialSubnetLimit::get()
    }
    /// Default value for network rate limit.
    #[pallet::type_value]
    pub fn DefaultNetworkRateLimit<T: Config>() -> u64 {
        if cfg!(feature = "pow-faucet") {
            return 0;
        }

        T::InitialNetworkRateLimit::get()
    }

    #[pallet::storage] // --- ITEM( maximum_number_of_networks )
    pub type SubnetLimit<T> = StorageValue<_, u16, ValueQuery, DefaultSubnetLimit<T>>;
    #[pallet::storage] // --- ITEM( total_number_of_existing_networks )
    pub type TotalNetworks<T> = StorageValue<_, u16, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> subnetwork_n (Number of UIDs in the network).
    pub type SubnetworkN<T: Config> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultN<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> modality   TEXT: 0, IMAGE: 1, TENSOR: 2
    pub type NetworkModality<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultModality<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> network_is_added
    pub type NetworksAdded<T: Config> =
        StorageMap<_, Identity, u16, bool, ValueQuery, DefaultNeworksAdded<T>>;
    #[pallet::storage] // --- DMAP ( hotkey, netuid ) --> bool
    pub type IsNetworkMember<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        u16,
        bool,
        ValueQuery,
        DefaultIsNetworkMember<T>,
    >;
    #[pallet::storage] // --- MAP ( netuid ) --> network_registration_allowed
    pub type NetworkRegistrationAllowed<T: Config> =
        StorageMap<_, Identity, u16, bool, ValueQuery, DefaultRegistrationAllowed<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> network_pow_allowed
    pub type NetworkPowRegistrationAllowed<T: Config> =
        StorageMap<_, Identity, u16, bool, ValueQuery, DefaultRegistrationAllowed<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> block_created
    pub type NetworkRegisteredAt<T: Config> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultNetworkRegisteredAt<T>>;
    #[pallet::storage] // ITEM( network_immunity_period )
    pub type NetworkImmunityPeriod<T> =
        StorageValue<_, u64, ValueQuery, DefaultNetworkImmunityPeriod<T>>;
    #[pallet::storage] // ITEM( network_last_registered_block )
    pub type NetworkLastRegistered<T> =
        StorageValue<_, u64, ValueQuery, DefaultNetworkLastRegistered<T>>;
    #[pallet::storage] // ITEM( network_min_allowed_uids )
    pub type NetworkMinAllowedUids<T> =
        StorageValue<_, u16, ValueQuery, DefaultNetworkMinAllowedUids<T>>;
    #[pallet::storage] // ITEM( min_network_lock_cost )
    pub type NetworkMinLockCost<T> = StorageValue<_, u64, ValueQuery, DefaultNetworkMinLockCost<T>>;
    #[pallet::storage] // ITEM( last_network_lock_cost )
    pub type NetworkLastLockCost<T> =
        StorageValue<_, u64, ValueQuery, DefaultNetworkMinLockCost<T>>;
    #[pallet::storage] // ITEM( network_lock_reduction_interval )
    pub type NetworkLockReductionInterval<T> =
        StorageValue<_, u64, ValueQuery, DefaultNetworkLockReductionInterval<T>>;
    #[pallet::storage] // ITEM( subnet_owner_cut )
    pub type SubnetOwnerCut<T> = StorageValue<_, u16, ValueQuery, DefaultSubnetOwnerCut<T>>;
    #[pallet::storage] // ITEM( network_rate_limit )
    pub type NetworkRateLimit<T> = StorageValue<_, u64, ValueQuery, DefaultNetworkRateLimit<T>>;
    #[pallet::storage] // ITEM( nominator_min_required_stake )
    pub type NominatorMinRequiredStake<T> =
        StorageValue<_, u64, ValueQuery, DefaultNominatorMinRequiredStake<T>>;

    /// ==============================
    /// ==== Subnetwork Features =====
    /// ==============================

    /// Default value for emission values.
    #[pallet::type_value]
    pub fn DefaultEmissionValues<T: Config>() -> u64 {
        0
    }
    /// Default value for pending emission.
    #[pallet::type_value]
    pub fn DefaultPendingEmission<T: Config>() -> u64 {
        0
    }
    /// Default value for blocks since last step.
    #[pallet::type_value]
    pub fn DefaultBlocksSinceLastStep<T: Config>() -> u64 {
        0
    }
    /// Default value for last mechanism step block.
    #[pallet::type_value]
    pub fn DefaultLastMechanismStepBlock<T: Config>() -> u64 {
        0
    }
    /// Default value for subnet owner.
    #[pallet::type_value]
    pub fn DefaultSubnetOwner<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
            .expect("trailing zeroes always produce a valid account ID; qed")
    }
    /// Default value for subnet locked.
    #[pallet::type_value]
    pub fn DefaultSubnetLocked<T: Config>() -> u64 {
        0
    }
    /// Default value for network tempo
    #[pallet::type_value]
    pub fn DefaultTempo<T: Config>() -> u16 {
        T::InitialTempo::get()
    }

    #[pallet::storage] // --- MAP ( netuid ) --> tempo
    pub type Tempo<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTempo<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> emission_values
    pub type EmissionValues<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultEmissionValues<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> pending_emission
    pub type PendingEmission<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultPendingEmission<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> blocks_since_last_step
    pub type BlocksSinceLastStep<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBlocksSinceLastStep<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> last_mechanism_step_block
    pub type LastMechansimStepBlock<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultLastMechanismStepBlock<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> subnet_owner
    pub type SubnetOwner<T: Config> =
        StorageMap<_, Identity, u16, T::AccountId, ValueQuery, DefaultSubnetOwner<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> subnet_locked
    pub type SubnetLocked<T: Config> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultSubnetLocked<T>>;

    /// =================================
    /// ==== Axon / Promo Endpoints =====
    /// =================================

    /// Struct for Axon.
    pub type AxonInfoOf = AxonInfo;

    /// Data structure for Axon information.
    #[freeze_struct("3545cfb0cac4c1f5")]
    #[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct AxonInfo {
        ///  Axon serving block.
        pub block: u64,
        ///  Axon version
        pub version: u32,
        ///  Axon u128 encoded ip address of type v6 or v4.
        pub ip: u128,
        ///  Axon u16 encoded port.
        pub port: u16,
        ///  Axon ip type, 4 for ipv4 and 6 for ipv6.
        pub ip_type: u8,
        ///  Axon protocol. TCP, UDP, other.
        pub protocol: u8,
        ///  Axon proto placeholder 1.
        pub placeholder1: u8,
        ///  Axon proto placeholder 2.
        pub placeholder2: u8,
    }

    ///  Struct for Prometheus.
    pub type PrometheusInfoOf = PrometheusInfo;
    /// Data structure for Prometheus information.
    #[freeze_struct("5dde687e63baf0cd")]
    #[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct PrometheusInfo {
        /// Prometheus serving block.
        pub block: u64,
        /// Prometheus version.
        pub version: u32,
        ///  Prometheus u128 encoded ip address of type v6 or v4.
        pub ip: u128,
        ///  Prometheus u16 encoded port.
        pub port: u16,
        /// Prometheus ip type, 4 for ipv4 and 6 for ipv6.
        pub ip_type: u8,
    }

    /// Default value for rate limiting
    #[pallet::type_value]
    pub fn DefaultTxRateLimit<T: Config>() -> u64 {
        T::InitialTxRateLimit::get()
    }
    /// Default value for delegate take rate limiting
    #[pallet::type_value]
    pub fn DefaultTxDelegateTakeRateLimit<T: Config>() -> u64 {
        T::InitialTxDelegateTakeRateLimit::get()
    }
    /// Default value for last extrinsic block.
    #[pallet::type_value]
    pub fn DefaultLastTxBlock<T: Config>() -> u64 {
        0
    }

    #[pallet::storage] // --- ITEM ( tx_rate_limit )
    pub(super) type TxRateLimit<T> = StorageValue<_, u64, ValueQuery, DefaultTxRateLimit<T>>;
    #[pallet::storage] // --- ITEM ( tx_rate_limit )
    pub(super) type TxDelegateTakeRateLimit<T> =
        StorageValue<_, u64, ValueQuery, DefaultTxDelegateTakeRateLimit<T>>;
    #[pallet::storage] // --- MAP ( key ) --> last_block
    pub type LastTxBlock<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultLastTxBlock<T>>;
    #[pallet::storage] // --- MAP ( key ) --> last_block
    pub(super) type LastTxBlockDelegateTake<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultLastTxBlock<T>>;

    /// Default value for serving rate limit.
    #[pallet::type_value]
    pub fn DefaultServingRateLimit<T: Config>() -> u64 {
        T::InitialServingRateLimit::get()
    }

    #[pallet::storage] // --- MAP ( netuid ) --> serving_rate_limit
    pub type ServingRateLimit<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultServingRateLimit<T>>;
    #[pallet::storage] // --- MAP ( netuid, hotkey ) --> axon_info
    pub type Axons<T: Config> =
        StorageDoubleMap<_, Identity, u16, Blake2_128Concat, T::AccountId, AxonInfoOf, OptionQuery>;
    #[pallet::storage] // --- MAP ( netuid, hotkey ) --> prometheus_info
    pub type Prometheus<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Blake2_128Concat,
        T::AccountId,
        PrometheusInfoOf,
        OptionQuery,
    >;

    /// =======================================
    /// ==== Subnetwork Hyperparam storage ====
    /// =======================================

    /// Default weights set rate limit.
    #[pallet::type_value]
    pub fn DefaultWeightsSetRateLimit<T: Config>() -> u64 {
        100
    }
    /// Default block at registration.
    #[pallet::type_value]
    pub fn DefaultBlockAtRegistration<T: Config>() -> u64 {
        0
    }
    /// Default Rho parameter value.
    #[pallet::type_value]
    pub fn DefaultRho<T: Config>() -> u16 {
        T::InitialRho::get()
    }
    /// Default Kai parameter value.
    #[pallet::type_value]
    pub fn DefaultKappa<T: Config>() -> u16 {
        T::InitialKappa::get()
    }
    /// Default max allowed uids.
    #[pallet::type_value]
    pub fn DefaultMaxAllowedUids<T: Config>() -> u16 {
        T::InitialMaxAllowedUids::get()
    }
    /// Default immunity period value.
    #[pallet::type_value]
    pub fn DefaultImmunityPeriod<T: Config>() -> u16 {
        T::InitialImmunityPeriod::get()
    }
    /// Default activity cutoff value.
    #[pallet::type_value]
    pub fn DefaultActivityCutoff<T: Config>() -> u16 {
        T::InitialActivityCutoff::get()
    }
    /// Default max weights limit.
    #[pallet::type_value]
    pub fn DefaultMaxWeightsLimit<T: Config>() -> u16 {
        T::InitialMaxWeightsLimit::get()
    }
    /// Default weights version key.
    #[pallet::type_value]
    pub fn DefaultWeightsVersionKey<T: Config>() -> u64 {
        T::InitialWeightsVersionKey::get()
    }
    /// Default minimal allowed weights.
    #[pallet::type_value]
    pub fn DefaultMinAllowedWeights<T: Config>() -> u16 {
        T::InitialMinAllowedWeights::get()
    }
    /// Default max allowed validators.
    #[pallet::type_value]
    pub fn DefaultMaxAllowedValidators<T: Config>() -> u16 {
        T::InitialMaxAllowedValidators::get()
    }
    /// Default adjustment interval.
    #[pallet::type_value]
    pub fn DefaultAdjustmentInterval<T: Config>() -> u16 {
        T::InitialAdjustmentInterval::get()
    }
    /// Default bonds moving average.
    #[pallet::type_value]
    pub fn DefaultBondsMovingAverage<T: Config>() -> u64 {
        T::InitialBondsMovingAverage::get()
    }
    /// Default validator prune length.
    #[pallet::type_value]
    pub fn DefaultValidatorPruneLen<T: Config>() -> u64 {
        T::InitialValidatorPruneLen::get()
    }
    /// Default scaling law power.
    #[pallet::type_value]
    pub fn DefaultScalingLawPower<T: Config>() -> u16 {
        T::InitialScalingLawPower::get()
    }
    /// Default target registrations per interval.
    #[pallet::type_value]
    pub fn DefaultTargetRegistrationsPerInterval<T: Config>() -> u16 {
        T::InitialTargetRegistrationsPerInterval::get()
    }
    /// Default adjustment alpha.
    #[pallet::type_value]
    pub fn DefaultAdjustmentAlpha<T: Config>() -> u64 {
        T::InitialAdjustmentAlpha::get()
    }
    /// Default weights min stake.
    #[pallet::type_value]
    pub fn DefaultWeightsMinStake<T: Config>() -> u64 {
        0
    }
    /// Provides the default value for the upper bound of the alpha parameter.

    #[pallet::type_value]
    pub fn DefaultAlphaValues<T: Config>() -> (u16, u16) {
        (45875, 58982) // (alpha_low: 0.7, alpha_high: 0.9)
    }

    #[pallet::storage] // ITEM( weights_min_stake )
    pub type WeightsMinStake<T> = StorageValue<_, u64, ValueQuery, DefaultWeightsMinStake<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> Rho
    pub type Rho<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRho<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> Kappa
    pub type Kappa<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultKappa<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> uid, we use to record uids to prune at next epoch.
    pub type NeuronsToPruneAtNextEpoch<T: Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> registrations_this_interval
    pub type RegistrationsThisInterval<T: Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> pow_registrations_this_interval
    pub type POWRegistrationsThisInterval<T: Config> =
        StorageMap<_, Identity, u16, u16, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> burn_registrations_this_interval
    pub type BurnRegistrationsThisInterval<T: Config> =
        StorageMap<_, Identity, u16, u16, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> max_allowed_uids
    pub type MaxAllowedUids<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedUids<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> immunity_period
    pub type ImmunityPeriod<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultImmunityPeriod<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> activity_cutoff
    pub type ActivityCutoff<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultActivityCutoff<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> max_weight_limit
    pub type MaxWeightsLimit<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxWeightsLimit<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> weights_version_key
    pub type WeightsVersionKey<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultWeightsVersionKey<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> min_allowed_weights
    pub type MinAllowedWeights<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMinAllowedWeights<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> max_allowed_validators
    pub type MaxAllowedValidators<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedValidators<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> adjustment_interval
    pub type AdjustmentInterval<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultAdjustmentInterval<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> bonds_moving_average
    pub type BondsMovingAverage<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBondsMovingAverage<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> weights_set_rate_limit
    pub type WeightsSetRateLimit<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultWeightsSetRateLimit<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> validator_prune_len
    pub type ValidatorPruneLen<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultValidatorPruneLen<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> scaling_law_power
    pub type ScalingLawPower<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultScalingLawPower<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> target_registrations_this_interval
    pub type TargetRegistrationsPerInterval<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTargetRegistrationsPerInterval<T>>;
    #[pallet::storage] // --- DMAP ( netuid, uid ) --> block_at_registration
    pub type BlockAtRegistration<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Identity,
        u16,
        u64,
        ValueQuery,
        DefaultBlockAtRegistration<T>,
    >;
    #[pallet::storage] // --- DMAP ( netuid ) --> adjustment_alpha
    pub type AdjustmentAlpha<T: Config> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultAdjustmentAlpha<T>>;

    //  MAP ( netuid ) --> (alpha_low, alpha_high)
    #[pallet::storage]
    pub type AlphaValues<T> =
        StorageMap<_, Identity, u16, (u16, u16), ValueQuery, DefaultAlphaValues<T>>;

    #[pallet::storage] // --- MAP (netuid, who) --> (hash, weight) | Returns the hash and weight committed by an account for a given netuid.
    pub type WeightCommits<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        u16,
        Twox64Concat,
        T::AccountId,
        (H256, u64),
        OptionQuery,
    >;

    /// Default value for weight commit reveal interval.
    #[pallet::type_value]
    pub fn DefaultWeightCommitRevealInterval<T: Config>() -> u64 {
        1000
    }
    // --- DMAP ( netuid ) --> interval
    #[pallet::storage]
    pub type WeightCommitRevealInterval<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultWeightCommitRevealInterval<T>>;

    /// Default value for weight commit/reveal enabled.
    #[pallet::type_value]
    pub fn DefaultCommitRevealWeightsEnabled<T: Config>() -> bool {
        false
    }
    // --- DMAP ( netuid ) --> interval
    #[pallet::storage]
    pub type CommitRevealWeightsEnabled<T> =
        StorageMap<_, Identity, u16, bool, ValueQuery, DefaultCommitRevealWeightsEnabled<T>>;

    /// =======================================
    /// ==== Subnetwork Consensus Storage  ====
    /// =======================================

    /// Value definition for vector of u16.
    #[pallet::type_value]
    pub fn EmptyU16Vec<T: Config>() -> Vec<u16> {
        vec![]
    }
    /// Value definition for vector of u64.
    #[pallet::type_value]
    pub fn EmptyU64Vec<T: Config>() -> Vec<u64> {
        vec![]
    }
    /// Value definition for vector of bool.
    #[pallet::type_value]
    pub fn EmptyBoolVec<T: Config>() -> Vec<bool> {
        vec![]
    }
    /// Value definition for bonds with type vector of (u16, u16).
    #[pallet::type_value]
    pub fn DefaultBonds<T: Config>() -> Vec<(u16, u16)> {
        vec![]
    }
    /// Value definition for weights with vector of (u16, u16).
    #[pallet::type_value]
    pub fn DefaultWeights<T: Config>() -> Vec<(u16, u16)> {
        vec![]
    }
    /// Default value for key with type T::AccountId derived from trailing zeroes.
    #[pallet::type_value]
    pub fn DefaultKey<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
            .expect("trailing zeroes always produce a valid account ID; qed")
    }

    #[pallet::storage] // --- DMAP ( netuid, hotkey ) --> uid
    pub type Uids<T: Config> =
        StorageDoubleMap<_, Identity, u16, Blake2_128Concat, T::AccountId, u16, OptionQuery>;
    #[pallet::storage] // --- DMAP ( netuid, uid ) --> hotkey
    pub type Keys<T: Config> =
        StorageDoubleMap<_, Identity, u16, Identity, u16, T::AccountId, ValueQuery, DefaultKey<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> (hotkey, se, ve)
    pub type LoadedEmission<T: Config> =
        StorageMap<_, Identity, u16, Vec<(T::AccountId, u64, u64)>, OptionQuery>;

    #[pallet::storage] // --- DMAP ( netuid ) --> active
    pub(super) type Active<T: Config> =
        StorageMap<_, Identity, u16, Vec<bool>, ValueQuery, EmptyBoolVec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> rank
    pub(super) type Rank<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> trust
    pub(super) type Trust<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> consensus
    pub(super) type Consensus<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> incentive
    pub(super) type Incentive<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> dividends
    pub(super) type Dividends<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> emission
    pub(super) type Emission<T: Config> =
        StorageMap<_, Identity, u16, Vec<u64>, ValueQuery, EmptyU64Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> last_update
    pub(super) type LastUpdate<T: Config> =
        StorageMap<_, Identity, u16, Vec<u64>, ValueQuery, EmptyU64Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> validator_trust
    pub(super) type ValidatorTrust<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> pruning_scores
    pub(super) type PruningScores<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> validator_permit
    pub(super) type ValidatorPermit<T: Config> =
        StorageMap<_, Identity, u16, Vec<bool>, ValueQuery, EmptyBoolVec<T>>;

    #[pallet::storage] // --- DMAP ( netuid, uid ) --> weights
    pub(super) type Weights<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Identity,
        u16,
        Vec<(u16, u16)>,
        ValueQuery,
        DefaultWeights<T>,
    >;
    #[pallet::storage] // --- DMAP ( netuid, uid ) --> bonds
    pub(super) type Bonds<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Identity,
        u16,
        Vec<(u16, u16)>,
        ValueQuery,
        DefaultBonds<T>,
    >;

    #[pallet::storage] // --- Storage for migration run status
    pub type HasMigrationRun<T: Config> = StorageMap<_, Identity, Vec<u8>, bool, ValueQuery>;

    /// ==================
    /// ==== Genesis =====
    /// ==================

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// Stakes record in genesis.
        pub stakes: Vec<(T::AccountId, Vec<(T::AccountId, (u64, u16))>)>,
        /// The total issued balance in genesis
        pub balances_issuance: u64,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                stakes: Default::default(),
                balances_issuance: 0,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Set initial total issuance from balances
            TotalIssuance::<T>::put(self.balances_issuance);

            // Subnet config values
            let netuid: u16 = 3;
            let tempo = 99;
            let max_uids = 4096;

            // The functions for initializing new networks/setting defaults cannot be run directly from genesis functions like extrinsics would
            // --- Set this network uid to alive.
            NetworksAdded::<T>::insert(netuid, true);

            // --- Fill tempo memory item.
            Tempo::<T>::insert(netuid, tempo);

            // --- Fill modality item.
            // Only modality 0 exists (text)
            NetworkModality::<T>::insert(netuid, 0);

            // Make network parameters explicit.
            if !Tempo::<T>::contains_key(netuid) {
                Tempo::<T>::insert(netuid, Tempo::<T>::get(netuid));
            }
            if !Kappa::<T>::contains_key(netuid) {
                Kappa::<T>::insert(netuid, Kappa::<T>::get(netuid));
            }
            if !Difficulty::<T>::contains_key(netuid) {
                Difficulty::<T>::insert(netuid, Difficulty::<T>::get(netuid));
            }
            if !MaxAllowedUids::<T>::contains_key(netuid) {
                MaxAllowedUids::<T>::insert(netuid, MaxAllowedUids::<T>::get(netuid));
            }
            if !ImmunityPeriod::<T>::contains_key(netuid) {
                ImmunityPeriod::<T>::insert(netuid, ImmunityPeriod::<T>::get(netuid));
            }
            if !ActivityCutoff::<T>::contains_key(netuid) {
                ActivityCutoff::<T>::insert(netuid, ActivityCutoff::<T>::get(netuid));
            }
            if !EmissionValues::<T>::contains_key(netuid) {
                EmissionValues::<T>::insert(netuid, EmissionValues::<T>::get(netuid));
            }
            if !MaxWeightsLimit::<T>::contains_key(netuid) {
                MaxWeightsLimit::<T>::insert(netuid, MaxWeightsLimit::<T>::get(netuid));
            }
            if !MinAllowedWeights::<T>::contains_key(netuid) {
                MinAllowedWeights::<T>::insert(netuid, MinAllowedWeights::<T>::get(netuid));
            }
            if !RegistrationsThisInterval::<T>::contains_key(netuid) {
                RegistrationsThisInterval::<T>::insert(
                    netuid,
                    RegistrationsThisInterval::<T>::get(netuid),
                );
            }
            if !POWRegistrationsThisInterval::<T>::contains_key(netuid) {
                POWRegistrationsThisInterval::<T>::insert(
                    netuid,
                    POWRegistrationsThisInterval::<T>::get(netuid),
                );
            }
            if !BurnRegistrationsThisInterval::<T>::contains_key(netuid) {
                BurnRegistrationsThisInterval::<T>::insert(
                    netuid,
                    BurnRegistrationsThisInterval::<T>::get(netuid),
                );
            }

            // Set max allowed uids
            MaxAllowedUids::<T>::insert(netuid, max_uids);

            let mut next_uid = 0u16;

            for (coldkey, hotkeys) in self.stakes.iter() {
                for (hotkey, stake_uid) in hotkeys.iter() {
                    let (stake, uid) = stake_uid;

                    // Expand Yuma Consensus with new position.
                    Rank::<T>::mutate(netuid, |v| v.push(0));
                    Trust::<T>::mutate(netuid, |v| v.push(0));
                    Active::<T>::mutate(netuid, |v| v.push(true));
                    Emission::<T>::mutate(netuid, |v| v.push(0));
                    Consensus::<T>::mutate(netuid, |v| v.push(0));
                    Incentive::<T>::mutate(netuid, |v| v.push(0));
                    Dividends::<T>::mutate(netuid, |v| v.push(0));
                    LastUpdate::<T>::mutate(netuid, |v| v.push(0));
                    PruningScores::<T>::mutate(netuid, |v| v.push(0));
                    ValidatorTrust::<T>::mutate(netuid, |v| v.push(0));
                    ValidatorPermit::<T>::mutate(netuid, |v| v.push(false));

                    // Insert account information.
                    Keys::<T>::insert(netuid, uid, hotkey.clone()); // Make hotkey - uid association.
                    Uids::<T>::insert(netuid, hotkey.clone(), uid); // Make uid - hotkey association.
                    BlockAtRegistration::<T>::insert(netuid, uid, 0); // Fill block at registration.
                    IsNetworkMember::<T>::insert(hotkey.clone(), netuid, true); // Fill network is member.

                    // Fill stake information.
                    Owner::<T>::insert(hotkey.clone(), coldkey.clone());

                    // Update OwnedHotkeys map
                    let mut hotkeys = OwnedHotkeys::<T>::get(coldkey);
                    if !hotkeys.contains(hotkey) {
                        hotkeys.push(hotkey.clone());
                        OwnedHotkeys::<T>::insert(coldkey, hotkeys);
                    }

                    TotalHotkeyStake::<T>::insert(hotkey.clone(), stake);
                    TotalColdkeyStake::<T>::insert(
                        coldkey.clone(),
                        TotalColdkeyStake::<T>::get(coldkey).saturating_add(*stake),
                    );

                    // Update total issuance value
                    TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_add(*stake));

                    Stake::<T>::insert(hotkey.clone(), coldkey.clone(), stake);

                    // Update StakingHotkeys map
                    let mut staking_hotkeys = StakingHotkeys::<T>::get(coldkey);
                    if !staking_hotkeys.contains(hotkey) {
                        staking_hotkeys.push(hotkey.clone());
                        StakingHotkeys::<T>::insert(coldkey, staking_hotkeys);
                    }

                    next_uid = next_uid.checked_add(1).expect(
                        "should not have total number of hotkey accounts larger than u16::MAX",
                    );
                }
            }

            // Set correct length for Subnet neurons
            SubnetworkN::<T>::insert(netuid, next_uid);

            // --- Increase total network count.
            TotalNetworks::<T>::mutate(|n| {
                *n = n.checked_add(1).expect(
                    "should not have total number of networks larger than u16::MAX in genesis",
                )
            });

            // Get the root network uid.
            let root_netuid: u16 = 0;

            // Set the root network as added.
            NetworksAdded::<T>::insert(root_netuid, true);

            // Increment the number of total networks.
            TotalNetworks::<T>::mutate(|n| {
                *n = n.checked_add(1).expect(
                    "should not have total number of networks larger than u16::MAX in genesis",
                )
            });
            // Set the number of validators to 1.
            SubnetworkN::<T>::insert(root_netuid, 0);

            // Set the maximum number to the number of senate members.
            MaxAllowedUids::<T>::insert(root_netuid, 64u16);

            // Set the maximum number to the number of validators to all members.
            MaxAllowedValidators::<T>::insert(root_netuid, 64u16);

            // Set the min allowed weights to zero, no weights restrictions.
            MinAllowedWeights::<T>::insert(root_netuid, 0);

            // Set the max weight limit to infinity, no weight restrictions.
            MaxWeightsLimit::<T>::insert(root_netuid, u16::MAX);

            // Add default root tempo.
            Tempo::<T>::insert(root_netuid, 100);

            // Set the root network as open.
            NetworkRegistrationAllowed::<T>::insert(root_netuid, true);

            // Set target registrations for validators as 1 per block.
            TargetRegistrationsPerInterval::<T>::insert(root_netuid, 1);
        }
    }

    // ================
    // ==== Hooks =====
    // ================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_idle(_n: BlockNumberFor<T>, _remaining_weight: Weight) -> Weight {
            Weight::zero()
        }

        fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
            let mut total_weight = Weight::zero();

            // Create a Weight::MAX value to pass to swap_coldkeys_this_block
            let max_weight = Weight::MAX;

            // Perform coldkey swapping
            let swap_weight = match Self::swap_coldkeys_this_block(&max_weight) {
                Ok(weight_used) => weight_used,
                Err(e) => {
                    log::error!("Error while swapping coldkeys: {:?}", e);
                    Weight::zero()
                }
            };
            total_weight = total_weight.saturating_add(swap_weight);

            // Perform block step
            let block_step_result = Self::block_step();
            match block_step_result {
                Ok(_) => {
                    log::debug!("Successfully ran block step.");
                    total_weight = total_weight.saturating_add(
                        Weight::from_parts(110_634_229_000_u64, 0)
                            .saturating_add(T::DbWeight::get().reads(8304_u64))
                            .saturating_add(T::DbWeight::get().writes(110_u64)),
                    );
                }
                Err(e) => {
                    log::error!("Error while stepping block: {:?}", e);
                    total_weight = total_weight.saturating_add(
                        Weight::from_parts(110_634_229_000_u64, 0)
                            .saturating_add(T::DbWeight::get().reads(8304_u64))
                            .saturating_add(T::DbWeight::get().writes(110_u64)),
                    );
                }
            }

            total_weight
        }

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            // --- Migrate storage
            use crate::migration;
            let mut weight = frame_support::weights::Weight::from_parts(0, 0);

            // Hex encoded foundation coldkey
            let hex = hex_literal::hex![
                "feabaafee293d3b76dae304e2f9d885f77d2b17adab9e17e921b321eccd61c77"
            ];
            weight = weight
                // Initializes storage version (to 1)
                .saturating_add(migration::migrate_to_v1_separate_emission::<T>())
                // Storage version v1 -> v2
                .saturating_add(migration::migrate_to_v2_fixed_total_stake::<T>())
                // Doesn't check storage version. TODO: Remove after upgrade
                .saturating_add(migration::migrate_create_root_network::<T>())
                // Storage version v2 -> v3
                .saturating_add(migration::migrate_transfer_ownership_to_foundation::<T>(
                    hex,
                ))
                // Storage version v3 -> v4
                .saturating_add(migration::migrate_delete_subnet_21::<T>())
                // Storage version v4 -> v5
                .saturating_add(migration::migrate_delete_subnet_3::<T>())
                // Doesn't check storage version. TODO: Remove after upgrade
                .saturating_add(migration::migration5_total_issuance::<T>(false))
                // Populate OwnedHotkeys map for coldkey swap. Doesn't update storage vesion.
                .saturating_add(migration::migrate_populate_owned::<T>())
                // Populate StakingHotkeys map for coldkey swap. Doesn't update storage vesion.
                .saturating_add(migration::migrate_populate_staking_hotkeys::<T>())
                // Fix total coldkey stake.
                .saturating_add(migration::migrate_fix_total_coldkey_stake::<T>());

            weight
        }
    }

    /// Dispatchable functions allow users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// --- Sets the caller weights for the incentive mechanism. The call can be
        /// made from the hotkey account so is potentially insecure, however, the damage
        /// of changing weights is minimal if caught early. This function includes all the
        /// checks that the passed weights meet the requirements. Stored as u16s they represent
        /// rational values in the range [0,1] which sum to 1 and can be interpreted as
        /// probabilities. The specific weights determine how inflation propagates outward
        /// from this peer.
        ///
        /// Note: The 16 bit integers weights should represent 1.0 as the max u16.
        /// However, the function normalizes all integers to u16_max anyway. This means that if the sum of all
        /// elements is larger or smaller than the amount of elements * u16_max, all elements
        /// will be corrected for this deviation.
        ///
        /// # Args:
        /// * `origin`: (<T as frame_system::Config>Origin):
        ///     - The caller, a hotkey who wishes to set their weights.
        ///
        /// * `netuid` (u16):
        /// 	- The network uid we are setting these weights on.
        ///
        /// * `dests` (Vec<u16>):
        /// 	- The edge endpoint for the weight, i.e. j for w_ij.
        ///
        /// * 'weights' (Vec<u16>):
        /// 	- The u16 integer encoded weights. Interpreted as rational
        /// 		values in the range [0,1]. They must sum to in32::MAX.
        ///
        /// * 'version_key' ( u64 ):
        /// 	- The network version key to check if the validator is up to date.
        ///
        /// # Event:
        /// * WeightsSet;
        /// 	- On successfully setting the weights on chain.
        ///
        /// # Raises:
        /// * 'SubNetworkDoesNotExist':
        /// 	- Attempting to set weights on a non-existent network.
        ///
        /// * 'NotRegistered':
        /// 	- Attempting to set weights from a non registered account.
        ///
        /// * 'WeightVecNotEqualSize':
        /// 	- Attempting to set weights with uids not of same length.
        ///
        /// * 'DuplicateUids':
        /// 	- Attempting to set weights with duplicate uids.
        ///
        ///     * 'UidsLengthExceedUidsInSubNet':
        /// 	- Attempting to set weights above the max allowed uids.
        ///
        /// * 'UidVecContainInvalidOne':
        /// 	- Attempting to set weights with invalid uids.
        ///
        /// * 'WeightVecLengthIsLow':
        /// 	- Attempting to set weights with fewer weights than min.
        ///
        /// * 'MaxWeightExceeded':
        /// 	- Attempting to set weights with max value exceeding limit.
        #[pallet::call_index(0)]
        #[pallet::weight((Weight::from_parts(22_060_000_000, 0)
        .saturating_add(T::DbWeight::get().reads(4106))
        .saturating_add(T::DbWeight::get().writes(2)), DispatchClass::Normal, Pays::No))]
        pub fn set_weights(
            origin: OriginFor<T>,
            netuid: u16,
            dests: Vec<u16>,
            weights: Vec<u16>,
            version_key: u64,
        ) -> DispatchResult {
            if !Self::get_commit_reveal_weights_enabled(netuid) {
                return Self::do_set_weights(origin, netuid, dests, weights, version_key);
            }

            Err(Error::<T>::CommitRevealEnabled.into())
        }

        /// ---- Used to commit a hash of your weight values to later be revealed.
        ///
        /// # Args:
        /// * `origin`: (`<T as frame_system::Config>::RuntimeOrigin`):
        ///   - The signature of the committing hotkey.
        ///
        /// * `netuid` (`u16`):
        ///   - The u16 network identifier.
        ///
        /// * `commit_hash` (`H256`):
        ///   - The hash representing the committed weights.
        ///
        /// # Raises:
        /// * `WeightsCommitNotAllowed`:
        ///   - Attempting to commit when it is not allowed.
        ///
        #[pallet::call_index(96)]
        #[pallet::weight((Weight::from_parts(46_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(1))
		.saturating_add(T::DbWeight::get().writes(1)), DispatchClass::Normal, Pays::No))]
        pub fn commit_weights(
            origin: T::RuntimeOrigin,
            netuid: u16,
            commit_hash: H256,
        ) -> DispatchResult {
            Self::do_commit_weights(origin, netuid, commit_hash)
        }

        /// ---- Used to reveal the weights for a previously committed hash.
        ///
        /// # Args:
        /// * `origin`: (`<T as frame_system::Config>::RuntimeOrigin`):
        ///   - The signature of the revealing hotkey.
        ///
        /// * `netuid` (`u16`):
        ///   - The u16 network identifier.
        ///
        /// * `uids` (`Vec<u16>`):
        ///   - The uids for the weights being revealed.
        ///
        /// * `values` (`Vec<u16>`):
        ///   - The values of the weights being revealed.
        ///
        /// * `salt` (`Vec<u8>`):
        ///   - The random salt to protect from brute-force guessing attack in case of small weight changes bit-wise.
        ///
        /// * `version_key` (`u64`):
        ///   - The network version key.
        ///
        /// # Raises:
        /// * `NoWeightsCommitFound`:
        ///   - Attempting to reveal weights without an existing commit.
        ///
        /// * `InvalidRevealCommitHashNotMatchTempo`:
        ///   - Attempting to reveal weights outside the valid tempo.
        ///
        /// * `InvalidRevealCommitHashNotMatch`:
        ///   - The revealed hash does not match the committed hash.
        ///
        #[pallet::call_index(97)]
        #[pallet::weight((Weight::from_parts(103_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(11))
		.saturating_add(T::DbWeight::get().writes(3)), DispatchClass::Normal, Pays::No))]
        pub fn reveal_weights(
            origin: T::RuntimeOrigin,
            netuid: u16,
            uids: Vec<u16>,
            values: Vec<u16>,
            salt: Vec<u16>,
            version_key: u64,
        ) -> DispatchResult {
            Self::do_reveal_weights(origin, netuid, uids, values, salt, version_key)
        }

        /// # Args:
        /// * `origin`: (<T as frame_system::Config>Origin):
        /// 	- The caller, a hotkey who wishes to set their weights.
        ///
        /// * `netuid` (u16):
        /// 	- The network uid we are setting these weights on.
        ///
        /// * `hotkey` (T::AccountId):
        /// 	- The hotkey associated with the operation and the calling coldkey.
        ///
        /// * `dests` (Vec<u16>):
        /// 	- The edge endpoint for the weight, i.e. j for w_ij.
        ///
        /// * 'weights' (Vec<u16>):
        /// 	- The u16 integer encoded weights. Interpreted as rational
        /// 		values in the range [0,1]. They must sum to in32::MAX.
        ///
        /// * 'version_key' ( u64 ):
        /// 	- The network version key to check if the validator is up to date.
        ///
        /// # Event:
        ///
        /// * WeightsSet;
        /// 	- On successfully setting the weights on chain.
        ///
        /// # Raises:
        ///
        /// * NonAssociatedColdKey;
        /// 	- Attempting to set weights on a non-associated cold key.
        ///
        /// * 'SubNetworkDoesNotExist':
        /// 	- Attempting to set weights on a non-existent network.
        ///
        /// * 'NotRootSubnet':
        /// 	- Attempting to set weights on a subnet that is not the root network.
        ///
        /// * 'WeightVecNotEqualSize':
        /// 	- Attempting to set weights with uids not of same length.
        ///
        /// * 'UidVecContainInvalidOne':
        /// 	- Attempting to set weights with invalid uids.
        ///
        /// * 'NotRegistered':
        /// 	- Attempting to set weights from a non registered account.
        ///
        /// * 'WeightVecLengthIsLow':
        /// 	- Attempting to set weights with fewer weights than min.
        ///
        ///  * 'IncorrectWeightVersionKey':
        ///      - Attempting to set weights with the incorrect network version key.
        ///
        ///  * 'SettingWeightsTooFast':
        ///      - Attempting to set weights too fast.
        ///
        /// * 'WeightVecLengthIsLow':
        /// 	- Attempting to set weights with fewer weights than min.
        ///
        /// * 'MaxWeightExceeded':
        /// 	- Attempting to set weights with max value exceeding limit.
        ///
        #[pallet::call_index(8)]
        #[pallet::weight((Weight::from_parts(10_151_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(4104))
		.saturating_add(T::DbWeight::get().writes(2)), DispatchClass::Normal, Pays::No))]
        pub fn set_root_weights(
            origin: OriginFor<T>,
            netuid: u16,
            hotkey: T::AccountId,
            dests: Vec<u16>,
            weights: Vec<u16>,
            version_key: u64,
        ) -> DispatchResult {
            Self::do_set_root_weights(origin, netuid, hotkey, dests, weights, version_key)
        }

        /// --- Sets the key as a delegate.
        ///
        /// # Args:
        /// * 'origin': (<T as frame_system::Config>Origin):
        /// 	- The signature of the caller's coldkey.
        ///
        /// * 'hotkey' (T::AccountId):
        /// 	- The hotkey we are delegating (must be owned by the coldkey.)
        ///
        /// * 'take' (u64):
        /// 	- The stake proportion that this hotkey takes from delegations.
        ///
        /// # Event:
        /// * DelegateAdded;
        /// 	- On successfully setting a hotkey as a delegate.
        ///
        /// # Raises:
        /// * 'NotRegistered':
        /// 	- The hotkey we are delegating is not registered on the network.
        ///
        /// * 'NonAssociatedColdKey':
        /// 	- The hotkey we are delegating is not owned by the calling coldket.
        ///
        #[pallet::call_index(1)]
        #[pallet::weight((Weight::from_parts(79_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(6))
		.saturating_add(T::DbWeight::get().writes(3)), DispatchClass::Normal, Pays::No))]
        pub fn become_delegate(origin: OriginFor<T>, hotkey: T::AccountId) -> DispatchResult {
            Self::do_become_delegate(origin, hotkey, Self::get_default_take())
        }

        /// --- Allows delegates to decrease its take value.
        ///
        /// # Args:
        /// * 'origin': (<T as frame_system::Config>::Origin):
        /// 	- The signature of the caller's coldkey.
        ///
        /// * 'hotkey' (T::AccountId):
        /// 	- The hotkey we are delegating (must be owned by the coldkey.)
        ///
        /// * 'netuid' (u16):
        /// 	- Subnet ID to decrease take for
        ///
        /// * 'take' (u16):
        /// 	- The new stake proportion that this hotkey takes from delegations.
        ///        The new value can be between 0 and 11_796 and should be strictly
        ///        lower than the previous value. It T is the new value (rational number),
        ///        the the parameter is calculated as [65535 * T]. For example, 1% would be
        ///        [0.01 * 65535] = [655.35] = 655
        ///
        /// # Event:
        /// * TakeDecreased;
        /// 	- On successfully setting a decreased take for this hotkey.
        ///
        /// # Raises:
        /// * 'NotRegistered':
        /// 	- The hotkey we are delegating is not registered on the network.
        ///
        /// * 'NonAssociatedColdKey':
        /// 	- The hotkey we are delegating is not owned by the calling coldkey.
        ///
        /// * 'DelegateTakeTooLow':
        /// 	- The delegate is setting a take which is not lower than the previous.
        ///
        #[pallet::call_index(65)]
        #[pallet::weight((0, DispatchClass::Normal, Pays::No))]
        pub fn decrease_take(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            take: u16,
        ) -> DispatchResult {
            Self::do_decrease_take(origin, hotkey, take)
        }

        /// --- Allows delegates to increase its take value. This call is rate-limited.
        ///
        /// # Args:
        /// * 'origin': (<T as frame_system::Config>::Origin):
        /// 	- The signature of the caller's coldkey.
        ///
        /// * 'hotkey' (T::AccountId):
        /// 	- The hotkey we are delegating (must be owned by the coldkey.)
        ///
        /// * 'take' (u16):
        /// 	- The new stake proportion that this hotkey takes from delegations.
        ///        The new value can be between 0 and 11_796 and should be strictly
        ///        greater than the previous value. T is the new value (rational number),
        ///        the the parameter is calculated as [65535 * T]. For example, 1% would be
        ///        [0.01 * 65535] = [655.35] = 655
        ///
        /// # Event:
        /// * TakeIncreased;
        /// 	- On successfully setting a increased take for this hotkey.
        ///
        /// # Raises:
        /// * 'NotRegistered':
        /// 	- The hotkey we are delegating is not registered on the network.
        ///
        /// * 'NonAssociatedColdKey':
        /// 	- The hotkey we are delegating is not owned by the calling coldkey.
        ///
        /// * 'DelegateTakeTooHigh':
        /// 	- The delegate is setting a take which is not greater than the previous.
        ///
        #[pallet::call_index(66)]
        #[pallet::weight((0, DispatchClass::Normal, Pays::No))]
        pub fn increase_take(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            take: u16,
        ) -> DispatchResult {
            Self::do_increase_take(origin, hotkey, take)
        }

        /// --- Adds stake to a hotkey. The call is made from the
        /// coldkey account linked in the hotkey.
        /// Only the associated coldkey is allowed to make staking and
        /// unstaking requests. This protects the neuron against
        /// attacks on its hotkey running in production code.
        ///
        /// # Args:
        ///  * 'origin': (<T as frame_system::Config>Origin):
        /// 	- The signature of the caller's coldkey.
        ///
        ///  * 'hotkey' (T::AccountId):
        /// 	- The associated hotkey account.
        ///
        ///  * 'amount_staked' (u64):
        /// 	- The amount of stake to be added to the hotkey staking account.
        ///
        /// # Event:
        ///  * StakeAdded;
        /// 	- On the successfully adding stake to a global account.
        ///
        /// # Raises:
        ///  * 'NotEnoughBalanceToStake':
        /// 	- Not enough balance on the coldkey to add onto the global account.
        ///
        ///  * 'NonAssociatedColdKey':
        /// 	- The calling coldkey is not associated with this hotkey.
        ///
        ///  * 'BalanceWithdrawalError':
        ///  	- Errors stemming from transaction pallet.
        ///
        #[pallet::call_index(2)]
        #[pallet::weight((Weight::from_parts(124_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(10))
		.saturating_add(T::DbWeight::get().writes(7)), DispatchClass::Normal, Pays::No))]
        pub fn add_stake(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            amount_staked: u64,
        ) -> DispatchResult {
            Self::do_add_stake(origin, hotkey, amount_staked)
        }

        /// Remove stake from the staking account. The call must be made
        /// from the coldkey account attached to the neuron metadata. Only this key
        /// has permission to make staking and unstaking requests.
        ///
        /// # Args:
        /// * 'origin': (<T as frame_system::Config>Origin):
        /// 	- The signature of the caller's coldkey.
        ///
        /// * 'hotkey' (T::AccountId):
        /// 	- The associated hotkey account.
        ///
        /// * 'amount_unstaked' (u64):
        /// 	- The amount of stake to be added to the hotkey staking account.
        ///
        /// # Event:
        /// * StakeRemoved;
        /// 	- On the successfully removing stake from the hotkey account.
        ///
        /// # Raises:
        /// * 'NotRegistered':
        /// 	- Thrown if the account we are attempting to unstake from is non existent.
        ///
        /// * 'NonAssociatedColdKey':
        /// 	- Thrown if the coldkey does not own the hotkey we are unstaking from.
        ///
        /// * 'NotEnoughStakeToWithdraw':
        /// 	- Thrown if there is not enough stake on the hotkey to withdwraw this amount.
        ///
        #[pallet::call_index(3)]
        #[pallet::weight((Weight::from_parts(111_000_000, 0)
		.saturating_add(Weight::from_parts(0, 43991))
		.saturating_add(T::DbWeight::get().reads(10))
		.saturating_add(T::DbWeight::get().writes(7)), DispatchClass::Normal, Pays::No))]
        pub fn remove_stake(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            amount_unstaked: u64,
        ) -> DispatchResult {
            Self::do_remove_stake(origin, hotkey, amount_unstaked)
        }

        /// Serves or updates axon /promethteus information for the neuron associated with the caller. If the caller is
        /// already registered the metadata is updated. If the caller is not registered this call throws NotRegistered.
        ///
        /// # Args:
        /// * 'origin': (<T as frame_system::Config>Origin):
        /// 	- The signature of the caller.
        ///
        /// * 'netuid' (u16):
        /// 	- The u16 network identifier.
        ///
        /// * 'version' (u64):
        /// 	- The bittensor version identifier.
        ///
        /// * 'ip' (u64):
        /// 	- The endpoint ip information as a u128 encoded integer.
        ///
        /// * 'port' (u16):
        /// 	- The endpoint port information as a u16 encoded integer.
        ///
        /// * 'ip_type' (u8):
        /// 	- The endpoint ip version as a u8, 4 or 6.
        ///
        /// * 'protocol' (u8):
        /// 	- UDP:1 or TCP:0
        ///
        /// * 'placeholder1' (u8):
        /// 	- Placeholder for further extra params.
        ///
        /// * 'placeholder2' (u8):
        /// 	- Placeholder for further extra params.
        ///
        /// # Event:
        /// * AxonServed;
        /// 	- On successfully serving the axon info.
        ///
        /// # Raises:
        /// * 'SubNetworkDoesNotExist':
        /// 	- Attempting to set weights on a non-existent network.
        ///
        /// * 'NotRegistered':
        /// 	- Attempting to set weights from a non registered account.
        ///
        /// * 'InvalidIpType':
        /// 	- The ip type is not 4 or 6.
        ///
        /// * 'InvalidIpAddress':
        /// 	- The numerically encoded ip address does not resolve to a proper ip.
        ///
        /// * 'ServingRateLimitExceeded':
        /// 	- Attempting to set prometheus information withing the rate limit min.
        ///
        #[pallet::call_index(4)]
        #[pallet::weight((Weight::from_parts(46_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(4))
		.saturating_add(T::DbWeight::get().writes(1)), DispatchClass::Normal, Pays::No))]
        pub fn serve_axon(
            origin: OriginFor<T>,
            netuid: u16,
            version: u32,
            ip: u128,
            port: u16,
            ip_type: u8,
            protocol: u8,
            placeholder1: u8,
            placeholder2: u8,
        ) -> DispatchResult {
            Self::do_serve_axon(
                origin,
                netuid,
                version,
                ip,
                port,
                ip_type,
                protocol,
                placeholder1,
                placeholder2,
            )
        }

        /// ---- Set prometheus information for the neuron.
        /// # Args:
        /// * 'origin': (<T as frame_system::Config>Origin):
        /// 	- The signature of the calling hotkey.
        ///
        /// * 'netuid' (u16):
        /// 	- The u16 network identifier.
        ///
        /// * 'version' (u16):
        /// 	-  The bittensor version identifier.
        ///
        /// * 'ip' (u128):
        /// 	- The prometheus ip information as a u128 encoded integer.
        ///
        /// * 'port' (u16):
        /// 	- The prometheus port information as a u16 encoded integer.
        ///
        /// * 'ip_type' (u8):
        /// 	- The ip type v4 or v6.
        ///
        #[pallet::call_index(5)]
        #[pallet::weight((Weight::from_parts(45_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(4))
		.saturating_add(T::DbWeight::get().writes(1)), DispatchClass::Normal, Pays::No))]
        pub fn serve_prometheus(
            origin: OriginFor<T>,
            netuid: u16,
            version: u32,
            ip: u128,
            port: u16,
            ip_type: u8,
        ) -> DispatchResult {
            Self::do_serve_prometheus(origin, netuid, version, ip, port, ip_type)
        }

        /// ---- Registers a new neuron to the subnetwork.
        ///
        /// # Args:
        /// * 'origin': (<T as frame_system::Config>Origin):
        /// 	- The signature of the calling hotkey.
        ///
        /// * 'netuid' (u16):
        /// 	- The u16 network identifier.
        ///
        /// * 'block_number' ( u64 ):
        /// 	- Block hash used to prove work done.
        ///
        /// * 'nonce' ( u64 ):
        /// 	- Positive integer nonce used in POW.
        ///
        /// * 'work' ( Vec<u8> ):
        /// 	- Vector encoded bytes representing work done.
        ///
        /// * 'hotkey' ( T::AccountId ):
        /// 	- Hotkey to be registered to the network.
        ///
        /// * 'coldkey' ( T::AccountId ):
        /// 	- Associated coldkey account.
        ///
        /// # Event:
        /// * NeuronRegistered;
        /// 	- On successfully registering a uid to a neuron slot on a subnetwork.
        ///
        /// # Raises:
        /// * 'SubNetworkDoesNotExist':
        /// 	- Attempting to register to a non existent network.
        ///
        /// * 'TooManyRegistrationsThisBlock':
        /// 	- This registration exceeds the total allowed on this network this block.
        ///
        /// * 'HotKeyAlreadyRegisteredInSubNet':
        /// 	- The hotkey is already registered on this network.
        ///
        /// * 'InvalidWorkBlock':
        /// 	- The work has been performed on a stale, future, or non existent block.
        ///
        /// * 'InvalidDifficulty':
        /// 	- The work does not match the difficulty.
        ///
        /// * 'InvalidSeal':
        /// 	- The seal is incorrect.
        ///
        #[pallet::call_index(6)]
        #[pallet::weight((Weight::from_parts(192_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(24))
		.saturating_add(T::DbWeight::get().writes(22)), DispatchClass::Normal, Pays::No))]
        pub fn register(
            origin: OriginFor<T>,
            netuid: u16,
            block_number: u64,
            nonce: u64,
            work: Vec<u8>,
            hotkey: T::AccountId,
            coldkey: T::AccountId,
        ) -> DispatchResult {
            Self::do_registration(origin, netuid, block_number, nonce, work, hotkey, coldkey)
        }

        /// Register the hotkey to root network
        #[pallet::call_index(62)]
        #[pallet::weight((Weight::from_parts(164_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(23))
		.saturating_add(T::DbWeight::get().writes(20)), DispatchClass::Normal, Pays::No))]
        pub fn root_register(origin: OriginFor<T>, hotkey: T::AccountId) -> DispatchResult {
            Self::do_root_register(origin, hotkey)
        }

        /// Attempt to adjust the senate membership to include a hotkey
        #[pallet::call_index(63)]
        #[pallet::weight((Weight::from_parts(0, 0)
		.saturating_add(T::DbWeight::get().reads(0))
		.saturating_add(T::DbWeight::get().writes(0)), DispatchClass::Normal, Pays::Yes))]
        pub fn adjust_senate(origin: OriginFor<T>, hotkey: T::AccountId) -> DispatchResult {
            Self::do_adjust_senate(origin, hotkey)
        }

        /// User register a new subnetwork via burning token
        #[pallet::call_index(7)]
        #[pallet::weight((Weight::from_parts(177_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(26))
		.saturating_add(T::DbWeight::get().writes(24)), DispatchClass::Normal, Pays::No))]
        pub fn burned_register(
            origin: OriginFor<T>,
            netuid: u16,
            hotkey: T::AccountId,
        ) -> DispatchResult {
            Self::do_burned_registration(origin, netuid, hotkey)
        }

        /// The extrinsic for user to change its hotkey
        ///#[pallet::call_index(70)]
        ///#[pallet::weight((Weight::from_parts(1_940_000_000, 0)
        ///.saturating_add(T::DbWeight::get().reads(272))
        ///.saturating_add(T::DbWeight::get().writes(527)), DispatchClass::Operational, Pays::No))]
        ///pub fn swap_hotkey(
        ///    origin: OriginFor<T>,
        ///    hotkey: T::AccountId,
        ///    new_hotkey: T::AccountId,
        ///) -> DispatchResultWithPostInfo {
        ///    Self::do_swap_hotkey(origin, &hotkey, &new_hotkey)
        ///}

        /// The extrinsic for user to change the coldkey associated with their account.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the call, must be signed by the old coldkey.
        /// * `old_coldkey` - The current coldkey associated with the account.
        /// * `new_coldkey` - The new coldkey to be associated with the account.
        ///
        /// # Returns
        ///
        /// Returns a `DispatchResultWithPostInfo` indicating success or failure of the operation.
        ///
        /// # Weight
        ///
        /// Weight is calculated based on the number of database reads and writes.
        #[pallet::call_index(71)]
        #[pallet::weight((Weight::from_parts(1_940_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(272))
		.saturating_add(T::DbWeight::get().writes(527)), DispatchClass::Operational, Pays::No))]
        pub fn swap_coldkey(
            origin: OriginFor<T>,
            new_coldkey: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            Self::do_swap_coldkey(origin, &new_coldkey)
        }
        /// Unstakes all tokens associated with a hotkey and transfers them to a new coldkey.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the call, must be signed by the current coldkey.
        /// * `hotkey` - The hotkey associated with the stakes to be unstaked.
        /// * `new_coldkey` - The new coldkey to receive the unstaked tokens.
        ///
        /// # Returns
        ///
        /// Returns a `DispatchResult` indicating success or failure of the operation.
        ///
        /// # Weight
        ///
        /// Weight is calculated based on the number of database reads and writes.
        #[cfg(test)]
        #[pallet::call_index(72)]
        #[pallet::weight((Weight::from_parts(21_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(3))
		.saturating_add(T::DbWeight::get().writes(3)), DispatchClass::Operational, Pays::No))]
        pub fn schedule_coldkey_swap(
            origin: OriginFor<T>,
            new_coldkey: T::AccountId,
            work: Vec<u8>,
            block_number: u64,
            nonce: u64,
        ) -> DispatchResult {
            // Attain the calling coldkey from the origin.
            let old_coldkey: T::AccountId = ensure_signed(origin)?;
            Self::do_schedule_coldkey_swap(&old_coldkey, &new_coldkey, work, block_number, nonce)
        }

        // ---- SUDO ONLY FUNCTIONS ------------------------------------------------------------

        // ==================================
        // ==== Parameter Sudo calls ========
        // ==================================
        // Each function sets the corresponding hyper paramter on the specified network
        // Args:
        // 	* 'origin': (<T as frame_system::Config>Origin):
        // 		- The caller, must be sudo.
        //
        // 	* `netuid` (u16):
        // 		- The network identifier.
        //
        // 	* `hyperparameter value` (u16):
        // 		- The value of the hyper parameter.
        //

        /// Authenticates a council proposal and dispatches a function call with `Root` origin.
        ///
        /// The dispatch origin for this call must be a council majority.
        ///
        /// ## Complexity
        /// - O(1).
        #[pallet::call_index(51)]
        #[pallet::weight((Weight::from_parts(0, 0), DispatchClass::Operational, Pays::No))]
        pub fn sudo(
            origin: OriginFor<T>,
            call: Box<T::SudoRuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            // This is a public call, so we ensure that the origin is a council majority.
            T::CouncilOrigin::ensure_origin(origin)?;

            let result = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
            let error = result.map(|_| ()).map_err(|e| e.error);
            Self::deposit_event(Event::Sudid(error));

            return result;
        }

        /// Authenticates a council proposal and dispatches a function call with `Root` origin.
        /// This function does not check the weight of the call, and instead allows the
        /// user to specify the weight of the call.
        ///
        /// The dispatch origin for this call must be a council majority.
        ///
        /// ## Complexity
        /// - O(1).
        #[allow(deprecated)]
        #[pallet::call_index(52)]
        #[pallet::weight((*weight, call.get_dispatch_info().class, Pays::No))]
        pub fn sudo_unchecked_weight(
            origin: OriginFor<T>,
            call: Box<T::SudoRuntimeCall>,
            weight: Weight,
        ) -> DispatchResultWithPostInfo {
            // We dont need to check the weight witness, suppress warning.
            // See https://github.com/paritytech/polkadot-sdk/pull/1818.
            let _ = weight;

            // This is a public call, so we ensure that the origin is a council majority.
            T::CouncilOrigin::ensure_origin(origin)?;

            let result = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
            let error = result.map(|_| ()).map_err(|e| e.error);
            Self::deposit_event(Event::Sudid(error));

            return result;
        }

        /// User vote on a proposal
        #[pallet::call_index(55)]
        #[pallet::weight((Weight::from_parts(0, 0)
		.saturating_add(Weight::from_parts(0, 0))
		.saturating_add(T::DbWeight::get().reads(0))
		.saturating_add(T::DbWeight::get().writes(0)), DispatchClass::Operational))]
        pub fn vote(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            proposal: T::Hash,
            #[pallet::compact] index: u32,
            approve: bool,
        ) -> DispatchResultWithPostInfo {
            Self::do_vote_root(origin, &hotkey, proposal, index, approve)
        }

        /// User register a new subnetwork
        #[pallet::call_index(59)]
        #[pallet::weight((Weight::from_parts(157_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(16))
		.saturating_add(T::DbWeight::get().writes(30)), DispatchClass::Operational, Pays::No))]
        pub fn register_network(origin: OriginFor<T>) -> DispatchResult {
            Self::user_add_network(origin)
        }

        /// Facility extrinsic for user to get taken from faucet
        /// It is only available when pow-faucet feature enabled
        /// Just deployed in testnet and devnet for testing purpose
        #[pallet::call_index(60)]
        #[pallet::weight((Weight::from_parts(91_000_000, 0)
        .saturating_add(T::DbWeight::get().reads(27))
		.saturating_add(T::DbWeight::get().writes(22)), DispatchClass::Normal, Pays::No))]
        pub fn faucet(
            origin: OriginFor<T>,
            block_number: u64,
            nonce: u64,
            work: Vec<u8>,
        ) -> DispatchResult {
            if cfg!(feature = "pow-faucet") {
                return Self::do_faucet(origin, block_number, nonce, work);
            }

            Err(Error::<T>::FaucetDisabled.into())
        }

        /// Remove a user's subnetwork
        /// The caller must be the owner of the network
        #[pallet::call_index(61)]
        #[pallet::weight((Weight::from_parts(119_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(6))
		.saturating_add(T::DbWeight::get().writes(31)), DispatchClass::Operational, Pays::No))]
        pub fn dissolve_network(origin: OriginFor<T>, netuid: u16) -> DispatchResult {
            Self::user_remove_network(origin, netuid)
        }

        /// Sets values for liquid alpha
        #[pallet::call_index(64)]
        #[pallet::weight((0, DispatchClass::Operational, Pays::No))]
        pub fn sudo_hotfix_swap_coldkey_delegates(
            _origin: OriginFor<T>,
            _old_coldkey: T::AccountId,
            _new_coldkey: T::AccountId,
        ) -> DispatchResult {
            Ok(())
        }
    }

    // ---- Subtensor helper functions.
    impl<T: Config> Pallet<T> {
        /// Returns the transaction priority for setting weights.
        pub fn get_priority_set_weights(hotkey: &T::AccountId, netuid: u16) -> u64 {
            if let Ok(uid) = Self::get_uid_for_net_and_hotkey(netuid, hotkey) {
                let _stake = Self::get_total_stake_for_hotkey(hotkey);
                let current_block_number: u64 = Self::get_current_block_as_u64();
                let default_priority: u64 =
                    current_block_number.saturating_sub(Self::get_last_update_for_uid(netuid, uid));
                return default_priority.saturating_add(u32::MAX as u64);
            }
            0
        }

        /// Is the caller allowed to set weights
        pub fn check_weights_min_stake(hotkey: &T::AccountId) -> bool {
            // Blacklist weights transactions for low stake peers.
            Self::get_total_stake_for_hotkey(hotkey) >= Self::get_weights_min_stake()
        }

        /// Helper function to check if register is allowed
        pub fn checked_allowed_register(netuid: u16) -> bool {
            if netuid == Self::get_root_netuid() {
                return false;
            }
            if !Self::if_subnet_exist(netuid) {
                return false;
            }
            if !Self::get_network_registration_allowed(netuid) {
                return false;
            }
            if Self::get_registrations_this_block(netuid)
                >= Self::get_max_registrations_per_block(netuid)
            {
                return false;
            }
            if Self::get_registrations_this_interval(netuid)
                >= Self::get_target_registrations_per_interval(netuid).saturating_mul(3)
            {
                return false;
            }
            true
        }
    }
}

/************************************************************
    CallType definition
************************************************************/
#[derive(Debug, PartialEq, Default)]
pub enum CallType {
    SetWeights,
    AddStake,
    RemoveStake,
    AddDelegate,
    Register,
    Serve,
    RegisterNetwork,
    #[default]
    Other,
}

#[freeze_struct("61e2b893d5ce6701")]
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub struct SubtensorSignedExtension<T: Config + Send + Sync + TypeInfo>(pub PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> Default for SubtensorSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Config + Send + Sync + TypeInfo> SubtensorSignedExtension<T>
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

    pub fn get_priority_set_weights(who: &T::AccountId, netuid: u16) -> u64 {
        Pallet::<T>::get_priority_set_weights(who, netuid)
    }

    pub fn check_weights_min_stake(who: &T::AccountId) -> bool {
        Pallet::<T>::check_weights_min_stake(who)
    }
}

impl<T: Config + Send + Sync + TypeInfo> sp_std::fmt::Debug for SubtensorSignedExtension<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "SubtensorSignedExtension")
    }
}

impl<T: Config + Send + Sync + TypeInfo + pallet_balances::Config> SignedExtension
    for SubtensorSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<BalancesCall<T>>,
{
    const IDENTIFIER: &'static str = "SubtensorSignedExtension";

    type AccountId = T::AccountId;
    type Call = T::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = (CallType, u64, Self::AccountId);

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        // Check if the call is one of the balance transfer types we want to reject
        if let Some(balances_call) = call.is_sub_type() {
            match balances_call {
                BalancesCall::transfer_allow_death { .. }
                | BalancesCall::transfer_keep_alive { .. }
                | BalancesCall::transfer_all { .. } => {
                    if Pallet::<T>::coldkey_in_arbitration(who) {
                        return Err(TransactionValidityError::Invalid(InvalidTransaction::Call));
                    }
                }
                _ => {} // Other Balances calls are allowed
            }
        }
        match call.is_sub_type() {
            Some(Call::commit_weights { netuid, .. }) => {
                if Self::check_weights_min_stake(who) {
                    let priority: u64 = Self::get_priority_set_weights(who, *netuid);
                    Ok(ValidTransaction {
                        priority,
                        longevity: 1,
                        ..Default::default()
                    })
                } else {
                    Err(InvalidTransaction::Call.into())
                }
            }
            Some(Call::reveal_weights { netuid, .. }) => {
                if Self::check_weights_min_stake(who) {
                    let priority: u64 = Self::get_priority_set_weights(who, *netuid);
                    Ok(ValidTransaction {
                        priority,
                        longevity: 1,
                        ..Default::default()
                    })
                } else {
                    Err(InvalidTransaction::Call.into())
                }
            }
            Some(Call::set_weights { netuid, .. }) => {
                if Self::check_weights_min_stake(who) {
                    let priority: u64 = Self::get_priority_set_weights(who, *netuid);
                    Ok(ValidTransaction {
                        priority,
                        longevity: 1,
                        ..Default::default()
                    })
                } else {
                    Err(InvalidTransaction::Call.into())
                }
            }
            Some(Call::set_root_weights { netuid, hotkey, .. }) => {
                if Self::check_weights_min_stake(hotkey) {
                    let priority: u64 = Self::get_priority_set_weights(hotkey, *netuid);
                    Ok(ValidTransaction {
                        priority,
                        longevity: 1,
                        ..Default::default()
                    })
                } else {
                    Err(InvalidTransaction::Call.into())
                }
            }
            Some(Call::add_stake { .. }) => Ok(ValidTransaction {
                priority: Self::get_priority_vanilla(),
                ..Default::default()
            }),
            Some(Call::remove_stake { .. }) => Ok(ValidTransaction {
                priority: Self::get_priority_vanilla(),
                ..Default::default()
            }),
            Some(Call::register { netuid, .. } | Call::burned_register { netuid, .. }) => {
                let registrations_this_interval =
                    Pallet::<T>::get_registrations_this_interval(*netuid);
                let max_registrations_per_interval =
                    Pallet::<T>::get_target_registrations_per_interval(*netuid);
                if registrations_this_interval >= (max_registrations_per_interval.saturating_mul(3))
                {
                    // If the registration limit for the interval is exceeded, reject the transaction
                    return InvalidTransaction::ExhaustsResources.into();
                }
                Ok(ValidTransaction {
                    priority: Self::get_priority_vanilla(),
                    ..Default::default()
                })
            }
            Some(Call::register_network { .. }) => Ok(ValidTransaction {
                priority: Self::get_priority_vanilla(),
                ..Default::default()
            }),
            Some(Call::dissolve_network { .. }) => {
                if Pallet::<T>::coldkey_in_arbitration(who) {
                    Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
                } else {
                    Ok(ValidTransaction {
                        priority: Self::get_priority_vanilla(),
                        ..Default::default()
                    })
                }
            }
            _ => Ok(ValidTransaction {
                priority: Self::get_priority_vanilla(),
                ..Default::default()
            }),
        }
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
            Some(Call::add_stake { .. }) => {
                let transaction_fee = 100000;
                Ok((CallType::AddStake, transaction_fee, who.clone()))
            }
            Some(Call::remove_stake { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::RemoveStake, transaction_fee, who.clone()))
            }
            Some(Call::set_weights { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::SetWeights, transaction_fee, who.clone()))
            }
            Some(Call::commit_weights { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::SetWeights, transaction_fee, who.clone()))
            }
            Some(Call::reveal_weights { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::SetWeights, transaction_fee, who.clone()))
            }
            Some(Call::register { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::Register, transaction_fee, who.clone()))
            }
            Some(Call::serve_axon { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::Serve, transaction_fee, who.clone()))
            }
            Some(Call::register_network { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::RegisterNetwork, transaction_fee, who.clone()))
            }
            _ => {
                let transaction_fee = 0;
                Ok((CallType::Other, transaction_fee, who.clone()))
            }
        }
    }

    fn post_dispatch(
        maybe_pre: Option<Self::Pre>,
        _info: &DispatchInfoOf<Self::Call>,
        _post_info: &PostDispatchInfoOf<Self::Call>,
        _len: usize,
        _result: &dispatch::DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        if let Some((call_type, _transaction_fee, _who)) = maybe_pre {
            match call_type {
                CallType::SetWeights => {
                    log::debug!("Not Implemented!");
                }
                CallType::AddStake => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::RemoveStake => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::Register => {
                    log::debug!("Not Implemented!");
                }
                _ => {
                    log::debug!("Not Implemented!");
                }
            }
        }
        Ok(())
    }
}

use sp_std::vec;

// TODO: unravel this rats nest, for some reason rustc thinks this is unused even though it's
// used not 25 lines below
#[allow(unused)]
use sp_std::vec::Vec;
use subtensor_macros::freeze_struct;

/// Trait for managing a membership pallet instance in the runtime
pub trait MemberManagement<AccountId> {
    /// Add member
    fn add_member(account: &AccountId) -> DispatchResultWithPostInfo;

    /// Remove a member
    fn remove_member(account: &AccountId) -> DispatchResultWithPostInfo;

    /// Swap member
    fn swap_member(remove: &AccountId, add: &AccountId) -> DispatchResultWithPostInfo;

    /// Get all members
    fn members() -> Vec<AccountId>;

    /// Check if an account is apart of the set
    fn is_member(account: &AccountId) -> bool;

    /// Get our maximum member count
    fn max_members() -> u32;
}

impl<T> MemberManagement<T> for () {
    /// Add member
    fn add_member(_: &T) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    // Remove a member
    fn remove_member(_: &T) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    // Swap member
    fn swap_member(_: &T, _: &T) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    // Get all members
    fn members() -> Vec<T> {
        vec![]
    }

    // Check if an account is apart of the set
    fn is_member(_: &T) -> bool {
        false
    }

    fn max_members() -> u32 {
        0
    }
}

/// Trait for interacting with collective pallets
pub trait CollectiveInterface<AccountId, Hash, ProposalIndex> {
    /// Remove vote
    fn remove_votes(hotkey: &AccountId) -> Result<bool, DispatchError>;

    fn add_vote(
        hotkey: &AccountId,
        proposal: Hash,
        index: ProposalIndex,
        approve: bool,
    ) -> Result<bool, DispatchError>;
}

impl<T, H, P> CollectiveInterface<T, H, P> for () {
    fn remove_votes(_: &T) -> Result<bool, DispatchError> {
        Ok(true)
    }

    fn add_vote(_: &T, _: H, _: P, _: bool) -> Result<bool, DispatchError> {
        Ok(true)
    }
}
