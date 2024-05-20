use codec::{Encode, MaxEncodedLen};
use core::fmt::Debug;
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::traits::Get;
use static_assertions::_core::cmp::Ordering;

#[derive(Encode, MaxEncodedLen, Default, TypeInfo)]
#[scale_info(skip_type_params(N))]
pub struct BoundedString<N: Get<u32>>(BoundedVec<u8, N>);

impl<N: Get<u32>> BoundedString<N> {
	pub fn truncate_from(data: &str) -> Self {
		Self(BoundedVec::truncate_from(data.as_bytes().to_vec()))
	}
}

impl<N: Get<u32>> codec::Decode for BoundedString<N> {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let inner = BoundedVec::<u8, N>::decode(input)?;
		core::str::from_utf8(&inner).map_err(|_| "Invalid UTF-8 string")?;
		Ok(Self(inner))
	}
}

impl<N: Get<u32>> Clone for BoundedString<N> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<N: Get<u32>> PartialEq for BoundedString<N> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<N: Get<u32>> Eq for BoundedString<N> {}

impl<N: Get<u32>> Debug for BoundedString<N> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		if let Ok(s) = core::str::from_utf8(&self.0) {
			write!(f, "{:?}", s)
		} else {
			write!(f, "<invalid utf8 string>")
		}
	}
}

impl<N: Get<u32>> TryFrom<&str> for BoundedString<N> {
	type Error = ();
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Ok(Self(value.as_bytes().to_vec().try_into().map_err(|_| ())?))
	}
}

impl<N: Get<u32>> PartialOrd for BoundedString<N> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<N: Get<u32>> Ord for BoundedString<N> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.cmp(&other.0)
	}
}
