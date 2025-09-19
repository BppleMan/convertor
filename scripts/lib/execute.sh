#!/bin/bash

# 命令执行工具模块

# 导入日志模块
source "$(dirname "${BASH_SOURCE[0]}")/log.sh"

# 执行命令并记录日志
execute_with_log() {
    local description="$1"
    shift
    local cmd="$*"
    
    log_info "正在执行: $description"
    log_debug "命令: $cmd"
    
    if eval "$cmd"; then
        log_success "$description 完成"
        return 0
    else
        log_error "$description 失败"
        return 1
    fi
}

# 检查命令是否存在
check_command() {
    local cmd="$1"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        log_error "命令 '$cmd' 未找到，请先安装"
        return 1
    fi
    return 0
}

# 检查Docker是否运行
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker 未运行或无权限访问"
        return 1
    fi
    return 0
}

# 检查必需的环境变量
check_required_env() {
    local vars=("$@")
    local missing=()
    
    for var in "${vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            missing+=("$var")
        fi
    done
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "缺少必需的环境变量: ${missing[*]}"
        return 1
    fi
    
    return 0
}

# 设置脚本的错误处理
set_error_handling() {
    set -euo pipefail
    
    # 捕获错误并记录
    trap 'log_error "脚本在第 $LINENO 行出错，退出码: $?"' ERR
}

# 验证参数数量
validate_args() {
    local expected="$1"
    local actual="$2"
    local usage="$3"
    
    if [[ "$actual" -lt "$expected" ]]; then
        log_error "参数数量不足，期望至少 $expected 个，实际 $actual 个"
        printf "\033[1;33m用法: %s\033[0m\n" "$usage"
        return 1
    fi
    return 0
}