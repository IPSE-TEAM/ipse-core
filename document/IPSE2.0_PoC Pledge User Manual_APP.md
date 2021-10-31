# IPSE2.0_PoC质押者使用手册
质押者业务流程图:
![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/PoC_Staking/PoC质押者1.jpg)

## 1 创建/导入账户
[IPSE下载地址](https://www.ipse.io/app/ipse.apk)
(1)打开ipse手机客户端，创建/导入账户，账户需要有足够的IPSE余额，支付相关交易手续费。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/PoC_Staking/创建_导入账户.png)          
## 2 质押者质押出块
非冷却期才能进行质押及解押操作，同时节点方也可以质押自己。
### 2.1 质押者进行质押
(1)进入“质押”-“参与质押”页面，选择推荐列表中的节点方进行质押（可查看节点方的出块详情），提交质押成功；在“质押”-“我的质押”列表可查看到所质押的节点方。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/PoC_Staking/质押者进行质押.png)

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/PoC_Staking/质押者进行质押2.png)

(2)进入“质押”-“我的质押”列表，选择任意已质押的节点方，质押者可查看自己的出块记录
当节点方提交deadline出块获得奖励时，质押者可以根据质押金额占比，获得相应的IPSE奖励，公式:  

节点方奖励金额=本次出块奖励金额*节点方佣金比例（若无质押者，则节点方奖励金额=本次出块奖励金额）

所有质押者总佣金=本次出块奖励金额-节点方奖励金额（存在质押者）

某个质押者佣金=所有质押者总佣金*该质押者质押金额/总质押金额

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/PoC_Staking/质押者进行质押3.png)

### 2.2 质押者更新质押
进入“质押”-“我的质押”列表，任选一个已质押的节点方,进行“更新质押金额”操作；可增加或减少质押金额,出块奖励也会随之增加或者减少；如果减少质押金额，该质押金额进入锁定期，锁定期结束需手动领取。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/PoC_Staking/质押者更新质押.png)  
      
### 2.3 质押者退出质押 
进入“质押”-“我的质押”列表，任选一个已质押的节点方,进行退出质押操作, 退出质押出块成功，质押金额进入锁定期，锁定期结束需手动领取，不能再获得该节点方出块的奖励分佣；可重新进行质押。

![avatar](https://cdn.jsdelivr.net/gh/IPSE-TEAM/ipse-core@ipse/document/ipse_img/PoC_Staking/质押者退出质押.png)

