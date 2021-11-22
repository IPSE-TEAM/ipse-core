## 一、部署节点

此步略过，[请参照此文档的第一章搭建本地节点](https://github.com/IPSE-TEAM/ipse-core/blob/ipse/document/IPSE2.0_PoC Miner Manual_Linux.md)，部署并同步节点数据完成，返回当前文档。

## 二、成为验证人节点

### 2.1 浏览器连接节点

浏览器输入https://polkadot.js.org/apps打开网页，左上角点击进行设置连接节点，保存自动刷新网络。

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/page-home.png)

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/page-home-setNode.png)



### 2.2 创建controller和stash账户

进入“账户”-“账户”页面，创建两个账号，或者导入2个账号，分别自定义命名(如controller-26、stash-26）

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/accounts.png)

### 2.3 绑定账号

网络->质押，选择“账户操作”-“存储账户”，进行stash和controller账户相互绑定，**账户需要有足够的可用余额**。

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/open_staking-page1.png)

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/open_staking-stash.png)

### 2.4 生成rotateKeys

进入“开发者”-“RPC calls”页面，选择"author"-"rotateKeys()",提交RPC调用，复制生成的rotateKeys，且此**rotateKeys务必与stash和controller账户一一对应进行备份。**

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/rotateKeys1.png)

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/rotateKeys2.png)



### 2.5 绑定sessionKey(rotateKeys)

网络->质押，选择“账户操作”-"session 密钥"，输入rotateKeys进行进行提交。

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/sessionkeys2.png)

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/sessionkeys3.png)



### 2.6 成为验证人

网络->质押，选择“账户操作”-"验证"，输入奖励佣金百分比，进行提交，进入候选队列，等待时代到期进行选举成为验证人节点。

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/validate1.png)

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/validate2.png)

![](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/validators/waiting.png)



## 三、重启节点程序

页面操作完成，则重启节点，重启命令加入--validator参数，运行验证模式。

```
sudo ./IPSE --chain  main --validator --ws-port 9948 --rpc-port 30339 --execution=NativeElseWasm  --unsafe-ws-external --unsafe-rpc-external  --rpc-cors=all --base-path ./db --rpc-methods=Unsafe  --pruning=archive --wasm-execution Compiled --name 节点名字自定义   > ipse.log 2>&1 &
```

