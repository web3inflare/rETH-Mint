# EthsScan

## 项目简介

rETH-Mint 是一个使用 Rust 编写的具有开源高性能,多线程，多平台,直接自动设定 Gas 费用的 rETH-Mint 工具。

## 使用方法

1. 克隆项目仓库：

```
git clone https://github.com/web3inflare/rETH-Mint.git
```

2. 安装 Rust 环境：

```
# 默认安装即可
linux & mac
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
window
https://forge.rust-lang.org/infra/other-installation-methods.html
rustup-init.exe

```

3. 修改并配置 Settings.toml 文件，用于指定参数和配置信息。示例配置如下

```
private_key = "0x00000" # 密钥
max_transactions = 2 # Mint 数量
rpc_url = "https://eth-goerli.public.blastapi.io"  # rpc
max_attempts = 30 # 检测上链时间
num_threads = 100 # 线程
difficulty = "0x7777" # 难度 0x7777 0x77777 0x777777
gas_type = "standard"  # rapid 超快 fast 快  标准 standard 慢 slow
network = "Mainnet" # Goerli或Mainnet

```

4. 在目录下运行命令

```
# 请注意 如果使用国内网络可能编译不出来。请使用代理或者服务器编译
cargo run
```

5. 在 Log 目录下查看日志文件

6. 检测 ID 是否正确

```
1.打开对应hash值
https://etherscan.io
2.点击 Click to show more
3.选择UTF-8格式取出json中的ID hash值
4.修改test目录下的keccak_test.rs文件
5.运行 cargo test
6.或查看官方引索是否成功
```

## 要求

- Rust 环境

## 联系方式:

- x(twitter): Web3inflare
