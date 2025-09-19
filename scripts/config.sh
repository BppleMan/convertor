#!/bin/bash

# 参数转换和配置管理

# 导入通用函数
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

# 将环境参数转换为构建配置
# 参数: profile (dev|prod|alpine)
# 输出: 设置 PROFILE, DASHBOARD, REGISTRY 等环境变量
convert_profile() {
    local input_profile="${1:-dev}"
    
    case "$input_profile" in
        "dev"|"development")
            export PROFILE="debug"      # cargo 构建目录使用 debug
            export CARGO_PROFILE="dev"  # cargo 命令使用 dev
            export DASHBOARD="development"
            export REGISTRY="local"
            ;;
        "prod"|"production"|"release")
            export PROFILE="release"
            export CARGO_PROFILE="release"
            export DASHBOARD="production"
            export REGISTRY="ghcr.io/bppleman/convertor"
            ;;
        "alpine")
            export PROFILE="alpine"     # cargo 构建目录使用 alpine
            export CARGO_PROFILE="alpine"  # cargo 命令使用 alpine
            export DASHBOARD="production"
            export REGISTRY="ghcr.io/bppleman/convertor"
            ;;
        *)
            log_error "不支持的环境: $input_profile"
            log_info "支持的环境: dev, prod, alpine"
            return 1
            ;;
    esac
    
    log_debug "环境转换: $input_profile -> PROFILE=$PROFILE, DASHBOARD=$DASHBOARD, REGISTRY=$REGISTRY"
    return 0
}

# 获取Docker相关配置
get_docker_config() {
    export TARGET_TRIPLE="x86_64-unknown-linux-musl"
    export BIN_NAME="convd"
    
    log_debug "Docker配置: TARGET_TRIPLE=$TARGET_TRIPLE, BIN_NAME=$BIN_NAME"
}

# 获取版本信息
get_version() {
    local target_triple="${TARGET_TRIPLE:-x86_64-unknown-linux-musl}"
    local bin_name="${BIN_NAME:-convd}"
    local profile="${PROFILE:-debug}"  # 默认使用 debug 而不是 dev
    
    local binary_path="./target/$target_triple/$profile/$bin_name"
    
    if [[ -f "$binary_path" ]]; then
        docker run --rm -v "$binary_path:/app/$bin_name" alpine:3.20 "/app/$bin_name" tag
    else
        log_error "二进制文件不存在: $binary_path"
        return 1
    fi
}

# 获取构建日期
get_build_date() {
    date +%Y-%m-%dT%H:%M:%S%z
}

# 验证环境配置
validate_environment() {
    local profile="$1"
    
    if [[ -z "$profile" ]]; then
        log_error "必须指定环境参数"
        return 1
    fi
    
    convert_profile "$profile" || return 1
    
    # 检查必要的目录
    if [[ ! -d "dashboard" ]]; then
        log_error "dashboard 目录不存在"
        return 1
    fi
    
    if [[ ! -f "Cargo.toml" ]]; then
        log_error "Cargo.toml 文件不存在，请确保在项目根目录执行"
        return 1
    fi
    
    return 0
}

# 为特定组件设置环境
setup_component_env() {
    local component="$1"
    local profile="$2"
    
    convert_profile "$profile" || return 1
    
    case "$component" in
        "convd")
            get_docker_config
            ;;
        "docker"|"image")
            get_docker_config
            export VERSION="$(get_version)"
            export BUILD_DATE="$(get_build_date)"
            ;;
        *)
            # 通用配置已在 convert_profile 中设置
            ;;
    esac
    
    return 0
}

# 显示当前环境配置
show_environment() {
    log_info "当前环境配置:"
    echo "  PROFILE: ${PROFILE:-未设置}"
    echo "  CARGO_PROFILE: ${CARGO_PROFILE:-未设置}"
    echo "  DASHBOARD: ${DASHBOARD:-未设置}"
    echo "  REGISTRY: ${REGISTRY:-未设置}"
    echo "  TARGET_TRIPLE: ${TARGET_TRIPLE:-未设置}"
    echo "  BIN_NAME: ${BIN_NAME:-未设置}"
    echo "  VERSION: ${VERSION:-未设置}"
    echo "  BUILD_DATE: ${BUILD_DATE:-未设置}"
}