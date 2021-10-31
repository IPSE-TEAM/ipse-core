# 1 How to build IPSE nodes
Build a local node
## Environment Description:
Ubuntu18.04 or Ubuntu20.04 system
## 1.1 Download the program
[Program download address](https://github.com/IPSE-TEAM/ipse-core/releases/download/3.0.1/IPSE)

The node server creates the folder locally and downloads the chain program
```
sudo mkdir -p ipse2.0/ipse2.0-node && cd ipse2.0/ipse2.0-node && sudo wget https://github.com/IPSE-TEAM/ipse-core/releases/download/3.0.1/IPSE
```
Give executable permissions:
```
sudo chmod +x IPSE
```
## 1.2 Synchronize node data
Start synchronizing your nodes by running the following instructions (write to log files and run in the background):

### 1.2.1 Connect To Our Test Network

Custom node name and node data store path

```
sudo ./IPSE --chain staging --ws-port 9948 --base-path ./db --pruning=archive  --execution=NativeElseWasm --wasm-execution Compiled --name NodeNameCustom  > ipse.log 2>&1 &
```
### 1.2.2 Connect To Our Main Network

Custom node name and node data store path

```
sudo ./IPSE --chain main --ws-port 9948 --base-path ./db --pruning=archive  --execution=NativeElseWasm --wasm-execution Compiled --name NodeNameCustom  > ipse.log 2>&1 &
```

If you don't want to run validation mode right away.
The --pruning=archive option implies the --validator and -sentry options, so it must be explicitly requested only if the node is started without either of these options.If you are not set up as an ARCHIVE node, you will need to resynchronize the database when switching, even when you are not running the validator and sentinel modes.
Depending on the size of the chain at the time, this step may take anywhere from a few minutes to a few hours.
If you want to estimate how much more time is needed, the server log (via the command tail -f ipse.log) shows the processed and newly validated blocks for your node.You can then compare with Telemetry or the current PolkadotJS blockchain browser.

## 1.3 Start the local node
When the node synchronization data is completed, close the IPSE program, restart the local node(Test Network or Main Network), and run the following commands (write to log files and run in the background):
Custom node name and node data store path
```
sudo ./IPSE --chain main --ws-port 9948 --rpc-port 30339 --execution=NativeElseWasm  --unsafe-ws-external --unsafe-rpc-external  --rpc-cors=all --base-path ./db --rpc-methods=Unsafe  --pruning=archive --wasm-execution Compiled --name NodeNameCustom   > ipse.log 2>&1 &
```
For log details, use tail -f ipse.log.
```
Jun 09 17:11:31.208  INFO IPSE2.0 Node
Jun 09 17:11:31.208  INFO ✌️  version 3.0.0-a65ef41-x86_64-linux-gnu
Jun 09 17:11:31.208  INFO ❤️  by IPSE TEAM, 2020-2021
Jun 09 17:11:31.208  INFO 📋 Chain specification: IPSE Mainnet
Jun 09 17:11:31.208  INFO 🏷 Node name: V-ipse-81
Jun 09 17:11:31.208  INFO 👤 Role: AUTHORITY
Jun 09 17:11:31.208  INFO 💾 Database: RocksDb at /data/ipse_db/chains/IPSE2.0/db
Jun 09 17:11:31.208  INFO ⛓  Native runtime: ipse-node-2021060201 (ipse-node-1.tx1.au10)
Jun 09 17:11:32.611  INFO 🏷 Local node identity is: 12D3KooWMTfZuF94Vx5p1qQBovnzAx4FR49MSc79mQB42kxoXyh6
Jun 09 17:11:32.615  INFO 📦 Highest known block at #85065
Jun 09 17:11:32.615  INFO 〽️ Prometheus server started at 127.0.0.1:9615
Jun 09 17:11:32.637  INFO Listening for new connections on 0.0.0.0:9948.
Jun 09 17:11:32.637  INFO 👶 Starting BABE Authorship worker
```
Then the mining program can directly connect ws://localhost:9948 to mine.

## 1.4 Close the local node
View the IPSE process number and kill the process with the following command:
```
ps -ef | grep IPSE
```
```
Root 1795222 1 2 Mar24?00:47:46./IPSE --chain main --execution= nativeElseasm --unsafe-ws-external -- rpc-external --rpc-cors=all--ws-port 9948 --rpc-port 30339 --base-path db --rpc-methods=Unsafe --pool-limit 100000 --ws-max-connections 50000
Root 1833766 1833711 0 15:26 PTS /0 00:00:00 00 grep --color=auto --exclude-dir=.bzr --exclude-dir=CVS--exclude-dir=.git --exclude-dir=.hg --exclude-dir=.svn --exclude-dir=.idea --exclude-dir=.tox IPSE
```

```
sudo kill -9 1795222
```

If you want to be a validator node, refer to the following documentation:

[Polkadot set the validator nodes on the network](https://wiki.polkadot.network/docs/zh-CN/maintain-guides-how-to-validate-polkadot)

# 2 IPSE2.0 Instructions for PoC Miners
PoC mining supports Ubuntu and Windows systems.
## 2.1 Ubuntu systems mining

Please refer to Chapters 2 and 3 in the [IPSE2.0_PoC Miner Manual_Linux](https://github.com/IPSE-TEAM/ipse-core/blob/ipse/document/IPSE2.0_PoC%20Miner%20Manual%20_Linux.md)

## 2.2 Windows systems mining

Please refer to Chapters 2 and 3 in the [IPSE2.0_PoC Miner Manual_Windows](https://github.com/IPSE-TEAM/ipse-core/blob/ipse/document/IPSE2.0_PoC%20Miner%20Manual_Windows.md)

# 3 IPSE2.0 Instructions for IPSE storage miner
If you become an IPSE storage miner and a user uploads a file to your storage server, you will be charged based on the size of the uploaded file, and the storage miner will receive this portion of the reward.
Calculation formula:
```
total_price = file_size(Byte) * unit_price * days
```

[How to become an IPSE storage miner](https://github.com/IPSE-TEAM/ipse-core/blob/ipse/document/IPSE2.0_IPSE%20Storage%20Miner%20and%20User%20Manual_APP.md)

# 4 IPSE2.0 Instructions for PoC Pledgers
Users can earn mining rewards by pledging PoC miners. Please refer to [IPSE2.0_PoC Pledge User Manual](https://github.com/IPSE-TEAM/ipse-core/blob/ipse/document/IPSE2.0_PoC%20Pledge%20User%20Manual_APP.md) for details.
