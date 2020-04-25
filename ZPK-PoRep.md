### 零知识证明运用

零知识证明的工作流程主要分为4步：

- 首先将欲证明的计算性问题，转换成门电路 Circuit
- 然后将门电路 Circuit 转换成 R1CS
- 再将 R1CS 转变成 QAP
- 最后，基于 QAP 实现 zkSNARK 的算法


### 参与方

- 数据提供方Alice（花钱存储）
- 数据存储方Bob（矿工）
- 区块链主网
- 数据索引方（IPSE搜索引擎）

### 流程

- 数据提供方存储数据，本地加密或者不加密，向链上提交存储数据请求（出价）。
    - 订单要说明数据存储周期，数据冷热，备份数量，数据提取次数。
    - 数据提供方支付定金，锁定在链上合约里。
- 数据存储方接单，提供存储。接单矿工需要有公网ip，要能从ipfs网络接收数据提供方的数据。
    - 如果是多个备份，多个数据存储方可以一起接单。
    - 如果是热数据，就必须在矿工保存一份准备随时提取，如果是冷数据，可以让其他存储节点保存。
- Alice本地将数据传到ipfs网络，Bob从ipfs网络拉取数据，BoB存储完毕后，做出一个数据存储零知识证明上链。
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

矿工作为证明者，需要将原始数据编码成复制数据，并且需要对这个复制数据进行承诺，PoRep对这个承诺提供一个证明。

Sector处理，理解为precommit阶段，把原始数据用sha256构造默克尔树tree_d，树根为comm_d。

原始数据，每32字节，称为一个Node，每32M分为一个Window，128M的Sector就会有4个Window，每个Window按照Stacked DRG算法，生成2个layer的数据，从上一个layer，通过Encode计算生成下一个layer的数据，Encode计算可以采用简单模加操作。具体也是超级简单，将Window的编号和Stacked DRG的节点关系通过sha256算法，生成“key”，然后用来和原始数据模加操作Encode计算得出结果。

最后一层layer生成的数据，构造默克尔树tree_q，可以采用pedersen或者poseidon的Hash函数，树根为comm_q。最后一层layer的生成数据，再经过一层exp的依赖关系，构造默克尔树tree_r_last，树根为comm_r_last。

layer2的4个Window的数据中，同一个位置上的Node拼接在一起，hash后生成Column Hash，针对Column Hash的计算结果，构造默克尔树tree_c，树根为comm_c。


> VDF的主要目的是抵抗并行计算，串行计算，后一个会依赖前一个

comm_r是podersen_hash(comm_c|comm_q|comm_r_last)的结果。

证明过程：结合链上生成的随机数，在replica复制数据中随机挑选1个Node数据。让矿工证明这个Node数据是从原始数据一步步处理生成的，而且，能构造comm_d,comm_c,comm_q,comm_r_last。

使用VDE函数，目的是让临时下载数据再来构建证明所花费的时间很长，从而挑战-证明流程中，就不符合在规定时间内完成证明的要求，也就可以防止矿工不在本地存储数据，临时下载的可能性。


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



### Filecoin的PoRep解读

首先专注在storage-proofs这部分，里面有三个重要的组成目录：

- core :复制证明底层的逻辑部分。
- porep :复制证明的逻辑。
- post :在复制证明的基础上进行时空证明的逻辑。

#### core部分

从`Cargo.toml`分析，除了依赖另外一个porep部分，还有依赖上一个目录的sha2raw模块，那让我们先分析下`sha2raw`部分。

##### sha2

SHA-2加密哈希函数的实现，有6个标准算法实现，算法上，只有两个核心算法:Sha256和Sha512。所有其他算法都是这些算法的应用，它们具有不同的初始散列值，并被截断为不同的摘要位长度。

Sha256的实现，重点是对固定大小的块进行hashing，不需要填充。

##### drgraph 

drgraph是core中最为重要的模块，这里的Graph就是DRG（depth robust graoh）。

用于所有DRG图的`BASE_DEGREE`=6。此值的一个DEGREE值用于确保给定节点始终将其直接前辈作为父节点，从而确保图节点具有惟一的拓扑次序。也就是能让计算具有串行计算的属性，这是VDF所必须要求的属性。

这个`Graph` Trait具有哪些方法：

- expected_size: 返回在图中所有节点加起来的大小。NODE_SIZE配置为32字节。
- merkle_tree_depth: 返回merkle树的深度，根据叶子节点的数量推算出默克尔树的高度。
- parents：返回一个节点其排好序的父节点列表。其父节点可能是重复的。如果一个节点没有父节点，其父节点列表向量需要在第一个元素放置一个该请求的节点，用来表示这个节点是没有父节点的。
- size: 返回这个图的节点数量，也就是图的大小。
- degree: 返回在图中每个节点的父节点数量。
- new: 构建一个图。
- create_key: 创建用于encoding的key，将Window的编号和Stacked DRG的节点关系通过sha256算法，生成“key“。

##### data

data模块是一个包装器，磁盘上或内存片上的数据包装器，可以将其删除并读入内存，以便更好地控制内存消耗。

对原始数据进行加载，包装成`Data`结构体，其中有一些构造方法from，也有对原始数据`RawData`进行的解引用方法`deref`,`deref_mut`，还有给`Data`的引用方法`as_ref`,`as_mut`。

##### parameter_cache

这个是非常简单的，就是加载zk-snark所需的setup参数。

##### fr32

`Fr32`是什么？

	pub type Fr32 = [u8];
	pub type Fr32Vec = Vec<u8>;	
	pub type Fr32Any = [u8;32];
	
> 想要理解Fr是什么，就需要了解有限域上的椭圆曲线基础知识。Fp是有限域，在这个基础上建立的椭圆曲线点点运算都是在这个域范围内。有限域上的椭圆曲线上有很多循环子群Fr，具有加法同态的性质。核心利用的就是离散对数问题：在循环子群上已知两点，却很难知道两点的标量。
> 
> 循环子裙是怎么来的？在有限域上的椭圆曲线中一个点标量乘法的结果，组成一个在加法操作下的循环子群。在子群中的点，所有的加法的结果都还在子群中。而且，存在一个点，幂次（加法操作）能生成子群中的所有点。这样的点，称为“生成元”。

具体要了解更多椭圆曲线的基础知识可以参考这篇博客 [零知识证明 - 椭圆曲线基础](https://mp.weixin.qq.com/s?__biz=MzU5MzMxNTk2Nw==&mid=2247486862&idx=1&sn=38b326ce8d694617252e58ea3f0c3a3c&chksm=fe131c9ec96495888fe990458b5f164440a4ca386db93904ce30ce5096344d17083daab9030e&scene=21#wechat_redirect)
	
`Fr32`包含一个或多个32字节的chunks，其小端值表示的就是Frs。有两点需要注意，每个32字节的chunk一定表示有效的Frs，总长度必须是32的倍数。也就是说，单独使用的每个32字节chunk必须是有效的Fr32。

- bytes_info_fr: 输入一个字节数组，如果其刚好是32字节，返回一个Fr，否则，返回一个`BadFrBytesError`。
- trim_bytes_to_fr_safe: 去掉多余的，返回一个Fr。
- bytes_into_fr_repr_safe:返回一个FrRepr。
- fr_into_bytes: 输入一个Fr，返回一个刚好32字节长度的向量。
- bytes_into_frs:输入一个字节数组，返回一组Fr的向量，如果字节数量不是32的倍数，返回一个错误。
- frs_into_bytes:一组Fr的向量输入，返回一个字节向量。
- u32_into_fr:输入u32，返回一个Fr。

paired模块中引入了`bls12_381`方法，这是最新的对zk-snark椭圆曲线的构造方法。

	use paired::bls12_381::{Fr, FrRepr};

BLS12-381是BLS族的一种友好配对的椭圆曲线结构，嵌入度为12。它建立在一个381位的基本字段GF(p)上。

zk-snark的验证者需要一个paired，能够支持加法和乘法的同态隐藏。椭圆曲线能够帮助我们获得有限制的，但满足需求的支持乘法的同态隐藏的方法。

##### merkle

首先是`builders`模块：

- create_disk_tree:这里的DiskTree继承了MerkleTreeWrapper。从提供的参数，base_tree_len来创建一个DiskTree，每一个参数配置都代表一个基础层的树。
- create_lc_tree:对数据进行复制生成备份后，生成的LCTree。
- create_tree:输入树的参数配置，可选输入复制备份路径，返回一个DiskTree或者LCTree，是由树的参数配置来制定的。
- create_base_merkle_tree:创建基础默克尔树。
- create_base_lcmerkle_tree:
- split_config:
- split_config_wrapped:
- split_config_and_replica:
- get_base_tree_count:
- get_base_tree_leafs:

其次是`tree`模块：

- trait MerkleTreeTrait
- struct MerkleTreeWrapper
- new:直接输入data数据，生成Merkle树。
- new_with_config：输入可迭代的数据和参数配置，生成Merkle树。
- from_data_with_config：输入可迭代的数据和参数配置，生成Merkle树。
- from_data_store：输入存储数据和叶子节点，生成Merkle树。
- from_tree_slice：输入数组数据和叶子节点，生成Merkle树。
- from_tree_silice_with_config：输入数组数据和叶子节点和参数配置，生成Merkle树。
- from_trees：输入Merkle树包装器向量数据，生成Merkle树。
- from_sub_trees：输入Merkle树包装器向量数据，生成Merkle树。
- from_sub_trees_as_trees：输入Merkle树包装器向量数据，生成Merkle树。
- from_slices：输入由Merkle树组成的数组和叶子节点，生成Merkle树。
- from_slices_with_configs：输入由Merkle树组成的数组和叶子节点和参数配置，生成Merkle树。
- from_stores：输入叶子节点和存储向量数据，生成Merkle树。
- from_store_configs：输入叶子节点和存储配置，生成Merkle树。
- from_store_configs_and_replicas：输入叶子节点，存储配置和复制文件路径，生成Merkle树。
- from_sub_tree_store_configs：输入叶子节点和存储配置，生成Merkle树。
- try_from_iter：输入迭代器，生成Merkle树。
- from_sub_tree_store_configs_and_replicas：输入叶子节点，存储配置和复制文件路径，生成Merkle树。
- try_from_iter_with_config：输入迭代器和存储配置，生成Merkle树。
- from_par_iter：输入迭代器，生成Merkle树。
- from_par_iter_with_config：输入迭代器，生成Merkle树。

最后是`proof`模块：

- trait MerkleProofTrait: 默克尔树的证明Trait
- base_path_length：输入叶子节点leafs，得到基础路径长度，等于整个图的高度-1。
- compound_path_length:计算整个路径的期望长度，给定基层中的叶子数量。
- compound_tree_height:基础层、子树层和顶级树层高度组合起来。
- struct InclusionPath: 是PathElement组成的向量，代表着树的路径。
- root:输入叶子节点，计算组成路径的Merkle根。
- struct PathElement:由hash值组成的向量和索引构成的结构体。
- struct MerkleProof:一个默克尔证明的表示结构体。
- enum ProofData:证明数据有三种，单个叶子节点证明，子树证明，树根证明。
- struct SingleProof：包括三个部分，默克尔树根root，单个叶子节点数据leaf，叶子到根的路径path。
- struct SubProof：子树证明包括4个部分，base_proof和sub_proof是InclusionPath，root是默克尔树根，leaf是证明的叶子节点数据。
- struct TopProof：树根证明包括5个部分，base_proof、sub_proof和top_proof是InclusionPath，root是默克尔树根，leaf是证明的叶子节点数据。


##### por

目前这个por是最容易看懂的部分了，有如下这些数据结构体：

- struct DataProof:包括两部分，proof是MerkleProofTrait，data是叶子节点数据。这个结构体就是矿工进行PoR证明最终要生成的证明。
- struct PublicParams：矿工证明者和验证者都需要知道的公共参数，其实就一个，底层默克尔树有多少叶子节点。
- struct PublicInputs：验证者给出公共输入，矿工证明者要根据这些输入完成PoR的证明。包括底层默克尔树的根hash，另外就是挑战，随机挑选一个叶子，要让证明着生成证明。
- struct PrivateInputs：只有矿工作为证明着自己能看到的输入，包括两部分，叶子节点的数据和底层默克尔树。
- struct SetupParams：设置参数，PoR中首先就要设置参数，默克尔树有多少叶子节点作为参数设置进去。
- struct PoR：基于默克尔树的数据可检索证明。

证明验证逻辑主要是三个方法：

- setup：设置参数。
- prove：核心的一步就是`tree.gen_proof(challenge)`。
- verify：核心的两步是`proof.proof.validate_data(proof.data)`和 `proof.proof.validate(pub_inputs.challenge)`

###### tree.gen_proof(challenge)

从底层默克尔树，随机挑选一个叶子节点进行挑战，然后生成证明。

###### proof.proof.validate_data(proof.data)

首先是 `verify()`，针对不同的默克尔树，都需要验证，比如是TopProof，需要计算top_proof的根是否跟原有证明里的root是否相同。

然后才是输入的叶子节点数据是否相同。

###### proof.proof.validate(pub_inputs.challenge)

同样也是先 `verify()`，然后验证选中用来挑战的叶子节点谁不是同一个。

##### proof

`proof`是非常简单的，就一个`ProofScheme`的trait，然后就是`setup`，`prove`，`prove_all_partitions`，`verify`，`verify_all_partitions`几个方法。

- setup:从初识设置参数中生成公共参数。
- prove:生成证明。
- prove_all_partitions:生成多份证明。
- verify:验证。
- verify_all_partitions:对所有证明进行验证。

##### gadgets

DRG电路设计是PoRep和PoST所需要的。这个模块就是如何搭建相应的电路。

###### constrait

实现了限制条件的集中运算的方法。

- equal
- sum
- add
- sub
- difference

###### encode

- encode:encode就是模加操作。
- decode:decode就是模减操作。

###### insertion

插入排列，在任意位置向序列中插入AllocatedNum。

- insert
- select
- pick,根据输入condition判断真伪，返回输入a或者b，a和b都是AllocatedNum

###### multipack

- pack_bits:获取一个布尔值序列，并将它们暴露为单个紧凑Num。

###### pedersen



###### por

Proof of Retrievability的电路。

- struct PoRCircuit，包含了椭圆曲线参数，叶子节点的Merkle树路径，默克尔树的根，叶子节点的值。
- struct AuthPath
- struct SubPath
- struct PathElement
- struct PoRCompound

impl CompoundProof for PoRCompound

- fn circuit
- fn blank_circuit
- fn generate_public_inputs

这里的CompoundProof是上一个compund_proof模块的内容。

impl Circuit for PoRCircuit

- fn synthesize

###### variables

###### xor
	
##### compound_proof

这个模块就是将PoR的电路组合起来，形成一个证明的电路。底层使用的是bellperson的groth16.

- struct SetupParams
- struct PublicParams
- trait CircuitComponent
- trait CompoundProof
	- fn setup
	- fn partition_count
	- fn prove
	- fn verify
	- fn batch_verify
	
	

























