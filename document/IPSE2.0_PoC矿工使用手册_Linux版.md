# IPSE2.0矿工使用手册

## 矿工角色操作流程图:
![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.3.0/document/ipse_img/PoC_Staking/矿工流程图.jpg)

## 环境说明:
Ubuntu18.04或Ubuntu20.04系统
## 一、	搭建本地节点

### 1.1 下载程序
[链程序下载地址](https://github.com/IPSE-TEAM/ipse-core/releases/download/2.3.0/IPSE)

节点服务器本地创建文件夹，下载链程序
```
➜  ~ mkdir -p ipse2.0/ipse2.0-node
➜  ~ cd ipse2.0/ipse2.0-node 
➜  ipse2.0-node ll
total 0
➜  ipse2.0-node wget https://github.com/IPSE-TEAM/ipse-core/releases/download/2.3.0/IPSE
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

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.3.0/document/ipse_img/PoC_Staking/矿工注册.png)

(2)进入“质押”-“矿工注册”页面，输入P盘空间、P盘id、佣金比例，进行矿工注册（默认收益地址是矿工自己的地址）。注册成功后跳转至质押界面；完成这一步，矿工就可以启动挖矿软件进行挖矿了。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.3.0/document/ipse_img/PoC_Staking/矿工注册2.png)         

### 2.2 矿工修改信息
冷却期:只有矿工能修改信息
非冷却期：抵押者可以进行质押及退出质押操作
(1) 进入“质押”-“矿工管理”页面，分别选择P盘空间、P盘id、佣金比例，进行修改，修改成功后信息随之更新。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.3.0/document/ipse_img/PoC_Staking/矿工修改注册信息.png)          
### 2.3 矿工推荐列表(抵押排名)

#### 2.3.1 申请加入推荐列表

(1) 进入“质押”-“矿工管理”页面，选择“抵押排名”,输入amount，提交申请加入推荐列表

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.3.0/document/ipse_img/PoC_Staking/矿工申请进入推荐列表.png)
         
(2)进入“质押”-“参与质押”页面，选择矿工列表，可以查看正在推荐列表的矿工信息，点击地址右边可查看到该矿工的挖矿记录；质押者可选择抵押排名中指定的矿工进行质押，获得挖矿分佣奖励。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.3.0/document/ipse_img/PoC_Staking/参与质押-矿工列表.png)           



#### 2.3.2 退出推荐列表 

进入“质押”-“矿工管理”页面，点击“退出抵押排名”进行退出操作，提交后退出推荐列表成功，并锁定抵押排名金额进入锁定期；不影响抵押者已进行质押的质押金额及出块奖励分佣。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.3.0/document/ipse_img/PoC_Staking/矿工退出抵押排名.png)           



### 2.4 矿工删除抵押者

(1)进入“质押”-“矿工管理”页面，选择质押者列表的某个质押者进行删除，矿工删除质押者成功；自动返还质押者的质押金额—该质押者金额进入锁定期；扣除保留金额 1 ipse，作为惩罚。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.3.0/document/ipse_img/PoC_Staking/矿工删除质押者.png)  
          


### 2.5 矿工停止挖矿

(1)进入“质押”-“矿工管理”页面，点击“停止挖矿”进行操作，停止挖矿成功，矿工可查看之前的挖矿记录；
矿工需手动去退出抵押排名列表，操作会锁定抵押排名金额进入锁定期；
质押者需手动去减少质押或退出质押，操作会锁定质押金额进入锁定期。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.3.0/document/ipse_img/PoC_Staking/矿工停止挖矿.png) 
             
### 2.6 矿工重新启动挖矿
矿工挖矿状态为停止状态，需要启动挖矿，可以重新挖矿。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.3.0/document/ipse_img/PoC_Staking/矿工重启挖矿.png)  

 
   
## 三、矿工P盘
linux系统(如ubuntu18.04或ubuntu20.04)进行P盘及挖矿！

### 3.1 下载P盘工具并解压
输入以下命令进行下载P盘工具：
```
abc@abc:~/ipse2.0/ipse2.0-mining$ sudo wget https://github.com/PoC-Consortium/engraver/releases/download/2.4.0/engraver-2.4.0-x86_64-unknown-linux-gnu-cpu-gpu.tar.xz
```
下载过程中界面日志:
```
--2021-05-07 15:05:38--  https://github.com/PoC-Consortium/engraver/releases/download/2.4.0/engraver-2.4.0-x86_64-unknown-linux-gnu-cpu-gpu.tar.xz
Resolving github.com (github.com)... 13.250.177.223
Connecting to github.com (github.com)|13.250.177.223|:443... connected.
HTTP request sent, awaiting response... 302 Found
Location: 
.....
Resolving github-releases.githubusercontent.com (github-releases.githubusercontent.com)... 185.199.108.154, 185.199.110.154, 185.199.109.154, ...
Connecting to github-releases.githubusercontent.com (github-releases.githubusercontent.com)|185.199.108.154|:443... connected.
HTTP request sent, awaiting response... 200 OK
Length: 649208 (634K) [application/octet-stream]
Saving to: ‘engraver-2.4.0-x86_64-unknown-linux-gnu-cpu-gpu.tar.xz’

engraver-2.4.0-x86_64-unknown-linux-gnu 100%[============================================================================>] 633.99K   823KB/s    in 0.8s    

2021-05-07 15:05:41 (823 KB/s) - ‘engraver-2.4.0-x86_64-unknown-linux-gnu-cpu-gpu.tar.xz’ saved [649208/649208]
```
解压P盘工具软件:
```
abc@abc:~/ipse2.0/ipse2.0-mining$ sudo tar -xvf engraver-2.4.0-x86_64-unknown-linux-gnu-cpu-gpu.tar.xz 
engraver_cpu
engraver_gpu

```

### 3.2 P盘参数说明

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

### 3.3 执行P盘命令
通过执行以下命令进行P盘
```
abc@abc:~/ipse2.0/ipse2.0-mining$ sudo ./engraver_gpu --n 409600 --id 10064825431032897010 --path /data/data10064825431032897010 --sn 0  &
```
提示指定的P盘文件存放路径不存在，那么我们先新建该文件路径
```
abc@abc:~/ipse2.0/ipse2.0-mining$ sudo mkdir -p /data/data10064825431032897010                                                                          
abc@abc:~/ipse2.0/ipse2.0-mining$ ll /data/data10064825431032897010 
总用量 0

```

再次执行P盘操作
```
abc@abc:~/ipse2.0/ipse2.0-mining$ sudo ./engraver_gpu --n 409600 --id 10064825431032897010 --path /data/data10064825431032897010 --sn 0  &
```
 

完成P盘后或进行P盘的过程中，接着下一步操作。

## 四、矿工启动挖矿程序

### 4.1 下载挖矿相关配置文件
下载最新版本的挖矿软件poc-mining及挖矿配置文件config.yaml、miners_config.yaml文件、supervision、update_config，运行以下命令进行下载:
```
abc@abc:~/ipse2.0/ipse2.0-mining$ sudo wget -nc https://github.com/IPSE-TEAM/ipse2.0-mining/releases/download/v3.4.0/update_config && sudo ./update_config
```
并赋予可执行权限:
```
sudo chmod +x update_config
```

完成上述步骤后，您可以miners_config.yaml在当前文件夹中找到。请接下来进行修改。（提示：以下是默认配置，您应该使用自己的配置。）

```
miners: # 矿工的统一配置
   - {host: localhost, # remote server id
     account_id: 10717349404514113857, # plot id
     phase: cash mixture tongue cry roof glare monkey island unfair brown spirit inflict, # your secret key
     miner_proportion: 20,  # how much proportion that the miner should get in a rewad.
     url: 'ws://localhost:9944',  # synchronization node 
     plot_size: 50, # plot size (The unit is Gib).
     miner_reward_dest: 5FHb1AEeNui5ANvyT368dECmNEJeouLeeZ6a9z8GTvxPLaVs, # Miner income address
     plot_path: '/data/test_data',  # where is the plot file on your computer.
     max_deadline_value: 5000  # The maximum number of the deadline allowed.
   }
--------------------------------------------------------
host #当前目录生成矿工文件夹名称 
account_id #矿工的P盘ID
phase  #矿工的助记词
miner_proportion #矿工挖矿获得的奖励占比
url  #本地节点或远程节点的地址(如输入本地节点 “ws://localhost:9948”)
plot_size #矿工的P盘容量(GiB)
miner_reward_dest #矿工挖矿奖励的存放账户地址
plot_path #矿工P盘的路径(注: 如上面P盘文件存放路径为/data/data10064825431032897010,则这里写/data/data10064825431032897010)
max_deadline_value #允许提交的最大deadline值
--------------------------------------------------------
```
#### 4.1.1 修改配置文件
打开miners_config.yaml文件，对应修改文件中host 、account_id 、phase、 miner_proportion 、url 、plot_size 、miner_reward_dest、plot_path、max_deadline_value的值，并保存退出。
```
miners: # 矿工的统一配置
   #224
   - {host: localhost,
      account_id: 10064825431032897010,
      phase: defense ball area outside castle divert fortune crazy gather camp response yard,
      miner_proportion: 20,
      url: "ws://localhost:9948",
      plot_size: 3700,
      miner_reward_dest: 5GWX6izv5A7Ja2ik7jctcXDZ8MZFF8BePSKEgs37J5DMbXha,
      plot_path: /data/data10064825431032897010,
      max_deadline_value: 10000
   }
```
如果需要生成多个矿工，则按照格式添加信息就可以了，如下：
```
miners: # 矿工的统一配置
   #224
   - {host: localhost,
      account_id: 10064825431032897010,
      phase: defense ball area outside castle divert fortune crazy gather camp response yard,
      miner_proportion: 20,
      url: "ws://localhost:9948",
      plot_size: 3700,
      miner_reward_dest: 5GWX6izv5A7Ja2ik7jctcXDZ8MZFF8BePSKEgs37J5DMbXha,
      plot_path: /data/data10064825431032897010,
      max_deadline_value: 10000
   }
   - {host: localhost,
      account_id: 16045882063755536351,
      phase: increase cushion season lunar advice history urge ice color gas sport region,
      miner_proportion: 20,
      url: "ws://localhost:9948",
      plot_size: 3700,
      miner_reward_dest: 5GWX6izv5A7Ja2ik7jctcXDZ8MZFF8BePSKEgs37J5DMbXha,
      plot_path: /data/data16045882063755536351,
      max_deadline_value: 10000
   }
```
#### 4.1.2 生成挖矿目录

执行python脚本，生成挖矿程序及挖矿配置文件，如下:
```
abc@abc:~/ipse2.0/ipse2.0-mining$ sudo ./update_config
```
```
File ‘config.yaml’ already there; not retrieving.

File ‘supervision’ already there; not retrieving.

File ‘miners_config.yaml’ already there; not retrieving.

File ‘poc-mining’ already there; not retrieving.

0
update_config:7: YAMLLoadWarning: calling yaml.load() without Loader=... is deprecated, as the default Loader is unsafe. Please read https://msg.pyyaml.org/load for full details.
  x = yaml.load(result)
update_config:72: YAMLLoadWarning: calling yaml.load() without Loader=... is deprecated, as the default Loader is unsafe. Please read https://msg.pyyaml.org/load for full details.
  x = yaml.load(result)
<class 'dict'>
{'host': 'localhost', 'account_id': 10064825431032897010, 'phase': 'cash mixture tongue cry roof glare monkey island unfair brown spirit inflict', 'miner_proportion': 20, 'url': 'ws://localhost:9948', 'plot_size': 50, 'miner_reward_dest': '5FHb1AEeNui5ANvyT368dECmNEJeouLeeZ6a9z8GTvxPLaVs', 'plot_path': '/data/data10064825431032897010', 'max_deadline_value': 10000}
localhost/
localhost/10064825431032897010/
localhost/10064825431032897010/supervision-10064825431032897010
localhost/10064825431032897010/poc-mining-10064825431032897010
localhost/10064825431032897010/config.yaml

```
在该文件夹localhost中，您可以找到另一个以P盘ID命名的文件夹，然后进入该文件夹，如下:
```
abc@abc:~/ipse2.0/ipse2.0-mining$cd localhost/10064825431032897010  
abc@abc:~/ipse2.0/ipse2.0-mining$ls -l
```
```
总用量 15M
-rw-r--r-- 1 root root  470 3月  25 16:56 command.txt
-rw-r--r-- 1 root root 1.2K 3月  25 16:56 config.yaml
-rwxrwxrwx 1 root root  15M 3月  25 16:56 poc-mining-10064825431032897010
-rw-r--r-- 1 root root 4.0K 3月  25 16:56 supervision-10064825431032897010
```
command.txt文件里含启动挖矿/停止挖矿命令，如图:
```
cat command.txt 
```
内容如下:
```
/home/abc/ipse2.0/ipse2.0-mining/localhost/10064825431032897010/supervision-10064825431032897010 --mining /home/abc/ipse2.0/ipse2.0-mining/localhost/10064825431032897010/poc-mining-10064825431032897010 --log-max-size 10 
/home/abc/ipse2.0/ipse2.0-mining/localhost/10064825431032897010/supervision-10064825431032897010 --mining /home/abc/ipse2.0/ipse2.0-mining/localhost/10064825431032897010/poc-mining-10064825431032897010 --log-max-size 10 --stop
```

#### 4.1.3 启动挖矿
启动前给矿工地址转足够的IPSE代币，因为启动后自动进行矿工注册操作;

==启动挖矿有如下两种方式：==

##### 4.1.3.1 supervision启动挖矿(异常自动重启)
拷贝command.txt中的启动命令进行挖矿程序（末尾加 & 为了后台运行），如下:
```
sudo nohup /home/abc/ipse2.0/ipse2.0-mining/localhost/10064825431032897010/supervision-10064825431032897010 --mining /home/abc/ipse2.0/ipse2.0-mining/localhost/10064825431032897010/poc-mining-10064825431032897010 --log-max-size 10 &
```
查看动态日志：
```
tail -f /home/abc/ipse2.0/ipse2.0-mining/localhost/10064825431032897010/poc-mining-10064825431032897010.log 
```
##### 4.1.3.2 不使用supervision,直接启动挖矿(异常不会自动重启)

进入挖矿目录,启动挖矿（末尾加 & 为了后台运行），如下:
```
cd /home/abc/ipse2.0/ipse2.0-mining/localhost/202100123456003000
```
```
sudo./poc-mining-10064825431032897010   & 
```

#### 4.1.4 停止挖矿
拷贝command.txt中的停止命令执行停止挖矿操作，如下:
```
sudo /home/abc/ipse2.0/ipse2.0-mining/localhost/10064825431032897010/supervision-10064825431032897010 --mining /home/abc/ipse2.0/ipse2.0-mining/localhost/10064825431032897010/poc-mining-10064825431032897010 --log-max-size 10 --stop

```
通过ps -ef| grep poc-mining查看进程是否已杀死，如果无法杀死进程，则进行kill -9 进程id.

如果P盘文件增大空间或者P盘id已更换，则对应修改配置文件再重启挖矿,且需在链上更新对应矿工的account_id和plot_size！

