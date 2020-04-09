### 零知识证明运用

零知识证明的工作流程主要分为4步：

首先将欲证明的计算性问题，转换成门电路 Circuit
然后将门电路 Circuit 转换成 R1CS
再将 R1CS 转变成 QAP
最后，基于 QAP 实现 zkSNARK 的算法


### 参与方

- 数据提供方Alice（花钱存储）
- 数据存储方Bob（矿工）
- 区块链主网
- 数据索引方（IPSE搜索引擎）

### 流程

- 数据提供方存储数据，本地加密或者不加密，向链上提交存储数据请求（出价）。
    - 订单要说明数据存储周期，数据冷热，备份数量，数据提取次数。
    - 数据提供方支付定金，锁定在链上合约里。
- 数据存储方接单，提供存储。接单矿工需要有公网ip，要能远程接收数据提供方的数据。
    - 如果是多个备份，多个数据存储方可以一起接单。
    - 如果是热数据，就必须在矿工保存一份准备随时提取，如果是冷数据，可以让其他存储节点保存。
- Alice向Bob推送数据，BoB存储完毕后，做出一个数据存储零知识证明上链。
- 区块链主网验证后，按照合约发放一定比例的Token给Bob。
- 区块链主网定时验证Bob提供的数据存储零知识证明，逐渐释放Token给Bob。
- Alice需要提取数据时，需要在链上记录一次，然后向Bob提取数据。


### ZoKrates工具链

使用ZoKrates工具链可以很方便提供某种计算的证明，整个流程如下：

- 1. compile 编译电路，对于想证明的计算，需要设计和开发电路。ZoKrates采用DSL（Domain Specific Language）描述电路。ZoKrates也提供一些常用的电路库（SHA256，椭圆曲线的计算等）。
- 2. setup 设置，对于每个电路，在生成证明之前，必须setup一次，生成CRS。
- 3. compute-witness 生成witness，在提供了private/public输入的情况下，ZoKrates自动根据电路计算出对应的witness。
- 4. generate-proof 生成证明信息。
- 5. export-verifier 导出证明工具。比如可以在链上进行验证。


ZoKrates给出详细的电路描述和编译的说明：

[https://zokrates.github.io](https://zokrates.github.io/)

### PoR的电路

数据Retrieval证明的电路，通过PoRCircuit来实现，可以简单通过Merkle树来实现，PoRCircuit的电路通过结合叶子节点和路径信息，最后计算出来的树根和提供的树根是否一致。

Proof of retrievability base on Merkle tree。

验证者要去验证矿工证明的输入数据public_inputs有：

- commitment：底层Merkle tree的根hash
- challenge： 哪一个叶子节点接受挑战

矿工证明者完成证明需要的输入数据private_inputs有：

- leaf： 叶子节点数据
- tree： 底层 Merkle tree

当然还有证明者和验证者都知道的公共知识，那就是Merkle Tree有多少叶子节点。

