# 参数转换说明

## 概述

本文档说明构建系统中参数转换的逻辑，特别是用户输入参数与 Cargo 构建系统之间的映射关系。

## 参数映射表

| 用户输入 | CARGO_PROFILE (构建命令) | PROFILE (目标路径) | DASHBOARD | REGISTRY |
|----------|-------------------------|-------------------|-----------|----------|
| `dev` / `development` | `dev` | `debug` | `development` | `local` |
| `prod` / `production` / `release` | `release` | `release` | `production` | `ghcr.io/bppleman/convertor` |
| `alpine` | `alpine` | `alpine` | `production` | `ghcr.io/bppleman/convertor` |

## 关键点

### 1. 开发环境的特殊映射
- **输入**: `dev`
- **构建命令**: `cargo build --profile dev`
- **输出路径**: `./target/debug/` 或 `./target/x86_64-unknown-linux-musl/debug/`

这是因为 Cargo 的默认行为：
- `--profile dev` 等同于不指定 profile（开发模式）
- 输出目录使用 `debug` 而不是 `dev`

### 2. 生产环境的一致映射
- **输入**: `prod`
- **构建命令**: `cargo build --profile release`
- **输出路径**: `./target/release/`

### 3. Alpine 环境的自定义 Profile
- **输入**: `alpine`
- **构建命令**: `cargo build --profile alpine`
- **输出路径**: `./target/alpine/`
- **注意**: 需要在 `Cargo.toml` 中定义 `[profile.alpine]`

## 路径结构

### 本机构建
```
target/
├── debug/           # dev 环境输出
├── release/         # prod 环境输出
└── alpine/          # alpine 环境输出（如果支持本机构建）
```

### 跨平台构建（MUSL）
```
target/x86_64-unknown-linux-musl/
├── debug/           # dev 环境 MUSL 输出
├── release/         # prod 环境 MUSL 输出
└── alpine/          # alpine 环境 MUSL 输出
```

## 函数说明

### `convert_profile()`
负责将用户输入的环境参数转换为内部使用的变量：
- 设置 `CARGO_PROFILE`：用于 `cargo build --profile` 命令
- 设置 `PROFILE`：用于构建输出路径
- 设置 `DASHBOARD`：前端构建环境
- 设置 `REGISTRY`：Docker 镜像注册表

### `setup_component_env()`
调用 `convert_profile()` 并为特定组件设置环境变量。

### `check_build_result()`
检查指定环境和目标的构建结果，支持：
- `native`：本机构建（路径：`./target/$PROFILE/`）
- 其他目标：跨平台构建（路径：`./target/$TARGET/$PROFILE/`）

## 使用示例

```bash
# 开发环境
just build convd dev                    # -> target/debug/convd
just build convd dev x86_64-unknown-linux-musl  # -> target/x86_64-unknown-linux-musl/debug/convd

# 生产环境
just build convd prod                   # -> target/release/convd
just musl prod                          # -> target/x86_64-unknown-linux-musl/release/convd

# Alpine 环境
just musl alpine                        # -> target/x86_64-unknown-linux-musl/alpine/convd

# 检查构建结果
just check native dev convd             # 检查 ./target/debug/convd
just check x86_64-unknown-linux-musl prod convd  # 检查 ./target/x86_64-unknown-linux-musl/release/convd
```

## 注意事项

1. **dev vs debug**: 用户使用 `dev`，但路径使用 `debug`
2. **native 处理**: `native` 不是真实的目标名，需要特殊处理为本机路径
3. **环境一致性**: 确保所有脚本都使用相同的参数转换逻辑
4. **错误处理**: 不支持的环境参数会返回错误并显示帮助信息