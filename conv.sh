#!/bin/bash

# Convertor 项目统一入口脚本
# 这是唯一的事实来源，所有构建、测试、发布和部署功能都在这里定义

# 获取脚本目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 导入通用模块
LIB_DIR="$SCRIPT_DIR/scripts/lib"
source "$LIB_DIR/log.sh"
source "$LIB_DIR/execute.sh"
source "$LIB_DIR/fs.sh"
source "$LIB_DIR/config.sh"

# ╭──────────────────────────────────────────────╮
# │                   快速命令                   │
# ╰──────────────────────────────────────────────╯

# 快速开发环境构建
build-dev() {
    log_info "快速开发环境构建"
    # 先构建前端
    "$SCRIPT_DIR/scripts/build.sh" build_dashboard development
    # 构建 convd 二进制 (MUSL)
    "$SCRIPT_DIR/scripts/build.sh" build_component convd x86_64-unknown-linux-musl dev
}

# 快速生产环境构建
build-prod() {
    log_info "快速生产环境构建"
    # 先构建前端
    "$SCRIPT_DIR/scripts/build.sh" build_dashboard production
    # 构建 convd 二进制 (MUSL)
    "$SCRIPT_DIR/scripts/build.sh" build_component convd x86_64-unknown-linux-musl alpine
}

# 准备开发环境
prepare() {
    log_info "准备开发环境"
    "$SCRIPT_DIR/scripts/build.sh" prepare
}

# ╭──────────────────────────────────────────────╮
# │                   构建功能                   │
# ╰──────────────────────────────────────────────╯

# 构建所有组件
all() {
    local profile="${1:-dev}"
    "$SCRIPT_DIR/scripts/build.sh" build_component all "" "$profile"
}

# 构建 convertor 库
convertor() {
    local profile="${1:-dev}"
    "$SCRIPT_DIR/scripts/build.sh" build_component convertor "" "$profile"
}

# 构建 convd
convd() {
    local profile="${1:-dev}"
    "$SCRIPT_DIR/scripts/build.sh" build_component convd "" "$profile"
}

# 构建 confly
confly() {
    local profile="${1:-dev}"
    "$SCRIPT_DIR/scripts/build.sh" build_component confly "" "$profile"
}

# 构建指定组件和目标
build() {
    local component="${1:-convd}"
    local profile="${2:-dev}"
    local target="${3:-}"
    "$SCRIPT_DIR/scripts/build.sh" build_component "$component" "$target" "$profile"
}

# ╭──────────────────────────────────────────────╮
# │                   测试功能                   │
# ╰──────────────────────────────────────────────╯

# 运行测试
test() {
    local package="${1:-all}"
    "$SCRIPT_DIR/scripts/build.sh" test "$package"
}

# ╭──────────────────────────────────────────────╮
# │                 MUSL 构建                    │
# ╰──────────────────────────────────────────────╯

# MUSL 静态构建
musl() {
    local profile="${1:-dev}"
    "$SCRIPT_DIR/scripts/build.sh" build_component convd x86_64-unknown-linux-musl "$profile"
}

# ╭──────────────────────────────────────────────╮
# │                 前端构建                     │
# ╰──────────────────────────────────────────────╯

# 构建前端界面
dashboard() {
    local profile="${1:-development}"
    "$SCRIPT_DIR/scripts/build.sh" build_dashboard "$profile"
}

# ╭──────────────────────────────────────────────╮
# │                 发布功能                     │
# ╰──────────────────────────────────────────────╯

# 安装二进制文件
install() {
    local bin="${1:-convd}"
    "$SCRIPT_DIR/scripts/build.sh" install "$bin"
}

# 发布包
publish() {
    local package="${1:-}"
    local dry_run="${2:-false}"
    if [[ -z "$package" ]]; then
        log_error "请指定要发布的包: convertor, convd, confly"
        return 1
    fi
    "$SCRIPT_DIR/scripts/build.sh" publish "$package" "$dry_run"
}

# ╭──────────────────────────────────────────────╮
# │                 Docker 功能                  │
# ╰──────────────────────────────────────────────╯

# 构建镜像
image() {
    local profile="${1:-dev}"
    "$SCRIPT_DIR/scripts/docker.sh" image "$profile"
}

# 运行镜像
run() {
    local profile="${1:-dev}"
    "$SCRIPT_DIR/scripts/docker.sh" run "$profile"
}

# 发布到 GHCR (PAT)
publish-ghcr() {
    local profile="${1:-dev}"
    local dry_run="${2:-false}"
    "$SCRIPT_DIR/scripts/docker.sh" publish-ghcr "$profile" "$dry_run"
}

# 发布到 GHCR (GitHub CLI)
publish-ghcr-gh() {
    local profile="${1:-dev}"
    local dry_run="${2:-false}"
    "$SCRIPT_DIR/scripts/docker.sh" publish-gh "$profile" "$dry_run"
}

# 清理 Docker 镜像
clean-docker() {
    "$SCRIPT_DIR/scripts/docker.sh" clean
}

# ╭──────────────────────────────────────────────╮
# │                 实用工具                     │
# ╰──────────────────────────────────────────────╯

# 显示项目状态
status() {
    "$SCRIPT_DIR/scripts/build.sh" status
}

# 检查构建结果
check() {
    local target="${1:-x86_64-unknown-linux-musl}"
    local profile="${2:-dev}"
    local bin="${3:-convd}"
    "$SCRIPT_DIR/scripts/build.sh" check "$target" "$profile" "$bin"
}

# ╭──────────────────────────────────────────────╮
# │                   帮助信息                   │
# ╰──────────────────────────────────────────────╯

# 显示帮助信息
help() {
    show_help "conv.sh" "Convertor 项目构建系统" "conv.sh <command> [args...]"
    
    printf "\033[1;33m快速命令:\033[0m\n"
    echo "  build-dev            - 构建开发环境"
    echo "  build-prod           - 构建生产环境"
    echo "  prepare              - 准备开发环境"
    echo "  status               - 显示项目状态"
    echo ""
    printf "\033[1;33m构建命令:\033[0m\n"
    echo "  all [profile]        - 构建所有组件"
    echo "  convertor [profile]  - 构建 convertor 库"
    echo "  convd [profile]      - 构建 convd"
    echo "  confly [profile]     - 构建 confly"
    echo "  dashboard [profile]  - 构建前端界面"
    echo "  musl [profile]       - MUSL 静态构建"
    echo "  build <component> <profile> <target> - 构建指定组件"
    echo ""
    printf "\033[1;33m测试命令:\033[0m\n"
    echo "  test [package]       - 运行测试 (默认: all)"
    echo ""
    printf "\033[1;33m发布命令:\033[0m\n"
    echo "  install [bin]        - 安装二进制文件"
    echo "  publish <package> [dry_run] - 发布包"
    echo ""
    printf "\033[1;33mDocker 命令:\033[0m\n"
    echo "  image [profile]      - 构建 Docker 镜像"
    echo "  run [profile]        - 运行 Docker 容器"
    echo "  publish-ghcr [profile] [dry_run] - 发布到 GHCR (PAT)"
    echo "  publish-ghcr-gh [profile] [dry_run] - 发布到 GHCR (GitHub CLI)"
    echo "  clean-docker         - 清理 Docker 镜像"
    echo ""
    printf "\033[1;33m工具命令:\033[0m\n"
    echo "  check [target] [profile] [bin] - 检查构建结果"
    echo "  help                 - 显示此帮助信息"
    echo ""
    printf "\033[1;33m环境参数:\033[0m\n"
    echo "  dev, development     - 开发环境"
    echo "  prod, production     - 生产环境"
    echo "  alpine               - Alpine Linux 环境"
    echo ""
    printf "\033[1;33m示例:\033[0m\n"
    echo "  ./conv.sh build-dev         # 快速开发环境构建"
    echo "  ./conv.sh build-prod        # 快速生产环境构建"
    echo "  ./conv.sh convd prod        # 构建生产版本 convd"
    echo "  ./conv.sh test convertor    # 测试 convertor 包"
    echo "  ./conv.sh publish convd     # 发布 convd 包"
    echo "  ./conv.sh image alpine      # 构建 Alpine Docker 镜像"
}

# ╭──────────────────────────────────────────────╮
# │                   主函数                     │
# ╰──────────────────────────────────────────────╯

main() {
    set_error_handling
    
    local command="${1:-}"
    shift || true
    
    case "$command" in
        # 快速命令
        "build-dev")
            build-dev
            ;;
        "build-prod")
            build-prod
            ;;
        "prepare")
            prepare
            ;;
        
        # 构建命令
        "all")
            all "$@"
            ;;
        "convertor")
            convertor "$@"
            ;;
        "convd")
            convd "$@"
            ;;
        "confly")
            confly "$@"
            ;;
        "dashboard")
            dashboard "$@"
            ;;
        "musl")
            musl "$@"
            ;;
        "build")
            build "$@"
            ;;
        
        # 测试命令
        "test")
            test "$@"
            ;;
        
        # 发布命令
        "install")
            install "$@"
            ;;
        "publish")
            publish "$@"
            ;;
        
        # Docker 命令
        "image")
            image "$@"
            ;;
        "run")
            run "$@"
            ;;
        "publish-ghcr")
            publish-ghcr "$@"
            ;;
        "publish-ghcr-gh")
            publish-ghcr-gh "$@"
            ;;
        "clean-docker")
            clean-docker
            ;;
        
        # 工具命令
        "status")
            status
            ;;
        "check")
            check "$@"
            ;;
        "help"|"-h"|"--help"|"")
            help
            ;;
        
        *)
            log_error "未知命令: $command"
            echo ""
            help
            exit 1
            ;;
    esac
}

# 如果直接执行此脚本
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi