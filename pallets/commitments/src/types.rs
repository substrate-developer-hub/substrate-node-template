// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
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

use codec::{Codec, Decode, Encode, MaxEncodedLen};
use frame_support::{
    traits::{ConstU32, Get},
    BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use scale_info::{
    build::{Fields, Variants},
    Path, Type, TypeInfo,
};
use sp_runtime::{
    traits::{AppendZerosInput, AtLeast32BitUnsigned},
    RuntimeDebug,
};
use sp_std::{fmt::Debug, iter::once, prelude::*};
use subtensor_macros::freeze_struct;

/// Either underlying data blob if it is at most 32 bytes, or a hash of it. If the data is greater
/// than 32-bytes then it will be truncated when encoding.
///
/// Can also be `None`.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen)]
pub enum Data {
    /// No data here.
    None,
    /// The data is stored directly.
    Raw(BoundedVec<u8, ConstU32<128>>),
    /// Only the Blake2 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    BlakeTwo256([u8; 32]),
    /// Only the SHA2-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    Sha256([u8; 32]),
    /// Only the Keccak-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    Keccak256([u8; 32]),
    /// Only the SHA3-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    ShaThree256([u8; 32]),
}

impl Data {
    pub fn is_none(&self) -> bool {
        self == &Data::None
    }
}

impl Decode for Data {
    fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
        let b = input.read_byte()?;
        Ok(match b {
            0 => Data::None,
            n @ 1..=129 => {
                let mut r: BoundedVec<_, _> = vec![0u8; (n as usize).saturating_sub(1)]
                    .try_into()
                    .expect("bound checked in match arm condition; qed");
                input.read(&mut r[..])?;
                Data::Raw(r)
            }
            130 => Data::BlakeTwo256(<[u8; 32]>::decode(input)?),
            131 => Data::Sha256(<[u8; 32]>::decode(input)?),
            132 => Data::Keccak256(<[u8; 32]>::decode(input)?),
            133 => Data::ShaThree256(<[u8; 32]>::decode(input)?),
            _ => return Err(codec::Error::from("invalid leading byte")),
        })
    }
}

impl Encode for Data {
    fn encode(&self) -> Vec<u8> {
        match self {
            Data::None => vec![0u8; 1],
            Data::Raw(ref x) => {
                let l = x.len().min(128) as u8;
                let mut r = vec![l.saturating_add(1)];
                r.extend_from_slice(&x[..]);
                r
            }
            Data::BlakeTwo256(ref h) => once(130).chain(h.iter().cloned()).collect(),
            Data::Sha256(ref h) => once(131).chain(h.iter().cloned()).collect(),
            Data::Keccak256(ref h) => once(132).chain(h.iter().cloned()).collect(),
            Data::ShaThree256(ref h) => once(133).chain(h.iter().cloned()).collect(),
        }
    }
}
impl codec::EncodeLike for Data {}

/// Add a Raw variant with the given index and a fixed sized byte array
macro_rules! data_raw_variants {
    ($variants:ident, $(($index:literal, $size:literal)),* ) => {
		$variants
		$(
			.variant(concat!("Raw", stringify!($size)), |v| v
				.index($index)
				.fields(Fields::unnamed().field(|f| f.ty::<[u8; $size]>()))
			)
		)*
    }
}

impl TypeInfo for Data {
    type Identity = Self;

    fn type_info() -> Type {
        let variants = Variants::new().variant("None", |v| v.index(0));

        // create a variant for all sizes of Raw data from 0-32
        let variants = data_raw_variants!(
            variants,
            (1, 0),
            (2, 1),
            (3, 2),
            (4, 3),
            (5, 4),
            (6, 5),
            (7, 6),
            (8, 7),
            (9, 8),
            (10, 9),
            (11, 10),
            (12, 11),
            (13, 12),
            (14, 13),
            (15, 14),
            (16, 15),
            (17, 16),
            (18, 17),
            (19, 18),
            (20, 19),
            (21, 20),
            (22, 21),
            (23, 22),
            (24, 23),
            (25, 24),
            (26, 25),
            (27, 26),
            (28, 27),
            (29, 28),
            (30, 29),
            (31, 30),
            (32, 31),
            (33, 32),
            (34, 33),
            (35, 34),
            (36, 35),
            (37, 36),
            (38, 37),
            (39, 38),
            (40, 39),
            (41, 40),
            (42, 41),
            (43, 42),
            (44, 43),
            (45, 44),
            (46, 45),
            (47, 46),
            (48, 47),
            (49, 48),
            (50, 49),
            (51, 50),
            (52, 51),
            (53, 52),
            (54, 53),
            (55, 54),
            (56, 55),
            (57, 56),
            (58, 57),
            (59, 58),
            (60, 59),
            (61, 60),
            (62, 61),
            (63, 62),
            (64, 63),
            (65, 64),
            (66, 65),
            (67, 66),
            (68, 67),
            (69, 68),
            (70, 69),
            (71, 70),
            (72, 71),
            (73, 72),
            (74, 73),
            (75, 74),
            (76, 75),
            (77, 76),
            (78, 77),
            (79, 78),
            (80, 79),
            (81, 80),
            (82, 81),
            (83, 82),
            (84, 83),
            (85, 84),
            (86, 85),
            (87, 86),
            (88, 87),
            (89, 88),
            (90, 89),
            (91, 90),
            (92, 91),
            (93, 92),
            (94, 93),
            (95, 94),
            (96, 95),
            (97, 96),
            (98, 97),
            (99, 98),
            (100, 99),
            (101, 100),
            (102, 101),
            (103, 102),
            (104, 103),
            (105, 104),
            (106, 105),
            (107, 106),
            (108, 107),
            (109, 108),
            (110, 109),
            (111, 110),
            (112, 111),
            (113, 112),
            (114, 113),
            (115, 114),
            (116, 115),
            (117, 116),
            (118, 117),
            (119, 118),
            (120, 119),
            (121, 120),
            (122, 121),
            (123, 122),
            (124, 123),
            (125, 124),
            (126, 125),
            (127, 126),
            (128, 127),
            (129, 128)
        );

        let variants = variants
            .variant("BlakeTwo256", |v| {
                v.index(130)
                    .fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
            })
            .variant("Sha256", |v| {
                v.index(131)
                    .fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
            })
            .variant("Keccak256", |v| {
                v.index(132)
                    .fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
            })
            .variant("ShaThree256", |v| {
                v.index(133)
                    .fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
            });

        Type::builder()
            .path(Path::new("Data", module_path!()))
            .variant(variants)
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::None
    }
}

#[freeze_struct("25c84048dcc90813")]
#[derive(
    CloneNoBound, Encode, Decode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[codec(mel_bound())]
#[derive(frame_support::DefaultNoBound)]
#[scale_info(skip_type_params(FieldLimit))]
pub struct CommitmentInfo<FieldLimit: Get<u32>> {
    pub fields: BoundedVec<Data, FieldLimit>,
}

/// Information concerning the identity of the controller of an account.
///
/// NOTE: This is stored separately primarily to facilitate the addition of extra fields in a
/// backwards compatible way through a specialized `Decode` impl.
#[derive(
    CloneNoBound, Encode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxFields))]
pub struct Registration<
    Balance: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq,
    MaxFields: Get<u32>,
    BlockNumber: Codec + Clone + Ord + Eq + AtLeast32BitUnsigned + MaxEncodedLen + Debug,
> {
    /// Amount held on deposit for this information.
    pub deposit: Balance,

    pub block: BlockNumber,

    /// Information on the identity.
    pub info: CommitmentInfo<MaxFields>,
}

// impl<
//         Balance: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq + Zero + Add,
//         MaxFields: Get<u32>,
//         Block: Codec + Clone + Ord + Eq + AtLeast32BitUnsigned + MaxEncodedLen + Debug,
//     > Registration<Balance, MaxFields, Block>
// {
//     pub(crate) fn total_deposit(&self) -> Balance {
//         self.deposit
//     }
// }

impl<
        Balance: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq,
        MaxFields: Get<u32>,
        Block: Codec + Clone + Ord + Eq + AtLeast32BitUnsigned + MaxEncodedLen + Debug,
    > Decode for Registration<Balance, MaxFields, Block>
{
    fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
        let (deposit, block, info) = Decode::decode(&mut AppendZerosInput::new(input))?;
        Ok(Self {
            deposit,
            block,
            info,
        })
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn manual_data_type_info() {
        let mut registry = scale_info::Registry::new();
        let type_id = registry.register_type(&scale_info::meta_type::<Data>());
        let registry: scale_info::PortableRegistry = registry.into();
        let type_info = registry.resolve(type_id.id).unwrap();

        let check_type_info = |data: &Data| {
            let variant_name = match data {
                Data::None => "None".to_string(),
                Data::BlakeTwo256(_) => "BlakeTwo256".to_string(),
                Data::Sha256(_) => "Sha256".to_string(),
                Data::Keccak256(_) => "Keccak256".to_string(),
                Data::ShaThree256(_) => "ShaThree256".to_string(),
                Data::Raw(bytes) => format!("Raw{}", bytes.len()),
            };
            if let scale_info::TypeDef::Variant(variant) = &type_info.type_def {
                let variant = variant
                    .variants
                    .iter()
                    .find(|v| v.name == variant_name)
                    .unwrap_or_else(|| panic!("Expected to find variant {}", variant_name));

                let field_arr_len = variant
                    .fields
                    .first()
                    .and_then(|f| registry.resolve(f.ty.id))
                    .map(|ty| {
                        if let scale_info::TypeDef::Array(arr) = &ty.type_def {
                            arr.len
                        } else {
                            panic!("Should be an array type")
                        }
                    })
                    .unwrap_or(0);

                let encoded = data.encode();
                assert_eq!(encoded[0], variant.index);
                assert_eq!(encoded.len() as u32 - 1, field_arr_len);
            } else {
                panic!("Should be a variant type")
            };
        };

        let mut data = vec![
            Data::None,
            Data::BlakeTwo256(Default::default()),
            Data::Sha256(Default::default()),
            Data::Keccak256(Default::default()),
            Data::ShaThree256(Default::default()),
        ];

        // A Raw instance for all possible sizes of the Raw data
        for n in 0..128 {
            data.push(Data::Raw(vec![0u8; n as usize].try_into().unwrap()))
        }

        for d in data.iter() {
            check_type_info(d);
        }
    }
}
