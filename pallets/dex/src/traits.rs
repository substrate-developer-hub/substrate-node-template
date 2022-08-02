use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

/// Trait for exposing the pricing of asset swaps to other pallets.  
/// **Note:** Should ideally be defined in a separate crate for loose coupling
pub trait Price {
	// Means of identifying one asset class from another.
	type AssetId;

	/// Scalar type for representing balance of an account.
	type Balance;

	/// Calculates the output amount of asset `other`, given an input `amount` and `asset` type.
	/// # Arguments
	/// * `amount` - An amount to be valued.
	/// * `asset` - The asset type of the amount.
	/// * `other` - The required asset type.
	fn price(
		amount: Self::Balance,
		asset: Self::AssetId,
		other: Self::AssetId,
	) -> Result<Self::Balance, DispatchError>;
}

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
