#!/usr/bin/env just --justfile

# 快速开发环境构建
build-dev:
    source ./scripts/tools.sh && build_dev_env

# 快速生产环境构建
build-prod:
    source ./scripts/tools.sh && build_prod_env

# 准备开发环境
prepare:
    source ./scripts/build.sh && prepare_build_env

#╭──────────────────────────────────────────────╮
#│                   发布                       │
#╰──────────────────────────────────────────────╯

# 安装二进制文件
install bin="convd":
    source ./scripts/tools.sh && install_binary {{ bin }}

# 发布所有包
publish:
    source ./scripts/tools.sh && publish_all

# 发布 convertor 包
publish-convertor:
    source ./scripts/tools.sh && publish_convertor

# 发布 convd 包
publish-convd:
    source ./scripts/tools.sh && publish_convd

# 发布 confly 包
publish-confly:
    source ./scripts/tools.sh && publish_confly

#╭──────────────────────────────────────────────╮
#│                   构建                       │
#╰──────────────────────────────────────────────╯

# 构建所有组件
all profile="dev":
    source ./scripts/build.sh && build_all {{ profile }}

# 构建 convertor 库
convertor profile="dev":
    source ./scripts/build.sh && build_convertor {{ profile }}

# 构建 convd
convd profile="dev":
    source ./scripts/build.sh && build_convd_with_dashboard {{ profile }}

# 构建 confly
confly profile="dev":
    source ./scripts/build.sh && build_confly {{ profile }}

# 构建指定组件和目标
build component profile="dev" target="native":
    source ./scripts/build.sh && build_component {{ component }} {{ profile }} {{ target }}

#╭──────────────────────────────────────────────╮
#│                   测试                       │
#╰──────────────────────────────────────────────╯

# 测试 convertor
test-convertor:
    source ./scripts/tools.sh && test_convertor

# 测试 convd
test-convd:
    source ./scripts/tools.sh && test_convd

# 测试 confly
test-confly:
    source ./scripts/tools.sh && test_confly

#╭──────────────────────────────────────────────╮
#│                 MUSL 构建                    │
#╰──────────────────────────────────────────────╯

# MUSL 静态构建
musl profile="dev":
    source ./scripts/build.sh && build_musl {{ profile }}

#╭──────────────────────────────────────────────╮
#│                 前端构建                      │
#╰──────────────────────────────────────────────╯

# 构建前端界面
dashboard profile="dev":
    source ./scripts/build.sh && build_dashboard {{ profile }}

#╭──────────────────────────────────────────────╮
#│                 Docker                       │
#╰──────────────────────────────────────────────╯

# 构建镜像
image profile="dev":
    source ./scripts/docker.sh && build_image {{ profile }}

# 运行镜像
run profile="dev":
    source ./scripts/docker.sh && run_container {{ profile }}

# 发布到 GHCR (PAT)
publish-ghcr profile="dev" dry_run="false":
    source ./scripts/docker.sh && publish_ghcr {{ profile }} {{ dry_run }}

# 发布到 GHCR (GitHub CLI)
publish-ghcr-gh profile="dev" dry_run="false":
    source ./scripts/docker.sh && publish_ghcr_gh {{ profile }} {{ dry_run }}

#╭──────────────────────────────────────────────╮
#│                 实用工具                      │
#╰──────────────────────────────────────────────╯

# 显示项目状态
status:
    source ./scripts/tools.sh && show_status

# 清理 Docker 镜像
clean-docker:
    source ./scripts/docker.sh && clean_images

# 检查构建结果
check target="x86_64-unknown-linux-musl" profile="dev" bin="convd":
    source ./scripts/build.sh && check_build_result {{ target }} {{ profile }} {{ bin }}

# 运行所有测试
test-all:
    source ./scripts/tools.sh && test_all

# 显示帮助信息
help:
    @echo "Convertor 项目构建系统"
    @echo ""
    @echo "快速命令:"
    @echo "  just build-dev           - 构建开发环境"
    @echo "  just build-prod          - 构建生产环境"
    @echo "  just status              - 显示项目状态"
    @echo "  just test-all            - 运行所有测试"
    @echo ""
    @echo "详细使用方法:"
    @echo "  just --list              - 显示所有可用命令"
    @echo "  ./scripts/<script>.sh help - 查看脚本帮助"

