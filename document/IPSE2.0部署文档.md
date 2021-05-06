# 说明
* 这个文档适合普通用户使用， 但是不适合创世节点或是种子节点的部署
* 如果作为验证节点，可以参与IPSE挖矿检查，获得丰厚回报，但是应该保持controller账号上有余额， 否则不能参与
* 作为节点，当主网链上升级，应该及时进行本地native版本的更新
# 一、准备工作
##1. 环境准备
* 安装ipse-core环境依赖
  `curl https://getsubstrate.io -sSf | bash -s -- --fast`

* 安装rust
  `curl https://sh.rustup.rs -sSf | sh`

* 安装wasm
  `rustup update nightly`
   `rustup target add wasm32-unknown-unknown --toolchain nightly`
***
## 2.编译IPSE
* 克隆[IPSE官方代码库](https://github.com/IPSE-TEAM/ipse-core.git)
  * 未加速（方法一）
    `git clone https://github.com/IPSE-TEAM/ipse-core.git`
  ***
  * 加速（方法二）
    * 安装mclone
      ` sudo bash -c "$(curl -fsSL https://gitee.com/liesauer/mclone/raw/v1.5.0/script/install.sh)"`
      `git mclone https://github.com/IPSE-TEAM/ipse-core.git`
  ***
  >>> 国内qiang的原因， 方法一可能会导致你花上半天时间，这时候可以使用优秀项目[mclone]([https://github.com/nulastudio/mclone](https://github.com/nulastudio/mclone)
)来协助克隆。整个过程会缩短到几十秒。
  ***
* 编译
  进入刚才克隆下来的代码仓库主目录
  执行 命令：`cargo build --release`
>>> 通常情况下，这个步骤将花费很长时间，并且可能有些库没法下载，可以换rust源试试。
  ***
编译成功之后， `./target/release`文件夹中有可执行二进制文件`ipse-core`。编译成功
***
## 3. 检查安装`subkey`工具
查看`./target/release`文件夹下是否有二进制文件`subkey`

如果没有， 执行命令：`cargo install --force --path subkey subkey`
再次查看`./target/release/subkey`
***
## 4.生成raw文件
`./target/release/ipse-core build-spec --chain=staging > localspec.json`
`./target/release/ipse-core build-spec --chain localspec.json --raw > customspec.json`
>>> raw文件用于启动节点，是根据`chain_spec.rs`文件生成的, 不能更改项目chain_spec.rs中的代码，要不然会导致生成的raw文件hash值不一致，从而启动节点失败.
***
# 二、部署节点
## 1. 开放端口
`ufw enable`
`ufw allow 9933`
 `ufw allow 9944`
 `ufw allow 30333`
***
## 2. 选择节点类型并启动
### 作为数据同步节点
>>> 说明：数据同步节点仅仅用于与链上进行数据交互，并不参与数据验证，可以向外提供wss连接
* 用到的参数： --ws-port、--rpc-port、--port、--name(节点名字，节点启动后，在[监控服务]([https://telemetry.polkadot.io/#/Polkadot%20CC1](https://links.jianshu.com/go?to=https%3A%2F%2Ftelemetry.polkadot.io%2F%23%2FPolkadot%2520CC1)
)找得到，说明已经成功启动节点)、--rpc-external、 --ws-external、--rpc-cors(允许所有外部连接)、--ws-max-connections、--pool-limit、--pruning(保留区块数据深度)
* 执行命令：
  `./target/release/ipse-core  --chain customspec.json --rpc--port 9933  --port 30333  --base-path ./db --rpc-external  --ws-external --rpc-cors=all  --ws-max-connections 2048  --pool-limit 10000  --pruning  archive
 --name 你的节点名称`
* 查看控制台打印，有无连接信息或是错误信息
***
## 作为验证人节点
>>> 验证人节点参与出块，相对于同步节点，要考虑的安全问题是不同的，所以部署的时候复杂性也相对要大得多。验证节点不对外公开wss连接，所以下面的步骤是部署时候打开，部署成功后关闭
* 设置wss服务，详细步骤参考[wss设置](https://www.jianshu.com/p/705a88d3c29d)
***
* 启动节点：
`./target/release/ipse-core --chain customspec.json --pruning=archive  --base-path ./db  --rpc-port 9933  --port  30333  --ws-port 9944 --unsafe-ws-external --unsafe-rpc-external --rpc-cors=all --rpc-methods=Unsafe  --validator  --execution=NativeElseWasm`
>>> 注意查看是否启动成功
***
* 生成controller与stash账号
`./target/release/subkey generate`
`./target/release/subkey generate`
>>> 随意指定身份，但是要记住谁是controller,谁是stash
***
* 进入[polkadot的ui界面](https://polkadot.js.org/apps/), 连接自己刚才启动的节点(wss://主机地址: 9944)
>>> 如果第一次使用，这里通常会报安全问题。在浏览器作允许连接的设置就可以了
***
* 在前端给controller与stash账号转账一定金额（尽量不要太小)
***
* `质押  -> 账户操作 -> 存储账户`, 对controller与stash进行绑定操作
***
* `质押  -> 账户操作 ->验证人`, controller账号声明自己作为验证节点
***
* `工具箱  -> author -> rotatekeys`, 生成session_keys, 复制
***
* `交易 -> session -> setKeys`, 把上一步复制的session_keys作为keys参数值， proof参数任意，提交交易
***
* 重新启动节点：
`./target/release/ipse-core --chain customspec.json --pruning=archive  --base-path ./db  --rpc-port 9933  --port  30333  --ws-port 9944 --validator --pool-limit 10000  --name 你的节点名称  --execution=NativeElseWasm`

