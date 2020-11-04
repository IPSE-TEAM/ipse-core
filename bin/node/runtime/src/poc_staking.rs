
extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;
extern crate pallet_balances as balances;
extern crate pallet_babe as babe;
use crate::constants::time::MILLISECS_PER_BLOCK;

use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult, DispatchError,}, debug,
	traits::{Get, Currency, ReservableCurrency, OnUnbalanced},
    weights::Weight,
	StorageMap, StorageValue,
	decl_error, ensure,
};
use sp_std::result;

use system::{ensure_signed};
use sp_runtime::{traits::{SaturatedConversion, Saturating}, Percent};
use sp_std::vec::Vec;
use sp_std::vec;
use node_primitives::KIB;
use num_traits::{CheckedAdd, CheckedSub};

type BalanceOf<T> =
	<<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: system::Trait + timestamp::Trait + balances::Trait + babe::Trait {

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type ChillDuration: Get<Self::BlockNumber>;

	type StakingCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

	type StakingDeposit: Get<BalanceOf<Self>>;

	type StakingSlash: OnUnbalanced<NegativeImbalanceOf<Self>>;

	type StakerMaxNumber: Get<usize>;

}


/// 矿工的机器信息
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MachineInfo<BlockNumber> {
	/// 磁盘空间
	pub disk: KIB,
	/// 更新时间
	pub update_time: BlockNumber,
	/// 机器是否在运行（这个是用户抵押的依据)
	is_stop: bool,
}


/// 抵押信息
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct StakingInfo<AccountId, Balance> {
	/// 矿工
	pub miner: AccountId,
	/// 矿工分润占比
	pub miner_portation: Percent,
	/// 总的抵押金额
	pub total_staking: Balance,
	/// 其他人的抵押 （staker， 抵押金额， 保留金额)
	pub others: Vec<(AccountId, Balance, Balance)>,
}


/// 操作
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Oprate {
	/// 添加
	Add,
	/// 减少
	Sub,
}

impl Default for Oprate {
	fn default() -> Self {
		Self::Add
	}
}


decl_storage! {
    trait Store for Module<T: Trait> as IpseStakingModule {

		/// 矿工磁盘空间信息
		pub DiskOf get(fn disk_of): map hasher(twox_64_concat) T::AccountId => Option<MachineInfo<T::BlockNumber>>;

		/// 是否在非抵押操作期间（冷冻期，只有矿工能改变信息)
		pub IsChillTime get(fn is_chill_time): bool = true;

		/// 每个矿工对应的抵押信息
		pub StakingInfoOf get(fn stking_info_of): map hasher(twox_64_concat) T::AccountId => Option<StakingInfo<T::AccountId, BalanceOf<T>>>;

		/// 用户抵押的矿工
		pub MinersOf get(fn mminers_of): map hasher(twox_64_concat) T::AccountId => Option<Vec<T::AccountId>>;

		/// 自增的p盘id
		pub Pid get(fn p_id): u64;

		/// 矿工对应的p盘id
		pub PidOf get(fn account_id_of): map hasher(twox_64_concat) T::AccountId => Option<u64>;
    }
}

decl_event! {
pub enum Event<T>
    where
    AccountId = <T as system::Trait>::AccountId,
//    Balance = <T as balances::Trait>::Balance,
	Balance = <<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance,
    {

        UpdateDiskInfo(AccountId, KIB),
        Register(AccountId, u64),
        StopMining(AccountId),
        RemoveStaker(AccountId, AccountId),
        Staking(AccountId, AccountId, Balance),
        UpdatePortation(AccountId, Percent),
		UpdateStaking(AccountId, Balance),
		ExitStaking(AccountId, AccountId),
    }
}

decl_module! {
     pub struct Module<T: Trait> for enum Call where origin: T::Origin {
     	/// 冷却期时长（从每个era开始计算，前面的区块是冷却期)
     	const ChillDuration: T::BlockNumber = T::ChillDuration::get();
     	/// staking时候需要保留的余额
     	const StakingDeposit: BalanceOf<T> = T::StakingDeposit::get();
     	/// 一名矿工最多有多少名抵押者
     	const StakerMaxNumber: u32 = T::StakerMaxNumber::get() as u32;
        fn deposit_event() = default;


		/// 矿工注册
		#[weight = 10_000]
		fn register(origin, kib: KIB, miner_portation: Percent) {
			let miner = ensure_signed(origin)?;

			ensure!(kib != 0 as KIB, Error::<T>::DiskEmpty);

			// 把kib转变成b
			let disk = kib.checked_mul(1000 as KIB).ok_or(Error::<T>::Overflow)?;

			ensure!(Self::is_register(miner.clone())?, Error::<T>::NotRegister);

			let now = Self::now();
			<DiskOf<T>>::insert(miner.clone(), MachineInfo {
        		disk: disk,
        		update_time: now,
        		is_stop: false,

        	});

        	<StakingInfoOf<T>>::insert(&miner,
        		StakingInfo {

        			miner: miner.clone(),
        			miner_portation: miner_portation,
        			total_staking: <BalanceOf<T>>::from(0u32),
        			others: vec![],
        		}
        	);

			// 映射p盘id
        	let mut p_id = <Pid>::get();
        	<PidOf<T>>::insert(miner.clone(), p_id);
        	p_id = p_id.saturating_add(1);

        	<Pid>::put(p_id);

        	Self::deposit_event(RawEvent::Register(miner, disk));

		}


		/// 更新磁盘信息
        #[weight = 10_000]
        fn update_disk_info(origin, disk: KIB) {

        	let miner = ensure_signed(origin)?;

        	ensure!(disk != 0 as KIB, Error::<T>::DiskEmpty);

			/// 必须在非冷冻期
			ensure!(Self::is_chill_time(), Error::<T>::ChillTime);

        	let now = Self::now();

        	ensure!(Self::is_register(miner.clone())?, Error::<T>::NotRegister);

        	<DiskOf<T>>::insert(miner.clone(), MachineInfo {
        		disk: disk,
        		update_time: now,
        		is_stop: false,

        	});

        	Self::deposit_event(RawEvent::UpdateDiskInfo(miner, disk));

        }


		/// 矿工停止挖矿
		#[weight = 10_000]
        fn stop_mining(origin) {

        	let miner = ensure_signed(origin)?;

        	Self::is_can_mining(miner.clone())?;

			<DiskOf<T>>::mutate(miner.clone(), |h| {
				if let Some(x) = h {
					x.is_stop = true
				}
			});

			let mut staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();
			let others = staking_info.others;
			for staker_info in others.iter() {

				T::StakingCurrency::unreserve(&staker_info.0, staker_info.1.clone());
				T::StakingCurrency::unreserve(&staker_info.0, staker_info.2.clone());
			}

			staking_info.total_staking = <BalanceOf<T>>::from(0u32);

			staking_info.others = vec![];

			<StakingInfoOf<T>>::insert(&miner, staking_info);

			Self::deposit_event(RawEvent::StopMining(miner));
        }


        /// 矿工删除抵押者
        #[weight = 10_000]
        fn remove_staker(origin, staker: T::AccountId) {
			let miner = ensure_signed(origin)?;

			Self::update_staking_info(miner.clone(), staker.clone(), Oprate::Sub, None, true)?;

			Self::deposit_event(RawEvent::RemoveStaker(miner, staker));
        }


		/// 用户第一次抵押
        #[weight = 10_000]
        fn staking(origin, miner: T::AccountId, amount: BalanceOf<T>) {

        	let who = ensure_signed(origin)?;

			Self::is_can_mining(miner.clone())?;

			// 不在冷冻期
			ensure!(!<IsChillTime>::get(), Error::<T>::ChillTime);

			if Self::staker_pos(miner.clone(), who.clone()).is_some() {

				return Err(Error::<T>::AlreadyStaking)?;
			}

			let bond = amount.checked_add(&T::StakingDeposit::get()).ok_or(Error::<T>::Overflow)?;

			let staker_info = (who.clone(), amount.clone(), T::StakingDeposit::get());

			let mut staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();

			ensure!(staking_info.others.len() <= T::StakerMaxNumber::get(), Error::<T>::StakerNumberToMax);

			let total_amount = staking_info.clone().total_staking;

			let now_amount = total_amount.checked_add(&amount).ok_or(Error::<T>::Overflow)?;

			T::StakingCurrency::reserve(&who, bond)?;

			staking_info.total_staking = now_amount;

			staking_info.others.push(staker_info);

			<StakingInfoOf<T>>::insert(miner.clone(), staking_info);


			Self::deposit_event(RawEvent::Staking(who, miner, amount));

        }


		/// 抵押者更新抵押金额
        #[weight = 10_000]
        fn update_staking(origin, miner: T::AccountId, oprate: Oprate, amount: BalanceOf<T>) {

        	let staker = ensure_signed(origin)?;

			Self::update_staking_info(miner, staker.clone(), oprate, Some(amount), false)?;

			Self::deposit_event(RawEvent::UpdateStaking(staker, amount));

        }


        /// 用户退出抵押
        #[weight = 10_000]
        fn exit_Staking(origin, miner: T::AccountId) {
        	let staker = ensure_signed(origin)?;
        	Self::update_staking_info(miner.clone(), staker.clone(), Oprate::Sub, None, false)?;
        	Self::deposit_event(RawEvent::ExitStaking(staker, miner));
        }


		/// 矿工更改分润比
        #[weight = 10_000]
        fn update_portation(origin, portation: Percent) {

        	let miner = ensure_signed(origin)?;

			// 在冻结期内才能执行
        	ensure!(<IsChillTime>::get(), Error::<T>::NotChillTime);

        	Self::is_can_mining(miner.clone())?;

        	let mut staking_info = <StakingInfoOf<T>>::get(miner.clone()).unwrap();

        	staking_info.miner_portation = portation.clone();

        	<StakingInfoOf<T>>::insert(miner.clone(), staking_info);

			Self::deposit_event(RawEvent::UpdatePortation(miner, portation));
        }


		fn on_initialize(n: T::BlockNumber) -> Weight {
			let _ = Self::update_chill();
			0

       }

     }
}

impl<T: Trait> Module<T> {


	fn current_epoch_start() -> result::Result<u64, DispatchError> {

		let time = <babe::Module<T>>::current_epoch_start();
		let block_number = time.checked_div(MILLISECS_PER_BLOCK).ok_or((Error::<T>::Overflow))?;
		Ok(block_number)

	}


	/// 获取当前区块
	pub fn now() -> T::BlockNumber {

		<system::Module<T>>::block_number()
	}


	/// 判断自己是否是某个矿工的抵押者(是的话在什么位置)
	fn staker_pos(miner: T::AccountId, staker: T::AccountId) -> Option<usize> {
		let staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();
		let others = staking_info.others;
		let pos = others.iter().position(|h| h.0 == staker);
		pos
	}


	/// 判断是否进入冷却期
	fn update_chill() -> DispatchResult {

		let now = Self::now().saturated_into::<u64>();
		let chill_duration = T::ChillDuration::get().saturated_into::<u64>();
		let start_time = Self::current_epoch_start()?;

		let time = chill_duration.checked_add(start_time).ok_or(Error::<T>::Overflow)?;

		if now <= time {
			<IsChillTime>::put(true)
		}

		else {
			<IsChillTime>::put(false)
		}

		Ok(())

	}

	/// 判断是否已经注册
	fn is_register(miner: T::AccountId) -> result::Result<bool, DispatchError> {

		if <DiskOf<T>>::contains_key(&miner) && <StakingInfoOf<T>>::contains_key(&miner) {
			return Ok(true);

		}

		else {
			return Err(Error::<T>::NotRegister)?;
		}


	}


	/// 判断矿工是否可以挖矿
	pub fn is_can_mining(miner: T::AccountId) -> result::Result<bool, DispatchError> {
		ensure!(Self::is_register(miner.clone())?, Error::<T>::NotRegister);

		// 已经停止挖矿不能再操作
		ensure!(!<DiskOf<T>>::get(&miner).unwrap().is_stop, Error::<T>::AlreadyStopMining);

		Ok(true)
	}


	/// 更新已经抵押过的用户的抵押金额
	fn update_staking_info(miner: T::AccountId, staker: T::AccountId, oprate: Oprate, amount_opt: Option<BalanceOf<T>>, is_slash: bool) -> DispatchResult {
		// 如果操作是减仓 那么amount_opt是none意味着抵押者退出
		// 如果操作是加仓 那么amount_opt 不能是none值
		Self::is_can_mining(miner.clone())?;

		let mut amount: BalanceOf<T>;


		if let Some(pos) = Self::staker_pos(miner.clone(), staker.clone()) {

			let mut staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();

			let mut staker_info = staking_info.others.swap_remove(pos);

			/// 这个是减仓的时候
			if amount_opt.is_none() {
				amount = staker_info.1.clone()
			}
			else {
				amount = amount_opt.unwrap()
			}

			match  oprate {

				Oprate::Add => {
					let bond = staker_info.1.clone();
					let now_bond = bond.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
					let total_staking = staking_info.total_staking;
					let now_staking = total_staking.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
					T::StakingCurrency::reserve(&staker, amount)?;

					staker_info.1 = now_bond;

					staking_info.total_staking = now_staking;
				},

				_ => {
					let bond = staker_info.1.clone();
					let now_bond = bond.checked_sub(&amount).ok_or(Error::<T>::Overflow)?;
					let total_staking = staking_info.total_staking;
					let now_staking = total_staking.checked_sub(&amount).ok_or(Error::<T>::Overflow)?;

					T::StakingCurrency::unreserve(&staker, amount);

					staker_info.1 = now_bond;

					staking_info.total_staking = now_staking;

				},

			}

			if staker_info.1 == <BalanceOf<T>>::from(0u32) {
				if is_slash {

					T::StakingSlash::on_unbalanced(T::StakingCurrency::slash_reserved(&staker, staker_info.2.clone()).0);
				}

				else {
					T::StakingCurrency::unreserve(&staker, staker_info.2.clone());
				}

			}

			else{
				staking_info.others.push(staker_info);

			}

			<StakingInfoOf<T>>::insert(&miner, staking_info);


		} else {
			return Err(Error::<T>::NotYourStaker)?;

		}

		Ok(())

	}


}

decl_error! {
    /// Error for the ipse module.
    pub enum Error for Module<T: Trait> {
    	/// 已经注册过
		AlreadyRegister,
		/// 没有注册过
		NotRegister,
		/// p盘空间为0(不允许)
		DiskEmpty,
		/// 在冷冻期（只能矿工修改信息，用户不能进行抵押或是解抵押操作)
		ChillTime,
		/// 不在冷冻期
		NotChillTime,
		/// 已经停止挖矿
		AlreadyStopMining,
		/// 不是当前矿工的抵押者
		NotYourStaker,
		/// 已经抵押
		AlreadyStaking,
		/// 数据溢出
		Overflow,
		/// 抵押人数超过限制
		StakerNumberToMax,


	}
}
