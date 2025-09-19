#!/bin/bash

# 发布和测试相关脚本

# 导入通用函数和配置
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

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
    source "$(dirname "${BASH_SOURCE[0]}")/build.sh"
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

# 准备开发环境
prepare_dev() {
    log_info "准备开发环境"
    
    ensure_project_root
    
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
    
    log_success "开发环境准备完成"
}

# 完整的开发环境构建
build_dev_env() {
    log_info "构建开发环境"
    
    # 构建 MUSL 二进制（包含前端）
    source "$(dirname "${BASH_SOURCE[0]}")/build.sh"
    build_musl "dev" || return 1
    
    # 构建 Docker 镜像
    source "$(dirname "${BASH_SOURCE[0]}")/docker.sh"
    build_image "dev" || return 1
    
    log_success "开发环境构建完成"
}

# 完整的生产环境构建
build_prod_env() {
    log_info "构建生产环境"
    
    # 构建 MUSL 二进制（包含前端）
    source "$(dirname "${BASH_SOURCE[0]}")/build.sh"
    build_musl "alpine" || return 1
    
    # 构建 Docker 镜像
    source "$(dirname "${BASH_SOURCE[0]}")/docker.sh"
    build_image "alpine" || return 1
    
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
    
    echo ""
    printf "\\033[0;36mDocker 镜像:\\033[0m\\n"
    docker images | grep -E "(convertor|convd)" | sed 's/^/  /' || echo "  无相关镜像"
}

# 显示帮助
show_tools_help() {
    show_help "tools.sh" "发布和测试工具脚本" "tools.sh <command> [args...]"
    
    printf "\033[1;33m安装命令:\033[0m\n"
    echo "  install [bin_name]    - 安装二进制文件 (默认: convd)"
    echo ""
    printf "\033[1;33m发布命令:\033[0m\n"
    echo "  publish-all          - 发布所有包"
    echo "  publish-convertor    - 发布 convertor 包"
    echo "  publish-convd        - 发布 convd 包"
    echo "  publish-confly       - 发布 confly 包"
    echo ""
    printf "\033[1;33m测试命令:\033[0m\n"
    echo "  test-all            - 运行所有测试"
    echo "  test-convertor      - 测试 convertor 包"
    echo "  test-convd          - 测试 convd 包"
    echo "  test-confly         - 测试 confly 包"
    echo ""
    printf "\033[1;33m环境命令:\033[0m\n"
    echo "  prepare             - 准备开发环境"
    echo "  build-dev           - 构建开发环境"
    echo "  build-prod          - 构建生产环境"
    echo "  status              - 显示项目状态"
    echo ""
}

# 主函数
main() {
    set_error_handling
    
    local command="${1:-}"
    shift || true
    
    case "$command" in
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
        "prepare")
            prepare_dev
            ;;
        "build-dev")
            build_dev_env
            ;;
        "build-prod")
            build_prod_env
            ;;
        "status")
            show_status
            ;;
        "help"|"-h"|"--help"|"")
            show_tools_help
            ;;
        *)
            log_error "未知命令: $command"
            show_tools_help
            exit 1
            ;;
    esac
}

# 如果直接执行此脚本
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi