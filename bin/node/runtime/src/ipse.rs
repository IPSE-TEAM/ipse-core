#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;

use codec::{Decode, Encode};
use frame_support::traits::{Currency, BalanceStatus, ReservableCurrency};
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    StorageMap, StorageValue,
};
use sp_runtime::traits::SaturatedConversion;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;
use system::ensure_signed;
use core::{u64, u128};

pub const KB: u64 = 1024;
// When whose times of violation is more than 3,
// slash all funds of this miner.
pub const MAX_VIOLATION_TIMES: u64 = 3;
// millisec * sec * min * hour
pub const DAY: u64 = 1000 * 60 * 60 * 24;

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: ReservableCurrency<Self::AccountId>;
}

#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct Miner<Balance> {
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
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct Order<AccountId, Balance> {
    // the key of this data
    pub key: Vec<u8>,
    // the merkle root of data
    pub merkle_root: [u8; 32],
    // the length of storing data
    pub data_length: u64,
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
        pub Miners get(fn miner): map hasher(twox_64_concat) T::AccountId => Option<Miner<BalanceOf<T>>>;
        // order id is the index of vec.
        pub Orders get(fn order): Vec<Order<T::AccountId, BalanceOf<T>>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000]
        fn register_miner(origin, nickname: Vec<u8>, region: Vec<u8>, url: Vec<u8>, capacity: u64, unit_price: BalanceOf<T>) {
            let miner = ensure_signed(origin)?;
            // staking per kb is  1000;
            let total_staking_u64 = capacity * 1000 / KB;
            let total_staking = total_staking_u64.saturated_into::<BalanceOf<T>>();
            ensure!(T::Currency::can_reserve(&miner, total_staking), Error::<T>::CannotStake);
            // reserve for staking
            T::Currency::reserve(&miner,total_staking)?;
            Miners::<T>::insert(&miner, Miner {
                nickname,
                region,
                url,
                capacity,
                unit_price,
                violation_times: 0,
                total_staking,
            });
        }

        #[weight = 10_000]
        fn create_order(origin, key: Vec<u8>, merkle_root: [u8; 32], data_length: u64, miners: Vec<T::AccountId>, days: u64) {
            let user = ensure_signed(origin)?;
            let mut miner_orders = Vec::new();
            for m in miners {
                let miner = Self::miner(&m).ok_or(Error::<T>::MinerNotFound)?;
                let day_price = miner.unit_price * data_length.saturated_into::<BalanceOf<T>>() / KB.saturated_into::<BalanceOf<T>>();
                let total_price = day_price * days.saturated_into::<BalanceOf<T>>();
                let miner_order = MinerOrder {
                    miner: m,
                    day_price,
                    total_price,
                    verify_result: false,
                    verify_ts: 0,
                    confirm_ts: 0,
                    url: None,
                };
                miner_orders.push(miner_order);
            }
            Orders::<T>::mutate( |o| o.push(
                Order {
                    key,
                    merkle_root,
                    data_length,
                    user,
                    orders: miner_orders,
                    status: OrderStatus::Created,
                    update_ts: Self::get_now_ts(),
                    duration: days * DAY,
                }
            ));
        }

        #[weight = 10_000]
        fn confirm_order(origin, order_id: u64, url: Vec<u8>) {
            let miner = ensure_signed(origin)?;
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
                T::Currency::reserve(&order.user, miner_order.total_price)?;
                Ok(())
            })?;
        }

        #[weight = 10_000]
        fn delete(origin, order_id: u64) {
            let user = ensure_signed(origin)?;
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
                T::Currency::unreserve(&order.user, refund);
                Ok(())
            })?;
        }

        #[weight = 10_000]
        fn verify_storage(origin, order_id: u64) {
            let miner = ensure_signed(origin)?;
            let mut orders = Self::order();
            let order = orders.get_mut(order_id as usize).ok_or(Error::<T>::OrderNotFound)?;

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

        fn on_finalize(n: T::BlockNumber) {
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
                        if now > order.duration + confirm_ts {
                            order.status = OrderStatus::Expired
                        } else {
                            if now - mo.verify_ts < DAY && mo.verify_result {
                                // verify result is ok, transfer one day's funds to miner
                                T::Currency::repatriate_reserved(&order.user, &mo.miner, mo.day_price, BalanceStatus::Free);
                                Self::deposit_event(RawEvent::VerifyStorage(mo.miner, true));
                            } else {
                                // verify result expired or no verifying, punish miner
                                Self::punish(&mo.miner, order.data_length);
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
    fn get_now_ts() -> u64 {
        let now = <timestamp::Module<T>>::get();
        <T::Moment as TryInto<u64>>::try_into(now).ok().unwrap()
    }

    fn find_miner_order(
        miner: T::AccountId,
        os: &mut Vec<MinerOrder<T::AccountId, BalanceOf<T>>>
    ) -> Option<&mut MinerOrder<T::AccountId, BalanceOf<T>>> {
        for o in os {
            if o.miner == miner {
                return Some(o);
            }
        }
        return None;
    }

    fn punish(miner: &T::AccountId, data_length: u64) {
        Miners::<T>::mutate(miner, |mi| {
            let mut m = mi.as_mut().unwrap();
            let fine = if m.violation_times < MAX_VIOLATION_TIMES {
                m.unit_price * data_length.saturated_into::<BalanceOf<T>>()
            } else {
                u128::MAX.saturated_into::<BalanceOf<T>>()
            };
            T::Currency::slash_reserved(miner, fine);
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
        AccountId = <T as system::Trait>::AccountId
        {
            VerifyStorage(AccountId, bool),
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
    }
}
