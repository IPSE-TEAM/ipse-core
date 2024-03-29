// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
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

//! Address type that is union of index and id for an account.

use crate::Member;
use codec::{Decode, Encode, Error, Input, Output};
use sp_std::convert::TryInto;
#[cfg(feature = "std")]
use std::fmt;

/// An indices-aware address, which can be either a direct `AccountId` or
/// an index.
#[derive(PartialEq, Eq, Clone, sp_runtime::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum Address<AccountId, AccountIndex>
where
	AccountId: Member,
	AccountIndex: Member,
{
	/// It's an account ID (pubkey).
	Id(AccountId),
	/// It's an account index.
	Index(AccountIndex),
}

#[cfg(feature = "std")]
impl<AccountId, AccountIndex> fmt::Display for Address<AccountId, AccountIndex>
where
	AccountId: Member,
	AccountIndex: Member,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl<AccountId, AccountIndex> From<AccountId> for Address<AccountId, AccountIndex>
where
	AccountId: Member,
	AccountIndex: Member,
{
	fn from(a: AccountId) -> Self {
		Address::Id(a)
	}
}

fn need_more_than<T: PartialOrd>(a: T, b: T) -> Result<T, Error> {
	if a < b {
		Ok(b)
	} else {
		Err("Invalid range".into())
	}
}

impl<AccountId, AccountIndex> Decode for Address<AccountId, AccountIndex>
where
	AccountId: Member + Decode,
	AccountIndex: Member + Decode + PartialOrd<AccountIndex> + Ord + From<u32> + Copy,
{
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		Ok(match input.read_byte()? {
			x @ 0x00..=0xef => Address::Index(AccountIndex::from(x as u32)),
			0xfc => Address::Index(AccountIndex::from(
				need_more_than(0xef, u16::decode(input)?)? as u32
			)),
			0xfd =>
				Address::Index(AccountIndex::from(need_more_than(0xffff, u32::decode(input)?)?)),
			0xfe => Address::Index(need_more_than(0xffffffffu32.into(), Decode::decode(input)?)?),
			0xff => Address::Id(Decode::decode(input)?),
			_ => return Err("Invalid address variant".into()),
		})
	}
}

impl<AccountId, AccountIndex> Encode for Address<AccountId, AccountIndex>
where
	AccountId: Member + Encode,
	AccountIndex:
		Member + Encode + PartialOrd<AccountIndex> + Ord + Copy + From<u32> + TryInto<u32>,
{
	fn encode_to<T: Output>(&self, dest: &mut T) {
		match *self {
			Address::Id(ref i) => {
				dest.push_byte(255);
				dest.push(i);
			},
			Address::Index(i) => {
				let maybe_u32: Result<u32, _> = i.try_into();
				if let Ok(x) = maybe_u32 {
					if x > 0xffff {
						dest.push_byte(253);
						dest.push(&x);
					} else if x >= 0xf0 {
						dest.push_byte(252);
						dest.push(&(x as u16));
					} else {
						dest.push_byte(x as u8);
					}
				} else {
					dest.push_byte(254);
					dest.push(&i);
				}
			},
		}
	}
}

impl<AccountId, AccountIndex> codec::EncodeLike for Address<AccountId, AccountIndex>
where
	AccountId: Member + Encode,
	AccountIndex:
		Member + Encode + PartialOrd<AccountIndex> + Ord + Copy + From<u32> + TryInto<u32>,
{
}

impl<AccountId, AccountIndex> Default for Address<AccountId, AccountIndex>
where
	AccountId: Member + Default,
	AccountIndex: Member,
{
	fn default() -> Self {
		Address::Id(Default::default())
	}
}

#[cfg(test)]
mod tests {
	use codec::{Decode, Encode};

	type Address = super::Address<[u8; 8], u32>;
	fn index(i: u32) -> Address {
		super::Address::Index(i)
	}
	fn id(i: [u8; 8]) -> Address {
		super::Address::Id(i)
	}

	fn compare(a: Option<Address>, d: &[u8]) {
		if let Some(ref a) = a {
			assert_eq!(d, &a.encode()[..]);
		}
		assert_eq!(Address::decode(&mut &d[..]).ok(), a);
	}

	#[test]
	fn it_should_work() {
		compare(Some(index(2)), &[2][..]);
		compare(None, &[240][..]);
		compare(None, &[252, 239, 0][..]);
		compare(Some(index(240)), &[252, 240, 0][..]);
		compare(Some(index(304)), &[252, 48, 1][..]);
		compare(None, &[253, 255, 255, 0, 0][..]);
		compare(Some(index(0x10000)), &[253, 0, 0, 1, 0][..]);
		compare(
			Some(id([42, 69, 42, 69, 42, 69, 42, 69])),
			&[255, 42, 69, 42, 69, 42, 69, 42, 69][..],
		);
	}
}
