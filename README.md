# IPSE Node
A fresh FRAME-based [Substrate](https://www.substrate.io/) node, ready for hacking :rocket:
***
## Getting Started
Follow these steps to get started with the Node Template :hammer_and_wrench:
***
### Rust Setup

First, complete the [basic Rust setup instructions](./doc/rust-setup.md).
***
### Clone Project From Github
`https://github.com/IPSE-TEAM/ipse-core.git`
***
### Init Project
* `cd ipse-core`
* `make init`
***
### Build
`make build`
***
### Run
#### Single-Node Development Chain
* `cd target/release`
* `./IPSE --dev`
***
#### Connect To Our Test Network
#### Synchronization Node
`./IPSE --chain main

### Connect To Our Mainnet
#### Synchronization Node
./IPSE --chain main

`
#### Node For Validator
Generate three mnemonics and see the sr25519 key and address associated with them. these key will be used by Babe, Audi, and Imon for block production.
```buildoutcfg
# subkey command
./subkey generate --scheme sr25519
./subkey generate --scheme sr25519
./subkey generate --scheme sr25519
```
```buildoutcfg
# subkey output
Secret phrase `infant salmon buzz patrol maple subject turtle cute legend song vital leisure` is account:
  Secret seed:      0xa2b0200f9666b743402289ca4f7e79c9a4a52ce129365578521b0b75396bd242
  Public key (hex): 0x0a11c9bcc81f8bd314e80bc51cbfacf30eaeb57e863196a79cccdc8bf4750d21
  Account ID:       0x0a11c9bcc81f8bd314e80bc51cbfacf30eaeb57e863196a79cccdc8bf4750d21
  SS58 Address:     5CHucvTwrPg8L2tjneVoemApqXcUaEdUDsCEPyE7aDwrtR8D


Secret phrase `whale embark scorpion enroll toward tackle mass more entire wrong word sure` is account:
  Secret seed:      0xe1fa469a99a5d1622335add4c71d0e9effc5b54a93c752b3507120f8eae017c7
  Public key (hex): 0xcc6503724801f06a8499a39e1d21f3a4bbe9be055fe8f2cbdbd21a18a72bcd34
  Account ID:       0xcc6503724801f06a8499a39e1d21f3a4bbe9be055fe8f2cbdbd21a18a72bcd34
  SS58 Address:     5GghdfMHBWcWRjkjXK18siYMcgrc3meabgzdtJFMu58LgVNd


Secret phrase `please crash ketchup excite squirrel sniff kick original square slim illness banner` is account:
  Secret seed:      0xef762b6d751cc817fa0cfe45f8e32887ce3e1638b417fda869788c4f0b7858ef
  Public key (hex): 0x52ab23407a0a9d0fdf649a84062f1a9fcc5294a4532094bb60d22e16fb41d266
  Account ID:       0x52ab23407a0a9d0fdf649a84062f1a9fcc5294a4532094bb60d22e16fb41d266
  SS58 Address:     5Dw6d3Qoac9ZPo2Kjk7FArFhMDySyeJRmQcHmwjmEeKBGGWu
```
Now see the ed25519 key and address associated with the same mnemonic. This key will be used by GRANDPA for block finalization.
```buildoutcfg
# subkey command
./subkey generate --scheme ed25519
```
```buildoutcfg
# subkey output
Secret phrase `goat involve type wait genuine exile husband beach iron undo tissue sorry` is account:
  Secret seed:      0x3042cd74b791cd94442534a9584d855deeecfd2f6b2b0a2a0267bceaa483f301
  Public key (hex): 0x2755c6311fa1fb6d38261db17e165a92fe7faff3ce9e2720c5c26442af4b6fce
  Account ID:       0x2755c6311fa1fb6d38261db17e165a92fe7faff3ce9e2720c5c26442af4b6fce
  SS58 Address:     5CxHCqbwoMkTmaNvNT617JrPs8phZRBWgbLjSuHCtLNiDTdd

```

```buildoutcfg
./IPSE --chain staging \
--base-path /tmp/mynode \
--name MyNode \
--validator \
--execution=NativeElseWasm \
--unsafe-ws-external \
--unsafe-rpc-external \
--rpc-cors=all \
--rpc-methods=Unsafe

```
You should see the console outputs something as follows:
```buildoutcfg
Apr 20 10:41:28.268  INFO Ipse Node
Apr 20 10:41:28.268  INFO ✌️  version 2.0.0-ac1a42f-x86_64-linux-gnu
Apr 20 10:41:28.268  INFO ❤️  by Parity Technologies <admin@parity.io>, 2017-2021
Apr 20 10:41:28.268  INFO 📋 Chain specification: IpseLocalTestnet
Apr 20 10:41:28.268  INFO 🏷 Node name: MyNode
Apr 20 10:41:28.268  INFO 👤 Role: FULL
Apr 20 10:41:28.269  INFO 💾 Database: RocksDb at /tmp/mynode/chains/ipse_local_testnet/db
Apr 20 10:41:28.269  INFO ⛓  Native runtime: ipse-node-2021041901 (ipse-node-1.tx1.au10)
Apr 20 10:41:29.971  INFO 🔨 Initializing Genesis block/state (state: 0x07ff…9a02, header-hash: 0xb892…600d)
Apr 20 10:41:29.990  INFO 👴 Loading GRANDPA authority set from genesis on what appears to be first startup.
Apr 20 10:41:30.187  INFO ⏱  Loaded block-time = 12000 milliseconds from genesis on first-launch
Apr 20 10:41:30.187  INFO 👶 Creating empty BABE epoch changes on what appears to be first startup.
Apr 20 10:41:30.189  INFO 🏷 Local node identity is: 12D3KooWG11YCm3mfGej3DuXdtZtSfunvt5WtUhdmBz4tGsAL9Fm
Apr 20 10:41:30.193  INFO 📦 Highest known block at #0
```
```buildoutcfg
# Submit a new key via RPC for grandpa module, connect to where your `rpc-port` is listening
curl http://localhost:9933 -H "Content-Type:application/json; charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["gran","xxxxx(your grandpa secret key)","xxxxx(your grandpa public key)"]}'
```
If you enter the command and parameters correctly, the node will return a JSON response as follows.
```buildoutcfg
{ "jsonrpc": "2.0", "result": null, "id": 1 }
```
```buildoutcfg
# Submit a new key via RPC for babe module, connect to where your `rpc-port` is listening
curl http://localhost:9933 -H "Content-Type:application/json; charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["babe","xxxxx(your babe secret key)","xxxxx(your babe public key)"]}'
```
If you enter the command and parameters correctly, the node will return a JSON response as follows.
```buildoutcfg
{ "jsonrpc": "2.0", "result": null, "id": 1 }
```
```buildoutcfg
# Submit a new key via RPC for IM_ONLINE module, connect to where your `rpc-port` is listening
curl http://localhost:9933 -H "Content-Type:application/json; charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["imon","xxxxx(your imon secret key)","xxxxx(your imon public key)"]}'
```
If you enter the command and parameters correctly, the node will return a JSON response as follows.
```buildoutcfg
{ "jsonrpc": "2.0", "result": null, "id": 1 }
```

```buildoutcfg
# Submit a new key via RPC for AUTHORITY_DISCOVERY module, connect to where your `rpc-port` is listening
curl http://localhost:9933 -H "Content-Type:application/json; charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["audi","xxxxx(your audi secret key)","xxxxx(your audi public key)"]}'
```
If you enter the command and parameters correctly, the node will return a JSON response as follows.
```buildoutcfg
{ "jsonrpc": "2.0", "result": null, "id": 1 }
```


>If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).

***

