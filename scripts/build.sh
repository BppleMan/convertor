#!/bin/bash

# 构建相关脚本

# 导入通用函数和配置
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

# 统一构建函数
# 参数: component, profile, target_triple
build_component() {
    local component="${1:-convd}"
    local profile="${2:-dev}"
    local target_triple="${3:-}"
    
    log_info "开始构建 $component (环境: $profile, 目标: ${target_triple:-native})"
    
    setup_component_env "$component" "$profile" || return 1
    ensure_project_root
    
    # 为 convd 构建前端（如果需要）
    if [[ "$component" == "convd" ]]; then
        build_dashboard "$DASHBOARD" || return 1
    fi
    
    # 准备构建命令
    local build_cmd_suffix=""
    local build_args=""
    
    # 根据目标平台选择构建工具
    if [[ -n "$target_triple" && "$target_triple" != "native" ]]; then
        # 交叉编译使用 zigbuild
        check_command "zig" || {
            log_error "未找到 zig，请先安装"
            log_info "macOS: brew install zig"
            log_info "或访问: https://ziglang.org/"
            return 1
        }
        
        if ! cargo zigbuild --help >/dev/null 2>&1; then
            log_error "未找到 cargo-zigbuild，请先安装"
            log_info "运行: cargo install cargo-zigbuild"
            return 1
        fi
        
        build_cmd_suffix="zigbuild"
        build_args="--target $target_triple"
    else
        build_cmd_suffix="build"
    fi
    
    # 根据组件类型设置构建参数
    case "$component" in
        "all")
            build_args="$build_args --workspace --all-targets"
            ;;
        "convertor")
            build_args="$build_args --package convertor"
            ;;
        "convd")
            build_args="$build_args --bin convd"
            ;;
        "confly")
            build_args="$build_args --bin confly"
            ;;
        *)
            log_error "未知组件: $component"
            return 1
            ;;
    esac
    
    # 添加 profile 参数
    build_args="$build_args --profile $CARGO_PROFILE"
    
    # 执行构建
    execute_with_log "构建 $component" \
        "time cargo $build_cmd_suffix $build_args"
}

# 构建所有组件
build_all() {
    local profile="${1:-dev}"
    local target_triple="${2:-}"
    
    build_component "all" "$profile" "$target_triple"
}

# 构建 convertor 库
build_convertor() {
    local profile="${1:-dev}"
    local target_triple="${2:-}"
    
    build_component "convertor" "$profile" "$target_triple"
}

# 构建 convd
build_convd() {
    local profile="${1:-dev}"
    local target_triple="${2:-}"
    
    build_component "convd" "$profile" "$target_triple"
}

# 构建 confly
build_confly() {
    local profile="${1:-dev}"
    local target_triple="${2:-}"
    
    build_component "confly" "$profile" "$target_triple"
}

# Linux MUSL 静态构建
build_musl() {
    local profile="${1:-dev}"
    
    build_component "convd" "$profile" "x86_64-unknown-linux-musl"
}

# 构建前端界面
build_dashboard() {
    local profile="${1:-development}"
    
    log_info "开始构建前端界面 (环境: $profile)"
    
    ensure_project_root
    
    # 检查 pnpm 是否可用
    check_command "pnpm" || return 1
    
    # 进入前端目录并安装依赖
    execute_with_log "安装前端依赖" \
        "(cd dashboard && pnpm install)"
    
    # 构建前端
    execute_with_log "构建前端应用" \
        "(cd dashboard && pnpm ng build --configuration $profile)"
    
    # 复制构建结果
    local target_dir="./crates/convd/assets/$profile"
    execute_with_log "清理旧的前端资源" \
        "rm -rf $target_dir"
    
    execute_with_log "复制前端资源" \
        "cp -rf ./dashboard/dist/dashboard/$profile/browser $target_dir"
    
    log_success "前端构建完成: $target_dir"
}

# 开发环境快速构建
build_dev_quick() {
    log_info "开始开发环境快速构建"
    
    build_convd "dev"
}

# 生产环境完整构建（包含 MUSL 静态构建）
build_prod_full() {
    log_info "开始生产环境完整构建"
    
    build_musl "alpine"
}

# 检查构建结果
check_build_result() {
    local target="${1:-x86_64-unknown-linux-musl}"
    local profile="${2:-dev}"
    local bin_name="${3:-convd}"
    
    # 转换 profile 以获取正确的目录名
    convert_profile "$profile" || return 1
    
    local binary_path="./target/$target/$CARGO_PROFILE/$bin_name"
    
    if [[ -f "$binary_path" ]]; then
        local size=$(ls -lh "$binary_path" | awk '{print $5}')
        log_success "构建成功: $binary_path (大小: $size)"
        
        # 显示文件信息
        log_info "文件信息:"
        file "$binary_path" | sed 's/^/  /'
        
        return 0
    else
        log_error "构建失败: 未找到二进制文件 $binary_path"
        return 1
    fi
}

# 环境准备
prepare_build_env() {
    log_info "准备构建环境"
    
    # 检查和安装 cargo-zigbuild
    if ! cargo zigbuild --help >/dev/null 2>&1; then
        execute_with_log "安装 cargo-zigbuild" \
            "cargo install cargo-zigbuild"
    else
        log_info "cargo-zigbuild 已安装"
    fi
    
    # 检查 zig (macOS)
    if [[ "$(uname)" == "Darwin" ]]; then
        if ! command -v zig >/dev/null 2>&1; then
            if command -v brew >/dev/null 2>&1; then
                execute_with_log "安装 zig" "brew install zig"
            else
                log_warn "请手动安装 zig: https://ziglang.org/"
            fi
        else
            log_info "zig 已安装"
        fi
    fi
    
    log_success "构建环境准备完成"
}

# 显示构建帮助
show_build_help() {
    show_help "build.sh" "统一构建脚本" "build.sh <command> [args...]"
    
    printf "\033[1;33m基础命令:\033[0m\n"
    echo "  all [profile] [target]       - 构建所有组件"
    echo "  convertor [profile] [target] - 构建 convertor 库"
    echo "  convd [profile] [target]     - 构建 convd"
    echo "  confly [profile] [target]    - 构建 confly"
    echo "  dashboard [profile]          - 构建前端界面"
    echo "  musl [profile]               - MUSL 静态构建"
    echo ""
    printf "\033[1;33m快速命令:\033[0m\n"
    echo "  dev-quick                    - 开发环境快速构建"
    echo "  prod-full                    - 生产环境完整构建"
    echo "  prepare                      - 准备构建环境"
    echo "  check <target> <profile> [bin] - 检查构建结果"
    echo ""
    printf "\033[1;33m环境参数:\033[0m\n"
    echo "  dev, development             - 开发环境"
    echo "  prod, production             - 生产环境"
    echo "  alpine                       - Alpine Linux 环境"
    echo ""
    printf "\033[1;33m目标平台:\033[0m\n"
    echo "  x86_64-unknown-linux-musl    - Linux MUSL (静态链接)"
    echo "  x86_64-unknown-linux-gnu     - Linux GNU (动态链接)"
    echo "  native 或留空                - 本机平台"
    echo ""
    printf "\033[1;33m示例:\033[0m\n"
    echo "  build.sh convertor dev"
    echo "  build.sh convd prod x86_64-unknown-linux-musl"
    echo "  build.sh musl alpine"
}

# 主函数
main() {
    set_error_handling
    
    local command="${1:-}"
    shift || true
    
    case "$command" in
        "all")
            build_all "$@"
            ;;
        "convertor")
            build_convertor "$@"
            ;;
        "convd")
            build_convd "$@"
            ;;
        "confly")
            build_confly "$@"
            ;;
        "dashboard")
            build_dashboard "$@"
            ;;
        "musl")
            build_musl "$@"
            ;;
        "dev-quick")
            build_dev_quick
            ;;
        "prod-full")
            build_prod_full
            ;;
        "prepare")
            prepare_build_env
            ;;
        "check")
            check_build_result "$@"
            ;;
        "help"|"-h"|"--help"|"")
            show_build_help
            ;;
        *)
            log_error "未知命令: $command"
            show_build_help
            exit 1
            ;;
    esac
}

# 如果直接执行此脚本
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi