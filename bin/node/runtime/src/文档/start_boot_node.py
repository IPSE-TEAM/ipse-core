import os
import sys
import getopt
import time
from subprocess import run

def kill(rpc_port):
	info = os.popen("ps -ef | grep IPSE").readlines()
	if info:
		for i in info:
			if rpc_port in i:
				print("关闭节点！")
				j = i.split()[1].strip()
				return os.system("kill -9 " + j)

# 生成raw文件
# file = r"./TransX build-spec --chain=staging > localspec.json"
# raw = r"./TransX build-spec --chain localspec.json --raw > customspec.json"
# all1 =[file, raw]
# for i in all1:
# 	a = os.system(i)
# 	time.sleep(10)
# 	if a != 0:
# 		exec("编译二进制文件没有成功!")

opts, args = getopt.getopt(sys.argv[1:], "", ["babe-key=", "gran-key=", "rpc-port=", "log-file=", "port=", "ws-port=", "node-key-file=", "name=", "base-path=", ])
print(opts)
keys = [i[0] for i in opts]

print(keys)

# rpc-port与babe-key、gran-key和log-file是必须的
if all([x in keys for x in ["--babe-key", "--gran-key", "--rpc-port", "--log-file"]]):
	pass
else:
	exec("rpc-port、 babe-key、gran-key是必须输入的参数项！")


rpc_port = None
gran_key = None
babe_key = None
append_string = ""
log_file = None
for opt, arg in opts:
	if opt == "--rpc-port":
		append_string = append_string + " " + opt + " " + arg
		rpc_port = arg
	elif opt == "--babe-key":
		babe_key = arg
	elif opt == "--gran-key":
		gran_key = arg
	elif opt == "--log-file":
		log_file = arg
	else:
		append_string = append_string + " " + opt + " " + arg
print(append_string)
print(rpc_port)
print(gran_key)
print(babe_key)


# 启动节点
# 注意 如果需要node-key-file一定要与--rpc-port对得上
cmd = r"./IPSE --chain customspec.json --validator --ws-external --rpc-external --rpc-methods=Unsafe --rpc-cors=all --execution=NativeElseWasm" + " " + append_string + " > %s"%log_file + ".log 2>&1 &"
print(cmd)

result = os.system(cmd)
if result != 0:
	exit("启动节点失败！")

time.sleep(30)
# rpc_port = str(9933)
# 根据babe和gran私钥生成对应的密钥文件
# gran_key = "0x6254e516076e1eb471185a2c4b5f56e4311784e12a4936c35768ee84cee10cc3"
gran = r"./subkey inspect --scheme ed25519 %s" % gran_key
gran_public = os.popen(gran).readlines()[3].split()[-1].strip()
print("grand-key", gran_key)
print("grand-public", gran_public)

send = """curl http://localhost:%s -H "Content-Type:application/json; charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["gran","%s","%s"]}'""" % (rpc_port, gran_key, gran_public)
print(send)
send_result = os.system(send)
if send_result != 0:
	kill(rpc_port)
	exit("发送gran-key失败")

# babe_key = "0xae8c1d6c09e011e476a541807947f6299cc398f65ea9ae56d79b480abddc836f"
babe = r"./subkey inspect %s" % babe_key
babe_public = os.popen(babe).readlines()[3].split()[-1].strip()
print("babe-key", babe_key)
print("babe-public", babe_public)

send = """curl http://localhost:%s -H "Content-Type:application/json; charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["babe","%s","%s"]}'""" % (rpc_port, babe_key, babe_public)
send_result = os.system(send)
if send_result != 0:
	kill(rpc_port)
	exit("发送babe-key失败")


# 关闭节点重启
if kill(rpc_port) == 0:
	result = os.system(cmd)
	if result != 0:
		exit("启动节点失败！")
	else:
		print("重新启动节点成功！")



