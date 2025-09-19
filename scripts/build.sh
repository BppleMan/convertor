#!/bin/bash

# 构建、测试和发布相关脚本

# 导入通用模块
LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/lib" && pwd)"
source "$LIB_DIR/log.sh"
source "$LIB_DIR/execute.sh"
source "$LIB_DIR/fs.sh"
source "$LIB_DIR/config.sh"

# ╭──────────────────────────────────────────────╮
# │                   构建功能                   │
# ╰──────────────────────────────────────────────╯

# 统一构建函数
# 参数: component, profile, target_triple
build_component() {
    local component="${1:-convd}"
    local profile="${2:-dev}"
    local target_triple="${3:-}"
    
    log_info "开始构建 $component (环境: $profile, 目标: ${target_triple:-native})"
    
    setup_component_env "$component" "$profile" || return 1
    ensure_project_root
    
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

# 构建 convd (包含前端)
build_convd_with_dashboard() {
    local profile="${1:-dev}"
    local target_triple="${2:-}"
    
    setup_component_env "convd" "$profile" || return 1
    
    # 先构建前端
    build_dashboard "$DASHBOARD" || return 1
    
    # 再构建 convd
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
    
    build_convd_with_dashboard "$profile" "x86_64-unknown-linux-musl"
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

# 检查构建结果
check_build_result() {
    local target="${1:-x86_64-unknown-linux-musl}"
    local profile="${2:-dev}"
    local bin_name="${3:-convd}"
    
    # 转换 profile 以获取正确的目录名
    convert_profile "$profile" || return 1
    
    # 处理 native 目标（本机构建）
    if [[ "$target" == "native" ]]; then
        local binary_path="./target/$PROFILE/$bin_name"
    else
        local binary_path="./target/$target/$PROFILE/$bin_name"
    fi
    
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
    
    # 检查前端依赖
    if [[ -d "dashboard" ]]; then
        if ! command -v pnpm >/dev/null 2>&1; then
            if command -v npm >/dev/null 2>&1; then
                execute_with_log "安装 pnpm" "npm install -g pnpm"
            else
                log_warn "请先安装 Node.js 和 npm"
            fi
        else
            log_info "pnpm 已安装"
        fi
    fi
    
    log_success "构建环境准备完成"
}

# ╭──────────────────────────────────────────────╮
# │                   测试功能                   │
# ╰──────────────────────────────────────────────╯

# 测试 convertor
test_convertor() {
    log_info "测试 convertor 包"
    
    ensure_project_root
    check_command "cargo" || return 1
    
    execute_with_log "测试 convertor" \
        "cargo insta test -p convertor --features=testkit"
}

# 测试 convd
test_convd() {
    log_info "测试 convd 包"
    
    ensure_project_root
    check_command "cargo" || return 1
    
    execute_with_log "测试 convd" \
        "cargo insta test -p convd"
}

# 测试 confly
test_confly() {
    log_info "测试 confly 包"
    
    ensure_project_root
    check_command "cargo" || return 1
    
    execute_with_log "测试 confly" \
        "cargo insta test -p confly"
}

# 运行所有测试
test_all() {
    log_info "运行所有测试"
    
    test_convertor || return 1
    test_convd || return 1
    test_confly || return 1
    
    log_success "所有测试完成"
}

# ╭──────────────────────────────────────────────╮
# │                   发布功能                   │
# ╰──────────────────────────────────────────────╯

# 安装二进制文件
install_binary() {
    local bin_name="${1:-convd}"
    
    log_info "安装二进制文件: $bin_name"
    
    ensure_project_root
    check_command "cargo" || return 1
    
    execute_with_log "安装 $bin_name" \
        "cargo install --bin $bin_name --path ."
}

# 发布所有包
publish_all() {
    log_info "开始发布所有包"
    
    ensure_project_root
    
    # 按顺序发布，convertor 是基础库
    publish_convertor || return 1
    publish_convd || return 1
    publish_confly || return 1
    
    log_success "所有包发布完成"
}

# 发布 convertor 包
publish_convertor() {
    log_info "发布 convertor 包"
    
    ensure_project_root
    check_command "cargo" || return 1
    
    execute_with_log "发布 convertor" \
        "cargo publish -p convertor"
}

# 发布 convd 包
publish_convd() {
    log_info "发布 convd 包"
    
    ensure_project_root
    check_command "cargo" || return 1
    
    # 确保前端资源已构建（同时构建开发和生产版本）
    build_dashboard "development" || return 1
    build_dashboard "production" || return 1
    
    execute_with_log "发布 convd" \
        "cargo publish -p convd"
}

# 发布 confly 包
publish_confly() {
    log_info "发布 confly 包"
    
    ensure_project_root
    check_command "cargo" || return 1
    
    execute_with_log "发布 confly" \
        "cargo publish -p confly"
}

# ╭──────────────────────────────────────────────╮
# │                   快速命令                   │
# ╰──────────────────────────────────────────────╯

# 开发环境快速构建
build_dev_env() {
    log_info "构建开发环境"
    
    # 构建 MUSL 二进制（包含前端）
    build_musl "dev" || return 1
    
    log_success "开发环境构建完成"
}

# 生产环境完整构建
build_prod_env() {
    log_info "构建生产环境"
    
    # 构建 MUSL 二进制（包含前端）
    build_musl "alpine" || return 1
    
    log_success "生产环境构建完成"
}

# 显示项目状态
show_status() {
    log_info "项目状态概览"
    
    printf "\\033[0;36m构建状态:\\033[0m\\n"
    
    # 检查本机二进制文件
    for target in "debug" "release"; do
        for bin in "convd" "confly"; do
            local path="./target/$target/$bin"
            if [[ -f "$path" ]]; then
                local size=$(ls -lh "$path" | awk '{print $5}')
                echo "  ✓ $target/$bin ($size)"
            else
                echo "  ✗ $target/$bin"
            fi
        done
    done
    
    # 检查跨平台二进制文件
    for target in "debug" "release" "alpine"; do
        for bin in "convd" "confly"; do
            local path="./target/x86_64-unknown-linux-musl/$target/$bin"
            if [[ -f "$path" ]]; then
                local size=$(ls -lh "$path" | awk '{print $5}')
                echo "  ✓ musl/$target/$bin ($size)"
            else
                echo "  ✗ musl/$target/$bin"
            fi
        done
    done
    
    echo ""
    printf "\\033[0;36m前端资源:\\033[0m\\n"
    for env in "development" "production"; do
        local path="./crates/convd/assets/$env"
        if [[ -d "$path" ]]; then
            echo "  ✓ $env"
        else
            echo "  ✗ $env"
        fi
    done
}

# ╭──────────────────────────────────────────────╮
# │                   帮助信息                   │
# ╰──────────────────────────────────────────────╯

show_build_help() {
    show_help "build.sh" "构建、测试和发布脚本" "build.sh <command> [args...]"
    
    printf "\033[1;33m构建命令:\033[0m\n"
    echo "  all [profile] [target]       - 构建所有组件"
    echo "  convertor [profile] [target] - 构建 convertor 库"
    echo "  convd [profile] [target]     - 构建 convd"
    echo "  confly [profile] [target]    - 构建 confly"
    echo "  dashboard [profile]          - 构建前端界面"
    echo "  musl [profile]               - MUSL 静态构建"
    echo "  prepare                      - 准备构建环境"
    echo "  check <target> <profile> [bin] - 检查构建结果"
    echo ""
    printf "\033[1;33m测试命令:\033[0m\n"
    echo "  test-all            - 运行所有测试"
    echo "  test-convertor      - 测试 convertor 包"
    echo "  test-convd          - 测试 convd 包"
    echo "  test-confly         - 测试 confly 包"
    echo ""
    printf "\033[1;33m发布命令:\033[0m\n"
    echo "  install [bin_name]    - 安装二进制文件 (默认: convd)"
    echo "  publish-all          - 发布所有包"
    echo "  publish-convertor    - 发布 convertor 包"
    echo "  publish-convd        - 发布 convd 包"
    echo "  publish-confly       - 发布 confly 包"
    echo ""
    printf "\033[1;33m快速命令:\033[0m\n"
    echo "  build-dev           - 构建开发环境"
    echo "  build-prod          - 构建生产环境"
    echo "  status              - 显示项目状态"
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
    echo "  build.sh test-all"
    echo "  build.sh publish-convd"
}

# ╭──────────────────────────────────────────────╮
# │                   主函数                     │
# ╰──────────────────────────────────────────────╯

main() {
    set_error_handling
    
    local command="${1:-}"
    shift || true
    
    case "$command" in
        # 构建命令
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
        "prepare")
            prepare_build_env
            ;;
        "check")
            check_build_result "$@"
            ;;
        # 测试命令
        "test-all")
            test_all
            ;;
        "test-convertor")
            test_convertor
            ;;
        "test-convd")
            test_convd
            ;;
        "test-confly")
            test_confly
            ;;
        # 发布命令
        "install")
            install_binary "$@"
            ;;
        "publish-all")
            publish_all
            ;;
        "publish-convertor")
            publish_convertor
            ;;
        "publish-convd")
            publish_convd
            ;;
        "publish-confly")
            publish_confly
            ;;
        # 快速命令
        "build-dev")
            build_dev_env
            ;;
        "build-prod")
            build_prod_env
            ;;
        "status")
            show_status
            ;;
        # 帮助
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