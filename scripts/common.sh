#!/bin/bash

# 通用日志和工具函数库

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 获取当前时间戳
get_timestamp() {
    date '+%Y-%m-%d %H:%M:%S'
}

# 日志级别函数
log_info() {
    printf "\033[0;34m[%s] [INFO]\033[0m %s\n" "$(get_timestamp)" "$1" >&2
}

log_warn() {
    printf "\033[1;33m[%s] [WARN]\033[0m %s\n" "$(get_timestamp)" "$1" >&2
}

log_error() {
    printf "\033[0;31m[%s] [ERROR]\033[0m %s\n" "$(get_timestamp)" "$1" >&2
}

log_success() {
    printf "\033[0;32m[%s] [SUCCESS]\033[0m %s\n" "$(get_timestamp)" "$1" >&2
}

log_debug() {
    if [[ "${DEBUG:-}" == "true" ]]; then
        printf "\033[0;35m[%s] [DEBUG]\033[0m %s\n" "$(get_timestamp)" "$1" >&2
    fi
}

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

# 获取项目根目录
get_project_root() {
    local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    echo "$(dirname "$script_dir")"
}

# 确保在项目根目录执行
ensure_project_root() {
    local project_root="$(get_project_root)"
    if [[ "$(pwd)" != "$project_root" ]]; then
        log_info "切换到项目根目录: $project_root"
        cd "$project_root" || {
            log_error "无法切换到项目根目录: $project_root"
            exit 1
        }
    fi
}

# 设置脚本的错误处理
set_error_handling() {
    set -euo pipefail
    
    # 捕获错误并记录
    trap 'log_error "脚本在第 $LINENO 行出错，退出码: $?"' ERR
}

# 显示帮助信息
show_help() {
    local script_name="$1"
    local description="$2"
    local usage="$3"
    
    printf "\033[0;36m%s\033[0m\n" "$script_name"
    printf "\033[0;36m%s\033[0m\n" "$description"
    echo ""
    printf "\033[1;33m用法:\033[0m\n"
    echo "  $usage"
    echo ""
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