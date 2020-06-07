#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;

use codec::{Decode, Encode};
use frame_support::traits::{Currency, WithdrawReasons, ReservableCurrency};
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    weights::SimpleDispatchInfo,
};
use sp_runtime::traits::SaturatedConversion;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;
use system::ensure_signed;
use core::{u64, u128};
use crate::constants::time::DAYS;

pub const KB: u64 =  1024;

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
    // number of violations
    pub violation_num: u64,
}

#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct Order<AccountId, Balance> {
    // the key of this data
    pub key: Vec<u8>,
    // the merkle root of data
    pub merkle_root: Vec<u8>,
    pub user: AccountId,
    pub orders: Vec<MinerOrder<AccountId, Balance>>,
    pub status: OrderStatus,
    // last update-status timestamp
    pub update_ts: u64,
    // how long this data keep
    pub duration: u64,
}

#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MinerOrder<AccountId, Balance> {
    pub miner: AccountId,
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

#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub enum OrderStatus {
    Created,
    Expired,
    Deleted,
}

decl_storage! {
    trait Store for Module<T: Trait> as Ipse {
        pub Miners get(miner): map hasher(twox_64_concat) T::AccountId => Miner<BalanceOf<T>>;
        // order id is the index of vec.
        pub Orders get(order): Vec<Order<T::AccountId, BalanceOf<T>>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        const STAKING_PER_KB: BalanceOf<T> = 1000;

        #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
        fn register_miner(origin, nickname: Vec<u8>, region: Vec<u8>, url: Vec<u8>, capacity: u64, unit_price: BalanceOf<T>) {
            let miner = ensure_signed(origin)?;
            let total_staking = capacity * Self::STAKING_PER_KB / KB;
            ensure!(T::Currency::can_reserve(&miner, total_staking), Error::<T>::CannotStake);
            // lock for staking
            T::Currency::reserve(&miner,total_staking)?;
            Miners::<T>::insert(&miner, Miner {
                nickname,
                region,
                url,
                capacity,
                unit_price,
                violation_num: 0,
            });
        }

        #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
        fn create_order(origin, key: Vec<u8>, merkle_root: Vec<u8>, data_length: u64, miners: Vec<T::AccountId>, days: u64) {
            let user = ensure_signed(origin)?;
            let mut miner_orders = Vec::new();
            for m in miners {
                let miner = Self::miner(&m).ok_or(Error::<T>::MinerNotFound)?;
                let total_price = miner.unit_price * data_length * days / KB ;
                let miner_order = MinerOrder {
                    miner,
                    total_price,
                    verify_result: false,
                    verify_ts: 0,
                    confirm_ts: 0,
                    url: None,
                };
                miner_orders.push(miner_order);
            }
            Orders::<T>::mutate( |o| o.push(
                key,
                merkle_root,
                user,
                orders: miner_orders,
                status: OrderStatus::Created,
                update_ts: Self::get_now_ts(),
                duration: days * DAYS,
            ));
        }

        #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
        fn confirm_order(origin, order_id: u64, url: Vec<u8>) {
            let miner = ensure_signed(origin)?;
            Orders::<T>::try_mutate( |os| -> DispatchResult {
                let mut order = os.get_mut(order_id).ok_or(Error::<T>::OrderNotFound)?;
                let mut miner_order = Self::find_miner_order(miner, order.orders).ok_or(Error::<T>::MinerOrderNotFound)?;
                miner_order.confirm_ts = Self::get_now_ts();
                miner_order.url = Some(url);
                // lock some user's balance
                T::Currency::reserve(&order.user,miner_order.total_price)?;
                Ok(())
            })?;
        }

        #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
        fn delete(origin, order_id: u64) {
            let user = ensure_signed(origin)?;
            Orders::<T>::try_mutate( |os| -> DispatchResult {
                let mut order = os.get_mut(order_id).ok_or(Error::<T>::OrderNotFound)?;
                ensure!(&order.status == &OrderStatus::Deleted, Error::<T>::OrderDeleted);
                ensure!(&order.status == &OrderStatus::Expired, Error::<T>::OrderExpired);
                order.status = OrderStatus::Deleted;
                order.update_ts = Self::get_now_ts();
                // unlock some user's balance
                let now = Self::get_now_ts();
                let days = order.duration/DAYS;
                let days_to_deadline = (order.duration + order.update_ts - now)/DAYS;
                let mut refund = 0;
                for mo in order.orders {
                    refund += mo.total_price;
                }
                refund = refund * days_to_deadline/days;
                T::Currency::unreserve(&order.user, refund);
                OK(())
            })?;
        }

        #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
        fn verify_storage(origin) {
            let miner = ensure_signed(origin)?;
            // todo: zk verify

            Self::deposit_event(RawEvent::VerifyStorage(miner, true));
        }

        fn on_finalize(n: T::BlockNumber) {
            let n = n.saturated_into::<u64>();
            // Check verifying result per 20 blocks,
            // 20 blocks just 1 minute.
            if n%20 != 0 { return }
            let now = Self::get_now_ts();
            let orders = Self::order();
            for order in orders {
                if &order.status == &OrderStatus::Created {
                    let create_ts = order.update_ts;
                    for mo in order.orders {
                        if now > mo.duration + create_ts {
                            order.status = OrderStatus::Expired
                        } else {
                            if now - mo.verify_ts < DAYS && mo.verify_result {
                                // todo: verify result is ok, transfer one day's funds to miner
                                Self::deposit_event(RawEvent::VerifyStorage(miner, true));
                            } else {
                                // verify result expired or no verifying, punish miner
                                Self::punish(&miner);
                                Self::deposit_event(RawEvent::VerifyStorage(miner, false));
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
        os: Vec<MinerOrder<T::AccountId, BalanceOf<T>>>,
    ) -> Option<MinerOrder<T::AccountId, BalanceOf<T>>> {
        for o in os {
            if o.miner == miner {
                return Some(o);
            }
        }
        return None;
    }

    fn transfer(user: T::AccountId, miner: T::AccountId) {

    }

    fn punish(miner: &T::AccountId) {

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
        /// Balance is not enough to stake.
        CannotStake,
    }
}