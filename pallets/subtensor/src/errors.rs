use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines the errors for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod errors {
    #[pallet::error]
    pub enum Error<T> {
        /// The subnet does not exist.
        SubNetworkDoesNotExist,
        /// The root network does not exist.
        RootNetworkDoesNotExist,
        /// The user is trying to serve an axon which is not of type 4 (IPv4) or 6 (IPv6).
        InvalidIpType,
        /// An invalid IP address is passed to the serve function.
        InvalidIpAddress,
        /// An invalid port is passed to the serve function.
        InvalidPort,
        /// The hotkey is not registered in subnet
        HotKeyNotRegisteredInSubNet,
        /// The hotkey does not exists
        HotKeyAccountNotExists,
        /// The hotkey is not registered in any subnet.
        HotKeyNotRegisteredInNetwork,
        /// Request to stake, unstake or subscribe is made by a coldkey that is not associated with the hotkey account.
        NonAssociatedColdKey,
        /// The hotkey is not a delegate and the signer is not the owner of the hotkey.
        HotKeyNotDelegateAndSignerNotOwnHotKey,
        /// Stake amount to withdraw is zero.
        StakeToWithdrawIsZero,
        /// The caller is requesting removing more stake than there exists in the staking account. See: "[remove_stake()]".
        NotEnoughStakeToWithdraw,
        /// The caller is requesting to set weights but the caller has less than minimum stake required to set weights (less than WeightsMinStake).
        NotEnoughStakeToSetWeights,
        /// The caller is requesting adding more stake than there exists in the coldkey account. See: "[add_stake()]"
        NotEnoughBalanceToStake,
        /// The caller is trying to add stake, but for some reason the requested amount could not be withdrawn from the coldkey account.
        BalanceWithdrawalError,
        /// Unsuccessfully withdraw, balance could be zero (can not make account exist) after withdrawal.
        ZeroBalanceAfterWithdrawn,
        /// The caller is attempting to set non-self weights without being a permitted validator.
        NeuronNoValidatorPermit,
        /// The caller is attempting to set the weight keys and values but these vectors have different size.
        WeightVecNotEqualSize,
        /// The caller is attempting to set weights with duplicate UIDs in the weight matrix.
        DuplicateUids,
        /// The caller is attempting to set weight to at least one UID that does not exist in the metagraph.
        UidVecContainInvalidOne,
        /// The dispatch is attempting to set weights on chain with fewer elements than are allowed.
        WeightVecLengthIsLow,
        /// Number of registrations in this block exceeds the allowed number (i.e., exceeds the subnet hyperparameter "max_regs_per_block").
        TooManyRegistrationsThisBlock,
        /// The caller is requesting registering a neuron which already exists in the active set.
        HotKeyAlreadyRegisteredInSubNet,
        /// The new hotkey is the same as old one
        NewHotKeyIsSameWithOld,
        /// The supplied PoW hash block is in the future or negative.
        InvalidWorkBlock,
        /// The supplied PoW hash block does not meet the network difficulty.
        InvalidDifficulty,
        /// The supplied PoW hash seal does not match the supplied work.
        InvalidSeal,
        /// The dispatch is attempting to set weights on chain with weight value exceeding the MaxWeightLimit (max_weight_limit subnet hyperparameter).
        MaxWeightExceeded,
        /// The hotkey is attempting to become a delegate when the hotkey is already a delegate.
        HotKeyAlreadyDelegate,
        /// A transactor exceeded the rate limit for setting weights.
        SettingWeightsTooFast,
        /// A validator is attempting to set weights from a validator with incorrect weight version.
        IncorrectWeightVersionKey,
        /// An axon or prometheus serving exceeded the rate limit for a registered neuron.
        ServingRateLimitExceeded,
        /// The caller is attempting to set weights with more UIDs than allowed.
        UidsLengthExceedUidsInSubNet,
        /// A transactor exceeded the rate limit for add network transaction.
        NetworkTxRateLimitExceeded,
        /// A transactor exceeded the rate limit for delegate transaction.
        DelegateTxRateLimitExceeded,
        /// A transactor exceeded the rate limit for setting or swapping hotkey.
        HotKeySetTxRateLimitExceeded,
        /// A transactor exceeded the rate limit for staking.
        StakeRateLimitExceeded,
        /// A transactor exceeded the rate limit for unstaking.
        UnstakeRateLimitExceeded,
        /// Registration is disabled.
        SubNetRegistrationDisabled,
        /// The number of registration attempts exceeded the allowed number in the interval.
        TooManyRegistrationsThisInterval,
        /// The hotkey is required to be the origin.
        TransactorAccountShouldBeHotKey,
        /// A hotkey is attempting to do something only senate members can do.
        NotSenateMember,
        /// Faucet is disabled.
        FaucetDisabled,
        /// Not a subnet owner.
        NotSubnetOwner,
        /// Operation is not permitted on the root subnet.
        RegistrationNotPermittedOnRootSubnet,
        /// A hotkey with too little stake is attempting to join the root subnet.
        StakeTooLowForRoot,
        /// All subnets are in the immunity period.
        AllNetworksInImmunity,
        /// Not enough balance to pay swapping hotkey.
        NotEnoughBalanceToPaySwapHotKey,
        /// Netuid does not match for setting root network weights.
        NotRootSubnet,
        /// Can not set weights for the root network.
        CanNotSetRootNetworkWeights,
        /// No neuron ID is available.
        NoNeuronIdAvailable,
        /// Stake amount below the minimum threshold for nominator validations.
        NomStakeBelowMinimumThreshold,
        /// Delegate take is too low.
        DelegateTakeTooLow,
        /// Delegate take is too high.
        DelegateTakeTooHigh,
        /// Not allowed to commit weights.
        WeightsCommitNotAllowed,
        /// No commit found for the provided hotkey+netuid combination when attempting to reveal the weights.
        NoWeightsCommitFound,
        /// Not the correct block/range to reveal weights.
        InvalidRevealCommitTempo,
        /// Committed hash does not equal the hashed reveal data.
        InvalidRevealCommitHashNotMatch,
        /// Attempting to call set_weights when commit/reveal is enabled
        CommitRevealEnabled,
        /// Attemtping to commit/reveal weights when disabled.
        CommitRevealDisabled,
        /// Not able to join the senate.
        CouldNotJoinSenate,
        /// Attempting to set alpha high/low while disabled
        LiquidAlphaDisabled,
        /// Alpha high is too low: alpha_high > 0.8
        AlphaHighTooLow,
        /// Alpha low is out of range: alpha_low > 0 && alpha_low < 0.8
        AlphaLowOutOfRange,
        /// The coldkey has already been swapped
        ColdKeyAlreadyAssociated,
        /// The coldkey swap transaction rate limit exceeded
        ColdKeySwapTxRateLimitExceeded,
        /// The new coldkey is the same as the old coldkey
        NewColdKeyIsSameWithOld,
        /// The coldkey does not exist
        NotExistColdkey,
        /// The coldkey balance is not enough to pay for the swap
        NotEnoughBalanceToPaySwapColdKey,
        /// No balance to transfer
        NoBalanceToTransfer,
        /// Same coldkey
        SameColdkey,
        /// The coldkey is in arbitration
        ColdkeyIsInArbitration,
        /// The new coldkey is already registered for the drain
        DuplicateColdkey,
        /// Error thrown on a coldkey swap.
        ColdkeySwapError,
        /// Insufficient Balance to Schedule coldkey swap
        InsufficientBalanceToPerformColdkeySwap,
        /// The maximum number of coldkey destinations has been reached
        MaxColdkeyDestinationsReached,
    }
}
