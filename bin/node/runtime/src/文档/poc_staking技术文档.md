## ***poc_staking文档***
***
### 一、模块主要说明
* 冷却期是指矿工用来修改数据， 其他人不能进行抵押操作的时间段。用来保证矿工信息的稳定性
* 一旦poc被选中，立马分发奖励
* 不需要去绑定账号（stash与controller)
* 只有在这个模块进行注册过，才能进行poc挖矿
* 挖矿奖励的总量是3500万， 两年减半

***
### 数据结构
* 抵押信息
```
/// 矿工的机器信息
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MachineInfo<BlockNumber> {
	/// 磁盘空间
	pub disk: KIB,
	/// P盘id
	pub pid: u128,
	/// 更新时间
	pub update_time: BlockNumber,
	/// 机器是否在运行（这个是用户抵押的依据)
	is_stop: bool,
}
```
```
// 抵押信息
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct StakingInfo<AccountId, Balance> {
	/// 矿工
	pub miner: AccountId,
	/// 矿工分润占比
	pub miner_proportion: Percent,
	/// 总的抵押金额
	pub total_staking: Balance,
	/// 其他人的抵押 （staker， 抵押金额， 保留金额)
	pub others: Vec<(AccountId, Balance, Balance)>,
}
```
```
/// 操作
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Oprate {
	/// 添加
	Add,
	/// 减少
	Sub,
}
```
***
* 操作方向
```
enum oprate {
    add, // 增加金额
    sub, // 减少金额
}
```
### ***storage***

```buildoutcfg
        /// 矿工磁盘空间信息
		pub DiskOf get(fn disk_of): map hasher(twox_64_concat) T::AccountId => Option<MachineInfo<T::BlockNumber>>;

		/// 是否在非抵押操作期间（冷冻期，只有矿工能改变信息)
		pub IsChillTime get(fn is_chill_time): bool = true;

		/// 每个矿工对应的抵押信息
		pub StakingInfoOf get(fn stking_info_of): map hasher(twox_64_concat) T::AccountId => Option<StakingInfo<T::AccountId, BalanceOf<T>>>;

		/// 用户现在抵押的矿工
		pub MinersOf get(fn mminers_of): map hasher(twox_64_concat) T::AccountId => Option<Vec<T::AccountId>>;

	    /// P盘id对应的矿工
		pub AccountIdOfPid get(fn accouont_id_of_pid): map hasher(twox_64_concat) u128 => Option<T::AccountId>;

```
### 主要方法

1. 矿工注册
    * 代码: `fn register(origin, plot_size: GIB, numeric_id: u128, miner_proportion: u32, reward_dest: Option<T::AccountId>)`
    * 参数:
        * plot_size: P盘空间（以gib为单位)
        * numeric_id: P盘id
        * miner_proportion: 矿工分润比
        * reward_dest: 矿工收益地址
    * 逻辑：
        * 签名
        * plot_size不能为0
        * 如果reward_dest是None，那么默认收益地址是自己
        * 没有注册过
***
2. 矿工修改P盘id
    * 代码：`fn update_numeric_id(origin, numeric_id: u128)`
    * 参数：
        *numeric_id: 用来p盘的id
    * 逻辑:
        * 签名
        * 自己是能挖矿的矿工
        * pid还没有被使用过
        * 不需要删除之前的挖矿记录，也不需要修改更新时间
***
3. 更新p盘空间大小
    * 代码：`fn update_plot_size(origin, plot_size: GIB)`
    * 参数:
        * plot_size: P盘空间（以gib为单位)
    * 逻辑：
        * 签名:
        * plot_size不能为0
        * 在非冷却期
        * 删除掉之前的挖矿历史记录
***
4.  矿工停止挖矿
    * 代码：`fn stop_mining(origin)'
    * 参数: 无
    * 逻辑：
        * 自己是能挖矿的矿工

***
5. 矿工删除抵押者
    * 代码: `fn remove_staker(origin, staker: T::AccountId)`
    * 参数:
        * staker: 抵押者
    * 逻辑:
        * 签名
        * 自己是矿工
        * 抵押者在自己名下有抵押
        * 惩罚抵押者保留金额， 琐仓抵押者抵押金额
        * 更新抵押信息
***
6. 用户第一次抵押
    * 代码：`fn staking(origin, miner: T::AccountId, amount: BalanceOf<T>)'
    * 参数:
        * miner: 矿工
        * amount： 抵押金额
    * 逻辑:
        * 签名
        * 矿工可以挖矿
        * 不在冷却期
        * 抵押人数还没有达到上限
        * 自己没有进行抵押过
        * 需要保留余额（如果被矿工删除， 那么就惩罚掉, 作为抵押成本)
***
7. 抵押者更新抵押金额
    * 代码：`fn update_staking(origin, miner: T::AccountId, oprate: Oprate, amount: BalanceOf<T>)`
    * 参数:
        * miner: 矿工
        * oprate： 增加或减少
        * amount: 增加或减少的金额
    * 逻辑：
        * 签名
        * 矿工可以挖矿
        * 自己抵押过这个矿工
        * 如果是减少抵押金额，减少的部分要进行锁仓
***
8. 用户退出抵押
    * 代码：`fn exit_Staking(origin, miner: T::AccountId)`
    * 参数：
        * miner: 矿工
    * 逻辑：
        * 签名
        * 抵押过这个矿工
        * 归还保留金额， 琐仓抵押金额
        * 更新矿工名下抵押信息
***
9. 矿工更改分润比
    * 代码：`fn update_proportion(origin, proportion: Percent) `
    * 参数:
        * proportion: 分润占比
    * 逻辑：
        * 签名
        * 自己是矿工， 并且可以挖矿
        * 在冷却期内
***
10. 矿工申请进入推荐列表
    * 代码: `fn request_up_to_list(origin, amount: BalanceOf<T>)`
    * 参数:
        * amount: 自己愿意抵押的金额
    * 逻辑：
        * 矿工才能操作
        * 自己正在挖矿
        * 自己如果之前在列表里，那么算是增加金额（累加)
***
11. 矿工退出推荐列表
    * 代码： `fn request_down_from_list(origin)`
    * 逻辑：
        * 自己在列表中
        * 从列表中删除
        * 对抵押的金额进行琐仓
***
12. 修改矿工收益地址
    * 代码: `fn update_reward_dest(origin, dest: T::AccountId)`
    * 逻辑：
        * 自己是矿工
***
13. 矿工重新开始挖矿
    * 代码: `fn restart_mining(origin)`
    * 逻辑：
        * 自己是矿工
        * 挖矿已经停止过
        * 把矿工的机器状态改成可挖矿
***
14. 用户手动领取琐仓金额
    * 代码: `fn unlock(origin)`
    * 逻辑：
        * 一键领取自己所有到期的琐仓
***



