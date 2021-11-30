// Copyright 2021 IPSE  Developer.
// This file is part of IPSE

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

use alt_serde::de::{Error, Unexpected}; // declare_error_trait
use alt_serde::{Deserialize, Deserializer};
use app_crypto::sr25519;
use codec::{Decode, Encode};
use frame_support::{debug, Parameter};
use frame_support::{
	traits::{Currency, LockableCurrency},
	StorageMap, StorageValue,
};
use frame_system::{self as system};
use hex;
use pallet_authority_discovery as authority_discovery;
use pallet_timestamp as timestamp;
use sp_core::{crypto::KeyTypeId, offchain::Timestamp};
use sp_runtime::offchain::http;
use sp_runtime::RuntimeAppPublic;
use sp_std::{convert::TryInto, prelude::*};

pub const CONTRACT_ACCOUNT: &[u8] = b"ipsecontract";
pub const DESTROY_ACCOUNT: &[u8] = b"eosio.saving";
pub const CONTRACT_SYMBOL: &[u8] = b"POST";
pub const VERIFY_STATUS: &[u8] = b"verify_status";
pub const PENDING_TIME_OUT: &'static str = "Error in waiting http response back";
pub const WAIT_HTTP_CONVER_REPONSE: &'static str = "Error in waiting http_result convert response";

#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum AddressStatus {
	Active,
	InActive,
}

impl Default for AddressStatus {
	fn default() -> Self {
		Self::InActive
	}
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct PostTxTransferData {
	pub code: u64,
	pub irreversible: bool,
	pub is_post_transfer: bool,

	#[serde(deserialize_with = "de_string_to_bytes")]
	pub contract_account: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	pub from: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	pub to: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	pub contract_symbol: Vec<u8>,
	#[serde(deserialize_with = "de_float_to_integer")]
	pub quantity: u64,
	#[serde(deserialize_with = "de_string_to_bytes")]
	pub memo: Vec<u8>,
	#[serde(deserialize_with = "de_string_decode_bytes")]
	pub pk: Vec<u8>,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	Ok(s.as_bytes().to_vec())
}

pub fn de_string_decode_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;

	if s.len() < 2 {
		return Err(D::Error::invalid_value(Unexpected::Str(s), &"0x..."));
	}
	match hex::decode(&s[2..]) {
		Ok(s_vec) => Ok(s_vec),
		Err(e) => {
			debug::error!("{:?}", e);
			Err(D::Error::invalid_value(Unexpected::Str(s), &""))
		}
	}
	// Ok(s.as_bytes().to_vec())
}

pub fn de_float_to_integer<'de, D>(de: D) -> Result<u64, D::Error>
where
	D: Deserializer<'de>,
{
	let f: f64 = Deserialize::deserialize(de)?;
	Ok(f as u64)
}

pub(crate) const POST_KEYWORD: [&[u8]; 5] = [
	b"{",   // {
	b"\"",  // "
	b"\":", // ":
	b",",   // ,
	b"}",   // }
];

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode, Clone)]
pub struct FetchFailed<BlockNumber> {
	pub block_num: BlockNumber,
	pub tx: Vec<u8>,
	pub err: Vec<u8>,
}

pub type FetchFailedOf<T> = FetchFailed<<T as system::Trait>::BlockNumber>;

pub type BlockNumberOf<T> = <T as system::Trait>::BlockNumber; // u32
pub type StdResult<T> = core::result::Result<T, &'static str>;

pub type StrDispatchResult = core::result::Result<(), &'static str>;

pub fn vecchars_to_vecbytes<I: IntoIterator<Item = char> + Clone>(it: &I) -> Vec<u8> {
	it.clone().into_iter().map(|c| c as u8).collect::<_>()
}

pub fn int_covert_str(inner: u64) -> Vec<u8> {
	let mut x: u32 = 0;
	let mut s: Vec<&str> = vec![];
	loop {
		let r = inner / ((10 as u64).pow(x));
		if r == 0 {
			s.reverse();
			return s.join("").as_bytes().to_vec();
		}
		let r = r % 10;
		s.push(num_to_char(r));
		x += 1;
	}
}

pub fn num_to_char<'a>(n: u64) -> &'a str {
	if n > 10 {
		return "";
	}
	match n {
		0 => "0",
		1 => "1",
		2 => "2",
		3 => "3",
		4 => "4",
		5 => "5",
		6 => "6",
		7 => "7",
		8 => "8",
		9 => "9",
		_ => "",
	}
}

pub fn hex_to_u8(param: &[u8]) -> Vec<u8> {
	let hex_0x = "0x".as_bytes();
	let tx_hex = hex::encode(param);
	let tx_vec = &[hex_0x, tx_hex.as_bytes()].concat();

	return tx_vec.to_vec();
}

pub trait AccountIdPublicConver {
	type AccountId;
	fn into_account32(self) -> Self::AccountId; // 转化为accountId
}
