#!/bin/bash

# 日志记录工具模块

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