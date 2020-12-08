import os
import time

def kill_process(FileName):
	info = os.popen("ps -ef | grep {0}".format(FileName)).readlines()
	for i in info:
		try:
			j = i.split()[1].strip()
			os.system("kill -9 " + j)
			print("杀掉进程! {0}".format(i))
		except Exception as e:
			print("删除进程错误! e = {0}, info = {1}".format(e, i))


def run(FileName):
	# 检查5分钟 如果有5条日志以上相同 那么判定挖矿异常

	info = None
	start = None
	count = 0
	dir_url = r'./{0}.log'.format(FileName)

	while True:
		try:
			with open(dir_url, "r") as f:

				info = f.readlines()[-1]  # .split()
				print(info)
				if info != start and "Client Error" not in info:
					count = 0
					start = info

				else:
					count += 1
					print(count)

				if count >= 5:
					kill_process(FileName)
					print("关闭挖矿软件!")
					time.sleep(5)

					os.system(r'./{0} > {1}.log 2>&1 &'.format(FileName, FileName))
					print("启动挖矿软件!")

					count = 0


		# 没有日志记录或是没有日志文件 说明没有启动软件
		except Exception as e:
			print("没有启动挖矿软件！")
			kill_process(FileName)
			print("关闭挖矿软件!")
			result = os.system(r'./{0} > {1}.log 2>&1 &'.format(FileName, FileName))
			print("启动挖矿软件!")
			count = 0

		time.sleep(10)


if __name__ == "__main__":
	# 监控节点 放在与挖矿软件相同的文件夹中
	FileName = "Alice"
	run(FileName)








