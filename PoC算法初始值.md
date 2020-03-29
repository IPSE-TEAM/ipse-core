# PoC算法初始值

矿工从众多nonce中找deadline，每一个nonce都会提供一个deadline，每个nonce的大小为256kb，4个为1M大小。每个deadline是一个64位的整数，这个64位的整数分布范围是0到2^64-1。而这个deadline是独立均匀分布的，符合independent identically distributed（i.i.d)。扫盘找寻deadline就意味着找到最小的deadline（相比于base_deadline）。

E(X) = (b + a*n)/(n+1) 

- (a,b)是deadline 分布在(0..2^64)之间的一个可能的值。
- n是表示有多少nonce被扫描到，也就是有多少个deadline。

如果我们希望在9s的时间内全部扫盘完，那么意味着E(X) = 9。

如果没有任何调整的话，我们在9s时间内需要计算的nonce数量：

nonces = (2^64-1)/9-1 = 2 049 638 230 412 173 000

折算成空间大小 TiB： 2 049 638 230 412 173 000 / 4 / 1024 / 1024 = 488 671 834 567 TiB

那genesis base target需要多少TiB呢， 488 671 834 567 TiB * 9 = 4 398 046 511 104 TiB

这个base_target其实取决于blocktime，也就是多长时间内扫描完盘。

base_target = 4 398 046 511 104 / 9 = 488 671 834 567

net_difficulty = 4 398 046 511 104 / 9 / base_target 


全网容量的估算：？？？


### 改写挖矿软件版本0.1

- 1. 循环从链上获取上一个挖矿的区块高度数据、上一个挖矿transaction的tx作为32 byte的GenSig。
- 2. 用这两个数据计算出scoop_number。扫描所有P盘的scoop_number。
- 3. 计算deadline和比较选出最小的deadline。
- 4. 获取当前提交的最佳deadline，如果比自己找到的还要小，就丢弃掉。如果自己的更小，就提交deadline。（如果调用Substrate的API，需要钱包签名请求挖矿接口）。

### 链上新增挖矿接口

- 1. 出块时间调整为3s，每3个区块高度进行一次挖矿，每次挖矿请求都比较上一次挖矿时记录的区块高度，如果高度差 >= 3,就进行验证。
- 2. 如果验证通过，就进行奖励（奖励模块后面写），如果不通过，进行惩罚，还是保持这一次挖矿。
- 3. on_finalize()方法检查当前区块高度是否跟上一次挖矿时高度差 >= 3，如果是的，那就奖励目前最佳deadline，如果目前还没有提交任何deadline，那就更新挖矿信息，将这3个区块周期内的挖矿算作国库的（奖励模块结合国库模块，后面写）。


