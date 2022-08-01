use super::*;

// todo: document
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Listing<T: Config> {
	/// The identifier of the liquidity pool asset
	pub(super) id: AssetIdOf<T>,
}
