#!/bin/bash

# 文件系统和项目管理工具模块

# 导入日志模块
source "$(dirname "${BASH_SOURCE[0]}")/log.sh"

# 获取项目根目录
get_project_root() {
    local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    # 从 scripts/lib 向上两级到项目根目录
    echo "$(dirname "$(dirname "$script_dir")")"
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