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

genesis_base_target = 4 398 046 511 104 / 9 = 488 671 834 567

net_difficulty = 4 398 046 511 104 / 9 / base_target 


全网容量的估算：？？？

### 缩短挖矿周期时间

将常规的挖矿时间从240s缩短到9s，会带来下面的挑战：

- 最好的deadline和第二好的deadline差距非常小。很多比较好的deadline都在一个非常狭小的范围内。
- 如果依靠最佳deadline来确认出块的话，就会因为网络延迟和大量矿工提交“最佳deadline”而导致链有很多fork，很难进行收敛。特别是当矿工不断增加的时候，如果参与网络同步和验证的节点越来越多，链的收敛性就会受到越来越多的挑战。

IPSE的解决方案：

- 减少网络延迟，参与IPSE区块网络的节点都是配置极好的云主机，降低网络延迟，同时将出块节点进行一定的限制，保持在数百的规模。
- 减少矿工提交deadline的数量，在降低网络延迟的前提下，矿工能尽可能获得目前最佳deadline，只要自己计算出来的deadline不是更好的，就没必要提交了，因为会有高昂的链上计算手续费。
- 出块算法选用BABE+Grandpa组合，而不是依靠PoC算力来决定区块，此算法能让链拥有非常好的最终一致性，符合拜占庭容错。这样就能拒绝链出现过多fork而无法收敛的情况。


### 改写挖矿软件版本0.1

改写的挖矿软件来源[scavenger](https://github.com/PoC-Consortium/scavenger)

Rust语言整合Substrate的导入私钥、签名、请求接口等，来源： [substrate-subxt](https://github.com/paritytech/substrate-subxt)

- 1. 循环从链上获取上一个挖矿的区块高度数据、上一个挖矿transaction的tx作为32 byte的GenSig。
- 2. 用这两个数据计算出scoop_number。扫描所有P盘的scoop_number。
- 3. 计算deadline和比较选出最小的deadline。
- 4. 获取当前提交的最佳deadline，如果比自己找到的还要小，就丢弃掉。如果自己的更小，就提交deadline。（如果调用Substrate的API，需要钱包签名请求挖矿接口）。

### 链上新增挖矿接口

- 1. 出块时间调整为3s，每3个区块高度进行一次挖矿，每次挖矿请求都比较上一次挖矿时记录的区块高度，如果高度差 >= 3,就进行验证。
- 2. 如果验证通过，就进行奖励（奖励模块后面写），如果不通过，进行惩罚，还是保持这一次挖矿。
- 3. on_finalize()方法检查当前区块高度是否跟上一次挖矿时高度差 >= 3，如果是的，那就奖励目前最佳deadline，如果目前还没有提交任何deadline，那就更新挖矿信息，将这3个区块周期内的挖矿算作国库的（奖励模块结合国库模块，后面写）。
- 4. on_finalize()每30个区块高度进行一次难度调整，重新计算base_target和difficulty。之前每次挖矿所花费的时间（timestamp模块）和base_target都要进行记录。调整的时候，依据这些数据进行调整。base_target调整和difficulty调整是一回事，主要目的是为了保持大家提交最佳答案的时间基本能维持在7-8秒左右。


