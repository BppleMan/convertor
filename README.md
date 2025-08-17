# convertor

[![Crates.io](https://img.shields.io/crates/v/convertor)](https://crates.io/crates/convertor)
[![Crates.io downloads](https://img.shields.io/crates/d/convertor)](https://crates.io/crates/convertor)
[![Docs](https://docs.rs/convertor/badge.svg)](https://docs.rs/convertor)
[![CI](https://github.com/BppleMan/convertor/actions/workflows/build-and-push.yml/badge.svg)](https://github.com/BppleMan/convertor/actions/workflows/build-and-push.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](https://opensource.org/licenses/Apache-2.0)
[![Last commit](https://img.shields.io/github/last-commit/BppleMan/convertor)](https://github.com/BppleMan/convertor)
[![Code size](https://img.shields.io/github/languages/code-size/BppleMan/convertor)](https://github.com/BppleMan/convertor)
[![GitHub stars](https://img.shields.io/github/stars/BppleMan/convertor)](https://github.com/BppleMan/convertor/stargazers)

A high-performance profile converter for Surge and Clash.  
convertor 可以将订阅提供商的配置转换为 Surge/Clash 客户端可用的配置，提供命令行工具和可选的 HTTP 服务。

## ✨ 特性

- 支持 Surge 与 Clash 订阅互转
- 内置 Axum HTTP 服务，可通过 API 获取转换后的配置
- 提供 CLI 子命令，生成订阅链接、安装服务、修改配置等
- 基于 Tokio 异步运行时与 Redis 缓存，性能优越
- 使用 Rust 编写，单个可执行文件便于部署

## 🚀 安装

```bash
cargo install convertor
# 或者克隆仓库自行编译
git clone https://github.com/BppleMan/convertor.git
cd convertor
cargo build --release
```

## 🛠️ 用法

启动服务（默认监听 `127.0.0.1:8080`）：

```bash
convertor
```

获取订阅链接：

```bash
convertor sub get clash bos-life --server http://127.0.0.1:8080
```

### 命令帮助

顶层命令：

```text
启动 Convertor 服务

Usage: convertor [OPTIONS] [LISTEN] [COMMAND]

Commands:
  config   配置相关的子命令 获取配置模板, 生成配置文件等
  sub      获取订阅提供商的订阅链接
  install  安装 systemd 服务
```

配置子命令：

```text
配置相关的子命令 获取配置模板, 生成配置文件等

Usage: convertor config [OPTIONS] [FILE] [COMMAND]

Commands:
  template  获取配置模板
  redis     从 Redis 获取配置

Options:
  -p, --publish  是否发布配置到 Redis
```

订阅子命令：

```text
获取订阅提供商的订阅链接

Usage: convertor sub [OPTIONS] <CLIENT> [PROVIDER] [COMMAND]

Commands:
  get     使用 订阅提供商API 获取最新订阅链接
  reset   使用重置的原始订阅链接
  raw     解码 订阅提供商 的原始订阅链接
  decode  解码 convertor 的完整订阅链接

Options:
  -s, --server <SERVER>      convertor 所在服务器的地址 格式为 `http://ip:port`
  -i, --interval <INTERVAL>  订阅更新的间隔时间，单位为秒
  -S, --strict <STRICT>      是否严格模式 [possible values: true, false]
  -u, --update               是否更新本地订阅文件
```

## 📦 开发

运行测试：

```bash
cargo test
```

## 📄 许可

本项目使用 [Apache-2.0](https://opensource.org/licenses/Apache-2.0) 许可证。
