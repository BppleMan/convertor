#!/usr/bin/env just --justfile
# Convertor 项目构建系统 (conv.sh 代理模式)
# 注意: 这个文件只是为了兼容 just 用户的代理层
# 真正的实现在 conv.sh 中，这里只是调用转发

# 快速开发环境构建
build-dev:
    ./conv.sh build-dev

# 快速生产环境构建
build-prod:
    ./conv.sh build-prod

# 准备开发环境
prepare:
    ./conv.sh prepare

#╭──────────────────────────────────────────────╮
#│                   发布                       │
#╰──────────────────────────────────────────────╯

# 安装二进制文件
install bin="convd":
    ./conv.sh install {{ bin }}

# 发布所有包
publish:
    ./conv.sh publish

# 发布 convertor 包
publish-convertor:
    ./conv.sh publish-convertor

# 发布 convd 包
publish-convd:
    ./conv.sh publish-convd

# 发布 confly 包
publish-confly:
    ./conv.sh publish-confly

#╭──────────────────────────────────────────────╮
#│                   构建                       │
#╰──────────────────────────────────────────────╯

# 构建所有组件
all profile="dev":
    ./conv.sh all {{ profile }}

# 构建 convertor 库
convertor profile="dev":
    ./conv.sh convertor {{ profile }}

# 构建 convd
convd profile="dev":
    ./conv.sh convd {{ profile }}

# 构建 confly
confly profile="dev":
    ./conv.sh confly {{ profile }}

# 构建指定组件和目标
build component profile="dev" target="native":
    ./conv.sh build {{ component }} {{ profile }} {{ target }}

#╭──────────────────────────────────────────────╮
#│                   测试                       │
#╰──────────────────────────────────────────────╯

# 测试 convertor
test-convertor:
    ./conv.sh test convertor

# 测试 convd
test-convd:
    ./conv.sh test convd

# 测试 confly
test-confly:
    ./conv.sh test confly

#╭──────────────────────────────────────────────╮
#│                 MUSL 构建                    │
#╰──────────────────────────────────────────────╯

# MUSL 静态构建
musl profile="dev":
    ./conv.sh musl {{ profile }}

#╭──────────────────────────────────────────────╮
#│                 前端构建                      │
#╰──────────────────────────────────────────────╯

# 构建前端界面
dashboard profile="dev":
    ./conv.sh dashboard {{ profile }}

#╭──────────────────────────────────────────────╮
#│                 Docker                       │
#╰──────────────────────────────────────────────╯

# 构建镜像
image profile="dev":
    ./conv.sh image {{ profile }}

# 运行镜像
run profile="dev":
    ./conv.sh run {{ profile }}

# 发布到 GHCR (PAT)
publish-ghcr profile="dev" dry_run="false":
    ./conv.sh publish-ghcr {{ profile }} {{ dry_run }}

# 发布到 GHCR (GitHub CLI)
publish-ghcr-gh profile="dev" dry_run="false":
    ./conv.sh publish-ghcr-gh {{ profile }} {{ dry_run }}

#╭──────────────────────────────────────────────╮
#│                 实用工具                      │
#╰──────────────────────────────────────────────╯

# 显示项目状态
status:
    ./conv.sh status

# 清理 Docker 镜像
clean-docker:
    ./conv.sh clean-docker

# 检查构建结果
check target="x86_64-unknown-linux-musl" profile="dev" bin="convd":
    ./conv.sh check {{ target }} {{ profile }} {{ bin }}

# 运行所有测试
test-all:
    ./conv.sh test-all

# 显示帮助信息
help:
    @echo "Convertor 项目构建系统 (Just 代理模式)"
    @echo ""
    @echo "注意: 这是 conv.sh 的代理层，实际功能请参考："
    @echo "  ./conv.sh help"
    @echo ""
    @echo "快速命令:"
    @echo "  just build-dev           - 构建开发环境"
    @echo "  just build-prod          - 构建生产环境"
    @echo "  just status              - 显示项目状态"
    @echo "  just test-all            - 运行所有测试"
    @echo ""
    @echo "详细使用方法:"
    @echo "  just --list              - 显示所有可用命令"
    @echo "  ./conv.sh help           - 查看完整帮助信息"
