#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// Some arithmetic operations can't use the saturating equivalent, such as the PerThing types
#![allow(clippy::arithmetic_side_effects)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod check_nonce;
mod migrations;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    dispatch::DispatchResultWithPostInfo,
    genesis_builder_helper::{build_config, create_default_config},
    pallet_prelude::{DispatchError, Get},
    traits::{fungible::HoldConsideration, Contains, LinearStoragePrice},
};
use frame_system::{EnsureNever, EnsureRoot, EnsureRootWithSuccess, RawOrigin};
use pallet_commitments::CanCommit;
use pallet_grandpa::{
    fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_registry::CanRegisterIdentity;
use scale_info::TypeInfo;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, RuntimeDebug};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{
        AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, One, Verify,
    },
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, MultiSignature,
};
use sp_std::cmp::Ordering;
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{
        ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, InstanceFilter, KeyOwnerProofSystem,
        PrivilegeCmp, Randomness, StorageInfo,
    },
    weights::{
        constants::{
            BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
        IdentityFee, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    },
    StorageValue,
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{CurrencyAdapter, Multiplier};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

// Subtensor module
pub use pallet_subtensor;

// An index to a block.
pub type BlockNumber = u32;

// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

// Some way of identifying an account on the chain. We intentionally make it equivalent
// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// Balance of an account.
pub type Balance = u64;

// Index of a transaction in the chain.
pub type Index = u32;

// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

// Member type for membership
type MemberCount = u32;

pub type Nonce = u32;

// Method used to calculate the fee of an extrinsic
pub const fn deposit(items: u32, bytes: u32) -> Balance {
    pub const ITEMS_FEE: Balance = 2_000 * 10_000;
    pub const BYTES_FEE: Balance = 100 * 10_000;
    (items as Balance)
        .saturating_mul(ITEMS_FEE)
        .saturating_add((bytes as Balance).saturating_mul(BYTES_FEE))
}

// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
// the specifics of the runtime. They can then be made to be agnostic over specific formats
// of data like extrinsics, allowing for them to continue syncing the network through upgrades
// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    // Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    // Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    // Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: Aura,
            pub grandpa: Grandpa,
        }
    }
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("node-subtensor"),
    impl_name: create_runtime_str!("node-subtensor"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    spec_version: 165,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
#[cfg(not(feature = "fast-blocks"))]
pub const MILLISECS_PER_BLOCK: u64 = 12000;

/// Fast blocks for development
#[cfg(feature = "fast-blocks")]
pub const MILLISECS_PER_BLOCK: u64 = 250;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    // We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::with_sensible_defaults(
            Weight::from_parts(4u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
            NORMAL_DISPATCH_RATIO,
        );
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(10 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    // The basic call filter to use in dispatchable.
    type BaseCallFilter = SafeMode;
    // Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    // The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    // The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    // The aggregated dispatch type that is available for extrinsics.
    type RuntimeCall = RuntimeCall;
    // The aggregated runtime tasks.
    type RuntimeTask = RuntimeTask;
    // The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    // The type for hashing blocks and tries.
    type Hash = Hash;
    // The hashing algorithm used.
    type Hashing = BlakeTwo256;
    // The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    // The ubiquitous origin type.
    type RuntimeOrigin = RuntimeOrigin;
    // Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    // The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    // Version of the runtime.
    type Version = Version;
    // Converts a module to the index of the module in `construct_runtime!`.
    //
    // This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    // What to do if a new account is created.
    type OnNewAccount = ();
    // What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    // The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    // Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    // This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    // The set code logic, just the default since we're not a parachain.
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type Nonce = Nonce;
    type Block = Block;
    type SingleBlockMigrations = Migrations;
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<32>;
    type AllowMultipleBlocksPerSlot = ConstBool<false>;
    type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type KeyOwnerProof = sp_core::Void;

    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type MaxNominators = ConstU32<20>;

    type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
    // A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const DisallowPermissionlessEnterDuration: BlockNumber = 0;
    pub const DisallowPermissionlessExtendDuration: BlockNumber = 0;

    pub const RootEnterDuration: BlockNumber = 5 * 60 * 24; // 24 hours

    pub const RootExtendDuration: BlockNumber = 5 * 60 * 12; // 12 hours

    pub const DisallowPermissionlessEntering: Option<Balance> = None;
    pub const DisallowPermissionlessExtending: Option<Balance> = None;
    pub const DisallowPermissionlessRelease: Option<BlockNumber> = None;
}

pub struct SafeModeWhitelistedCalls;
impl Contains<RuntimeCall> for SafeModeWhitelistedCalls {
    fn contains(call: &RuntimeCall) -> bool {
        matches!(
            call,
            RuntimeCall::Sudo(_)
                | RuntimeCall::Multisig(_)
                | RuntimeCall::System(_)
                | RuntimeCall::SafeMode(_)
                | RuntimeCall::Timestamp(_)
                | RuntimeCall::SubtensorModule(
                    pallet_subtensor::Call::set_weights { .. }
                        | pallet_subtensor::Call::set_root_weights { .. }
                        | pallet_subtensor::Call::serve_axon { .. }
                )
                | RuntimeCall::Commitments(pallet_commitments::Call::set_commitment { .. })
        )
    }
}

impl pallet_safe_mode::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type WhitelistedCalls = SafeModeWhitelistedCalls;
    type EnterDuration = DisallowPermissionlessEnterDuration;
    type ExtendDuration = DisallowPermissionlessExtendDuration;
    type EnterDepositAmount = DisallowPermissionlessEntering;
    type ExtendDepositAmount = DisallowPermissionlessExtending;
    type ForceEnterOrigin = EnsureRootWithSuccess<AccountId, RootEnterDuration>;
    type ForceExtendOrigin = EnsureRootWithSuccess<AccountId, RootExtendDuration>;
    type ForceExitOrigin = EnsureRoot<AccountId>;
    type ForceDepositOrigin = EnsureRoot<AccountId>;
    type Notify = ();
    type ReleaseDelay = DisallowPermissionlessRelease;
    type WeightInfo = pallet_safe_mode::weights::SubstrateWeight<Runtime>;
}

// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u64 = 500;

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    // The type for recording an account's balance.
    type Balance = Balance;
    // The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;

    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = ConstU32<50>;
}

pub struct LinearWeightToFee<C>(sp_std::marker::PhantomData<C>);

impl<C> WeightToFeePolynomial for LinearWeightToFee<C>
where
    C: Get<Balance>,
{
    type Balance = Balance;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        let coefficient = WeightToFeeCoefficient {
            coeff_integer: 0,
            coeff_frac: Perbill::from_parts(1_000_000),
            negative: false,
            degree: 1,
        };

        smallvec!(coefficient)
    }
}

parameter_types! {
    // Used with LinearWeightToFee conversion.
    pub const FeeWeightRatio: u64 = 1;
    pub const TransactionByteFee: u128 = 1;
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
    //type TransactionByteFee = TransactionByteFee;

    // Convert dispatch weight to a chargeable fee.
    type WeightToFee = LinearWeightToFee<FeeWeightRatio>;

    type FeeMultiplierUpdate = ();

    type OperationalFeeMultiplier = ConstU8<1>;

    type LengthToFee = IdentityFee<Balance>;
    //type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

// Configure collective pallet for council
parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 12 * HOURS;
    pub const CouncilMaxProposals: u32 = 10;
    pub const CouncilMaxMembers: u32 = 3;
}

// Configure collective pallet for Senate
parameter_types! {
    pub const SenateMaxMembers: u32 = 12;
}

use pallet_collective::{CanPropose, CanVote, GetVotingMembers};
pub struct CanProposeToTriumvirate;
impl CanPropose<AccountId> for CanProposeToTriumvirate {
    fn can_propose(account: &AccountId) -> bool {
        Triumvirate::is_member(account)
    }
}

pub struct CanVoteToTriumvirate;
impl CanVote<AccountId> for CanVoteToTriumvirate {
    fn can_vote(_: &AccountId) -> bool {
        //Senate::is_member(account)
        false // Disable voting from pallet_collective::vote
    }
}

use pallet_subtensor::{CollectiveInterface, MemberManagement};
pub struct ManageSenateMembers;
impl MemberManagement<AccountId> for ManageSenateMembers {
    fn add_member(account: &AccountId) -> DispatchResultWithPostInfo {
        let who = Address::Id(account.clone());
        SenateMembers::add_member(RawOrigin::Root.into(), who)
    }

    fn remove_member(account: &AccountId) -> DispatchResultWithPostInfo {
        let who = Address::Id(account.clone());
        SenateMembers::remove_member(RawOrigin::Root.into(), who)
    }

    fn swap_member(rm: &AccountId, add: &AccountId) -> DispatchResultWithPostInfo {
        let remove = Address::Id(rm.clone());
        let add = Address::Id(add.clone());

        Triumvirate::remove_votes(rm)?;
        SenateMembers::swap_member(RawOrigin::Root.into(), remove, add)
    }

    fn is_member(account: &AccountId) -> bool {
        SenateMembers::members().contains(account)
    }

    fn members() -> Vec<AccountId> {
        SenateMembers::members().into()
    }

    fn max_members() -> u32 {
        SenateMaxMembers::get()
    }
}

pub struct GetSenateMemberCount;
impl GetVotingMembers<MemberCount> for GetSenateMemberCount {
    fn get_count() -> MemberCount {
        SenateMembers::members().len() as u32
    }
}
impl Get<MemberCount> for GetSenateMemberCount {
    fn get() -> MemberCount {
        SenateMaxMembers::get()
    }
}

pub struct TriumvirateVotes;
impl CollectiveInterface<AccountId, Hash, u32> for TriumvirateVotes {
    fn remove_votes(hotkey: &AccountId) -> Result<bool, sp_runtime::DispatchError> {
        Triumvirate::remove_votes(hotkey)
    }

    fn add_vote(
        hotkey: &AccountId,
        proposal: Hash,
        index: u32,
        approve: bool,
    ) -> Result<bool, sp_runtime::DispatchError> {
        Triumvirate::do_vote(hotkey.clone(), proposal, index, approve)
    }
}

type EnsureMajoritySenate =
    pallet_collective::EnsureProportionMoreThan<AccountId, TriumvirateCollective, 1, 2>;

// We call pallet_collective TriumvirateCollective
type TriumvirateCollective = pallet_collective::Instance1;
impl pallet_collective::Config<TriumvirateCollective> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = GetSenateMemberCount;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
    type SetMembersOrigin = EnsureNever<AccountId>;
    type CanPropose = CanProposeToTriumvirate;
    type CanVote = CanVoteToTriumvirate;
    type GetVotingMembers = GetSenateMemberCount;
}

// We call council members Triumvirate
type TriumvirateMembership = pallet_membership::Instance1;
impl pallet_membership::Config<TriumvirateMembership> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type PrimeOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = Triumvirate;
    type MembershipChanged = Triumvirate;
    type MaxMembers = CouncilMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

// We call our top K delegates membership Senate
type SenateMembership = pallet_membership::Instance2;
impl pallet_membership::Config<SenateMembership> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type PrimeOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = ();
    type MembershipChanged = ();
    type MaxMembers = SenateMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;

    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // According to multisig pallet, key and value size be computed as follows:
    // value size is `4 + sizeof((BlockNumber, Balance, AccountId))` bytes
    // key size is `32 + sizeof(AccountId)` bytes.
    // For our case, One storage item; key size is 32+32=64 bytes; value is size 4+4+8+32 bytes = 48 bytes.
    pub const DepositBase: Balance = deposit(1, 112);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u32 = 100;
}

impl pallet_multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

// Proxy Pallet config
parameter_types! {
    // One storage item; key size sizeof(AccountId) = 32, value sizeof(Balance) = 8; 40 total
    pub const ProxyDepositBase: Balance = deposit(1, 40);
    // Adding 32 bytes + sizeof(ProxyType) = 32 + 1
    pub const ProxyDepositFactor: Balance = deposit(0, 33);
    pub const MaxProxies: u32 = 20; // max num proxies per acct
    pub const MaxPending: u32 = 15 * 5; // max blocks pending ~15min
    // 16 bytes
    pub const AnnouncementDepositBase: Balance =  deposit(1, 16);
    // 68 bytes per announcement
    pub const AnnouncementDepositFactor: Balance = deposit(0, 68);
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    RuntimeDebug,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum ProxyType {
    Any,
    Owner, // Subnet owner Calls
    NonCritical,
    NonTransfer,
    Senate,
    NonFungibile, // Nothing involving moving TAO
    Triumvirate,
    Governance, // Both above governance
    Staking,
    Registration,
}
impl Default for ProxyType {
    fn default() -> Self {
        Self::Any
    }
} // allow all Calls; required to be most permissive
impl InstanceFilter<RuntimeCall> for ProxyType {
    fn filter(&self, c: &RuntimeCall) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::NonTransfer => !matches!(c, RuntimeCall::Balances(..)),
            ProxyType::NonFungibile => !matches!(
                c,
                RuntimeCall::Balances(..)
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::burned_register { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register { .. })
            ),
            ProxyType::Owner => matches!(c, RuntimeCall::AdminUtils(..)),
            ProxyType::NonCritical => !matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::dissolve_network { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::burned_register { .. })
                    | RuntimeCall::Triumvirate(..)
            ),
            ProxyType::Triumvirate => matches!(
                c,
                RuntimeCall::Triumvirate(..) | RuntimeCall::TriumvirateMembers(..)
            ),
            ProxyType::Senate => matches!(c, RuntimeCall::SenateMembers(..)),
            ProxyType::Governance => matches!(
                c,
                RuntimeCall::SenateMembers(..)
                    | RuntimeCall::Triumvirate(..)
                    | RuntimeCall::TriumvirateMembers(..)
            ),
            ProxyType::Staking => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake { .. })
            ),
            ProxyType::Registration => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::burned_register { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::register { .. })
            ),
        }
    }
    fn is_superset(&self, o: &Self) -> bool {
        match (self, o) {
            (x, y) if x == y => true,
            (ProxyType::Any, _) => true,
            (_, ProxyType::Any) => false,
            (ProxyType::NonTransfer, _) => true,
            (ProxyType::Governance, ProxyType::Triumvirate | ProxyType::Senate) => true,
            _ => false,
        }
    }
}

impl pallet_proxy::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = MaxProxies;
    type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
    type MaxPending = MaxPending;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        BlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const NoPreimagePostponement: Option<u32> = Some(10);
}

/// Used the compare the privilege of an origin inside the scheduler.
pub struct OriginPrivilegeCmp;

impl PrivilegeCmp<OriginCaller> for OriginPrivilegeCmp {
    fn cmp_privilege(left: &OriginCaller, right: &OriginCaller) -> Option<Ordering> {
        if left == right {
            return Some(Ordering::Equal);
        }

        match (left, right) {
            // Root is greater than anything.
            (OriginCaller::system(frame_system::RawOrigin::Root), _) => Some(Ordering::Greater),
            // Check which one has more yes votes.
            (
                OriginCaller::Triumvirate(pallet_collective::RawOrigin::Members(
                    l_yes_votes,
                    l_count,
                )),
                OriginCaller::Triumvirate(pallet_collective::RawOrigin::Members(
                    r_yes_votes,
                    r_count,
                )), // Equivalent to (l_yes_votes / l_count).cmp(&(r_yes_votes / r_count))
            ) => Some(
                l_yes_votes
                    .saturating_mul(*r_count)
                    .cmp(&r_yes_votes.saturating_mul(*l_count)),
            ),
            // For every other origin we don't care, as they are not used for `ScheduleOrigin`.
            _ => None,
        }
    }
}

impl pallet_scheduler::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = OriginPrivilegeCmp;
    type Preimages = Preimage;
}

parameter_types! {
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub const PreimageBaseDeposit: Balance = deposit(2, 64);
    pub const PreimageByteDeposit: Balance = deposit(0, 1);
    pub const PreimageHoldReason: RuntimeHoldReason =
        RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
    type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type Consideration = HoldConsideration<
        AccountId,
        Balances,
        PreimageHoldReason,
        LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
    >;
}

pub struct AllowIdentityReg;

impl CanRegisterIdentity<AccountId> for AllowIdentityReg {
    #[cfg(not(feature = "runtime-benchmarks"))]
    fn can_register(address: &AccountId, identified: &AccountId) -> bool {
        if address != identified {
            SubtensorModule::coldkey_owns_hotkey(address, identified)
                && SubtensorModule::is_hotkey_registered_on_network(0, identified)
        } else {
            SubtensorModule::is_subnet_owner(address)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn can_register(_: &AccountId, _: &AccountId) -> bool {
        true
    }
}

// Configure registry pallet.
parameter_types! {
    pub const MaxAdditionalFields: u32 = 1;
    pub const InitialDeposit: Balance = 100_000_000; // 0.1 TAO
    pub const FieldDeposit: Balance = 100_000_000; // 0.1 TAO
}

impl pallet_registry::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type Currency = Balances;
    type CanRegister = AllowIdentityReg;
    type WeightInfo = pallet_registry::weights::SubstrateWeight<Runtime>;

    type MaxAdditionalFields = MaxAdditionalFields;
    type InitialDeposit = InitialDeposit;
    type FieldDeposit = FieldDeposit;
}

parameter_types! {
    pub const MaxCommitFields: u32 = 1;
    pub const CommitmentInitialDeposit: Balance = 0; // Free
    pub const CommitmentFieldDeposit: Balance = 0; // Free
    pub const CommitmentRateLimit: BlockNumber = 100; // Allow commitment every 100 blocks
}

pub struct AllowCommitments;
impl CanCommit<AccountId> for AllowCommitments {
    #[cfg(not(feature = "runtime-benchmarks"))]
    fn can_commit(netuid: u16, address: &AccountId) -> bool {
        SubtensorModule::is_hotkey_registered_on_network(netuid, address)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn can_commit(_: u16, _: &AccountId) -> bool {
        true
    }
}

impl pallet_commitments::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = pallet_commitments::weights::SubstrateWeight<Runtime>;

    type CanCommit = AllowCommitments;

    type MaxFields = MaxCommitFields;
    type InitialDeposit = CommitmentInitialDeposit;
    type FieldDeposit = CommitmentFieldDeposit;
    type RateLimit = CommitmentRateLimit;
}

// Configure the pallet subtensor.
parameter_types! {
    pub const SubtensorInitialRho: u16 = 10;
    pub const SubtensorInitialKappa: u16 = 32_767; // 0.5 = 65535/2
    pub const SubtensorInitialMaxAllowedUids: u16 = 4096;
    pub const SubtensorInitialIssuance: u64 = 0;
    pub const SubtensorInitialMinAllowedWeights: u16 = 1024;
    pub const SubtensorInitialEmissionValue: u16 = 0;
    pub const SubtensorInitialMaxWeightsLimit: u16 = 1000; // 1000/2^16 = 0.015
    pub const SubtensorInitialValidatorPruneLen: u64 = 1;
    pub const SubtensorInitialScalingLawPower: u16 = 50; // 0.5
    pub const SubtensorInitialMaxAllowedValidators: u16 = 128;
    pub const SubtensorInitialTempo: u16 = 99;
    pub const SubtensorInitialDifficulty: u64 = 10_000_000;
    pub const SubtensorInitialAdjustmentInterval: u16 = 100;
    pub const SubtensorInitialAdjustmentAlpha: u64 = 0; // no weight to previous value.
    pub const SubtensorInitialTargetRegistrationsPerInterval: u16 = 2;
    pub const SubtensorInitialImmunityPeriod: u16 = 4096;
    pub const SubtensorInitialActivityCutoff: u16 = 5000;
    pub const SubtensorInitialMaxRegistrationsPerBlock: u16 = 1;
    pub const SubtensorInitialPruningScore : u16 = u16::MAX;
    pub const SubtensorInitialBondsMovingAverage: u64 = 900_000;
    pub const SubtensorInitialDefaultTake: u16 = 11_796; // 18% honest number.
    pub const SubtensorInitialMinTake: u16 = 5_898; // 9%
    pub const SubtensorInitialWeightsVersionKey: u64 = 0;
    pub const SubtensorInitialMinDifficulty: u64 = 10_000_000;
    pub const SubtensorInitialMaxDifficulty: u64 = u64::MAX / 4;
    pub const SubtensorInitialServingRateLimit: u64 = 50;
    pub const SubtensorInitialBurn: u64 = 1_000_000_000; // 1 tao
    pub const SubtensorInitialMinBurn: u64 = 1_000_000_000; // 1 tao
    pub const SubtensorInitialMaxBurn: u64 = 100_000_000_000; // 100 tao
    pub const SubtensorInitialTxRateLimit: u64 = 1000;
    pub const SubtensorInitialTxDelegateTakeRateLimit: u64 = 216000; // 30 days at 12 seconds per block
    pub const SubtensorInitialRAORecycledForRegistration: u64 = 0; // 0 rao
    pub const SubtensorInitialSenateRequiredStakePercentage: u64 = 1; // 1 percent of total stake
    pub const SubtensorInitialNetworkImmunity: u64 = 7 * 7200;
    pub const SubtensorInitialMinAllowedUids: u16 = 128;
    pub const SubtensorInitialMinLockCost: u64 = 1_000_000_000_000; // 1000 TAO
    pub const SubtensorInitialSubnetOwnerCut: u16 = 11_796; // 18 percent
    pub const SubtensorInitialSubnetLimit: u16 = 12;
    pub const SubtensorInitialNetworkLockReductionInterval: u64 = 14 * 7200;
    pub const SubtensorInitialNetworkRateLimit: u64 = 7200;
    pub const SubtensorInitialTargetStakesPerInterval: u16 = 1;
    pub const SubtensorInitialKeySwapCost: u64 = 1_000_000_000;
    pub const InitialAlphaHigh: u16 = 58982; // Represents 0.9 as per the production default
    pub const InitialAlphaLow: u16 = 45875; // Represents 0.7 as per the production default
    pub const InitialLiquidAlphaOn: bool = false; // Default value for LiquidAlphaOn
    pub const SubtensorInitialBaseDifficulty: u64 = 10_000_000; // Base difficulty
}

impl pallet_subtensor::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SudoRuntimeCall = RuntimeCall;
    type Currency = Balances;
    type CouncilOrigin = EnsureMajoritySenate;
    type SenateMembers = ManageSenateMembers;
    type TriumvirateInterface = TriumvirateVotes;

    type InitialRho = SubtensorInitialRho;
    type InitialKappa = SubtensorInitialKappa;
    type InitialMaxAllowedUids = SubtensorInitialMaxAllowedUids;
    type InitialBondsMovingAverage = SubtensorInitialBondsMovingAverage;
    type InitialIssuance = SubtensorInitialIssuance;
    type InitialMinAllowedWeights = SubtensorInitialMinAllowedWeights;
    type InitialEmissionValue = SubtensorInitialEmissionValue;
    type InitialMaxWeightsLimit = SubtensorInitialMaxWeightsLimit;
    type InitialValidatorPruneLen = SubtensorInitialValidatorPruneLen;
    type InitialScalingLawPower = SubtensorInitialScalingLawPower;
    type InitialTempo = SubtensorInitialTempo;
    type InitialDifficulty = SubtensorInitialDifficulty;
    type InitialAdjustmentInterval = SubtensorInitialAdjustmentInterval;
    type InitialAdjustmentAlpha = SubtensorInitialAdjustmentAlpha;
    type InitialTargetRegistrationsPerInterval = SubtensorInitialTargetRegistrationsPerInterval;
    type InitialImmunityPeriod = SubtensorInitialImmunityPeriod;
    type InitialActivityCutoff = SubtensorInitialActivityCutoff;
    type InitialMaxRegistrationsPerBlock = SubtensorInitialMaxRegistrationsPerBlock;
    type InitialPruningScore = SubtensorInitialPruningScore;
    type InitialMaxAllowedValidators = SubtensorInitialMaxAllowedValidators;
    type InitialDefaultTake = SubtensorInitialDefaultTake;
    type InitialMinTake = SubtensorInitialMinTake;
    type InitialWeightsVersionKey = SubtensorInitialWeightsVersionKey;
    type InitialMaxDifficulty = SubtensorInitialMaxDifficulty;
    type InitialMinDifficulty = SubtensorInitialMinDifficulty;
    type InitialServingRateLimit = SubtensorInitialServingRateLimit;
    type InitialBurn = SubtensorInitialBurn;
    type InitialMaxBurn = SubtensorInitialMaxBurn;
    type InitialMinBurn = SubtensorInitialMinBurn;
    type InitialTxRateLimit = SubtensorInitialTxRateLimit;
    type InitialTxDelegateTakeRateLimit = SubtensorInitialTxDelegateTakeRateLimit;
    type InitialRAORecycledForRegistration = SubtensorInitialRAORecycledForRegistration;
    type InitialSenateRequiredStakePercentage = SubtensorInitialSenateRequiredStakePercentage;
    type InitialNetworkImmunityPeriod = SubtensorInitialNetworkImmunity;
    type InitialNetworkMinAllowedUids = SubtensorInitialMinAllowedUids;
    type InitialNetworkMinLockCost = SubtensorInitialMinLockCost;
    type InitialNetworkLockReductionInterval = SubtensorInitialNetworkLockReductionInterval;
    type InitialSubnetOwnerCut = SubtensorInitialSubnetOwnerCut;
    type InitialSubnetLimit = SubtensorInitialSubnetLimit;
    type InitialNetworkRateLimit = SubtensorInitialNetworkRateLimit;
    type InitialTargetStakesPerInterval = SubtensorInitialTargetStakesPerInterval;
    type KeySwapCost = SubtensorInitialKeySwapCost;
    type AlphaHigh = InitialAlphaHigh;
    type AlphaLow = InitialAlphaLow;
    type LiquidAlphaOn = InitialLiquidAlphaOn;
    type InitialBaseDifficulty = SubtensorInitialBaseDifficulty;
}

use sp_runtime::BoundedVec;

pub struct AuraPalletIntrf;
impl pallet_admin_utils::AuraInterface<AuraId, ConstU32<32>> for AuraPalletIntrf {
    fn change_authorities(new: BoundedVec<AuraId, ConstU32<32>>) {
        Aura::change_authorities(new);
    }
}

pub struct SubtensorInterface;

impl
    pallet_admin_utils::SubtensorInterface<
        AccountId,
        <pallet_balances::Pallet<Runtime> as frame_support::traits::Currency<AccountId>>::Balance,
        RuntimeOrigin,
    > for SubtensorInterface
{
    fn set_max_delegate_take(max_take: u16) {
        SubtensorModule::set_max_delegate_take(max_take);
    }

    fn set_min_delegate_take(max_take: u16) {
        SubtensorModule::set_min_delegate_take(max_take);
    }

    fn set_tx_rate_limit(rate_limit: u64) {
        SubtensorModule::set_tx_rate_limit(rate_limit);
    }

    fn set_tx_delegate_take_rate_limit(rate_limit: u64) {
        SubtensorModule::set_tx_delegate_take_rate_limit(rate_limit);
    }

    fn set_serving_rate_limit(netuid: u16, rate_limit: u64) {
        SubtensorModule::set_serving_rate_limit(netuid, rate_limit);
    }

    fn set_max_burn(netuid: u16, max_burn: u64) {
        SubtensorModule::set_max_burn(netuid, max_burn);
    }

    fn set_min_burn(netuid: u16, min_burn: u64) {
        SubtensorModule::set_min_burn(netuid, min_burn);
    }

    fn set_burn(netuid: u16, burn: u64) {
        SubtensorModule::set_burn(netuid, burn);
    }

    fn set_max_difficulty(netuid: u16, max_diff: u64) {
        SubtensorModule::set_max_difficulty(netuid, max_diff);
    }

    fn set_min_difficulty(netuid: u16, min_diff: u64) {
        SubtensorModule::set_min_difficulty(netuid, min_diff);
    }

    fn set_difficulty(netuid: u16, diff: u64) {
        SubtensorModule::set_difficulty(netuid, diff);
    }

    fn set_weights_rate_limit(netuid: u16, rate_limit: u64) {
        SubtensorModule::set_weights_set_rate_limit(netuid, rate_limit);
    }

    fn set_weights_version_key(netuid: u16, version: u64) {
        SubtensorModule::set_weights_version_key(netuid, version);
    }

    fn set_bonds_moving_average(netuid: u16, moving_average: u64) {
        SubtensorModule::set_bonds_moving_average(netuid, moving_average);
    }

    fn set_max_allowed_validators(netuid: u16, max_validators: u16) {
        SubtensorModule::set_max_allowed_validators(netuid, max_validators);
    }

    fn get_root_netuid() -> u16 {
        SubtensorModule::get_root_netuid()
    }

    fn if_subnet_exist(netuid: u16) -> bool {
        SubtensorModule::if_subnet_exist(netuid)
    }

    fn create_account_if_non_existent(coldkey: &AccountId, hotkey: &AccountId) {
        SubtensorModule::create_account_if_non_existent(coldkey, hotkey)
    }

    fn coldkey_owns_hotkey(coldkey: &AccountId, hotkey: &AccountId) -> bool {
        SubtensorModule::coldkey_owns_hotkey(coldkey, hotkey)
    }

    fn increase_stake_on_coldkey_hotkey_account(
        coldkey: &AccountId,
        hotkey: &AccountId,
        increment: u64,
    ) {
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(coldkey, hotkey, increment);
    }

    fn add_balance_to_coldkey_account(coldkey: &AccountId, amount: Balance) {
        SubtensorModule::add_balance_to_coldkey_account(coldkey, amount);
    }

    fn get_current_block_as_u64() -> u64 {
        SubtensorModule::get_current_block_as_u64()
    }

    fn get_subnetwork_n(netuid: u16) -> u16 {
        SubtensorModule::get_subnetwork_n(netuid)
    }

    fn get_max_allowed_uids(netuid: u16) -> u16 {
        SubtensorModule::get_max_allowed_uids(netuid)
    }

    fn append_neuron(netuid: u16, new_hotkey: &AccountId, block_number: u64) {
        SubtensorModule::append_neuron(netuid, new_hotkey, block_number)
    }

    fn get_neuron_to_prune(netuid: u16) -> u16 {
        SubtensorModule::get_neuron_to_prune(netuid)
    }

    fn replace_neuron(netuid: u16, uid_to_replace: u16, new_hotkey: &AccountId, block_number: u64) {
        SubtensorModule::replace_neuron(netuid, uid_to_replace, new_hotkey, block_number);
    }

    fn set_total_issuance(total_issuance: u64) {
        SubtensorModule::set_total_issuance(total_issuance);
    }

    fn set_network_immunity_period(net_immunity_period: u64) {
        SubtensorModule::set_network_immunity_period(net_immunity_period);
    }

    fn set_network_min_lock(net_min_lock: u64) {
        SubtensorModule::set_network_min_lock(net_min_lock);
    }

    fn set_subnet_limit(limit: u16) {
        SubtensorModule::set_max_subnets(limit);
    }

    fn set_lock_reduction_interval(interval: u64) {
        SubtensorModule::set_lock_reduction_interval(interval);
    }

    fn set_tempo(netuid: u16, tempo: u16) {
        SubtensorModule::set_tempo(netuid, tempo);
    }

    fn set_subnet_owner_cut(subnet_owner_cut: u16) {
        SubtensorModule::set_subnet_owner_cut(subnet_owner_cut);
    }

    fn set_network_rate_limit(limit: u64) {
        SubtensorModule::set_network_rate_limit(limit);
    }

    fn set_max_registrations_per_block(netuid: u16, max_registrations_per_block: u16) {
        SubtensorModule::set_max_registrations_per_block(netuid, max_registrations_per_block);
    }

    fn set_adjustment_alpha(netuid: u16, adjustment_alpha: u64) {
        SubtensorModule::set_adjustment_alpha(netuid, adjustment_alpha);
    }

    fn set_target_registrations_per_interval(netuid: u16, target_registrations_per_interval: u16) {
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );
    }

    fn set_network_pow_registration_allowed(netuid: u16, registration_allowed: bool) {
        SubtensorModule::set_network_pow_registration_allowed(netuid, registration_allowed);
    }

    fn set_network_registration_allowed(netuid: u16, registration_allowed: bool) {
        SubtensorModule::set_network_registration_allowed(netuid, registration_allowed);
    }

    fn set_activity_cutoff(netuid: u16, activity_cutoff: u16) {
        SubtensorModule::set_activity_cutoff(netuid, activity_cutoff);
    }

    fn ensure_subnet_owner_or_root(o: RuntimeOrigin, netuid: u16) -> Result<(), DispatchError> {
        SubtensorModule::ensure_subnet_owner_or_root(o, netuid)
    }

    fn set_rho(netuid: u16, rho: u16) {
        SubtensorModule::set_rho(netuid, rho);
    }

    fn set_kappa(netuid: u16, kappa: u16) {
        SubtensorModule::set_kappa(netuid, kappa);
    }

    fn set_max_allowed_uids(netuid: u16, max_allowed: u16) {
        SubtensorModule::set_max_allowed_uids(netuid, max_allowed);
    }

    fn set_min_allowed_weights(netuid: u16, min_allowed_weights: u16) {
        SubtensorModule::set_min_allowed_weights(netuid, min_allowed_weights);
    }

    fn set_immunity_period(netuid: u16, immunity_period: u16) {
        SubtensorModule::set_immunity_period(netuid, immunity_period);
    }

    fn set_max_weight_limit(netuid: u16, max_weight_limit: u16) {
        SubtensorModule::set_max_weight_limit(netuid, max_weight_limit);
    }

    fn set_scaling_law_power(netuid: u16, scaling_law_power: u16) {
        SubtensorModule::set_scaling_law_power(netuid, scaling_law_power);
    }

    fn set_validator_prune_len(netuid: u16, validator_prune_len: u64) {
        SubtensorModule::set_validator_prune_len(netuid, validator_prune_len);
    }

    fn set_adjustment_interval(netuid: u16, adjustment_interval: u16) {
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
    }

    fn set_weights_set_rate_limit(netuid: u16, weights_set_rate_limit: u64) {
        SubtensorModule::set_weights_set_rate_limit(netuid, weights_set_rate_limit);
    }

    fn set_rao_recycled(netuid: u16, rao_recycled: u64) {
        SubtensorModule::set_rao_recycled(netuid, rao_recycled);
    }

    fn is_hotkey_registered_on_network(netuid: u16, hotkey: &AccountId) -> bool {
        SubtensorModule::is_hotkey_registered_on_network(netuid, hotkey)
    }

    fn init_new_network(netuid: u16, tempo: u16) {
        SubtensorModule::init_new_network(netuid, tempo);
    }

    fn set_weights_min_stake(min_stake: u64) {
        SubtensorModule::set_weights_min_stake(min_stake);
    }

    fn clear_small_nominations() {
        SubtensorModule::clear_small_nominations();
    }

    fn set_nominator_min_required_stake(min_stake: u64) {
        SubtensorModule::set_nominator_min_required_stake(min_stake);
    }

    fn get_nominator_min_required_stake() -> u64 {
        SubtensorModule::get_nominator_min_required_stake()
    }

    fn set_target_stakes_per_interval(target_stakes_per_interval: u64) {
        SubtensorModule::set_target_stakes_per_interval(target_stakes_per_interval)
    }

    fn set_commit_reveal_weights_interval(netuid: u16, interval: u64) {
        SubtensorModule::set_commit_reveal_weights_interval(netuid, interval);
    }

    fn set_commit_reveal_weights_enabled(netuid: u16, enabled: bool) {
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, enabled);
    }

    fn set_liquid_alpha_enabled(netuid: u16, enabled: bool) {
        SubtensorModule::set_liquid_alpha_enabled(netuid, enabled);
    }

    fn do_set_alpha_values(
        origin: RuntimeOrigin,
        netuid: u16,
        alpha_low: u16,
        alpha_high: u16,
    ) -> Result<(), DispatchError> {
        SubtensorModule::do_set_alpha_values(origin, netuid, alpha_low, alpha_high)
    }
}

impl pallet_admin_utils::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AuthorityId = AuraId;
    type MaxAuthorities = ConstU32<32>;
    type Aura = AuraPalletIntrf;
    type Balance = Balance;
    type Subtensor = SubtensorInterface;
    type WeightInfo = pallet_admin_utils::weights::SubstrateWeight<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub struct Runtime
    {
        System: frame_system,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
        Timestamp: pallet_timestamp,
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        SubtensorModule: pallet_subtensor,
        Triumvirate: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        TriumvirateMembers: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},
        SenateMembers: pallet_membership::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>},
        Utility: pallet_utility,
        Sudo: pallet_sudo,
        Multisig: pallet_multisig,
        Preimage: pallet_preimage,
        Scheduler: pallet_scheduler,
        Proxy: pallet_proxy,
        Registry: pallet_registry,
        Commitments: pallet_commitments,
        AdminUtils: pallet_admin_utils,
        SafeMode: pallet_safe_mode,
    }
);

// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    check_nonce::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    pallet_subtensor::SubtensorSignedExtension<Runtime>,
    pallet_commitments::CommitmentsSignedExtension<Runtime>,
);

type Migrations = pallet_grandpa::migrations::MigrateV4ToV5<Runtime>;

// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    Migrations,
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_subtensor, SubtensorModule]
        [pallet_timestamp, Timestamp]
        [pallet_registry, Registry]
        [pallet_commitments, Commitments]
        [pallet_admin_utils, AdminUtils]
    );
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn create_default_config() -> Vec<u8> {
            create_default_config::<RuntimeGenesisConfig>()
        }

        fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
            build_config::<RuntimeGenesisConfig>(config)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            pallet_aura::Authorities::<Runtime>::get().into_inner()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> fg_primitives::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
        for Runtime
    {
        fn query_call_info(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_call_info(call, len)
        }
        fn query_call_fee_details(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_call_fee_details(call, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();

            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
            use sp_storage::TrackedStorageKey;

            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            #[allow(non_local_definitions)]
            impl frame_system_benchmarking::Config for Runtime {}

            #[allow(non_local_definitions)]
            impl baseline::Config for Runtime {}

            use frame_support::traits::WhitelistedStorageKeys;
            let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);
            add_benchmarks!(params, batches);

            Ok(batches)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        #[allow(clippy::unwrap_used)]
        fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade(checks).unwrap();
            (weight, BlockWeights::get().max_block)
        }

        fn execute_block(
            block: Block,
            state_root_check: bool,
            signature_check: bool,
            select: frame_try_runtime::TryStateSelect
        ) -> Weight {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here.
            Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
        }
    }

    impl subtensor_custom_rpc_runtime_api::DelegateInfoRuntimeApi<Block> for Runtime {
        fn get_delegates() -> Vec<u8> {
            let result = SubtensorModule::get_delegates();
            result.encode()
        }

        fn get_delegate(delegate_account_vec: Vec<u8>) -> Vec<u8> {
            let _result = SubtensorModule::get_delegate(delegate_account_vec);
            if _result.is_some() {
                let result = _result.expect("Could not get DelegateInfo");
                result.encode()
            } else {
                vec![]
            }
        }

        fn get_delegated(delegatee_account_vec: Vec<u8>) -> Vec<u8> {
            let result = SubtensorModule::get_delegated(delegatee_account_vec);
            result.encode()
        }
    }

    impl subtensor_custom_rpc_runtime_api::NeuronInfoRuntimeApi<Block> for Runtime {
        fn get_neurons_lite(netuid: u16) -> Vec<u8> {
            let result = SubtensorModule::get_neurons_lite(netuid);
            result.encode()
        }

        fn get_neuron_lite(netuid: u16, uid: u16) -> Vec<u8> {
            let _result = SubtensorModule::get_neuron_lite(netuid, uid);
            if _result.is_some() {
                let result = _result.expect("Could not get NeuronInfoLite");
                result.encode()
            } else {
                vec![]
            }
        }

        fn get_neurons(netuid: u16) -> Vec<u8> {
            let result = SubtensorModule::get_neurons(netuid);
            result.encode()
        }

        fn get_neuron(netuid: u16, uid: u16) -> Vec<u8> {
            let _result = SubtensorModule::get_neuron(netuid, uid);
            if _result.is_some() {
                let result = _result.expect("Could not get NeuronInfo");
                result.encode()
            } else {
                vec![]
            }
        }
    }

    impl subtensor_custom_rpc_runtime_api::SubnetInfoRuntimeApi<Block> for Runtime {
        fn get_subnet_info(netuid: u16) -> Vec<u8> {
            let _result = SubtensorModule::get_subnet_info(netuid);
            if _result.is_some() {
                let result = _result.expect("Could not get SubnetInfo");
                result.encode()
            } else {
                vec![]
            }
        }

        fn get_subnets_info() -> Vec<u8> {
            let result = SubtensorModule::get_subnets_info();
            result.encode()
        }

        fn get_subnet_hyperparams(netuid: u16) -> Vec<u8> {
            let _result = SubtensorModule::get_subnet_hyperparams(netuid);
            if _result.is_some() {
                let result = _result.expect("Could not get SubnetHyperparams");
                result.encode()
            } else {
                vec![]
            }
        }
    }

    impl subtensor_custom_rpc_runtime_api::StakeInfoRuntimeApi<Block> for Runtime {
        fn get_stake_info_for_coldkey( coldkey_account_vec: Vec<u8> ) -> Vec<u8> {
            let result = SubtensorModule::get_stake_info_for_coldkey( coldkey_account_vec );
            result.encode()
        }

        fn get_stake_info_for_coldkeys( coldkey_account_vecs: Vec<Vec<u8>> ) -> Vec<u8> {
            let result = SubtensorModule::get_stake_info_for_coldkeys( coldkey_account_vecs );
            result.encode()
        }
    }

    impl subtensor_custom_rpc_runtime_api::SubnetRegistrationRuntimeApi<Block> for Runtime {
        fn get_network_registration_cost() -> u64 {
            SubtensorModule::get_network_lock_cost()
        }
    }
}

// #[cfg(test)]
// mod tests {

#[test]
fn check_whitelist() {
    use crate::*;
    use frame_support::traits::WhitelistedStorageKeys;
    use sp_core::hexdisplay::HexDisplay;
    use std::collections::HashSet;
    let whitelist: HashSet<String> = AllPalletsWithSystem::whitelisted_storage_keys()
        .iter()
        .map(|e| HexDisplay::from(&e.key).to_string())
        .collect();

    // Block Number
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac"));
    // Total Issuance
    assert!(whitelist.contains("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80"));
    // Execution Phase
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a"));
    // Event Count
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850"));
    // System Events
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7"));
}
// }
