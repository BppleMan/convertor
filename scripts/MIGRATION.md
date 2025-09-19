# Scripts 目录结构更新说明

## 新的目录结构

```
scripts/
├── lib/                    # 通用函数库
│   ├── log.sh             # 日志记录功能
│   ├── execute.sh         # 命令执行工具
│   ├── fs.sh              # 文件系统操作
│   └── config.sh          # 配置管理
├── build.sh               # 构建、测试和发布脚本 (统一)
└── docker.sh              # Docker相关脚本
```

## 主要变化

### 1. 通用功能模块化

- **`lib/log.sh`**: 日志记录、颜色输出、帮助信息显示
- **`lib/execute.sh`**: 命令执行、环境检查、错误处理
- **`lib/fs.sh`**: 文件系统操作、项目路径管理
- **`lib/config.sh`**: 环境配置、参数转换、配置验证

### 2. 业务脚本整合

原有的多个脚本文件：
- `common.sh` → 功能分散到 `lib/` 目录
- `config.sh` → 重构为 `lib/config.sh`
- `tools.sh` → 功能合并到 `build.sh`
- `build.sh` → 保留并扩展，包含所有构建、测试、发布功能

### 3. 统一入口

- **`conv.sh`** (项目根目录): 统一入口脚本，是唯一的事实来源
- **`justfile`** (项目根目录): 保留兼容性，但只作为 `conv.sh` 的代理

## 使用方式

### 推荐使用方式 (conv.sh)

```bash
# 查看帮助
./conv.sh help

# 快速构建
./conv.sh build-dev      # 开发环境
./conv.sh build-prod     # 生产环境

# 具体组件构建
./conv.sh convd prod     # 构建生产版本 convd
./conv.sh test-all       # 运行所有测试

# Docker 操作
./conv.sh image alpine   # 构建 Alpine 镜像
./conv.sh run dev        # 运行开发容器
```

### 兼容使用方式 (justfile)

```bash
# 仍然可以使用 just，但实际调用 conv.sh
just build-dev
just status
just test-all
```

### 直接调用脚本 (高级用法)

```bash
# 直接调用具体脚本
./scripts/build.sh help
./scripts/docker.sh help
```

## 优势

1. **简化维护**: 从5个脚本文件简化为2个主要脚本
2. **模块化**: 通用功能在 `lib/` 目录中复用
3. **统一入口**: `conv.sh` 作为唯一事实来源
4. **向后兼容**: `justfile` 仍然可用
5. **清晰结构**: 业务逻辑与通用功能分离

## 迁移指南

如果你之前使用：
- `source ./scripts/xxx.sh && function_name` → `./conv.sh command`
- `just xxx` → 继续使用（无变化）
- 直接调用脚本函数 → 使用 `conv.sh` 对应命令

## 开发者注意事项

- 新功能只需要在 `conv.sh` 中添加
- 通用功能添加到对应的 `lib/` 模块
- `justfile` 只需要添加对 `conv.sh` 的调用代理
- 所有脚本都引用 `lib/` 中的模块，确保一致性