#!/bin/bash

# 构建、测试和发布相关脚本

# 导入通用模块
LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/lib" && pwd)"
source "$LIB_DIR/log.sh"
source "$LIB_DIR/execute.sh"
source "$LIB_DIR/fs.sh"
source "$LIB_DIR/config.sh"

# ╭──────────────────────────────────────────────╮
# │                   核心功能                   │
# ╰──────────────────────────────────────────────╯

# 统一构建函数
# 参数: package, target_triple, profile
build_component() {
    local package="${1:-convd}"
    local target_triple="${2:-}"
    local profile="${3:-dev}"

    log_info "开始构建 $package (环境: $profile, 目标: ${target_triple:-native})"

    setup_component_env "$package" "$profile" || return 1
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
    case "$package" in
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
            log_error "未知组件: $package"
            return 1
            ;;
    esac

    # 添加 profile 参数
    build_args="$build_args --profile $CARGO_PROFILE"

    # 执行构建
    execute_with_log "构建 $package" \
        "time cargo $build_cmd_suffix $build_args"
}

# 统一测试函数
# 参数: package
test() {
    local package="${1:-all}"

    log_info "测试 $package 包"

    ensure_project_root
    check_command "cargo" || return 1

    case "$package" in
        "all")
            execute_with_log "测试 convertor" \
                "cargo insta test -p convertor --features=testkit"
            execute_with_log "测试 convd" \
                "cargo insta test -p convd"
            execute_with_log "测试 confly" \
                "cargo insta test -p confly"
            ;;
        "convertor")
            execute_with_log "测试 convertor" \
                "cargo insta test -p convertor --features=testkit"
            ;;
        "convd")
            execute_with_log "测试 convd" \
                "cargo insta test -p convd"
            ;;
        "confly")
            execute_with_log "测试 confly" \
                "cargo insta test -p confly"
            ;;
        *)
            log_error "未知包: $package"
            return 1
            ;;
    esac
}

# 统一发布函数
# 参数: package, dry_run
publish() {
    local package="${1:-}"
    local dry_run="${2:-false}"

    if [[ -z "$package" ]]; then
        log_error "请指定要发布的包: convertor, convd, confly"
        return 1
    fi

    log_info "发布 $package 包 (dry_run: $dry_run)"

    ensure_project_root
    check_command "cargo" || return 1

    local publish_args=""
    if [[ "$dry_run" == "true" ]]; then
        publish_args="--dry-run"
    fi

    case "$package" in
        "convertor")
            execute_with_log "发布 convertor" \
                "cargo publish -p convertor $publish_args"
            ;;
        "convd")
            # 确保前端资源已构建（构建开发和生产版本）
            build_dashboard "development" || return 1
            build_dashboard "production" || return 1
            execute_with_log "发布 convd" \
                "cargo publish -p convd $publish_args"
            ;;
        "confly")
            execute_with_log "发布 confly" \
                "cargo publish -p confly $publish_args"
            ;;
        *)
            log_error "未知包: $package"
            return 1
            ;;
    esac
}

# 构建前端界面
build_dashboard() {
    local dashboard_config="${1:-development}"

    log_info "开始构建前端界面 (配置: $dashboard_config)"

    ensure_project_root

    # 检查 pnpm 是否可用
    check_command "pnpm" || return 1

    # 进入前端目录并安装依赖
    execute_with_log "安装前端依赖" \
        "(cd dashboard && pnpm install)"

    # 构建前端
    execute_with_log "构建前端应用" \
        "(cd dashboard && pnpm ng build --configuration $dashboard_config)"

    # 复制构建结果
    local target_dir="./crates/convd/assets/$dashboard_config"
    execute_with_log "清理旧的前端资源" \
        "rm -rf $target_dir"

    execute_with_log "复制前端资源" \
        "cp -rf ./dashboard/dist/dashboard/$dashboard_config/browser $target_dir"

    log_success "前端构建完成: $target_dir"
}

# ╭──────────────────────────────────────────────╮
# │                   工具功能                   │
# ╰──────────────────────────────────────────────╯

# 安装二进制文件
install() {
    local bin_name="${1:-convd}"

    log_info "安装二进制文件: $bin_name"

    ensure_project_root
    check_command "cargo" || return 1

    execute_with_log "安装 $bin_name" \
        "cargo install --bin $bin_name --path ."
}

# 检查构建结果
check() {
    local target="${1:-x86_64-unknown-linux-musl}"
    local profile="${2:-dev}"
    local bin_name="${3:-convd}"

    # 转换 profile 以获取正确的目录名
    map_profile_to_configs "$profile" || return 1

    # 处理 native 目标（本机构建）
    if [[ "$target" == "native" ]]; then
        local binary_path="./target/$CARGO_PATH/$bin_name"
    else
        local binary_path="./target/$target/$CARGO_PATH/$bin_name"
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
prepare() {
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

# 显示项目状态
status() {
    log_info "项目状态概览"

    printf "\\033[0;36m构建状态:\\033[0m\\n"

    # 检查本机二进制文件
    for cargo_path in "debug" "release"; do
        for bin in "convd" "confly"; do
            local path="./target/$cargo_path/$bin"
            if [[ -f "$path" ]]; then
                local size=$(ls -lh "$path" | awk '{print $5}')
                echo "  ✓ $cargo_path/$bin ($size)"
            else
                echo "  ✗ $cargo_path/$bin"
            fi
        done
    done

    # 检查跨平台二进制文件
    for cargo_path in "debug" "release" "alpine"; do
        for bin in "convd" "confly"; do
            local path="./target/x86_64-unknown-linux-musl/$cargo_path/$bin"
            if [[ -f "$path" ]]; then
                local size=$(ls -lh "$path" | awk '{print $5}')
                echo "  ✓ musl/$cargo_path/$bin ($size)"
            else
                echo "  ✗ musl/$cargo_path/$bin"
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

help() {
    show_help "build.sh" "构建、测试和发布脚本" "build.sh <command> [args...]"

    printf "\033[1;33m核心功能:\033[0m\n"
    echo "  build_component <package> [target] [profile] - 构建组件"
    echo "  test [package]                              - 运行测试 (默认: all)"
    echo "  publish <package> [dry_run]                 - 发布包"
    echo "  build_dashboard [profile]                   - 构建前端"
    echo ""
    printf "\033[1;33m工具功能:\033[0m\n"
    echo "  install [bin_name]       - 安装二进制文件"
    echo "  check [target] [profile] [bin] - 检查构建结果"
    echo "  prepare                  - 准备构建环境"
    echo "  status                   - 显示项目状态"
    echo ""
    printf "\033[1;33m参数说明:\033[0m\n"
    echo "  package: all, convertor, convd, confly"
    echo "  target:  x86_64-unknown-linux-musl, native (default)"
    echo "  profile: dev, prod, alpine"
    echo "  dry_run: true, false (default)"
    echo ""
    printf "\033[1;33m示例:\033[0m\n"
    echo "  build.sh build_component convd x86_64-unknown-linux-musl prod"
    echo "  build.sh test convertor"
    echo "  build.sh publish convd true"
    echo "  build.sh build_dashboard production"
}

# ╭──────────────────────────────────────────────╮
# │                   主函数                     │
# ╰──────────────────────────────────────────────╯

main() {
    set_error_handling

    local command="${1:-}"
    shift || true

    case "$command" in
        # 核心功能
        "build_component")
            build_component "$@"
            ;;
        "test")
            test "$@"
            ;;
        "publish")
            publish "$@"
            ;;
        "build_dashboard")
            build_dashboard "$@"
            ;;
        # 工具功能
        "install")
            install "$@"
            ;;
        "check")
            check "$@"
            ;;
        "prepare")
            prepare
            ;;
        "status")
            status
            ;;
        # 帮助
        "help"|"-h"|"--help"|"")
            help
            ;;
        *)
            log_error "未知命令: $command"
            help
            exit 1
            ;;
    esac
}

# 如果直接执行此脚本
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
