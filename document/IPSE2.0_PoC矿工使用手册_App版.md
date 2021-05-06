# IPSE2.0矿工使用手册

## 矿工角色操作流程图:
![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@2.2.1/document/ipse_img/PoC_Staking/%E7%9F%BF%E5%B7%A5%E6%B5%81%E7%A8%8B%E5%9B%BE.jpg)

## 环境说明:
Ubuntu18.04或Ubuntu20.04系统
## 一、	搭建本地节点

### 1.1 下载程序
[链程序下载地址](https://github.com/IPSE-TEAM/ipse-core/releases下最新版本IPSE程序)

节点服务器本地创建文件夹，下载链程序
```
➜  ~ mkdir -p ipse2.0/ipse2.0-node
➜  ~ cd ipse2.0/ipse2.0-node 
➜  ipse2.0-node ll
total 0
➜  ipse2.0-node wget https://github.com/IPSE-TEAM/ipse-core/releases下最新版本IPSE程序
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

 
   
## 三、矿工启动挖矿（P盘及启动挖矿）
当前只支持linux系统(如ubuntu18.04或ubuntu20.04)进行P盘及挖矿！

### 3.1 手动操作P盘及挖矿
####  3.1.1 下载所需文件
(1)下载最新版本的P盘软件engraver、挖矿软件poc-mining及挖矿配置文件config.yaml、miners_config.yaml文件，启动py脚本supervision.py、update_config.py

下载路径: https://github.com/IPSE-TEAM/ipse2.0-mining/releases下

(2)解压P盘工具
```
➜  ipse2.0-mining tar -xvf engraver-2.4.0-x86_64-unknown-linux-gnu-cpu-gpu.tar.xz 
engraver_cpu
engraver_gpu 
```
#### 3.1.2 矿工P盘
在之前解压P盘软件所在目录下进行P盘操作
```
sudo ./engraver_cpu  --n nonce  --id numeric_id  --path plot_dirs  --sn 0 > plot.log 2>&1 &
-----------------------------------------------------------------------------
--n nonce  P盘大小对应的nonce
--id numeric_id  P盘id
--path plot_dirs  指定P盘文件存放目录(如/data/data2000001234000100)
--sn 0  起始随机数（计算：已使用随机数。其它P盘大小随机数之和）
> plot.log 2>&1 & 写入日志并后台运行

-----------------------------------------------------------------------------
nonce计算：
1GiB=（1024*1024*1024）B=1073741824B
1GB=（1000*1000*1000）B=1000000000B
1GiB/1GB=1073741824/1000000000=1.073741824
计算:
1 nonce=256KiB，1MiB= 4nonce，则1G= 1*1024*4=4096 nonce，以此类推。
```


##### 3.1.2.1 执行P盘命令
通过执行以下命令进行P盘
```
➜  ipse2.0-mining ./engraver_gpu --n 40940 --id 10064825431032897010 --path /data/data10064825431032897010 --sn 0  &
```
提示指定的P盘文件存放路径不存在，那么我们先新建该文件路径
```
➜  ipse2.0-mining mkdir -p /data/data10064825431032897010                                                                          
➜  ipse2.0-mining ll /data/data10064825431032897010 
总用量 0

```

再次执行P盘操作
```
➜  ipse2.0-mining ./engraver_gpu --n 40940 --id 10064825431032897010 --path /data/data10064825431032897010 --sn 0  &
```
 

完成P盘，接着下一步操作。

#### 3.1.3 修改配置文件
打开miners_config.yaml文件，对应修改文件中host 、account_id 、phase、 miner_proportion 、url 、plot_size 、miner_reward_dest、plot_path的值，并保存退出。
```
miners: # 矿工的统一配置
   #224
   - {host: poc_test,
      account_id: 11611391906548388081,
      phase: defense ball area outside castle divert fortune crazy gather camp response yard,
      miner_proportion: 20,
      url: "ws://localhost:9948",
      plot_size: 3700,
      miner_reward_dest: 5GWX6izv5A7Ja2ik7jctcXDZ8MZFF8BePSKEgs37J5DMbXha,
      plot_path: /data/data
   }
--------------------------------------------------------
host #当前目录生成矿工文件夹名称 
account_id #矿工的P盘ID
phase  #矿工的助记词
miner_proportion #矿工挖矿获得的奖励占比
url  #本地节点或远程节点的地址(如输入本地节点 “ws://localhost:9948”)
plot_size #矿工的P盘容量(GB)
miner_reward_dest #矿工挖矿奖励的存放账户地址
plot_path #矿工P盘的路径(注: 如上面P盘文件存放路径为/data/data10064825431032897010,则这里写/data/data 就可以了，执行python3 update_config.py会在后面自动account_id的数值)
--------------------------------------------------------
```
如果需要生成多个矿工，则按照格式添加信息就可以了，如下：
```
miners: # 矿工的统一配置
   #224
   - {host: poc_test,
      account_id: 11611391906548388081,
      phase: defense ball area outside castle divert fortune crazy gather camp response yard,
      miner_proportion: 20,
      url: "ws://localhost:9948",
      plot_size: 3700,
      miner_reward_dest: 5GWX6izv5A7Ja2ik7jctcXDZ8MZFF8BePSKEgs37J5DMbXha,
      plot_path: /data/data
   }
   - {host: poc_test,
      account_id: 16045882063755536351,
      phase: increase cushion season lunar advice history urge ice color gas sport region,
      miner_proportion: 20,
      url: "ws://localhost:9948",
      plot_size: 3700,
      miner_reward_dest: 5GWX6izv5A7Ja2ik7jctcXDZ8MZFF8BePSKEgs37J5DMbXha,
      plot_path: /data/data
   }
```

执行python脚本，生成挖矿程序及挖矿配置文件，如下:
```
➜  ipse2.0-mining python3 update_config.py
```
在当前路径下，生成host+ plot_path的文件路径，进入到这个路径下可查看生成的对应文件，如下:
```
➜  ipse2.0-mining cd poc_test1/10064825431032897010  
➜ 10064825431032897010 ll
总用量 15M
-rw-r--r-- 1 root root  470 3月  25 16:56 command.txt
-rw-r--r-- 1 root root 1.2K 3月  25 16:56 config.yaml
-rwxrwxrwx 1 root root  15M 3月  25 16:56 poc-mining-10064825431032897010
-rw-r--r-- 1 root root 4.0K 3月  25 16:56 supervision-10064825431032897010.py
```
生成的启动挖矿/停止挖矿命令在command.txt文件里
```
➜  10064825431032897010 cat command.txt                  
python3 /home/xjh/ipse2.0/ipse2.0-mining/poc_test1/10064825431032897010/supervision-10064825431032897010.py --mining /home/xjh/ipse2.0/ipse2.0-mining/poc_test1/10064825431032897010/poc-mining-10064825431032897010 --log-max-size 10
 
python3 /home/xjh/ipse2.0/ipse2.0-mining/poc_test1/10064825431032897010/supervision-10064825431032897010.py --mining /home/xjh/ipse2.0/ipse2.0-mining/poc_test1/10064825431032897010/poc-mining-10064825431032897010 --log-max-size 10 --stop
```

下面3.1.4章节中可使用以上的启动挖矿/停止挖矿命令，进行启动或停止挖矿操作。

#### 3.1.4 手动启动及停止挖矿操作
启动前提是该矿工地址有足够的币，因为启动后自动进行矿工注册操作
##### 3.1.4.1 python启动挖矿软件(异常自动重启)
(1)建议通过supervision-xxxxx.py启动，可实现程序异常自动重启

#进入挖矿程序所在路径
```
cd /home/xjh/ipse2.0/ipse2.0-mining/poc_test1/10064825431032897010
```
#查看command.txt文件的启动/停止挖矿命令
```
➜  10064825431032897010 cat command.txt
```
#启动挖矿命令（末尾加 & 为了后台运行）
```
python3 /home/xjh/ipse2.0/ipse2.0-mining/poc_test1/10064825431032897010/supervision-10064825431032897010.py --mining /home/xjh/ipse2.0/ipse2.0-mining/poc_test1/10064825431032897010/poc-mining-10064825431032897010 --log-max-size 10 &
```
#动态查看日志
```
➜  10064825431032897010 tail -f poc-mining-10064825431032897010.log  
```
#停止挖矿命令
```
➜  10064825431032897010 python3 /home/xjh/ipse2.0/ipse2.0-mining/poc_test1/10064825431032897010/supervision-10064825431032897010.py --mining /home/xjh/ipse2.0/ipse2.0-mining/poc_test1/10064825431032897010/poc-mining-10064825431032897010 --log-max-size 10 --stop
``` 

##### 3.1.4.2 直接启动挖矿软件(异常不会自动重启)
#启动挖矿命令（末尾加 & 为了后台运行）
```
➜  ./poc-mining-10064825431032897010 & 
```
#停止挖矿命令
```
➜  10064825431032897010 ps -ef |grep poc-mining
root     18194 17327  0 17:48 pts/1    00:00:00 ./poc-mining-10064825431032897010
root     18234 18104  0 17:49 pts/2    00:00:00 grep --color=auto --exclude-dir=.bzr --exclude-dir=CVS --exclude-dir=.git --exclude-dir=.hg --exclude-dir=.svn --exclude-dir=.idea --exclude-dir=.tox poc-mining
➜  10064825431032897010 kill -9 18194
```
如果P盘文件增大空间或者P盘id已更换，则对应修改配置文件再重启挖矿！

