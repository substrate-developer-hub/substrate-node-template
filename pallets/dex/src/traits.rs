use frame_support::dispatch::DispatchResult;

/// Trait for exposing asset swapping to other pallets.  
/// **Note:** Should ideally be defined in a separate crate for loose coupling
pub trait Swap<AccountId> {
	// Means of identifying one asset class from another.
	type AssetId;

	/// Scalar type for representing balance of an account.
	type Balance;

	/// Performs a swap of an `amount` of the specified `asset` to the `other` asset.  
	/// # Arguments
	/// * `amount` - An amount to be swapped.
	/// * `asset` - The identifier of the asset type to be swapped.
	/// * `other` - The identifier of the other asset type.
	/// * `buyer` - The identifier of the account initiating the swap.
	fn swap(
		amount: Self::Balance,
		asset: Self::AssetId,
		other: Self::AssetId,
		buyer: AccountId,
	) -> DispatchResult;
}
