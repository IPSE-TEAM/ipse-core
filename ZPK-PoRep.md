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


### PoRep的电路设计

PoRep就是Proof of Replicate，数据存储证明。

我们的逻辑是从对原始数据密封seal开始，用户存储的数据分为一个个Sector，比如这个扇区大小可以设置为128M。

首先是对原始存储数据进行密封，密封完后会有一个Replica_ID。数据的复制逻辑是比较复杂的，可以采用Stacked-DRG的方式组织，直观理解就是将Sector中未seal的原始数据依次切成一个个小数据node，比如每个小数据32个字节，这些小的数据按照DRG（Depth Robust Graph）建立连接关系。按照每个小数据的依赖关系，通过VDE（Verifiable Delay Encode）函数，计算出下一层的所有小数据。PoRep的计算过程是有好几层。

Sector处理，理解为precommit阶段，把原始数据用sha256构造默克尔树tree_d，树根为comm_d。

原始数据，每32字节，称为一个Node，每32M分为一个Window，128M的Sector就会有4个Window，每个Window按照Stacked DRG算法，生成2个layer的数据，从上一个layer，通过Encode计算生成下一个layer的数据，Encode计算可以采用简单模加操作。具体也是超级简单，将Window的编号和Stacked DRG的节点关系通过sha256算法，生成“key”，然后用来和原始数据模加操作Encode计算得出结果。

最后一层layer生成的数据，构造默克尔树tree_q，可以采用pedersen或者poseidon的Hash函数，树根为comm_q。最后一层layer的生成数据，再经过一层exp的依赖关系，构造默克尔树tree_r_last，树根为comm_r_last。

layer2的4个Window的数据中，同一个位置上的Node拼接在一起，hash后生成Column Hash，针对Column Hash的计算结果，构造默克尔树tree_c，树根为comm_c。


> VDF的主要目的是抵抗并行计算，串行计算，后一个会依赖前一个

comm_r是podersen_hash(comm_c|comm_q|comm_r_last)的结果。

证明过程：结合链上生成的随机数，在replica复制数据中随机挑选1个Node数据。让矿工证明这个Node数据是从原始数据一步步处理生成的，而且，能构造comm_d,comm_c,comm_q,comm_r_last。


> TAU和AUX
>  TAU：一棵或者多棵Merkle树的树根都称为TAU
> 
>  AUX：Auxiliary的简称，一棵或者多棵Merkle树的结构称为AUX

comm_r是综合了每一层的所有Encoder输出和Replica_ID的信息。

复制数据完成后，需要提供零知识证明。零知识证明核心三部曲：

- Setup：设置证明的参数。
- Prove：提供需要证明的内容。
- Verify：链上快速验证上链的证明。

而零知识证明的构建，需要有两组参数的输入：

- public inputs：原始数据的comm_d和comm_r。
- private inputs：原始数据tree_d，最后一层构造的默克尔树tree_q和tree_r_last。

然后在矿工完成复制证明后，需要提交到链上的数据是：

- PoRep的公共数据
- 复制证明数据Proof

最后就是链上对Proof进行验证的逻辑。


### Circom

`Circom`是一种用于编写零知识证明的算术电路的语言，Circom简化了创建`zk-snark`电路的复杂度。`Circom`的`template`关键字，类似java中面向对象语言中的class，和class一样，定义好的模版可以在不同的电路或者其他项目中重用。

`circom`提供的5个额外的操作符用来定义约束。

- `<==,==>` :用来给signal赋值，同时会生成一个等于约束。
- `<--,-->` :赋值符号，用于给signal赋值，但不生成任何约束。
- `===`	:定义了等于约束



- mimc ： SNARK-friendly hash Minimal Multiplicative Complexity.

- pedersen：Pesersen commitment，在承诺方和接收方之间，满足完美隐藏、计算绑定的同态加密承诺协议。例如财务审计，被审计方不想暴露具体财务流水秘密，将数据加密后交给审计方验证，被审计方通过pedersen承诺保证自己不能造假。其实这里的审计工作是一个恒等验证，被审计方提供验证v1+v2=v3，v1和v2被验证方持有（不可泄露的秘密交易数据），审计方只能从被验证方得到v3。通俗理解，被审计方不想告诉你我的所有交易过程数据，但是我可以告诉你我交易后最终的余额。有这样一个案例，根据固定的评估计算方法，对某个人的流水数据和余额进行信用分评估，但是又不泄露具体的数值，最终对信用分进行验证。
- poseidon: pedersen hash算法的升级版本，主要是优化了其约束性条件，相对pedersen hash而言，减少了8倍的复杂度，电路复杂度大大降低。证明和验证速度能更快。



