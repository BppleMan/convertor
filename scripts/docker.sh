#!/bin/bash

# Docker 相关脚本

# 导入通用函数和配置
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

# 构建 Docker 镜像
build_image() {
    local profile="${1:-dev}"
    
    log_info "开始构建 Docker 镜像 (环境: $profile)"
    
    setup_component_env "docker" "$profile" || return 1
    ensure_project_root
    check_docker || return 1
    
    # 检查二进制文件是否存在
    local binary_path="./target/$TARGET_TRIPLE/$PROFILE/$BIN_NAME"
    if [[ ! -f "$binary_path" ]]; then
        log_error "二进制文件不存在: $binary_path"
        log_info "请先运行构建命令构建二进制文件"
        return 1
    fi
    
    log_info "Docker 构建配置:"
    echo "  TARGET_TRIPLE: $TARGET_TRIPLE"
    echo "  BIN_NAME: $BIN_NAME"
    echo "  PROFILE: $PROFILE"
    echo "  VERSION: $VERSION"
    echo "  REGISTRY: $REGISTRY"
    echo "  BUILD_DATE: $BUILD_DATE"
    
    # 构建镜像
    execute_with_log "构建 Docker 镜像" \
        "docker build -f Dockerfile \
            --build-arg TARGET_TRIPLE=$TARGET_TRIPLE \
            --build-arg BIN_NAME=$BIN_NAME \
            --build-arg PROFILE=$PROFILE \
            --build-arg VERSION=$VERSION \
            --build-arg BUILD_DATE=$BUILD_DATE \
            -t $REGISTRY/$BIN_NAME:$VERSION ."
    
    # 标记为 latest
    execute_with_log "标记为 latest" \
        "docker tag $REGISTRY/$BIN_NAME:$VERSION $REGISTRY/$BIN_NAME:latest"
    
    log_success "Docker 镜像构建完成:"
    echo "  $REGISTRY/$BIN_NAME:$VERSION"
    echo "  $REGISTRY/$BIN_NAME:latest"
}

# 运行 Docker 容器
run_container() {
    local profile="${1:-dev}"
    
    log_info "开始运行 Docker 容器 (环境: $profile)"
    
    setup_component_env "docker" "$profile" || return 1
    check_docker || return 1
    
    local image_name="$REGISTRY/$BIN_NAME:$VERSION"
    
    # 检查镜像是否存在
    if ! docker images "$REGISTRY/$BIN_NAME" | grep -q "$VERSION"; then
        log_error "镜像不存在: $image_name"
        log_info "请先构建镜像"
        return 1
    fi
    
    log_info "运行容器: $image_name"
    
    # 运行容器
    execute_with_log "启动容器" \
        "docker run --rm -it \
            -v ~/.convertor/convertor.toml:/app/.convertor/convertor.toml \
            -e REDIS_ENDPOINT \
            -e REDIS_SCHEME \
            -e REDIS_CONVERTOR_USERNAME \
            -e REDIS_CONVERTOR_PASSWORD \
            -e REDIS_CA_CERT \
            --name convd-temp \
            $image_name"
}

# 发布到 GHCR (使用 Personal Access Token)
publish_ghcr() {
    local profile="${1:-dev}"
    local dry_run="${2:-false}"
    
    log_info "开始发布到 GHCR (环境: $profile, 预览: $dry_run)"
    
    setup_component_env "docker" "$profile" || return 1
    
    # 检查是否是本地镜像
    if [[ "$REGISTRY" == "local" ]]; then
        log_info "本地环境跳过 GHCR 发布"
        return 0
    fi
    
    # 检查环境变量
    if [[ -z "${CR_PAT:-}" ]]; then
        log_error "请设置环境变量 CR_PAT (GitHub Personal Access Token)"
        log_info "获取方式: GitHub Settings -> Developer settings -> Personal access tokens"
        return 1
    fi
    
    check_docker || return 1
    
    # 确保镜像已构建
    build_image "$profile" || return 1
    
    local image_with_version="$REGISTRY/$BIN_NAME:$VERSION"
    local image_latest="$REGISTRY/$BIN_NAME:latest"
    
    if [[ "$dry_run" == "true" ]]; then
        log_info "[预览模式] 将执行以下操作:"
        echo "  1. 登录 GHCR"
        echo "  2. 推送镜像: $image_with_version"
        echo "  3. 推送镜像: $image_latest"
        echo "  4. 退出登录"
        return 0
    fi
    
    # 登录 GHCR
    execute_with_log "登录 GHCR" \
        "echo \$CR_PAT | docker login ghcr.io -u TOKEN --password-stdin"
    
    # 推送带版本号的镜像
    execute_with_log "推送版本镜像" \
        "docker push $image_with_version"
    
    # 推送 latest 镜像
    execute_with_log "推送 latest 镜像" \
        "docker push $image_latest"
    
    # 退出登录
    execute_with_log "退出 GHCR 登录" \
        "docker logout ghcr.io"
    
    log_success "镜像发布完成:"
    echo "  $image_with_version"
    echo "  $image_latest"
}

# 发布到 GHCR (使用 GitHub CLI)
publish_ghcr_gh() {
    local profile="${1:-dev}"
    local dry_run="${2:-false}"
    
    log_info "开始发布到 GHCR via GitHub CLI (环境: $profile, 预览: $dry_run)"
    
    setup_component_env "docker" "$profile" || return 1
    
    # 检查是否是本地镜像
    if [[ "$REGISTRY" == "local" ]]; then
        log_info "本地环境跳过 GHCR 发布"
        return 0
    fi
    
    # 检查 GitHub CLI
    check_command "gh" || {
        log_error "请安装 GitHub CLI (gh)"
        log_info "macOS: brew install gh"
        log_info "或访问: https://cli.github.com/"
        return 1
    }
    
    # 检查登录状态
    if ! gh auth status >/dev/null 2>&1; then
        log_error "请先使用 GitHub CLI 登录"
        log_info "运行: gh auth login"
        return 1
    fi
    
    check_docker || return 1
    
    # 确保镜像已构建
    build_image "$profile" || return 1
    
    local gh_username="$(gh api user --jq .login)"
    local image_with_version="$REGISTRY/$BIN_NAME:$VERSION"
    local image_latest="$REGISTRY/$BIN_NAME:latest"
    
    log_info "使用 GitHub 用户: $gh_username"
    
    if [[ "$dry_run" == "true" ]]; then
        log_info "[预览模式] 将执行以下操作:"
        echo "  1. 使用 GitHub CLI 登录 GHCR (用户: $gh_username)"
        echo "  2. 推送镜像: $image_with_version"
        echo "  3. 推送镜像: $image_latest"
        echo "  4. 退出登录"
        return 0
    fi
    
    # 使用 GitHub CLI 登录 GHCR
    execute_with_log "使用 GitHub CLI 登录 GHCR" \
        "gh auth token | docker login ghcr.io -u $gh_username --password-stdin"
    
    # 推送带版本号的镜像
    execute_with_log "推送版本镜像" \
        "docker push $image_with_version"
    
    # 推送 latest 镜像
    execute_with_log "推送 latest 镜像" \
        "docker push $image_latest"
    
    # 退出登录
    execute_with_log "退出 GHCR 登录" \
        "docker logout ghcr.io"
    
    log_success "镜像发布完成 (via GitHub CLI):"
    echo "  $image_with_version"
    echo "  $image_latest"
}

# 显示镜像信息
show_images() {
    log_info "本地 Docker 镜像:"
    docker images | grep -E "(convertor|convd)" || {
        log_warn "未找到相关镜像"
    }
}

# 清理本地镜像
clean_images() {
    local force="${1:-false}"
    
    log_info "清理本地 Docker 镜像"
    
    if [[ "$force" != "true" ]]; then
        log_warn "这将删除所有 convertor/convd 相关镜像"
        read -p "确认继续? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "取消清理"
            return 0
        fi
    fi
    
    # 删除镜像
    local images=$(docker images | grep -E "(convertor|convd)" | awk '{print $3}' || true)
    if [[ -n "$images" ]]; then
        execute_with_log "删除相关镜像" \
            "echo '$images' | xargs docker rmi -f"
    else
        log_info "未找到需要清理的镜像"
    fi
}

# 显示帮助
show_docker_help() {
    show_help "docker.sh" "Docker 相关脚本" "docker.sh <command> [args...]"
    
    printf "\033[1;33m命令:\033[0m\n"
    echo "  image [profile]           - 构建 Docker 镜像"
    echo "  run [profile]             - 运行 Docker 容器"
    echo "  publish-ghcr [profile] [dry_run] - 发布到 GHCR (PAT)"
    echo "  publish-gh [profile] [dry_run]   - 发布到 GHCR (GitHub CLI)"
    echo "  images                    - 显示本地镜像"
    echo "  clean [force]             - 清理本地镜像"
    echo ""
    printf "\033[1;33m环境参数:\033[0m\n"
    echo "  dev, development          - 开发环境"
    echo "  prod, production          - 生产环境"
    echo "  alpine                    - Alpine Linux 环境"
    echo ""
    printf "\033[1;33m示例:\033[0m\n"
    echo "  docker.sh image alpine"
    echo "  docker.sh publish-ghcr prod false"
    echo "  docker.sh publish-gh alpine true"
}

# 主函数
main() {
    set_error_handling
    
    local command="${1:-}"
    shift || true
    
    case "$command" in
        "image")
            build_image "$@"
            ;;
        "run")
            run_container "$@"
            ;;
        "publish-ghcr")
            publish_ghcr "$@"
            ;;
        "publish-gh")
            publish_ghcr_gh "$@"
            ;;
        "images")
            show_images
            ;;
        "clean")
            clean_images "$@"
            ;;
        "help"|"-h"|"--help"|"")
            show_docker_help
            ;;
        *)
            log_error "未知命令: $command"
            show_docker_help
            exit 1
            ;;
    esac
}

# 如果直接执行此脚本
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi