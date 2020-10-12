### Polkadot XCMP

Polkadot中消息传递可以简单分为两大类：水平消息传递（HMP）和垂直消息传递（VMP）。水平消息传递发生在平行链之间，其实就是XCMP；垂直消息传递发生在平行链和终极链之间。而VMP包括上传UMP和下传DMP两种方式。

XCMP(Cross-Chain Message Passing)主要用于平行链与平行链之间消息通信传递机制。在此过程中要尽量减少中继链的消耗。



要深刻理解波卡的共享安全机制，就要理解底层的消息通信，链与链之间的消息可以做到可验证，也就是可信的程度，但可信并不代表安全，比如一条只有几个节点达成共识的链的消息，传递过来是可以验证的，但因为其网络价值不足以保证安全，所以如果是这样的链之间进行通信，那价值网络会有很多安全黑洞。所以为了解决这个问题，就需要进行波卡的共享安全来保证，也就是接入波卡的平行链无论如何都需要付出一定的代价，不管是平行链拍卖，还是平行线程的抢购，只有这样才能将各个链之间的安全差异尽可能拉高到一个基准线，而只要接入波卡的平行链协议，就不存在安全上的巨大差异，而只有合理与不合理的差异，而合理与否就需要博弈了。


而在XCMP之前，会有一个阉割版的方案先出来，那就是HRMP（Horizontally Relay-routed Message Passing），与XCMP不同在于，他是将所有消息通过中继链来存储和中转，也就是说中继链的压力会比较大，但这只是早期的过度版本而已。

### XMP通信模型

- 异步性：发送者发送完消息后，该出块就出块，不需要管消息接下来如何的。
- 完备性：消息能够及时有序地传递和准确解释，中间不要对消息做手脚。
- 不对称性：消息并没有结果。返回的结果其实是另外一条消息了。
- 不可知性：消息对于传递消息的共识系统的性质是不可知的。传递消息是发生在平行链之间，还是平行链与转接桥之间，或者就是平行链与智能合约之间，消息是不需要知道的。

### 定义

- Consensus System：可以是链的，智能合约或者单个状态转换机的共识系统。这个共识系统只要能够发送和接收数据报。
- Location：可寻址的一个地址，比如国库地址，私人基金会地址，平行链地址，多签钱包地址等。
- Sovereign Account：一个被某个特定共识系统控制的账号，可以是一个账号或者多个账号，如果是多个账号，那多个账号指向一个单一主账号。
- Holding Account：一个临时的概念性“帐户”，其中暂时保留消息中固有的资产。
- Reserve location：保留地址，比如特定资产，在另外一个共识系统中有相应的一个衍生品，但原来的共识系统总能被衍生品所识别，而且在原来的共识系统中有一个主权账号，所以在原来共识系统中有衍生品的全部抵押品。
- Origin：消息发送源，从某个共识系统中传递消息出来，这个能够被接收者使用消息传递协议来查询。
- Recipient：消息接收源，某个共识系统中接收一个被传递到的消息。
- Teleport：传送，从一个地方销毁资产，在另外一个地方铸造相应资产，这就称为传送。传送的两个地方，在性质上没必要是一致的，一个地方可能是UTXO模型，另外一个地方可能是账号模型。传送发生的前提就是，双方有可信任的关系，包括状态转移函数STF，验证过程、finality过程和可用性都要是双方可信的。
- Transfer：从一个控制账号转移到另外一个账号的资产转移。这发生在同一类型的链，或者同样的整个资产所有权环境中，并且在相同的抽象级别内。比如同样用Substrate构建的链之间。

### 基础的消息格式

所有的数据都是SCALE编码，命名这些顶级消息XCM数据格式 `XcmPacket`。通常定义如下：

- `magic: [u8;2] = 0xff00`: 前缀符号。
- `version: Compact<u32>` : XCM版本号，目前支持0。
- `message: Xcm` : 消息体。

消息体格式：

- `type: u8`: 消息格式。
- `payload`: 消息参数。

消息格式有如下可选：

- 0: `WithdrawAsset`
- 1: `ReserveAssetTransfer`
- 2: `ReserveAssetCredit`
- 3: `TeleportAsset`
- 32: `RelayMessageParachain`
- 33: `ParachainRelayMessage`

在上面的消息格式里面，还有一个二级数据结构，是一些资产指令的包装， `Ai`, “Asset Instruction”,定义如下：

- `type: u8`: 指令类型。
- `payload` : 指令参数。

指令类型枚举在下面：

- 0: `DepositAsset` 
- 1: `ExchangeAsset`
- 2: `InitateReserveTransfer`
- 3: `InitiateTeleport`

#### 消息 ReserveAssetTransfer

这个消息指令就是指挥资产从一个有权限账号下转移到另外一个接收者账号。

这个消息指令下去后，一个 `ReserveAssetCredit`消息会被发送给目的地账号。

参数如下：

- `asset: MultiAsset` 被转移的资产。
- `destination: MultiDest` 资产转移目的地。
- `effect: Ai` 这个资产所携带的指令。

#### 消息 ReserveAssetCredit

这是一个传递到的通知消息，即消息发送源（保留资金账号）已将资金存入消息接收源拥有的主权账号。资产也将被铸造出来，并放入临时持有账号，并且如果有新的`effect`，则会被评估。

参数如下：

- `asset: MultiAsset` 被转移的资产。
- `effect:Ai` 资产所携带的指令。

#### TeleportAsset

一些可分割的资产从原来的消息发送源账号移除，并在接收者临时持有账号中铸造出来，如果消息资产携带新的指令，将会被评估。

参数如下：

- `asset: MultiAsset` 被借记的资产。
- `effect: Ai` 资产所携带的指令。

#### WithdrawAsset

原来消息发送源的主权账号，发送该消息到临时持有账号，要求清除相应资产，并且评估资产所携带的指令。

参数如下：

- `asset: MultiAsset` 被借记的资产。
- `effect: Ai` 资产所携带的指令。

#### RelayMessageParachain

指示性消息，中继链将消息中继到指定的目标链，其中实际消息呈现给目标链将会是`Parachain Relayed Message`格式，并且适当的保留消息发送源在里面。

参数如下：

- `destination: ParaId` 消息将要中继到的目标链。 
- `messages: Vec<Xcm>` 消息接收者将要解释的消息列表。


#### ParachainRelayMessage

平行链需要通过中继链给传递消息。

参数如下：

- `source: ParaId` 发出消息的平行链。
- `messages: Vec<Xcm>` 消息接收者将要解释的消息列表。

#### Balances

跨链查询资产余额。

参数如下：

- `query_id` 将要把这个余额查询请求发送的链。
- `assets` 将要查询的资产列表。

### AssetInstruction 指令类型

如果是多个指令，按照所给出致命的顺序依次执行。

- `instructions: Vec<Ai>` 将要被执行的指令

#### DepositAsset

从持有账号中将所有资产存进指定目标账户，不需要额外其他信息指令。

- `asset: MultiAsset` 将要被转移资产的标识符。
- `destination: MultiLocation` 转入地址的标识符。

#### ExchangeAsset

从持有账号中将指定资产，尝试去兑换相应目标资产。

- `give: MultiAsset` 从这个交易指令中借贷出去最大指定资产的标识符。
- `for: MultiAsset` 从这个交易指令中将可以被兑换最少目标资产的标识符。
 
#### InitiateReserveTransfer

从持有账号中销毁资产，发送一个 `ReserveAssetTransfer`消息指令到保留地址铸造出对应的资产。

- `asset: MultiAsset` 从持有账号转出的资产。
- `destination: MultiLocation` 资产转入的目标地址。
- `effect:Ai` 在目标地址，接下来需要执行的资产指令。

#### InitiateTeleport

从持有账号中销毁资产，发送一个`TeleportAsset`消息指令到保留地址，在另外一个共识系统中在该持有账户下铸造出相应资产。

- `asset: MultiAsset` 将要被转移到资产。
- `destination: MultiLocation` 将要把转移资产指令传达到的地址的标识符。
- `effect: Ai` 在目标地址，将要对转移资产接下来需要执行的资产指令。 

#### QueryHolding

给定查询哪些资产信息，查询目标地址，发送一个余额查询指令过去。

- `query_id: Compact<u64>` 查询指令的索引标识符。
- `destination: MultiLocation` 查询目标地址。
- `assets: Vec<MultiAsset>` 需要查询的资产。


### MultiAsset 通用资产标识符

基本格式：

- `version: u8` 版本/格式 编码，现在有两个编码是支持的 `0x00` 和 `0x01`。

**可分割资产**

- `version: u8 = 0x00` 目前的版本编号。
- `id: Vec<u8>` 不可分割资产标识符，通常表示成这样： `*b"BTC"` `*b"ETH"` `*b"DOT"` 。如果为空表示所有资产，在这种情况下，`amount`将会被无视掉，按照惯例直接默认为0。
- `amount: Compact<_>` 资产标识符的金额。


**不可分割资产**

- `version: u8 = 0x01` 目前的版本编号。
- `class: Vec<u8>` 通用不可分割资产类型代码。如果为空，表示所有资产，在这种情况下，`instance`表将会被无视掉，按照惯例默认被设置为`Undefined`。
- `instance: AssetInstance` 使用 NFA类型标准定义的通用不可分割资产实例，可以使用一个序号或者一个数据报来标识。

###### AssetInstance

给出的一些资产实例枚举如下：

- Undefined = 0: ()
- Index8 = 1: u8
- Index16 = 2: Compact<u16>
- Index32 = 3: Compact<u32>
- Index64 = 4: Compact<u64>
- Index128 = 5: Compact<u128>
- Array4 = 6: [u8; 4]
- Array8 = 7: [u8; 8]
- Array16 = 8: [u8; 16]
- Array32 = 9: [u8; 32]
- Blob = 10: Vec<u8>

### MultiLocation 通用地址标识符

目标标识符是自我描述的标识符，可以指定将某些加密资产放入其所有者。 它旨在在含义和性质上具有足够的抽象性，使其适用于各种链类型，包括UTXO，基于帐户和保护隐私。

基本格式：

- `version: u8 = 0x00` 版本编码，目前只有一个编码，那就是0.
- `type : u8` 目标地址的类型。
- `payload` 数据附载，格式取决于上面的地址类型。

**Type 0: Null**

指示在其下评估值的上下文本身就是目标。

用一个 `.` 来书写表示。

**Type 1: Parent**

相对于上下文，其上一个共识系统中的评估目标。

用一个 `..` 来书写表示。

**Type 2: ChildOf**

子账号或者下属账号类型。

用 `<primary>/<subordinate>` 来书写表示。

**Type 3: SiblingOf**

相当于一个主账号是同一个父账号，不同兄弟子账户。

用 `../<sibling>` 来书写表示。

**Type 7: QpaqueRemark**

不透明备注账号，不是真正的账号，而是一个伪地址。对于操作来说是没有意义的，但是会有语义方便人类理解操作。

**Type 8: AccountId32**

通用32字节多重加密账号，如果需要，其 `network` 可以被限制/指定。

- `network: MultiNetwork` 主要网络，链，合约的标识符。
- `id: [u8; 32]` 32字节的账号ID。

**Type 9: AccountId64**

索引对应到地址，那么索引就需要64位才能存储足够多的账号。

- `network: MultiNetwork` 主要网络，链，合约的标识符。
- `index: Compact<u64>` 64位的索引编号。

**Type 10: ParachainPrimaryAccount**

波卡的中继链会有模块收集平行链/平行线程的账号，每一个账号标识符都会有一个u32编码的`ParaId`。在平行链上有相应的主权账号被平行链控制。

- `network: MultiNetwork` 中继链的网络标识符。
- `index: Compact<u32>` 32位的 `ParaId`。

**Type 11: AccountKey20**

- `network: MultiNetwork` 主要网络，链，合约的标识符。
- `key: [u8; 20]` 帐户密钥，从SECP256k1/ECDSA公钥的Keccak-256散列末尾的20个字节派生而来，或者由该地址处的智能合约所拥有。

### MultiNetwork

基本格式：

- `version: u8` 版本/格式 编码。

**Version 0: Wildcard**

表示该标识符在所有共识系统中的所有语法合法位置中都可能相关。

- `version: u8 = 0u8`

**Version 1: Networks**

指示标识符仅与特定网络相关，如所述。

- `version: u8 = 1u8`
- `network_id: Vec<u8>`

合法的 `network_id`值包括：

- `b"dot"` 波卡主网。
- `b"ksm"` Kusama主网。
- `b"eth"` 以太坊主网， `AccountKey20` 对应到地址或者合约地址。
- `b"btc"` 比特币主网，`AccountKey20` 对应到标准的比特币地址。


## 案例

### 跨链兑换币

尝试在兑换链上用42DOT来兑换21BTC。

- `H` 自己的链。
- `X` 兑换链，`H`拥有临时账号。

代码：

	X.WithdrawAsset(
	    42 DOT,
	    Each(
	        ExchangeAsset(*, 21 BTC)
	        QueryHolding(../H)
	        DepositAsset(../H, *)
	    )
	)


### 跨链传送支付

两个链，互相信任彼此的STF（状态转移函数），使用传送功能将21 aUSD从自己链传送到目标链。

- `H` 自己的链。
- `D` 目标链。

代码：

	D.TeleportAsset(
   		21 aUSD,
    	DepositAsset(Bob, *)
	)

### 跨链担保支付

两个链，信任第三方链（担保链）的STF，使用担保支付方式发送交易。

- `H` 自己链。
- `D` 目标链。
- `R` 担保链。

代码：

	R.ReserveAssetTransfer(
	    21 DOT,
	    D,
	    DepositAsset(Bob, *)
	)

### 跨链通过不同担保链交易两个代币

Alice尝试交易42DOT换21BTC，但是自己链和交易链都互不信任对方的STF。

- `H` 自己链，平行链拥有42DOT。
- `X` 交易链。
- `R` 中继链，扮演DOT的担保链。
- `B` 比特币转接桥，扮演BTC的担保链。

代码：

	R.ReserveAssetTransfer(
	    42 DOT,
	    X,
	    Each(
	        ExchangeAsset(*, 21 BTC)
	        InitiateReserveTransfer(
	            * BTC,
	            ../H,
	            DepositAsset(Alice)
	        )
	        InitiateReserveTransfer(
	            * DOT,
	            H,
	            DepositAsset(Alice)
	        )
	    )
	)




