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
struct stking_info {
    矿工id: AccountId,
    矿工分润比：Permill,
    总抵押金额：Balance,
    具体抵押情况：Vec<(AccountId, Balance)>,
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

		/// 自增的p盘id
		pub Pid get(fn p_id): u64;

		/// 矿工对应的p盘id
		pub PidOf get(fn account_id_of): map hasher(twox_64_concat) T::AccountId => Option<u64>;

```
### 主要方法

1. 矿工注册
    * 代码: `fn register(origin, kib: KIB, miner_proportion: Percent)`
    * 参数:
        * kib: P盘空间（以kib为单位)
        * miner_proportion: 矿工分润比
    * 逻辑：
        * 签名
        * kib不能为0
        * 没有注册过
        * 获得一个自增的p盘id（p盘时候用到)
***
2. 矿工修改P盘id
    * 代码：`n update_pid(origin, pid: u64) `
    * 参数：
        *pid: 用来p盘的id
    * 逻辑:
        * 签名
        * 自己是能挖矿的矿工
***
3. 更新磁盘信息
    * 代码：`fn update_disk_info(origin, kib: KIB)`
    * 参数:
        * kib: P盘空间（以kib为单位)
    * 逻辑：
        * 签名:
        * kib不能为0
        * 在非冷却期
        * 删除掉之前的挖矿历史记录
***
4.  矿工停止挖矿
    * 代码：`fn stop_mining(origin)'
    * 参数: 无
    * 逻辑：
        * 自己是能挖矿的矿工
        * 归还每个人的抵押金额以及保留金额
        * 抵押消息初始化
***
5. 矿工删除抵押者
    * 代码: `fn remove_staker(origin, staker: T::AccountId)`
    * 参数:
        * staker: 抵押者
    * 逻辑:
        * 签名
        * 自己是矿工
        * 抵押者在自己名下有抵押
        * 惩罚抵押者保留金额， 归还抵押者抵押金额
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
***
8. 用户退出抵押
    * 代码：`fn exit_Staking(origin, miner: T::AccountId)`
    * 参数：
        * miner: 矿工
    * 逻辑：
        * 签名
        * 抵押过这个矿工
        * 归还抵押金额与保留余额
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


