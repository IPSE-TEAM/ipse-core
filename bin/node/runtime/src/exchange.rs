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

use sp_core::crypto::AccountId32 as AccountId;
use sp_core::{crypto::KeyTypeId, offchain::Timestamp};
use sp_std::convert::{Into, TryFrom, TryInto};
use sp_std::{fmt::Debug, prelude::*};

use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, print,
	traits::{Currency, Get, OnUnbalanced},
	weights::Weight,
	IterableStorageMap, Parameter, StorageDoubleMap,
};
use frame_system::{
	self as system, ensure_none, ensure_root, ensure_signed, offchain, Origin, RawOrigin,
};
use hex;

use pallet_authority_discovery as authority_discovery;
use pallet_timestamp as timestamp;

use codec::{Decode, Encode};
use frame_system::offchain::{SendTransactionTypes, SubmitTransaction};
use num_traits::float::FloatCore;
use sp_io::{self, misc::print_utf8 as print_bytes};
use sp_runtime::{
	traits::{MaybeDisplay, MaybeSerializeDeserialize},
	DispatchError, DispatchResult,
};

use app_crypto::sr25519;
use sp_runtime::{
	offchain::http,
	traits::{CheckedAdd, CheckedSub, IdentifyAccount, Member, Printable, Zero},
	transaction_validity::{
		InvalidTransaction, TransactionLongevity, TransactionPriority, TransactionSource,
		TransactionValidity, ValidTransaction,
	},
	AnySignature, MultiSignature, MultiSigner, RuntimeAppPublic,
};

use crate::constants::currency;
use crate::ocw_common::*;

const EOS_NODE_URL: &[u8] = b"http://localhost:8421/v1/eosio/tx/";

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
// type BalanceOf<T> = <<T as staking::Trait>::StakingCurrency as Currency<<T as
// frame_system::Trait>::AccountId>>::Balance;
type PositiveImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::PositiveImbalance;

type Signature = AnySignature;
pub mod eos_crypto {
	use super::{debug, AccountIdPublicConver, Signature};
	pub mod app_sr25519 {
		use super::{debug, AccountIdPublicConver};
		use sp_core::crypto::AccountId32 as AccountId;
		use sp_runtime::app_crypto::{app_crypto, key_types::ACCOUNT, sr25519};
		use sp_runtime::traits::IdentifyAccount; // AccountIdConversion,
		use sp_runtime::{MultiSignature, MultiSigner};
		app_crypto!(sr25519, ACCOUNT);

		impl From<Signature> for super::Signature {
			fn from(a: Signature) -> Self {
				sr25519::Signature::from(a).into()
			}
		}

		impl AccountIdPublicConver for Public {
			type AccountId = AccountId;
			fn into_account32(self) -> AccountId {
				let s: sr25519::Public = self.into();
				MultiSigner::from(s).into_account()
			}
		}

		impl From<AccountId> for Public {
			fn from(acct: AccountId) -> Self {
				let mut data = [0u8; 32];
				let acct_data: &[u8; 32] = acct.as_ref();
				for (index, val) in acct_data.iter().enumerate() {
					data[index] = *val;
				}
				Self(sr25519::Public(data))
			}
		}

		impl From<[u8; 32]> for Public {
			fn from(acct: [u8; 32]) -> Self {
				let mut data = [0u8; 32];
				for (index, val) in acct.iter().enumerate() {
					data[index] = *val;
				}
				Self(sr25519::Public(data))
			}
		}
	}

	app_crypto::with_pair! {
		/// An bridge-eos keypair using sr25519 as its crypto.
		pub type AuthorityPair = app_sr25519::Pair;
	}

	pub type AuthoritySignature = app_sr25519::Signature;

	pub type AuthorityId = app_sr25519::Public;
}

enum VerifyStatus {
	Continue,
	Failed,
	Pass,
}

/// The module's configuration trait.
pub trait Trait: system::Trait + timestamp::Trait + SendTransactionTypes<Call<Self>> {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	// type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord +
	// Default                 + AsRef<[u8]> + From<[u8; 32]>;

	/// The local AuthorityId
	type AuthorityId: RuntimeAppPublic
		+ Clone
		+ Parameter
		+ Into<sr25519::Public>
		+ From<sr25519::Public>
		+ AccountIdPublicConver<AccountId = Self::AccountId>
		+ From<<Self as frame_system::Trait>::AccountId>
		+ From<[u8; 32]>;

	type TxsMaxCount: Get<u32>;

	type Deadline: Get<Self::BlockNumber>;

	type Duration: Get<Self::BlockNumber>; // 对记录的清除周期

	type UnsignedPriority: Get<TransactionPriority>;

	type OnUnbalanced: OnUnbalanced<PositiveImbalanceOf<Self>>;

	type Currency: Currency<Self::AccountId>;
}

decl_error! {
	pub enum Error for Module<T: Trait> {
	  /// account not match
	  MemoMissMatch,

	  TxExisted,

	  TxInUsing,

	  /// txsoverlimit
	  OverMaximum,

	  Empty,

	  Exist,

	  /// account error
	  MemoInvalid,

	  /// tx exchaned
	  Exchanged,

	  /// Out of exchange time
	  DeadExchange,

	  NoPermission,
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		BlockNumber = <T as system::Trait>::BlockNumber,
		Amount = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance,
	{
		FetchedSuc(AccountId, BlockNumber, Vec<u8>, u64),

		FailedEvent(AccountId, BlockNumber, Vec<u8>),

		AddExchangeQueueEvent(Vec<u8>), //

		CreateToken(AccountId, Amount),
	}
);

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as PostExchange {

		RootDeadlineTime get(fn root_deadline_time) config(): T::BlockNumber;

		TokenStatus get(fn tx_status): map hasher(blake2_128_concat) Vec<u8> => (u64,T::AccountId); // 收款的账号

		pub TokenStatusLen: u32;

		/// The current set of notary keys that may send bridge transactions to Eos chain.
		NotaryKeys get(fn notary_keys) config(): Vec<T::AccountId>;   // 使用 json文件来配置

		SucTxExchange get(fn suc_tx_exchange): map hasher(blake2_128_concat) Vec<u8> => Option<bool>;

		EosExchangeInfo get(fn eos_exchange_info): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) Vec<u8> => (AddressStatus, u64);

	   FetchRecord get(fn fetch_record): double_map hasher(blake2_128_concat) T::BlockNumber,hasher(blake2_128_concat) T::AccountId => (u32,u32,u32);

	   pub FetchFailed get(fn fetch_failed): map hasher(blake2_128_concat) T::AccountId => Vec<FetchFailedOf<T>>;
  }
	  add_extra_genesis {
		build(|config: &GenesisConfig<T>| {
			NotaryKeys::<T>::put(config.notary_keys.clone());
			<RootDeadlineTime<T>>::put(T::Deadline::get());

		});
	}
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
	type Error = Error<T>;
	// Initializing events
	// this is needed only if you are using events in your module
	fn deposit_event() = default;

	// Clean the state on initialization of the block
	fn on_initialize(block: T::BlockNumber) -> Weight{
		if (block % T::Duration::get()).is_zero() {

		for key_value in <FetchFailed<T>>::iter().into_iter(){
			let (key,val) = key_value;
			 <FetchFailed<T>>::remove(&key);
			}
		}
		0
	   }

	 #[weight = 0]
	 fn SetExchangeDealine(origin, deadline_time: T::BlockNumber) -> DispatchResult{
			ensure_root(origin)?;
			<RootDeadlineTime<T>>::put(deadline_time);
			Ok(())
	 }

	 #[weight = 0]
	 fn set_notary_keys(origin, node: T::AccountId) -> DispatchResult{
			ensure_root(origin)?;
			<NotaryKeys<T>>::try_mutate(|keys| {
			if keys.len()>=50{
				return Err(Error::<T>::OverMaximum)?;
			}
			if keys.contains(&node){
				debug::info!("--------Exist {:?}----------",node);
				return Err(Error::<T>::Exist);
			}
			keys.push(node);
			Ok(())
			})?;
			// debug::info!("--------set_notary_keys----------");
			Ok(())
	 }

	 #[weight = 0]
	 fn del_notary_keys(origin, node: T::AccountId) -> DispatchResult{
		ensure_root(origin)?;
		let keys = NotaryKeys::<T>::get();
		 <NotaryKeys<T>>::try_mutate(|keys| {
			if keys.is_empty(){
				return Err(Error::<T>::Empty);
			}
			keys.retain(|key| *key != node);
			return Ok(());
		 })?;
		 //  debug::info!("--------del_notary_keys----------");
		 Ok(())
	 }


	 #[weight = 10_000]
	 fn exchange(origin, tx: Vec<u8>) -> DispatchResult{
		// let deadline_blocknum = sp_std::cmp::max(<RootDeadlineTime<T>>::get(),T::Deadline::get());
		let deadline_blocknum = <RootDeadlineTime<T>>::get();
		if deadline_blocknum < <system::Module<T>>::block_number(){
			return Err(Error::<T>::DeadExchange)?;
		}
		let who = ensure_signed(origin)?;
		// let account = Self::vec_convert_account(memo.clone()).ok_or(Error::<T>::MemoInvalid)?;
		// debug::info!("memo is {:?}",account);

		let tx_hex = hex::encode(&tx);
		debug::info!("verify tx = {:?}",tx_hex);
		match SucTxExchange::get(&tx){
			Some(tx) => return Err(Error::<T>::Exchanged)?,
			_ => (),
		}

		let curent_status = <TokenStatus<T>>::get(tx.clone()).0;

		ensure!(<TokenStatus<T>>::get(tx.clone()).0 == 0, Error::<T>::TxInUsing);
		debug::info!("TokenStatusLen: {:?}",TokenStatusLen::get());
		ensure!(TokenStatusLen::get() <= T::TxsMaxCount::get(), Error::<T>::OverMaximum);
		<TokenStatus<T>>::insert(tx.clone(),(1000,who.clone()));
		debug::info!("TokenStatus init status:{:?}",<TokenStatus<T>>::get(tx.clone()).0);
		TokenStatusLen::mutate(|n|*n += 1);
		Self::deposit_event(RawEvent::AddExchangeQueueEvent(tx.clone()));
		Ok(())
	 }


	#[weight = 0]
	fn record_suc_verify(
	  origin,
	  block_num: T::BlockNumber,
	  account: T::AccountId,
	  key: T::AuthorityId,
	  tx:Vec<u8>,
	  status: u64,
	  quantity: u64,
	  _signature: <T::AuthorityId as RuntimeAppPublic>::Signature
	) -> DispatchResult{
	  ensure_none(origin)?;
	  let now = <timestamp::Module<T>>::get();
	  let block_num = <system::Module<T>>::block_number();
	  let duration = block_num / T::Duration::get();

	   let (token_status,accept_account) = <TokenStatus<T>>::get(tx.clone());
	   debug::info!("token_status={:?},accept_account={:?}",token_status,accept_account);
	   ensure!(<TokenStatus<T>>::contains_key(tx.clone()), "tx removed from TokenStatus");
	   match SucTxExchange::get(&tx){
		Some(_) => return Err(Error::<T>::Exchanged)?,
		_ => (),
	  }
	  debug::info!("The return message of the local service was obtained.operate status");
	  // let status = post_tx_transfer_data.code;
	  if status == 0{   // 0

	   <FetchRecord<T>>::mutate(
		duration,account.clone(),
		|val|{
			val.0 = val.0.checked_add(1).unwrap();
		});
	   <TokenStatus<T>>::mutate(&tx,|val|val.0 = val.0.checked_add(100).unwrap());
	  }else if status == 1 { // query  ./token-query
		 <FetchRecord<T>>::mutate(
			duration,account.clone(),
			|val|{
				val.2 = val.2.checked_add(1).unwrap();
//                val
			});
			 <TokenStatus<T>>::mutate(&tx,|val|val.0 = val.0.checked_add(1).unwrap());
	  }else{
		<FetchRecord<T>>::mutate( // 200x
			duration,account.clone(),
			|val|{
				val.1 = val.1.checked_add(1).unwrap();
//                val
			});
			 <TokenStatus<T>>::mutate(&tx,|val|val.0 = val.0.checked_add(10).unwrap());
	  }
	  Self::verify_handle(&tx, quantity);
	  debug::info!("----onchain: record_suc_verify:{:?}-----", duration);
	  Ok(())
	}

	#[weight = 0]
	fn record_fail_verify(
		origin,
		block: T::BlockNumber,
		account: T::AccountId,
		key: T::AuthorityId,
		tx: Vec<u8>,
		err: Vec<u8>,
		_signature: <T::AuthorityId as RuntimeAppPublic>::Signature
		)->DispatchResult{
			ensure_none(origin)?;
			<TokenStatus<T>>::try_mutate(&tx,|val|{
				if val.0 == 0 {
					debug::error!("TokenStatus status 0.tx removed from TokenStatus");
					return Err("");
				}
				debug::info!("record_fail_verify status bit +1");
				val.0 = val.0.checked_add(1).unwrap();
				return Ok(&tx)}
			)?;

			let failed_struct = FetchFailedOf::<T> {
					block_num: block,
					tx: tx.clone(),
					err: err.clone()
			};
			let status:u64 = <TokenStatus<T>>::get(tx.clone()).0;
			debug::info!("------verify failed:status={:?},tx={:?}-------",status,hex::encode(&tx));
			Self::verify_handle(&tx, 0);
			<FetchFailed<T>>::mutate(&account, |fetch_failed| {
			if fetch_failed.len()>50{
				fetch_failed.pop();
			}
			fetch_failed.push(failed_struct)
			});
			/**
			if err == WAIT_HTTP_CONVER_REPONSE.as_bytes().to_vec(){
			}
			*/

			Self::deposit_event(RawEvent::FailedEvent(account.clone(),block,tx));
			debug::info!("------failed fetch  are recorded onchain: record_fail_verify--------");
			Ok(())
	}


	fn offchain_worker(block: T::BlockNumber) {
		if sp_io::offchain::is_validator() {
			 if let (Some(authority_id),Some(local_account)) = Self::local_authority_keys() {  // local_account
				debug::info!("-----------exchange offchain work------------");
				match Self::offchain(block,authority_id,&local_account){
					Err(e)=>{
						debug::error!("ocw excute error:{:?}",e);
					},
					_ => debug::info!("ocw excute suc"),
				}
			}
		}
	} // end of `fn offchain_worker()`



	}
}

impl<T: Trait> Module<T> {
	fn offchain(
		block_num: T::BlockNumber,
		key: T::AuthorityId,
		local_account: &T::AccountId,
	) -> DispatchResult {
		for (tx_key, value) in <TokenStatus<T>>::iter().into_iter() {
			let (status, accept_account) = value;
			let tx = tx_key;
			let tx_hex = hex::encode(&tx);
			debug::info!("iterator tx = {:?}", tx_hex);

			// post json
			// let body = Self::get_json(&tx,&accept_account).ok_or("get_json error");
			// let body = match body{
			//     Ok(body) => body,
			//     Err(e) => {
			//         debug::error!("---------{:?}---------",e);
			//         Self::call_record_fail_verify(block_num,key.clone(),local_account,&tx,e)?;
			//         return Err(DispatchError::Other("get_json error"));
			//     }
			// };

			// get
			let body = tx_hex.as_bytes().to_vec();

			match Self::fetch_http_result_status(EOS_NODE_URL, body, accept_account) {
				Ok(mut post_tx_transfer_data) => {
					let tx_hex = hex::encode(&tx);
					debug::info!(
						"*** fetch ***: {:?}:{:?}",
						core::str::from_utf8(EOS_NODE_URL).unwrap(),
						tx_hex
					);
					Self::call_record_address(
						block_num,
						key.clone(),
						local_account,
						&tx,
						post_tx_transfer_data,
					)?;
				},
				Err(e) => {
					debug::info!("~~~~~~ Error address fetching~~~~~~~~:  {:?}: {:?}", tx_hex, e);
					Self::call_record_fail_verify(block_num, key.clone(), local_account, &tx, e)?;
				},
			}
			break
		}
		Ok(())
	}

	fn call_record_fail_verify<'a>(
		block_num: T::BlockNumber,
		key: T::AuthorityId,
		account: &T::AccountId,
		tx: &'a [u8],
		e: &'a str,
	) -> StrDispatchResult {
		let signature = key
			.sign(&(block_num, account.clone(), tx.to_vec()).encode())
			.ok_or("signing failed!")?;
		debug::info!(
			"record_fail_verify signed,block_num = {:?},tx={:?}",
			block_num,
			hex::encode(&tx)
		);

		let call = Call::record_fail_verify(
			block_num,
			account.clone(),
			key.clone(),
			tx.to_vec(),
			e.as_bytes().to_vec(),
			signature,
		);
		SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).map_err(
			|_| {
				debug::error!("===record_fail_verify: submit_unsigned_call error===");
				"===record_fail_verify: submit_unsigned_call error==="
			},
		)?;
		debug::info!("+++++++record_fail_verify suc++++++++++++++");
		Ok(())
	}

	fn call_record_address(
		block_num: T::BlockNumber,
		key: T::AuthorityId,
		account: &T::AccountId,
		tx: &[u8], //tx
		post_tx_transfer_data: PostTxTransferData,
	) -> StrDispatchResult {
		let signature = key
			.sign(&(block_num, account.clone(), tx.to_vec()).encode())
			.ok_or("signing failed!")?;
		debug::info!(
			"record_suc_verify signed,block_num = {:?},tx={:?}",
			block_num,
			hex::encode(tx)
		);
		let call = Call::record_suc_verify(
			block_num,
			account.clone(),
			key.clone(),
			tx.to_vec(),
			post_tx_transfer_data.code,
			post_tx_transfer_data.quantity,
			signature,
		);

		// Unsigned tx
		SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).map_err(|e| {
			debug::info!("{:?}", e);
			"============fetch_price: submit_signed(call) error============"
		})?;

		debug::info!("***fetch price over ^_^***");
		Ok(())
	}

	fn get_json(tx: &[u8], accept_account: &T::AccountId) -> Option<Vec<u8>> {
		let keys: [&[u8]; 2] = [b"tx", b"account"];
		let tx_vec = hex_to_u8(tx);
		debug::info!("the converted tx={:?}", core::str::from_utf8(&tx_vec).ok()?);
		let acc = Self::account_convert_u8(accept_account.clone());
		debug::info!("the converted acc={:?}", core::str::from_utf8(&acc).ok()?);
		let vals: [&[u8]; 2] = [&tx_vec, &acc];

		let mut json_val = vec![POST_KEYWORD[0]];
		for (i, key) in keys.iter().enumerate() {
			json_val.push(POST_KEYWORD[1]); //json_val.push("\"");
			json_val.push(key);
			json_val.push(POST_KEYWORD[2]); //json_val.push("\":");
			json_val.push(POST_KEYWORD[1]); // json_val.push("\"");
			json_val.push(vals[i]);
			json_val.push(POST_KEYWORD[1]); //json_val.push("\"");
			json_val.push(POST_KEYWORD[3]); //json_val.push(",");
		}
		json_val.pop();
		json_val.push(POST_KEYWORD[4]); //json_val.push("}");

		let json_vec = json_val.concat().to_vec();
		debug::info!("requested json:{:?}", core::str::from_utf8(&json_vec).ok()?);

		Some(json_vec)
	}

	fn local_authority_keys() -> (Option<T::AuthorityId>, Option<T::AccountId>) {
		let authorities = NotaryKeys::<T>::get();
		let key_id = core::str::from_utf8(&T::AuthorityId::ID.0).unwrap();
		debug::info!("keytypeId: {:?}", key_id);
		for i in T::AuthorityId::all().iter() {
			let authority: T::AuthorityId = (*i).clone();
			let authority_sr25519: sr25519::Public = authority.clone().into();
			let s: T::AccountId = authority.clone().into_account32();
			debug::info!("local accounts:{:?}", s);
			if authorities.contains(&s) {
				debug::info!("matched account: {:?}", s);
				return (Some(authority), Some(s))
			}
		}
		return (None, None)
	}

	fn verify_handle(tx: &[u8], quantity: u64) -> StdResult<VerifyStatus> {
		let mut verify_status: VerifyStatus = VerifyStatus::Continue; //初始化一个值
		let (status, accept_account) = <TokenStatus<T>>::get(tx);
		let num = TokenStatusLen::get();
		if status < 1000 {
			debug::error!(
				"=====Binding validation failed: tx={:?},status {:?},=====",
				hex::encode(tx),
				status
			);
			<TokenStatus<T>>::remove(tx);
			if num > 0 {
				TokenStatusLen::mutate(|n| *n -= 1);
			}
			return Err("status < 1000")
		}

		debug::info!("--------onchain set status={:?}--------", status);
		let units_digit = status % 10;
		let tens_digit = status / 10 % 10;
		let hundreds_digit = status / 100 % 10;

		if tens_digit >= 6 {
			verify_status = VerifyStatus::Failed;
		} else if hundreds_digit >= 3 {
			verify_status = VerifyStatus::Pass;
		} else if units_digit >= 8 {
			if hundreds_digit >= 2 {
				verify_status = VerifyStatus::Pass;
			} else {
				verify_status = VerifyStatus::Failed;
			}
		} else {
			verify_status = VerifyStatus::Continue;
		}

		// let active_status = <EosExchangeInfo<T>>::get(accept_account.clone(), tx.clone());
		match verify_status {
			VerifyStatus::Failed => {
				debug::info!("--fail to register--");
				<TokenStatus<T>>::remove(tx);
				debug::info!("remove tx={:?},queue len:{:?} ", hex::encode(tx.clone()), num);
				if num > 0 {
					TokenStatusLen::mutate(|n| *n -= 1);
				}
				<EosExchangeInfo<T>>::insert(
					accept_account.clone(),
					tx.clone(),
					(AddressStatus::InActive, quantity),
				);
				// Self::insert_active_status(accept_account.clone(), tx, AddressStatus::InActive);
			},
			VerifyStatus::Pass => {
				debug::info!("--------exchanged suc--------");

				Self::create_token(accept_account.clone(), quantity);
				debug::info!("remove tx={:?},queue len::{:?}", hex::encode(tx.clone()), num);
				<TokenStatus<T>>::remove(tx);
				if num > 0 {
					TokenStatusLen::mutate(|n| *n -= 1);
				}
				<EosExchangeInfo<T>>::insert(
					accept_account.clone(),
					tx.clone(),
					(AddressStatus::Active, quantity),
				);
				<SucTxExchange>::insert(tx.clone(), true);
				// Self::insert_active_status(accept_account.clone(), tx, AddressStatus::active);
			},
			_ => {},
		}
		return Ok(verify_status)
	}

	// fn insert_active_status(accept_account: T::AccountId, tx:&[u8], active_status:
	// AddressStatus){     let position = register_list.iter().position(|p| p.3 == symbol.clone());
	//     match position{
	//         Some(x) => {
	//             debug::info!("---------AddressOf haved {:?}---------",hex::encode(tx));
	//             register_list[x] = (token_address,active_status,tx.to_vec(),symbol);
	//             <AddressOf<T>>::insert(register_account,register_list);
	//         },
	//         None => {
	//             debug::info!("------- AddressOf not found {:?}-----------",hex::encode(tx));
	//             <AddressOf<T>>::mutate(register_account, |v|{
	//                 v.push((token_address,active_status,tx.to_vec(),symbol));
	//             });
	//         }
	//     }
	//
	//
	// }

	fn fetch_http_result_status(
		remote_url: &[u8],
		body: Vec<u8>,
		accept_account: T::AccountId,
	) -> StdResult<PostTxTransferData> {
		let json = Self::fetch_json(remote_url, body)?;
		let mut post_tx_transfer_data: PostTxTransferData = Self::fetch_parse(json)?; // json parse
		Self::get_verify_status(&mut post_tx_transfer_data, accept_account);
		debug::info!("----verified status = {:?}----", post_tx_transfer_data.code);
		Ok(post_tx_transfer_data)
	}

	fn fetch_json<'a>(remote_url: &'a [u8], body: Vec<u8>) -> StdResult<Vec<u8>> {
		// http get
		let url = &[remote_url, body.as_slice()].concat();
		let remote_url_str =
			core::str::from_utf8(url).map_err(|_| "Error in converting remote_url to string")?;
		debug::info!("get url: {:?}", remote_url_str);
		let now = <timestamp::Module<T>>::get();
		let deadline: u64 = now
			.try_into()
			.map_err(|_| "An error occurred when moment was converted to usize")? //
			.try_into()
			.map_err(|_| "An error occurred when usize was converted to u64")?;
		let deadline = Timestamp::from_unix_millis(deadline + 20000); // 等待最多10s
															  // let body = sp_std::str::from_utf8(&body).map_err(|e|"symbol from utf8 to str failed")?;
		let mut new_reuest = http::Request::get(remote_url_str);
		new_reuest.deadline = Some(deadline);
		let pending = new_reuest.send().map_err(|_| "Error in sending http get request")?;

		let http_result = pending.try_wait(deadline).map_err(|_| PENDING_TIME_OUT)?; // "Error in waiting http response back"
		let response = http_result.map_err(|_| WAIT_HTTP_CONVER_REPONSE)?;

		if response.code != 200 {
			debug::warn!("Unexpected status code: {}", response.code);
			let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();
			debug::info!("error body:{:?}", core::str::from_utf8(&json_result).unwrap());
			return Err("Non-200 status code returned from http request")
		}

		let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();

		// Print out the whole JSON blob
		// debug::info!("---response---{:?}",&core::str::from_utf8(&json_result).unwrap());
		Ok(json_result)
	}

	fn fetch_parse(resp_bytes: Vec<u8>) -> StdResult<PostTxTransferData> {
		let resp_str = core::str::from_utf8(&resp_bytes).map_err(|_| "Error in fetch_parse")?;
		// Print out our fetched JSON string
		debug::info!("json: {}", resp_str);

		// Deserializing JSON to struct, thanks to `serde` and `serde_derive`
		let post_tx_transfer_data: PostTxTransferData =
			serde_json::from_str(&resp_str).map_err(|e| {
				debug::info!("parse error: {:?}", e);
				"convert to ResponseStatus failed"
			})?;

		debug::info!("http get status:{:?}", post_tx_transfer_data.code);
		Ok(post_tx_transfer_data)
	}

	fn get_verify_status(post_transfer_data: &mut PostTxTransferData, acc: T::AccountId) {
		if post_transfer_data.code != 1 {
			if post_transfer_data.code == 0 {
				post_transfer_data.code = 2100;
			}

			if post_transfer_data.irreversible && post_transfer_data.is_post_transfer {
				if post_transfer_data.contract_account == CONTRACT_ACCOUNT.to_vec() &&
					post_transfer_data.to == DESTROY_ACCOUNT.to_vec()
				{
					match Self::vec_convert_account(post_transfer_data.pk.clone()) {
						Some(new_acc) =>
							if acc == new_acc {
								debug::info!("expect acc = {:?}", acc);
								debug::info!("new_acc = {:?}", new_acc);
								debug::info!(
									"to = {:?}",
									core::str::from_utf8(&post_transfer_data.to).unwrap()
								);
								post_transfer_data.code = 0;
							} else {
								post_transfer_data.code = 2003;
							},
						None => debug::info!("can not parse account"),
					}
				} else {
					post_transfer_data.code = 2002;
				}
			} else {
				debug::info!("reversible or not post token");
				post_transfer_data.code = 2001;
			}
		}
		post_transfer_data.code;
	}

	fn create_token(who: T::AccountId, quantity: u64) {
		let decimal = match <BalanceOf<T> as TryFrom<u128>>::try_from(
			quantity as u128 * currency::DOLLARS / 10,
		)
		.ok()
		{
			Some(x) => x,

			None => {
				debug::error!("quantity convert balance error");
				return
			},
		};
		debug::info!("{:?}exchanged num:{:?}", &who, decimal);
		T::OnUnbalanced::on_unbalanced(T::Currency::deposit_creating(&who, decimal));
		Self::deposit_event(RawEvent::CreateToken(who, decimal));
	}

	fn vec_convert_account(acc: Vec<u8>) -> Option<T::AccountId> {
		// debug::info!("------ acc ={:?} -------", hex::encode(&acc.clone()));
		if acc.len() != 32 {
			debug::error!("acc len={:?}", acc.len());
			return None
		}
		let acc_u8: [u8; 32] = acc.as_slice().try_into().expect("");
		let authority_id: T::AuthorityId = acc_u8.into();
		// debug::info!("authority_id is {:?}",authority_id);
		Some(authority_id.into_account32())
	}

	fn account_convert_u8(acc: T::AccountId) -> Vec<u8> {
		debug::info!("acc={:?}", acc);
		let author: T::AuthorityId = acc.into();
		debug::info!("author={:?}", author);
		let author_vec = author.to_raw_vec();
		hex_to_u8(&author_vec)
	}
}

#[allow(deprecated)]
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		let now = <timestamp::Module<T>>::get();
		debug::info!("--------------validate_unsigned time:{:?}--------------------", now);
		match call {
			// Call::record_address(block_num,account_id,key,tx,.., signature)
			Call::record_suc_verify(block_num, account, key, tx, status, quantity, signature) => {
				debug::info!(
					"############## record_suc_verify : now = {:?},block_num = {:?}##############",
					now,
					block_num
				);

				// check signature (this is expensive so we do it last).
				let signature_valid = &(block_num, account, tx)
					.using_encoded(|encoded_sign| key.verify(&encoded_sign, &signature));

				if !signature_valid {
					debug::error!(
						"................ record_suc_verify signed fail ....................."
					);
					return InvalidTransaction::BadProof.into()
				}
				debug::info!("................ record_suc_verify signed suc .....................");
				Ok(ValidTransaction {
					priority: <T as Trait>::UnsignedPriority::get(),
					requires: vec![],
					provides: vec![(block_num, tx, status, account).encode()],
					longevity: TransactionLongevity::max_value(),
					propagate: true,
				})
			},

			Call::record_fail_verify(block, account, key, tx, err, signature) => {
				debug::info!(
					"############# record_fail_verify :block={:?},time={:?}##############",
					block,
					now
				);
				// check signature (this is expensive so we do it last).
				let signature_valid = &(block, account, tx)
					.using_encoded(|encoded_sign| key.verify(&encoded_sign, &signature));
				if !signature_valid {
					debug::error!(
						"................ record_fail_verify signed fail ....................."
					);
					return InvalidTransaction::BadProof.into()
				}
				Ok(ValidTransaction {
					priority: <T as Trait>::UnsignedPriority::get(),
					requires: vec![],
					provides: vec![(block, tx, err, account).encode()], // vec![(now).encode()],
					longevity: TransactionLongevity::max_value() - 1,
					propagate: true,
				})
			},

			_ => InvalidTransaction::Call.into(),
		}
	}
}
