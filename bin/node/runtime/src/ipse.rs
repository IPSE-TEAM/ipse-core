#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;
use sp_std::vec;
use codec::{Decode, Encode};
use frame_support::traits::{Currency, Get, BalanceStatus, ReservableCurrency};
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    StorageMap, StorageValue,
};

use sp_runtime::{traits::{
    SaturatedConversion,
}, ModuleId, DispatchError};

use sp_std::convert::TryInto;
use sp_std::vec::Vec;
use sp_std::result;
use system::ensure_signed;
use core::{u64, u128};
use sp_runtime::traits::AccountIdConversion;
// use pallet_staking as staking;
// use pallet_balances as balances;

pub const KB: u64 = 1024;
// When whose times of violation is more than 3,
// slash all funds of this miner.
pub const MAX_VIOLATION_TIMES: u64 = 3;
// millisecond * sec * min * hour
pub const DAY: u64 = 1000 * 60 * 60 * 24;


pub type BalanceOf<T> = <<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub type NegativeImbalanceOf<T> = <<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

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
    // capacity of data miner can store
    pub capacity: u64,
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
    pub hash: [u8; 32],
    // the size of storing data
    pub size: u64,
    pub user: AccountId,
    pub orders: Vec<MinerOrder<AccountId, Balance>>,
    pub status: OrderStatus,
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
    total_num: u64,
    history: Vec<(BlockNumber, Balance)>,
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
    	/// 矿工的信息
        pub Miners get(fn miner): map hasher(twox_64_concat) T::AccountId => Option<Miner<T::AccountId,BalanceOf<T>>>;

        /// order id is the index of vec.
        pub Orders get(fn order): Vec<Order<T::AccountId, BalanceOf<T>>>;

		/// 推荐的矿工列表
		pub RecommendList get(fn recommend_list): Vec<(T::AccountId, BalanceOf<T>)>;

		/// 矿工的挖矿记录
        pub History get(fn history): map hasher(twox_64_concat) T::AccountId => Option<MiningHistory<BalanceOf<T>, T::BlockNumber>>;

    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        fn deposit_event() = default;


        /// 矿工进行注册登记
        #[weight = 10_000]
        fn register_miner(origin,nickname: Vec<u8>, region: Vec<u8>, url: Vec<u8>, capacity: u64, unit_price: BalanceOf<T>) {
        	// 容量单位是kb
            let who = ensure_signed(origin)?;
            // staking per kb is  1000;
            let total_staking_u64 = capacity * 1000 / KB;
            let total_staking = total_staking_u64.saturated_into::<BalanceOf<T>>();
            ensure!(T::StakingCurrency::can_reserve(&who, total_staking), Error::<T>::CannotStake);
            // reserve for staking
            T::StakingCurrency::reserve(&who,total_staking)?;

            Miners::<T>::insert(&who, Miner {
                account_id:who.clone(),
                nickname,
                region,
                url,
                capacity,
                unit_price,
                violation_times: 0,
                total_staking,
                create_ts:Self::get_now_ts(),
                update_ts:Self::get_now_ts(),
            });
            Self::deposit_event(RawEvent::Registered(who));
        }

        /// 矿工注册信息更新(容量)-miner  Schedule job
        #[weight = 10_000]
        fn update_miner(origin, nickname: Vec<u8>, region: Vec<u8>, url: Vec<u8>, capacity: u64, unit_price: BalanceOf<T>) {
            let who = ensure_signed(origin)?;

            // must check total staking, if is zero, cannot confirm order.
            let miner_info = Self::miner(&who).ok_or(Error::<T>::MinerNotFound)?;
            ensure!(miner_info.total_staking > 0.saturated_into::<BalanceOf<T>>(), Error::<T>::NoneStaking);

            if let Some(miner) = Miners::<T>::get(&who).as_mut() {

                miner.nickname = nickname;
                miner.region = region;
                miner.url = url;
                miner.capacity = capacity;
                miner.unit_price = unit_price;
                miner.update_ts = Self::get_now_ts();

                Miners::<T>::insert(&who, miner);
            }

            Self::deposit_event(RawEvent::UpdatedMiner(who));
        }





        /// 用户创建订单(后面加上，unit_price)
        #[weight = 10_000]
        fn create_order(origin,miner: T::AccountId, label: Vec<u8>, hash: [u8; 32], size: u64, url: Option<Vec<u8>>, days: u64, unit_price: BalanceOf<T>) {
            let user = ensure_signed(origin)?;

            let mut order_list= Vec::new();

            let miner_cp = miner.clone();

            let miner = Self::miner(&miner).ok_or(Error::<T>::MinerNotFound)?;
            let day_price = miner.unit_price * size.saturated_into::<BalanceOf<T>>() / KB.saturated_into::<BalanceOf<T>>();
            let total_price = day_price * days.saturated_into::<BalanceOf<T>>();
            let miner_order = MinerOrder {
                miner: miner.account_id,
                day_price,
                total_price,
                verify_result: false,
                verify_ts: Self::get_now_ts(),
                confirm_ts: Self::get_now_ts(),
                url: url,
            };
            order_list.push(miner_order);

            Orders::<T>::mutate( |o| o.push(
                Order {
                    miner: miner_cp,
                    label,
                    hash,
                    size,
                    user: user.clone(),
                    orders: order_list,
                    status: OrderStatus::Created,
                    update_ts: Self::get_now_ts(),
                    duration: days * DAY,
                }
            ));

            Self::deposit_event(RawEvent::CreatedOrder(user));

        }

        /// 矿工确认订单-自动搞定
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


        /// 用户删除订单
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


        /// 数据验证
        #[weight = 10_000]
        fn verify_storage(origin, order_id: u64) {
            let miner = ensure_signed(origin)?;
            let mut orders = Self::order();
            let order = orders.get_mut(order_id as usize).ok_or(Error::<T>::OrderNotFound)?;

			/// 已经提交
            ensure!(order.status == OrderStatus::Confirmed, Error::<T>::OrderUnconfirmed);
            // todo: zk verify

            let now = Self::get_now_ts();

            for mut mo in &mut order.orders {
                if mo.miner ==  miner {
                    mo.verify_ts = now;
                    // temporarily assume verify_result is true
                    mo.verify_result = true;
                }
            }
            Self::deposit_event(RawEvent::VerifyStorage(miner, true));
        }


        /// 矿工申请进入推荐列表
		#[weight = 10_000]
		fn apply_to_recommended_list(origin, amount: BalanceOf<T>) {

			// 矿工才能操作
			let miner = ensure_signed(origin)?;

			ensure!(<Miners<T>>::contains_key(&miner), Error::<T>::MinerNotFound);

            // TODO: add user to recommended list
			Self::sort_account_by_amount(miner.clone(), amount)?;

			Self::deposit_event(RawEvent::RequestUpToList(miner, amount));

		}


		/// 矿工退出推荐列表
		#[weight = 10_000]
		fn drop_out_recommended_list(origin) {
			let miner = ensure_signed(origin)?;
			// 获取推荐列表
			let mut list = <RecommendList<T>>::get();
			if let Some(pos) = list.iter().position(|h| h.0 == miner) {
				let amount = list.swap_remove(pos).1;

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
            let orders = Self::order();
            for mut order in orders {
                if &order.status == &OrderStatus::Confirmed {
                    let confirm_ts = order.update_ts;
                    for mo in order.orders {
                        //
                        if now > order.duration + confirm_ts {
                            order.status = OrderStatus::Expired
                        } else {
                            if now - mo.verify_ts < DAY && mo.verify_result {
                                // verify result is ok, transfer one day's funds to miner
                                T::StakingCurrency::repatriate_reserved(&order.user, &mo.miner, mo.day_price, BalanceStatus::Free);

								Self::update_history(current_block, mo.miner.clone(), mo.day_price);

                                Self::deposit_event(RawEvent::VerifyStorage(mo.miner, true));
                            } else {
                                // verify result expired or no verifying, punish miner
                                Self::punish(&mo.miner, order.size);
                                Self::deposit_event(RawEvent::VerifyStorage(mo.miner, false));
                            }
                        }
                    }
                }
            }
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
                return Some(o);
            }
        }
        return None;
    }

    fn sort_account_by_amount(miner: T::AccountId, mut amount: BalanceOf<T>) -> result::Result<(), DispatchError> {
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
                    break;
                }
            }

            Self::sort_after(miner, amount, index, old_list)?;
        }

        Ok(())
    }

    fn update_history(n: T::BlockNumber, miner: T::AccountId, amount: BalanceOf<T>) {
    	let mut history = <History<T>>::get(miner.clone());
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
				total_num: num + 1u64,
				history: vec,
			});
		}

		else {
			let mut vec = vec![];
			vec.push((n, amount));
			history = Some(MiningHistory {
				total_num: 1u64,
				history: vec,
			});
		}

		<History<T>>::insert(miner, history.unwrap());
    }

    fn sort_after(miner: T::AccountId, amount: BalanceOf<T>, index: usize, mut old_list: Vec<(T::AccountId, BalanceOf<T>)>) -> result::Result<(), DispatchError> {
        // 先对矿工进行抵押

        T::StakingCurrency::reserve(&miner, amount)?;

        old_list.insert(index, (miner, amount));

        if old_list.len() > 20 {
            let abandon = old_list.split_off(20);
            // 对被淘汰的人进行释放
            for i in abandon {
                T::StakingCurrency::unreserve(&i.0, i.1);
            }
        }

        <RecommendList<T>>::put(old_list);

        Ok(())
    }

    fn punish(miner: &T::AccountId, size: u64) {
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
			CreatedOrder(AccountId),
			ConfirmedOrder(AccountId, u64),
			DeletedOrder(AccountId, u64),
            RequestUpToList(AccountId, Balance),
            RequestDownFromList(AccountId),
        }
}

decl_error! {
    /// Error for the ipse module.
    pub enum Error for Module<T: Trait> {
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
        /// Total staking is zero.
        NoneStaking,
        /// User has no op permission to order.
        PermissionDenyed,
        AmountNotEnough,
        NotInList,
    }
}
