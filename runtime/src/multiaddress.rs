// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! MultiAddress type is a wrapper for multiple downstream account formats.
//! Copied from `sp_runtime::multiaddress`, and can be removed and replaced when
//! updating to a newer version of Substrate.

use codec::{Encode, Decode, Codec};
use sp_std::{vec::Vec, marker::PhantomData, fmt::Debug};
use sp_runtime::{RuntimeDebug, traits::{LookupError, StaticLookup}};

/// A multi-format address wrapper for on-chain accounts.
#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum MultiAddress<AccountId, AccountIndex> {
	/// It's an account ID (pubkey).
	Id(AccountId),
	/// It's an account index.
	Index(#[codec(compact)] AccountIndex),
	/// It's some arbitrary raw bytes.
	Raw(Vec<u8>),
	/// It's a 32 byte representation.
	Address32([u8; 32]),
	/// Its a 20 byte representation.
	Address20([u8; 20]),
}

#[cfg(feature = "std")]
impl<AccountId, AccountIndex> std::fmt::Display for MultiAddress<AccountId, AccountIndex>
where
	AccountId: std::fmt::Debug,
	AccountIndex: std::fmt::Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		use sp_core::hexdisplay::HexDisplay;
		match self {
			MultiAddress::Raw(inner) => write!(f, "MultiAddress::Raw({})", HexDisplay::from(inner)),
			MultiAddress::Address32(inner) => write!(f, "MultiAddress::Address32({})", HexDisplay::from(inner)),
			MultiAddress::Address20(inner) => write!(f, "MultiAddress::Address20({})", HexDisplay::from(inner)),
			_ => write!(f, "{:?}", self),
		}
	}
}

impl<AccountId, AccountIndex> From<AccountId> for MultiAddress<AccountId, AccountIndex> {
	fn from(a: AccountId) -> Self {
		MultiAddress::Id(a)
	}
}

impl<AccountId: Default, AccountIndex> Default for MultiAddress<AccountId, AccountIndex> {
	fn default() -> Self {
		MultiAddress::Id(Default::default())
	}
}

/// A lookup implementation returning the `AccountId` from a `MultiAddress`.
pub struct AccountIdLookup<AccountId, AccountIndex>(PhantomData<(AccountId, AccountIndex)>);
impl<AccountId, AccountIndex> StaticLookup for AccountIdLookup<AccountId, AccountIndex>
where
	AccountId: Codec + Clone + PartialEq + Debug,
	AccountIndex: Codec + Clone + PartialEq + Debug,
	MultiAddress<AccountId, AccountIndex>: Codec,
{
	type Source = MultiAddress<AccountId, AccountIndex>;
	type Target = AccountId;
	fn lookup(x: Self::Source) -> Result<Self::Target, LookupError> {
		match x {
			MultiAddress::Id(i) => Ok(i),
			_ => Err(LookupError),
		}
	}
	fn unlookup(x: Self::Target) -> Self::Source {
		MultiAddress::Id(x)
	}
}
