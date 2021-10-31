# IPSE2.0_IPSE存储节点方及用户使用手册
IPSE节点方及用户业务流程图:
![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/IPSE/IPSE_mining1.jpg)

## 环境说明:

Ubuntu18.04或Ubuntu20.04
## 一、ipfs本地同步节点部署
### 1.1 下载ipfs
[IPFS下载链接](https://github.com/ipfs/go-ipfs/releases/download/v0.7.0/go-ipfs_v0.7.0_linux-amd64.tar.gz)
### 1.2 安装ipfs
```
➜  ipse2.0-ipfs tar -zxvf go-ipfs_v0.6.0_linux-amd64.tar.gz
go-ipfs/install.sh
go-ipfs/ipfs
go-ipfs/LICENSE
go-ipfs/LICENSE-APACHE
go-ipfs/LICENSE-MIT
go-ipfs/README.md
```
执行安装:
```
➜  ipse2.0-ipfs cd go-ipfs 
➜  go-ipfs ./install.sh 
Moved ./ipfs to /usr/local/bin
```
### 1.3 ipfs使用
安装之后，用法可以参考ipfs --help ,基本流程如下：

``step1: 初始化本地配置(ipfs init)``
```
➜  go-ipfs ipfs init
initializing IPFS node at /root/.ipfs
generating 2048-bit RSA keypair...done
peer identity: QmaSMT6x5iRmrHVUrHgVEaArfAkLGVyxcoTkpRrm4uvM2M
to get started, enter:

	ipfs cat /ipfs/QmQPeNsJPyVWPFDVHb77w8G42Fvo15z4bG2X8D2GhfbSXc/readme
```
``step2: ipfs后台运行命令(nohup ipfs daemon &)``
```
➜  lany nohup ipfs daemon &
[1] 127065
nohup: ignoring input and appending output to 'nohup.out'                                                                       
➜  lany tail -f nohup.out 
Swarm announcing /ip4/172.16.5.23/tcp/4001
Swarm announcing /ip4/172.16.5.23/udp/4001/quic
Swarm announcing /ip4/47.98.139.83/tcp/4001
Swarm announcing /ip4/47.98.139.83/udp/4001/quic
Swarm announcing /ip6/::1/tcp/4001
Swarm announcing /ip6/::1/udp/4001/quic
API server listening on /ip4/127.0.0.1/tcp/5001
WebUI: http://127.0.0.1:5001/webui
Gateway (readonly) server listening on /ip4/127.0.0.1/tcp/8080
Daemon is ready
```
到此ipfs暂告一段落。

## 二、ipse2.0-miner节点方出块
### 2.1 前提
本地已安装IPFS，请查看上一步ipfs安装！
已部署ipse节点，[部署请查阅如何搭建本地节点章节](https://github.com/IPSE-TEAM/ipse-core/blob/ipse/document/IPSE2.0_PoC Miner Manual_Linux.md)
### 2.2 miner安装
ipse2.0-miner 是存储出块的工具。下面展示miner工具的主要用法。安装和开发访问https://github.com/IPSE-TEAM/ipse2.0-miner/releases下

安装之后，用法可以参考 miner --help
```
➜  miner --help
miner 1.0.0
IPSE-TEAM

USAGE:
    miner [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>    Path to a config file other than config.toml in the root of project
    -r, --root <root>        Directory to use as root of project [default: .]

SUBCOMMANDS:
    generate    Generate a random account
    help        Prints this message or the help of the given subcommand(s)
    init        Create a new project
    job         Scheduling tasks for miner
    serve       Serve the miner serve
```
### 2.3 miner使用
下面展示基本流程

``step1: 生成节点方账号 ``

该账号是作为一个节点账号，把下面信息进行私密保存
```
➜  tmp miner generate
{
  "miner_id": "5CSpZW9572pstfm8HxvCCtVB72hv1zvxX6FHvEn3iSHAbnF4",
  "public_key": "10ddf77a9cf7a15f5d58bdebce2e7cecb72e6332250b07bf5d41112c6da56a02",
  "secret_phrase": "gallery egg water trial toddler into chunk space announce grief arena flip",
  "secret_seed": "ea43101d1ccf77e936fea27a7c7de8590448d04afb99310db4f06646edf5b341"
}
```

````step2: 初始化项目目录````

下面是一个项目初始化的目录,名称为cook,你可以换其他名字，该目录下有config.toml,db, keystore.
```
➜  tmp miner init cook
➜  tmp cd cook
➜  cook ls
config.toml  db  keystore
```
我们需要进行配置的是config.toml，除了[data]、[seaerch]、[ipfs]、[serve]部分不需要改，其他自定义修改
```
[miner]
nickname = "the_name_of_miner" #节点方昵称-别名
region = "the_regin_of_miner"  #地区
url = "http://localhost"       # 节点方对外上传访问的url
capacity = 1024000000          # 容量(10GB，则输入10*1024*1024*1024的结果)
unit_price = 100               # 单价（ipse/byte,每字节多少ipse;建议单价写小一点，如0.000001或0.0000001）
public_key = "刚刚生成的public_key"
secret_seed = "刚刚生成的secret_seed"
income_address = "你的收益地址" #如果非节点方自己的地址(其他地址)，则该地址可用余额必须大于1

[chain]
url = "ws://localhost:9944"  # ipse 链上节点地址
#下面的信息不需要更改
[data]
db = "db"
keystore = "keystore"

[seaerch]
url = "https://www.ipse.io/v3/machine/ipse/"


[ipfs]
uri = "http://127.0.0.1:5001"
local = false

[serve]
secret_key = "QlKHBX8H7RYN5nksrZf3R1ePoDceXLNMLvyxTQ7MldMf"
```

``step3: 启动项目``

配置完之后可以启动
```
miner serve
```
启动之后这个就是 存储出块的serve

后台启动使用如下命令：
```
nohup miner serve &
```
``step4: 查看日志``

动态查看日志
```
tail -f nohup.out
```
### 2.4 调度
调度的目的是删除过期的存储数据，可以直接启动miner job 来进行调度
```
➜  lany miner job
start rm expired data file
end rm expired data file
```
## 三、节点方app端使用ipse功能
### 3.1 前提
(1)miner账户可用余额足够, Miner程序已启动并自动注册成为ipse存储节点方。

(2)已安装ipse-app。
   [App下载链接](https://www.ipse.io/app/ipse.apk)


### 3.2 使用ipse存储功能
注：节点方也是用户，可以选择自己或其他节点方进行上传文件。

“标注”-“节点方管理”,可以查看存储节点方注册的详细信息。

#### 3.2.1 节点方申请抵押排名
进入”标注”-“节点方管理”,选择”抵押排名”输入金额并提交，进入抵押排名成功，标注页面就可以看到排名信息。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/IPSE/IPSE_applyToRecommendedList.png)      
#### 3.2.2 节点方查看已确认订单
进入”标注”-“节点方管理”，选择”已确认订单”，可以查看到自己账户下所有的订单。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/IPSE/ipse_confirmOrder.png)    
#### 3.2.3 节点方查看出块记录
进入”标注”-“节点方管理”，选择”出块记录”可查看到自己的出块记录。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/IPSE/ipse_miner_orderInfo.png)    

#### 3.2.4 节点方更新注册信息
两种方式:
(1)直接后台修改config.toml文件中的相关参数值，重启miner程序，更新注册信息成功
(2)网页端“开发者”-“交易”-(ipse.registerMiner)修改容量/单价并提交交易，更新注册信息成功
#### 3.2.5 节点方退出抵押排名
进入”标注”-“节点方管理”，选择”退出抵押排名”并提交交易，退出抵押排名成功，抵押金额实时返还给节点方。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/IPSE/ipse_dropOutRecommendedList.png)     

## 四、用户使用ipse存储功能
### 4.1 前提

用户已安装app且导入账户,账户余额充足。
[App下载链接](https://www.ipse.io/app/ipse.apk)

### 4.2 使用ipse存储功能
#### 4.2.1 查看节点方信息
进入”标注”-“标注”，选择任意节点方，点击右边的图标，可以查看该节点方的出块信息。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/IPSE/ipse_Check_minerInfo.png)      
#### 4.2.2 上传文件
进入”标注”-“标注”页面，选择指定节点方，进行上传文件。上传文件成功，根据上传文件大小进行计算费用；
保留金额=上传文件大小(单位:Byte)*节点方设定的单价(n IPSE/Byte)*存储天数

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/IPSE/ipse_uploadFile.png)         

#### 4.2.3 查看订单明细
用户上传文件成功，返回”标注”页面，最新订单数据列表增加一条订单记录，点击可查看订单明细

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/IPSE/ipse_orderInfo.png)    


#### 4.2.4 访问ipse.io搜索已上传文件
浏览器打开www.ipse.io，输入已确认订单的hash，可以查看到所上传的文件信息

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/IPSE/ipse_explorer.png) 
   
