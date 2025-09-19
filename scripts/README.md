# 构建脚本系统

这个项目使用模块化的 shell 脚本系统来管理构建、测试和部署流程。

## 🗂️ 脚本结构

```
scripts/
├── common.sh       # 通用函数和日志工具
├── config.sh       # 参数转换和配置管理
├── build.sh        # 构建相关脚本
├── linux.sh        # Linux 跨平台构建
├── docker.sh       # Docker 镜像管理
└── tools.sh        # 发布、测试和工具脚本
```

## 🚀 快速开始

### 基本命令

```bash
# 准备开发环境
just prepare

# 构建开发环境
just build-dev

# 构建生产环境
just build-prod

# 查看项目状态
just status

# 运行所有测试
just test-all
```

### 查看帮助

```bash
# 查看 justfile 命令列表
just --list

# 查看总体帮助
just help

# 查看具体脚本帮助
./scripts/build.sh help
./scripts/docker.sh help
./scripts/linux.sh help
./scripts/tools.sh help
```

## 📝 详细用法

### 构建系统

```bash
# 构建所有组件
just all [dev|prod|alpine]

# 构建 convd
just convd [dev|prod|alpine]

# 构建 confly
just confly [dev|prod|alpine]

# 构建前端
just dashboard [dev|prod]
```

### Linux 跨平台构建

```bash
# MUSL 静态构建 (推荐)
just musl [dev|prod|alpine]

# Linux 原生构建
just linux [dev|prod|alpine]

# Cross 交叉编译
just cross [dev|prod|alpine]

# 检查构建结果
just check [target] [profile] [bin]
```

### Docker 管理

```bash
# 构建镜像
just image [dev|prod|alpine]

# 运行容器
just run [dev|prod|alpine]

# 发布到 GHCR (Personal Access Token)
just publish-ghcr [dev|prod|alpine] [dry_run]

# 发布到 GHCR (GitHub CLI)
just publish-ghcr-gh [dev|prod|alpine] [dry_run]

# 清理本地镜像
just clean-docker
```

### 发布和测试

```bash
# 安装二进制文件
just install [bin_name]

# 发布所有包
just publish

# 测试单个包
just test-convertor
just test-convd
just test-confly
```

## 🔧 环境参数

| 参数 | 说明 | Cargo Profile | 前端环境 | 镜像仓库 |
|------|------|---------------|----------|----------|
| `dev` | 开发环境 | `dev` | `development` | `local` |
| `prod` | 生产环境 | `release` | `production` | `ghcr.io/bppleman/convertor` |
| `alpine` | Alpine 环境 | `alpine` | `production` | `ghcr.io/bppleman/convertor` |

## 🛠️ 直接使用脚本

脚本也可以直接使用，不依赖 `just`：

```bash
# 构建脚本
./scripts/build.sh convd prod
./scripts/build.sh dashboard production

# Linux 构建
./scripts/linux.sh musl alpine
./scripts/linux.sh prepare

# Docker 管理
./scripts/docker.sh image prod
./scripts/docker.sh publish-gh alpine false

# 工具脚本
./scripts/tools.sh test-all
./scripts/tools.sh status
```

## 📊 日志功能

所有脚本都内置了丰富的日志功能：

- **INFO** (蓝色): 一般信息
- **WARN** (黄色): 警告信息
- **ERROR** (红色): 错误信息
- **SUCCESS** (绿色): 成功信息
- **DEBUG** (紫色): 调试信息 (需要设置 `DEBUG=true`)

```bash
# 启用调试日志
DEBUG=true ./scripts/build.sh convd dev
```

## 🔄 预览模式

Docker 发布命令支持预览模式，可以在不实际执行的情况下查看将要执行的操作：

```bash
# 预览发布操作
just publish-ghcr prod true
just publish-ghcr-gh alpine true
```

## ⚡ 特殊功能

### 环境变量支持

- `CR_PAT`: GitHub Personal Access Token (用于 GHCR 发布)
- `DEBUG`: 启用调试日志
- `REDIS_*`: Redis 相关配置 (容器运行时)

### 自动检查

脚本会自动检查：
- 必要的命令是否可用
- Docker 是否运行
- 二进制文件是否存在
- 环境变量是否设置

### 错误处理

- 使用 `set -euo pipefail` 严格错误处理
- 自动捕获错误并显示行号
- 友好的错误提示和解决建议

## 🔧 依赖工具

### 必需
- `cargo` - Rust 构建工具
- `pnpm` - 前端包管理器

### 可选 (按需安装)
- `zig` + `cargo-zigbuild` - MUSL 静态构建
- `cross` - 交叉编译
- `docker` - 容器化
- `gh` - GitHub CLI (推荐的 GHCR 发布方式)

使用 `just prepare` 可以自动安装大部分依赖。