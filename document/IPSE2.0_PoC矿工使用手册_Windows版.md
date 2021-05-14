# IPSE2.0矿工使用手册

## 矿工角色操作流程图:
![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/矿工流程图.jpg)



## 一、	搭建本地节点
### 环境说明:
Ubuntu18.04或Ubuntu20.04系统
### 1.1 下载程序
[链程序下载地址](https://github.com/IPSE-TEAM/ipse-core/releases/download/2.1.2/IPSE)

节点服务器本地创建文件夹，下载链程序
```
➜  ~ mkdir -p ipse2.0/ipse2.0-node
➜  ~ cd ipse2.0/ipse2.0-node 
➜  ipse2.0-node ll
total 0
➜  ipse2.0-node wget hhttps://github.com/IPSE-TEAM/ipse-core/releases/download/2.1.2/IPSE
```



### 1.2 同步节点数据

通过运行以下指令来开始同步您的节点(写入日志文件及后台运行):
```
/IPSE --chain  staging --ws-port 9948 --base-path db --pruning=archive  --execution=NativeElseWasm --wasm-execution Compiled --name 节点名字自定义  > ipse.log 2>&1 &
```
如果您不想马上运行验证模式下。
--pruning=archive选项意味着--validator和-sentry选项，因此仅如果在没有这两个选项之一的情况下启动节点，则必须明确要求。 如果您不设置为 archive 节点，即使不在运行验证人和哨兵模式时，也需要切换时重新同步数据库。
根据当时链的大小，此步可能需要几分钟到几个小时不等。
如果您想估计还需要再多少时间，服务器日志(通过命令 tail –f ipse.log)显示了您的节点已处理和最新验证的区块。 然后您可以与 Telemetry 或当前 PolkadotJS 区块链浏览器比较。

### 1.3 启动本地节点

节点同步数据完成，关闭IPSE程序，重新启动本地节点，运行以下命令(写入日志文件及后台运行):
```
./IPSE --chain  staging  --ws-port 9948 --rpc-port 30339 --execution=NativeElseWasm  --unsafe-ws-external --unsafe-rpc-external  --rpc-cors=all --base-path db --rpc-methods=Unsafe  --pruning=archive --wasm-execution Compiled --name 节点名字自定义   > ipse.log 2>&1 &
```
通过tail –f ipse.log查看日志详情.
```
Mar 25 15:31:19.327  WARN It isn't safe to expose RPC publicly without a proxy server that filters available set of RPC methods.
Mar 25 15:31:19.328  WARN It isn't safe to expose RPC publicly without a proxy server that filters available set of RPC methods.
Mar 25 15:31:19.328  INFO Substrate Node
Mar 25 15:31:19.328  INFO ✌️  version 2.0.0-f5775ed-x86_64-linux-gnu
Mar 25 15:31:19.328  INFO ❤️  by Parity Technologies <admin@parity.io>, 2017-2021
Mar 25 15:31:19.328  INFO 📋 Chain specification: IpseLocalTestnet
Mar 25 15:31:19.328  INFO 🏷 Node name: Alice
Mar 25 15:31:19.328  INFO 👤 Role: AUTHORITY
Mar 25 15:31:19.328  INFO 💾 Database: RocksDb at db/chains/ipse_local_testnet/db
Mar 25 15:31:19.328  INFO ⛓  Native runtime: node-267 (substrate-node-1.tx1.au10)
Mar 25 15:31:19.401  INFO 🏷 Local node identity is: 12D3KooWMeuwhySA5nLYwWHRGyZxiSSxehfxthZTG5jMXU7ecaE4
Mar 25 15:31:19.692  INFO staking_poc----当前打印的高度是:69580
Mar 25 15:31:19.692  INFO poc_staking era start_time: 69449, chill end_time: 69499
Mar 25 15:31:19.719  INFO 📦 Highest known block at #69579
Mar 25 15:31:19.720  INFO 〽️ Prometheus server started at 127.0.0.1:9615
Mar 25 15:31:19.738  INFO Listening for new connections on 0.0.0.0:9948.
Mar 25 15:31:19.739  INFO 👶 Starting BABE Authorship worker
Mar 25 15:31:19.991  INFO Accepted a new tcp connection from 119.136.126.101:25472.
Mar 25 15:31:20.373  INFO 🔍 Discovered new external address for our node: /ip4/47.98.139.83/tcp/30333/p2p/12D3KooWMeuwhySA5nLYwWHRGyZxiSSxehfxthZTG5jMXU7ecaE4
Mar 25 15:31:20.447  INFO Accepted a new tcp connection from 119.136.126.101:25490.
Mar 25 15:31:21.034  INFO staking_poc----当前打印的高度是:69580
Mar 25 15:31:21.034  INFO poc_staking era start_time: 69449, chill end_time: 69499
Mar 25 15:31:21.077  INFO execute_block: staking_poc----当前打印的高度是:69580    {block}
Mar 25 15:31:21.079  INFO execute_block: poc_staking era start_time: 69449, chill end_time: 69499    {block}
Mar 25 15:31:21.081  INFO execute_block:apply_extrinsic: 矿工: 16c3ab6a5c4213de6a396cb1f899dbcadcc76f6865aae1c2b10e08339990de0a (5CaZ35VM...),  提交挖矿!, height = 69580, deadline = 494    {ext}
```

之后挖矿程序可以直接连ws://localhost:9948进行挖矿。

### 1.4 关闭本地节点
查看IPSE进程号，并杀掉进程，命令如下:
```
➜  ~ ps -ef |grep IPSE                                              
root     1795222       1  2 Mar24 ?        00:47:46 ./IPSE --chain   staging --execution=NativeElseWasm  --unsafe-ws-external --unsafe-rpc-external  --rpc-cors=all --ws-port 9948 --rpc-port 30339 --base-path db --rpc-methods=Unsafe  --pool-limit 100000 --ws-max-connections 50000
root     1833766 1833711  0 15:26 pts/0    00:00:00 grep --color=auto --exclude-dir=.bzr --exclude-dir=CVS --exclude-dir=.git --exclude-dir=.hg --exclude-dir=.svn --exclude-dir=.idea --exclude-dir=.tox IPSE
➜  ~ 
➜  ~ kill -9 1795222
```
如果想成为验证人节点，则参考以下文档:

[Polkadot网络上设置验证人节点](https://wiki.polkadot.network/docs/zh-CN/maintain-guides-how-to-validate-polkadot)

## 二、矿工app端操作
[App下载链接](https://www.ipse.io/app/ipse.apk )

### 2.1 矿工注册
(1)打开ipse手机客户端，创建/导入账户，账户需要有足够的IPSE余额，支付相关交易手续费。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/矿工注册.png)

(2)进入“质押”-“矿工注册”页面，输入P盘空间、P盘id、佣金比例，进行矿工注册（默认收益地址是矿工自己的地址）。注册成功后跳转至质押界面；完成这一步，矿工就可以启动挖矿软件进行挖矿了。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/矿工注册2.png)         

### 2.2 矿工修改信息
冷却期:只有矿工能修改信息
非冷却期：抵押者可以进行质押及退出质押操作
(1) 进入“质押”-“矿工管理”页面，分别选择P盘空间、P盘id、佣金比例，进行修改，修改成功后信息随之更新。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/矿工修改注册信息.png)          
### 2.3 矿工推荐列表(抵押排名)

#### 2.3.1 申请加入推荐列表

(1) 进入“质押”-“矿工管理”页面，选择“抵押排名”,输入amount，提交申请加入推荐列表

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/矿工申请进入推荐列表.png)
         
(2)进入“质押”-“参与质押”页面，选择矿工列表，可以查看正在推荐列表的矿工信息，点击地址右边可查看到该矿工的挖矿记录；质押者可选择抵押排名中指定的矿工进行质押，获得挖矿分佣奖励。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/参与质押-矿工列表.png)           



#### 2.3.2 退出推荐列表 

进入“质押”-“矿工管理”页面，点击“退出抵押排名”进行退出操作，提交后退出推荐列表成功，并锁定抵押排名金额进入锁定期；不影响抵押者已进行质押的质押金额及出块奖励分佣。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/矿工退出抵押排名.png)           



### 2.4 矿工删除抵押者

(1)进入“质押”-“矿工管理”页面，选择质押者列表的某个质押者进行删除，矿工删除质押者成功；自动返还质押者的质押金额—该质押者金额进入锁定期；扣除保留金额 1 ipse，作为惩罚。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/矿工删除质押者.png)  
          


### 2.5 矿工停止挖矿

(1)进入“质押”-“矿工管理”页面，点击“停止挖矿”进行操作，停止挖矿成功，矿工可查看之前的挖矿记录；
矿工需手动去退出抵押排名列表，操作会锁定抵押排名金额进入锁定期；
质押者需手动去减少质押或退出质押，操作会锁定质押金额进入锁定期。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/矿工停止挖矿.png) 
             
### 2.6 矿工重新启动挖矿
矿工挖矿状态为停止状态，需要启动挖矿，可以重新挖矿。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/矿工重启挖矿.png)  

 
## 三、矿工P盘
### 环境说明:
Windows7/10系统
### 3.1 下载windows版P盘工具并解压
下载适当版本的Engraver并解压缩存档后，目标文件夹将包含两个文件：Engraver可执行文件和Engraver图形用户界面（EngraverGui）

[window版P盘工具下载地址](https://github.com/PoC-Consortium/engraver/releases/download/2.4.0/engraver-2.4.0-x86_64-pc-windows-msvc.zip.zip)

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/plot_win_img/plot_dir.png) 

### 3.2 命令行界面说明

Windows系统当前目录打开CMD命令行界面方法:
```
Windows7:按住键盘shift键，点击鼠标右键，点击“在此处打开命令窗口（W）”，进入CMD命令行界面
Windows10:按住键盘shift键，点击鼠标右键，选择"在此处打开powershell窗口"，执行 start cmd,进入CMD命令行界面 
```
![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/plot_win_img/engraver_help_page.png) 

#### 3.2.1 P盘参数说明

P盘参数说明:
```
engraver_gpu.exe [FLAGS] [OPTIONS] --n <nonces> --id <numeric_ID> --sn <start_nonce> --path <path>

-----------------------------------------------------------------------------
--n <nonces>  P盘大小对应的nonce
--id <numeric_ID>  P盘id
--path <path>  指定P盘文件存放目录(如/data/data2000001234000100)，不指定则默认存放在当前目录
--sn <start_nonce>  起始随机数（计算：已使用随机数。其它P盘大小随机数之和）

-----------------------------------------------------------------------------
nonce计算：
1GiB=（1024*1024*1024）B=1073741824B
1GB=（1000*1000*1000）B=1000000000B
1GiB/1GB=1073741824/1000000000=1.073741824
计算:
1 nonce=256KiB，1MiB= 4nonce，则
1GiB= 1*1024*4=4096 nonce，
1TiB=1*1024*1024*4=4194304 nonce, 
2TiB=2*1024*1024*4=8388608 nonce，以此类推。
```
用户应注意，重叠的图会减小图的有效大小，因此应谨慎提供这些参数。
计算起始随机数和绘图随机数的策略可以是：
```
对于第一个绘图文件（0）：

开始随机数（0）= 0

随机数（0）=用于MiB的绘图文件（0）的磁盘空间乘以4

对于下一个绘图文件（i）

起始随机数（i）=起始随机数（i-1）+随机数（i-1）

随机数（i）的数量=用于文件（i）的磁盘空间，以4 x MiB乘以4

示例：创建前两个10Gib图文件：

第一个文件：

开始随机数（0）= 0

随机数（0）= 40960（10GiB = 10240 MiB = 40960随机数）

结果文件名：numeric_ID_0_40960


第二个文件：

开始随机数（1）= 0 + 40960 = 40960

随机数（1）的数量= 40960（10GiB = 10240 MiB = 40960随机数）

结果文件名：numeric_ID_40960_40960

-n，-n：要绘制的随机数（强制）

如果将此选项设置为零，则绘图文件将具有可容纳在驱动器上的尽可能多的随机数。
```
#### 3.2.2 命令行界面执行P盘

通过执行以下命令进行P盘(以1GiB为例):
```
F:\plot>engraver_gpu.exe --n 4096 --id 10008312345600028  --path F:\plot\data  --sn 0
```
提示指定的P盘文件存放路径不存在，那么我们先新建该文件路径
```
F:\plot>mkdir F:\plot\data
```
再次执行P盘操作
```
F:\plot>engraver_gpu.exe --n 4096 --id 10008312345600028  --path F:\plot\data  --sn 0
```
![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/plot_win_img/cmd_start_plotting.png) 


### 3.3 图形用户界面说明
下载适当版本的Engraver并解压缩存档后，目标文件夹将包含两个文件：Engraver可执行文件和Engraver图形用户界面（EngraverGui）。通过双击EngraverGui可执行文件来启动图形用户界面。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/plot_win_img/plot_dir.png) 

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/plot_win_img/GUI_homepage.png) 

#### 3.3.1 基本设置(basic Settings)

基本设置包括最低设置和开始绘图所需提供的信息。
```
Numeric ID：将用于创建绘图文件的数字帐户ID
Output Folder：系统上将要存储结果文件的文件夹。使用“浏览”按钮导航到所需位置
Drive Info：指目标驱动器的信息。用户选择输出文件夹后，将填充驱动器信息
```
![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/plot_win_img/basic_Settings.png) 

驱动器信息包括目标驱动器上的可用空间，可存储到可用空间中的最大随机数以及逻辑扇区大小。
```
Start Nonce(起始随机数)：定义要绘制的起始随机数。如果目标驱动器上已经存储了绘图文件，建议使用“自动从上一个文件开始”按钮，该按钮将确定正确的随机数以开始绘图，以防止绘图文件重叠
Size to plot(P盘大小)：
Maxinum(最大)：如果选择此选项，将创建打印文件以填充目标驱动器上的所有可用空间
Value(值)：如果用户选择此选项，则他们可以输入所需的打印文件大小在随机数，MiB，TiB或GiB中。根据所选择的单位，雕刻机将以随机数或存储大小单位显示相应的值。
```

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/plot_win_img/basic_Settings2.png) 


 
请注意，Engraver会根据逻辑扇区的大小舍入P盘文件的大小。这样做是为了启用快速直接I / O功能。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/plot_win_img/basic_Settings3.png) 
 
#### 3.3.2 高级设置(Advanced Settings)

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/plot_win_img/Advanced_Settings.png) 

高级设置选项卡包括以下选项：
 
```
XPU：选择要进行哈希处理的设备，并可以选择设置“线程限制”，这将允许设置所需的线程数以进行P盘。如果将该设置保留为0，将使用最大可用线程数。
Low priority(低优先级)：如果选中，将以低优先级执行绘图过程。
RAM：此设置允许配置将用于绘图的最大系统RAM内存。要限制用于绘图的RAM，请检查“内存限制”以允许输入所需的RAM内存量以用于绘图。
I/O：
--Direct I/O:直接I / O使用绕过OS缓存层。默认设置被选中。如果未选中此选项，则绘图过程将相当慢。
--Async I/O:如果选中了异步I / O，则会禁用异步写入（单缓冲区模式）。在默认设置异步写入模式下，缓冲区减半-一半用于写入，另一半用于同时生成Shabal256哈希。在单缓冲区模式下，完整缓冲区用于散列，然后刷新到磁盘，而无需并行化。
```

#### 3.3.3 执行P盘操作

设置所有所需参数后，单击“Start Plotting”按钮，开始创建绘图文件(P盘文件)。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/plot_win_img/GUI_start-plotting.png) 
 
单击“Stop Plotting”按钮可以停止绘图。确认后，绘图将停止。

Engraver支持绘图恢复。要恢复绘图文件(P盘文件)，用户将转到“file”菜单，然后选择“Resume file”选项，导航到不完整的绘图文件，选择它，然后单击“Start Plotting”按钮恢复绘图。

完成P盘后或进行P盘的过程中，接着下一步操作。

## 四、矿工启动挖矿程序

### 4.1 下载挖矿程序
[poc-mining(Windows版本)下载链接](https://github.com/IPSE-TEAM/ipse2.0-mining/releases)

下载挖矿软件poc-mining最新版本(Windows版本)，压缩包包含三个文件：挖矿配置文件config.yaml、监控工具supervision、挖矿软件poc-mining，解压到对应目录:


### 4.2 修改配置文件
##### 提示：配置文件config.yaml是默认配置，您应该使用自己的配置。
打开config.yaml文件，对应修改文件中account_id、plot_size、miner_proportion 、account_id_to_secret_phrase、url 、plot_dirs、url、max_deadline_value、miner_reward_dest 的值，其他值暂时不需要修改及关注，保存退出。
```


# ********************* 您可能需要修改以下这几个参数 ***********************#

account_id: &numeric_id 20210511000010    # 矿工p盘id
plot_size: 10                                  # p盘空间大小 单位是GiB
miner_proportion: 20                            # 矿工分润占比 （0 ～ 100）
account_id_to_secret_phrase:                    # 矿工的助记词（用于注册)
  *numeric_id: 'amount verify giant dumb wing acquire region cube cable lottery travel enroll'   #5F9sD6s7Be8GJKLHSx5S8oNRMakwUJNfFiEHfGkMgpU9thhP
plot_dirs:                                      # p盘数据存储路径
   - 'E:\IPSE\plot'
url: 'ws://localhost:9948'                 # IPSE节点地址
max_deadline_value: 10000                         # 允许的deadline最大提交值
miner_reward_dest: '5GWX6izv5A7Ja2ik7jctcXDZ8MZFF8BePSKEgs37J5DMbXha'    # 收益地址(一定要指定)

# ********************* 您可能需要修改以上这几个参数 ***********************#

```
如果要进行多矿工挖矿，则在当前目录或其他目录新建一个文件夹，拷贝挖矿配置文件config.yaml、监控工具supervision、挖矿软件poc-mining至该文件夹下,进入该文件夹。

(1)重名名poc-mining， 比如改成xxxx

(2)根据上面的第4.2章节进行修改配置文件，每个矿工的助记词及P盘id不能重复


### 4.3 启动挖矿
启动前给矿工地址转足够的IPSE代币，因为启动后自动进行矿工注册操作;

==启动挖矿有如下两种方式：==

#### 4.3.1 supervision启动挖矿(异常自动重启)
进入挖矿目录,启动挖矿(按住键盘shift键，点击鼠标右键，选择"在此处打开powershell窗口")，执行如下命令:
```
E:\IPSE\plot\win10> .\supervision.exe --mining .\poc-mining.exe
```
生成以poc-mining.exe.log的日志文件，打开即可看到最新日志。

#### 4.3.2 不使用supervision,直接启动挖矿(异常不会自动重启)

进入挖矿目录,启动挖矿(按住键盘shift键，点击鼠标右键，选择"在此处打开powershell窗口")，执行如下命令:
```
E:\IPSE\plot\win10> .\poc-mining.exe
```

#### 4.4 停止挖矿
cmd窗口直接ctrl + C，或者关闭cmd窗口。

如果P盘文件增大空间或者P盘id已更换，则对应修改配置文件再重启挖矿,且需在链上更新对应矿工的account_id和plot_size！

