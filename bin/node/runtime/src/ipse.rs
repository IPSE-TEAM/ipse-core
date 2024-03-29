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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;

use codec::{Decode, Encode};
use frame_support::traits::{BalanceStatus, Currency, Get, ReservableCurrency};
use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
	StorageMap, StorageValue,
};
use sp_std::vec;

use sp_runtime::{traits::SaturatedConversion, DispatchError, ModuleId};

use core::{u128, u64};
use sp_runtime::traits::AccountIdConversion;
use sp_std::{convert::TryInto, result, vec::Vec};
use system::ensure_signed;
// use pallet_staking as staking;
// use pallet_balances as balances;

pub const KB: u64 = 1024;

pub const MB: u64 = 1024 * 1024;

pub const GB: u64 = 1024 * 1024 * 1024;
pub const BASIC_BALANCE: u64 = 100000000000000;
// When whose times of violation is more than 3,
// slash all funds of this miner.
pub const MAX_VIOLATION_TIMES: u64 = 3;
// millisecond * sec * min * hour
// pub const DAY: u64 = 1000 * 60 * 60 * 24;
pub const DAY: u64 = 1000 * 60 * 60 * 24;
// max list order len
pub const NUM_LIST_ORDER_LEN: usize = 500;
// history len
pub const NUM_LIST_HISTORY_LEN: usize = 500;

pub type BalanceOf<T> =
	<<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub type NegativeImbalanceOf<T> = <<T as Trait>::StakingCurrency as Currency<
	<T as frame_system::Trait>::AccountId,
>>::NegativeImbalance;

pub trait Trait: system::Trait + timestamp::Trait {
	/// default event
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// currency
	type Currency: ReservableCurrency<Self::AccountId>;

	type StakingCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

	/// The treasury module account id to recycle assets.for default miner register
	type TreasuryModuleId: Get<ModuleId>;
}

#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct Miner<AccountId, Balance> {
	// account id
	pub account_id: AccountId,
	// miner nickname
	pub nickname: Vec<u8>,
	// where miner server locates
	pub region: Vec<u8>,
	// the miner's url
	pub url: Vec<u8>,
	// public_key
	pub public_key: Vec<u8>,
	// stash_address
	pub stash_address: AccountId,
	// capacity of data miner can store
	pub capacity: u128,
	// price per KB every day
	pub unit_price: Balance,
	// times of violations
	pub violation_times: u64,
	// total staking = unit_price * capacity
	pub total_staking: Balance,
	// register time
	pub create_ts: u64,
	// update timestamp
	pub update_ts: u64,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct Order<AccountId, Balance> {
	// miner account id
	pub miner: AccountId,
	// the label of this data
	pub label: Vec<u8>,
	// the hash of data
	pub hash: [u8; 46],
	// the size of storing data(byte)
	pub size: u128,
	pub user: AccountId,
	pub orders: Vec<MinerOrder<AccountId, Balance>>,
	pub status: OrderStatus,
	// register time
	pub create_ts: u64,
	// last update-status timestamp
	pub update_ts: u64,
	// how long this data keep
	pub duration: u64,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct MinerOrder<AccountId, Balance> {
	pub miner: AccountId,
	// one day price = unit_price * data_length
	pub day_price: Balance,
	// total_price = day_price * days
	pub total_price: Balance,
	// last verify result
	pub verify_result: bool,
	// last verify timestamp
	pub verify_ts: u64,
	// confirm order timestamp
	pub confirm_ts: u64,
	// use to be read data
	pub url: Option<Vec<u8>>,
}

/// History
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MiningHistory<Balance, BlockNumber> {
	// pub miner: AccountId,
	pub total_num: u64,
	pub history: Vec<(BlockNumber, Balance)>,
}

#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderStatus {
	Created,
	// Once one miner confirms it,
	// this order becomes confirmed.
	Confirmed,
	Expired,
	Deleted,
}

decl_storage! {
	trait Store for Module<T: Trait> as Ipse {
		/// the info of miners.
		pub Miners get(fn miner): map hasher(twox_64_concat) T::AccountId => Option<Miner<T::AccountId,BalanceOf<T>>>;

		/// order id is the index of vec.
		pub Orders get(fn order): Vec<Order<T::AccountId, BalanceOf<T>>>;

		/// exposed miners
		pub RecommendList get(fn recommend_list): Vec<(T::AccountId, BalanceOf<T>)>;

		/// the rewad history of miners.
		pub MinerHistory get(fn miner_history): map hasher(twox_64_concat) T::AccountId => Vec<Order<T::AccountId, BalanceOf<T>>>;

		/// the rewad history of users.
		pub History get(fn history): map hasher(twox_64_concat) T::AccountId => Option<MiningHistory<BalanceOf<T>, T::BlockNumber>>;

		/// whose url?.
		pub Url get(fn url): map hasher(twox_64_concat) Vec<u8> => T::AccountId;

		/// orders.
		pub ListOrder get(fn list_order): Vec<Order<T::AccountId, BalanceOf<T>>>;

	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;


		/// register
		#[weight = 10_000]
		fn register_miner(origin,nickname: Vec<u8>, region: Vec<u8>, url: Vec<u8>, public_key: Vec<u8>,stash_address: T::AccountId, capacity: u128, unit_price: BalanceOf<T>) {
			let who = ensure_signed(origin)?;

			let total_staking = capacity.saturated_into::<BalanceOf<T>>() * unit_price;

			ensure!(capacity > 0, Error::<T>::NoneCapacity);

			ensure!(!<Url<T>>::contains_key(url.clone()), Error::<T>::UrlExists);

			Url::<T>::insert(url.clone(),&who);

			Miners::<T>::insert(&who, Miner {
				account_id:who.clone(),
				nickname,
				region,
				url,
				public_key,
				stash_address,
				capacity,
				unit_price: unit_price,
				violation_times: 0,
				total_staking,
				create_ts:Self::get_now_ts(),
				update_ts:Self::get_now_ts(),
			});
			Self::deposit_event(RawEvent::Registered(who));
		}


		/// the user create the order.
		#[weight = 10_000]
		fn create_order(origin,miner: T::AccountId, label: Vec<u8>, hash: [u8; 46], size: u128, url: Option<Vec<u8>>, days: u64, unit_price: BalanceOf<T>) {
			let user = ensure_signed(origin)?;

			let mut order_list= Vec::new();

			let miner_cp = miner.clone();

			ensure!(<Miners<T>>::contains_key(&miner), Error::<T>::MinerNotFound);
			ensure!(days > 0, Error::<T>::NoneDays);


			if let Some(miner_info) = Miners::<T>::get(&miner).as_mut() {

				ensure!(miner_info.capacity > size, Error::<T>::InsufficientCapacity);

				miner_info.capacity = miner_info.capacity - size;

				let day_price = miner_info.unit_price * size.saturated_into::<BalanceOf<T>>();
				let total_price = day_price * days.saturated_into::<BalanceOf<T>>();

				let miner_order = MinerOrder {
					miner: miner_cp.clone(),
					day_price,
					total_price,
					verify_result: true,
					verify_ts: Self::get_now_ts(),
					confirm_ts: Self::get_now_ts(),
					url: url,
				};
				T::StakingCurrency::reserve(&user, miner_order.total_price)?;
				order_list.push(miner_order);

				Self::append_or_replace_orders(Order {
						miner: miner_cp.clone(),
						label: label.clone(),
						hash: hash.clone(),
						size: size.clone(),
						user: user.clone(),
						orders: order_list.clone(),
						status: OrderStatus::Confirmed,
						create_ts: Self::get_now_ts(),
						update_ts: Self::get_now_ts(),
						duration: days * DAY,
					});

				Self::update_miner_history(miner_cp.clone(),Order {
						miner: miner_cp.clone(),
						label: label.clone(),
						hash: hash.clone(),
						size: size.clone(),
						user: user.clone(),
						orders: order_list.clone(),
						status: OrderStatus::Confirmed,
						create_ts: Self::get_now_ts(),
						update_ts: Self::get_now_ts(),
						duration: days * DAY,
					});

				Orders::<T>::mutate( |o| o.push(
					Order {
						miner: miner_cp.clone(),
						label: label.clone(),
						hash: hash.clone(),
						size: size.clone(),
						user: user.clone(),
						orders: order_list,
						status: OrderStatus::Confirmed,
						create_ts: Self::get_now_ts(),
						update_ts: Self::get_now_ts(),
						duration: days * DAY,
					}
				));

				Miners::<T>::insert(&miner_cp.clone(), miner_info);

			}


			Self::deposit_event(RawEvent::CreatedOrder(user));

		}

		/// the miner confirm the order.
		#[weight = 10_000]
		fn confirm_order(origin, order_id: u64, url: Vec<u8>) {
			let miner = ensure_signed(origin)?;
			let miner_cp = miner.clone();

			// must check total staking, if is zero, cannot confirm order.
			let miner_info = Self::miner(&miner).ok_or(Error::<T>::MinerNotFound)?;
			ensure!(miner_info.total_staking > 0.saturated_into::<BalanceOf<T>>(), Error::<T>::NoneStaking);

			let now = Self::get_now_ts();
			Orders::<T>::mutate( |os| -> DispatchResult {

				let mut order = os.get_mut(order_id as usize).ok_or(Error::<T>::OrderNotFound)?;

				ensure!(order.status != OrderStatus::Deleted, Error::<T>::OrderDeleted);
				ensure!(order.status != OrderStatus::Expired, Error::<T>::OrderExpired);

				let mut miner_order = Self::find_miner_order(miner, &mut order.orders).ok_or(Error::<T>::MinerOrderNotFound)?;
				miner_order.confirm_ts = now;
				miner_order.url = Some(url);
				// update order's status and update_ts
				if order.status == OrderStatus::Created {
					order.status = OrderStatus::Confirmed;
					order.update_ts = now;
				}

				// reserve some user's funds for the order
				T::StakingCurrency::reserve(&order.user, miner_order.total_price)?;
				Ok(())
			})?;
			Self::deposit_event(RawEvent::ConfirmedOrder(miner_cp, order_id));
		}


		/// users delete their order.
		#[weight = 10_000]
		fn delete_order(origin, order_id: u64) {
			let user = ensure_signed(origin)?;
			let user_cp = user.clone();
			Orders::<T>::mutate( |os| -> DispatchResult {
				let mut order = os.get_mut(order_id as usize).ok_or(Error::<T>::OrderNotFound)?;
				ensure!(user == order.user , Error::<T>::PermissionDenyed);
				ensure!(order.status != OrderStatus::Deleted, Error::<T>::OrderDeleted);
				ensure!(order.status != OrderStatus::Expired, Error::<T>::OrderExpired);

				let now = Self::get_now_ts();
				order.status = OrderStatus::Deleted;
				order.update_ts = now;
				// unreserve some user's funds
				let days_to_deadline: u64 = (order.duration + order.update_ts - now)/DAY;
				let mut refund = 0.saturated_into::<BalanceOf<T>>();
				for mo in &order.orders {
					refund += mo.day_price;
				}
				refund = refund * days_to_deadline.saturated_into::<BalanceOf<T>>();
				T::StakingCurrency::unreserve(&order.user, refund);
				Ok(())
			})?;
			Self::deposit_event(RawEvent::DeletedOrder(user_cp, order_id));
		}



		/// verify
		#[weight = 10_000]
		fn verify_storage(origin, order_id: u64) {
			let miner = ensure_signed(origin)?;
			let mut orders = Self::order();
			let order = orders.get_mut(order_id as usize).ok_or(Error::<T>::OrderNotFound)?;

			ensure!(order.status == OrderStatus::Confirmed, Error::<T>::OrderUnconfirmed);

			let now = Self::get_now_ts();

			for mut mo in &mut order.orders {
				if mo.miner ==  miner {
					mo.verify_ts = now;
					mo.verify_result = true;
				}
			}
			Self::deposit_event(RawEvent::VerifyStorage(miner, true));
		}



		/// the miner apply to recommended list.
		#[weight = 10_000]
		fn apply_to_recommended_list(origin, amount: BalanceOf<T>) {

			let miner = ensure_signed(origin)?;

			ensure!(<Miners<T>>::contains_key(&miner), Error::<T>::MinerNotFound);

			// TODO: add user to recommended list
			Self::sort_account_by_amount(miner.clone(), amount)?;

			Self::deposit_event(RawEvent::RequestUpToList(miner, amount));

		}


		/// the miner drop out recommended list.
		#[weight = 10_000]
		fn drop_out_recommended_list(origin) {
			let miner = ensure_signed(origin)?;

			let mut list = <RecommendList<T>>::get();
			if let Some(pos) = list.iter().position(|h| h.0 == miner) {
				let amount = list.remove(pos).1;

				T::StakingCurrency::unreserve(&miner, amount);

				<RecommendList<T>>::put(list);
			}
			else {
				return Err(Error::<T>::NotInList)?;
			}

			Self::deposit_event(RawEvent::RequestDownFromList(miner));

		}

		fn on_finalize(n: T::BlockNumber) {
			let current_block = n;
			let n = n.saturated_into::<u64>();
			// Check verifying result per 20 blocks,
			// 20 blocks just 1 minute.
			if n%20 != 0 { return }
			let now = Self::get_now_ts();
			Orders::<T>::mutate( |orders| {
				for mut order in orders {
					if order.status == OrderStatus::Confirmed {
						let create_ts = order.create_ts;
						let confirm_ts = order.update_ts;
						for mo in &order.orders {
							if now > order.duration + create_ts + DAY{
								order.status = OrderStatus::Expired;
							} else {
								if now - order.update_ts >= DAY && mo.verify_result {
									// verify result is ok, transfer one day's funds to miner
									//  transfer to income address
								   if let Some(miner) = Miners::<T>::get(&mo.miner){
										T::StakingCurrency::repatriate_reserved(&order.user, &miner.stash_address, mo.day_price, BalanceStatus::Free);

										debug::info!("miner: {:?}",&miner);

										order.update_ts = now;

										Self::update_history(current_block, mo.miner.clone(), mo.day_price);

										Self::deposit_event(RawEvent::VerifyStorage(mo.miner.clone(), true));
									}

								} else {
									// verify result expired or no verifying, punish miner
									// Self::punish(&mo.miner, order.size);
									Self::deposit_event(RawEvent::VerifyStorage(mo.miner.clone(), false));
								}
							}
						}

					}
				}
			});
		}

	}
}

impl<T: Trait> Module<T> {
	/// Get treasury account id.
	pub fn miner_account_id() -> T::AccountId {
		T::TreasuryModuleId::get().into_account()
	}

	fn get_now_ts() -> u64 {
		let now = <timestamp::Module<T>>::get();
		<T::Moment as TryInto<u64>>::try_into(now).ok().unwrap()
	}

	fn find_miner_order(
		miner: T::AccountId,
		os: &mut Vec<MinerOrder<T::AccountId, BalanceOf<T>>>,
	) -> Option<&mut MinerOrder<T::AccountId, BalanceOf<T>>> {
		for o in os {
			if o.miner == miner {
				return Some(o)
			}
		}
		return None
	}

	fn sort_account_by_amount(
		miner: T::AccountId,
		mut amount: BalanceOf<T>,
	) -> result::Result<(), DispatchError> {
		let mut old_list = <RecommendList<T>>::get();
		let mut miner_old_info: Option<(T::AccountId, BalanceOf<T>)> = None;

		if let Some(pos) = old_list.iter().position(|h| h.0 == miner.clone()) {
			miner_old_info = Some(old_list.remove(pos));
		}
		if miner_old_info.is_some() {
			let old_amount = miner_old_info.clone().unwrap().1;

			ensure!(T::StakingCurrency::can_reserve(&miner, amount), Error::<T>::AmountNotEnough);

			T::StakingCurrency::unreserve(&miner, old_amount);

			amount = amount + old_amount;
		}

		if old_list.len() == 0 {
			Self::sort_after(miner, amount, 0, old_list)?;
		} else {
			let mut index = 0;
			for i in old_list.iter() {
				if i.1 >= amount {
					index += 1;
				} else {
					break
				}
			}

			Self::sort_after(miner, amount, index, old_list)?;
		}

		Ok(())
	}

	fn append_or_replace_orders(order: Order<T::AccountId, BalanceOf<T>>) {
		ListOrder::<T>::mutate(|orders| {
			let len = orders.len();
			if len == NUM_LIST_ORDER_LEN {
				let pre = len - NUM_LIST_HISTORY_LEN;
				let new_vec = orders.split_off(pre);
				let orders = new_vec;
			}
			orders.push(order);
			debug::info!("orders vector: {:?}", orders);
		});
	}

	fn update_miner_history(miner: T::AccountId, order: Order<T::AccountId, BalanceOf<T>>) {
		let mut miner_history = MinerHistory::<T>::get(&miner);
		let len = miner_history.len();

		if len >= NUM_LIST_HISTORY_LEN {
			let pre = len - NUM_LIST_HISTORY_LEN;
			let new_vec = miner_history.split_off(pre);
			let miner_history = new_vec;
		}
		miner_history.push(order);
		<MinerHistory<T>>::insert(miner, miner_history);
	}

	fn update_history(n: T::BlockNumber, miner: T::AccountId, amount: BalanceOf<T>) {
		let mut history = <History<T>>::get(miner.clone());
		let miner_cp = miner.clone();
		if history.is_some() {
			let mut vec = history.clone().unwrap().history;
			let num = history.clone().unwrap().total_num;
			vec.push((n, amount));

			let len = vec.len();
			if len >= 100 {
				let pre = len - 100;
				let new_vec = vec.split_off(pre);
				vec = new_vec;
			}

			history = Some(MiningHistory {
				// miner: miner_cp,
				total_num: num + 1u64,
				history: vec,
			});
		} else {
			let mut vec = vec![];
			vec.push((n, amount));
			history = Some(MiningHistory {
				// miner: miner_cp,
				total_num: 1u64,
				history: vec,
			});
		}

		<History<T>>::insert(miner, history.unwrap());
	}

	fn sort_after(
		miner: T::AccountId,
		amount: BalanceOf<T>,
		index: usize,
		mut old_list: Vec<(T::AccountId, BalanceOf<T>)>,
	) -> result::Result<(), DispatchError> {
		T::StakingCurrency::reserve(&miner, amount)?;

		old_list.insert(index, (miner, amount));

		if old_list.len() > 20 {
			let abandon = old_list.split_off(20);
			for i in abandon {
				T::StakingCurrency::unreserve(&i.0, i.1);
			}
		}

		<RecommendList<T>>::put(old_list);

		Ok(())
	}

	fn punish(miner: &T::AccountId, size: u128) {
		Miners::<T>::mutate(miner, |mi| {
			let mut m = mi.as_mut().unwrap();
			let fine = if m.violation_times < MAX_VIOLATION_TIMES {
				m.unit_price * size.saturated_into::<BalanceOf<T>>()
			} else {
				u128::MAX.saturated_into::<BalanceOf<T>>()
			};
			T::StakingCurrency::slash_reserved(miner, fine);
			m.violation_times += 1;
			if m.total_staking >= fine {
				m.total_staking -= fine;
			} else {
				m.total_staking = 0.saturated_into::<BalanceOf<T>>();
			}
		});
	}
}

decl_event! {
	pub enum Event<T>
		where
		AccountId = <T as system::Trait>::AccountId,
		Balance = <<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance,
		{
			Registered(AccountId),
			UpdatedMiner(AccountId),
			VerifyStorage(AccountId, bool),
			VerifyBlock(u64),
			CreatedOrder(AccountId),
			ListOrder(AccountId),
			ConfirmedOrder(AccountId, u64),
			DeletedOrder(AccountId, u64),
			RequestUpToList(AccountId, Balance),
			RequestDownFromList(AccountId),
		}
}

decl_error! {
	/// Error for the ipse module.
	pub enum Error for Module<T: Trait> {
		/// url already exists
		UrlExists,
		/// Miner not found.
		MinerNotFound,
		/// Order not found.
		OrderNotFound,
		/// Miner-order not found.
		MinerOrderNotFound,
		/// Order is already deleted.
		OrderDeleted,
		/// Order is already expired.
		OrderExpired,
		/// Order is unconfirmed.
		OrderUnconfirmed,
		/// Balance is not enough to stake.
		CannotStake,
		/// not have enough money
		NotEnoughMoney,
		/// Total staking is zero.
		NoneStaking,
		/// User has no op permission to order.
		PermissionDenyed,
		AmountNotEnough,
		NotInList,
		/// Miners provide insufficient storage capacity
		InsufficientCapacity,
		NoneCapacity,
		NoneDays,
	}
}
