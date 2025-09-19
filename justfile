#!/usr/bin/env just --justfile

# 开发环境构建
build-dev:
    cd dashboard && just dashboard dev
    cd convertor && just musl dev
    cd convertor && just image dev

# 生产环境构建
build-prod:
    cd dashboard && just dashboard prod
    cd convertor && just musl alpine
    cd convertor && just image alpine

# 准备开发环境
prepare:
    cargo install cargo-zigbuild
    brew install zig

#╭──────────────────────────────────────────────╮
#│                   发布                       │
#╰──────────────────────────────────────────────╯

# 安装二进制文件
install bin="convd":
    cargo install --bin {{ bin }} --path .

# 发布所有包
publish:
    just publish-convertor
    just publish-convd
    just publish-confly

# 发布 convertor 包
publish-convertor:
    cargo publish -p convertor

# 发布 convd 包
publish-convd:
    just dashboard dev
    just dashboard prod
    cargo publish -p convd

# 发布 confly 包
publish-confly:
    cargo publish -p confly

#╭──────────────────────────────────────────────╮
#│                   构建                       │
#╰──────────────────────────────────────────────╯

# 构建所有组件 (dev|prod|alpine)
all profile="dev":
    #!/bin/sh
    PROFILE="dev"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
    fi

    time cargo build --workspace --all-targets --profile ${PROFILE}

# 构建 convd (dev|prod|alpine)
convd profile="dev":
    #!/bin/sh
    PROFILE="dev"
    DASHBOARD="development"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
        DASHBOARD="production"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
        DASHBOARD="production"
    fi

    just dashboard ${DASHBOARD}
    time cargo build --bin convd --profile ${PROFILE}

# 构建 confly (dev|prod|alpine)
confly profile="dev":
    #!/bin/sh
    PROFILE="dev"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
    fi

    time cargo build --bin confly --profile ${PROFILE}

#╭──────────────────────────────────────────────╮
#│                   测试                       │
#╰──────────────────────────────────────────────╯

# 测试 convertor
test-convertor:
    cargo insta test -p convertor --features=testkit

# 测试 convd
test-convd:
    cargo insta test -p convd

# 测试 confly
test-confly:
    cargo insta test -p confly

#╭──────────────────────────────────────────────╮
#│                 Linux 构建                   │
#╰──────────────────────────────────────────────╯

# Linux 构建 (dev|prod|alpine)
linux profile="dev":
    #!/bin/sh
    PROFILE="dev"
    DASHBOARD="development"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
        DASHBOARD="production"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
        DASHBOARD="production"
    fi

    just dashboard ${DASHBOARD}
    time CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    cargo build  --profile ${PROFILE} --bin convd --target x86_64-unknown-linux-gnu

# MUSL 构建 (dev|prod|alpine)
musl profile="dev":
    #!/bin/sh
    PROFILE="dev"
    DASHBOARD="development"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
        DASHBOARD="production"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
        DASHBOARD="production"
    fi

    just dashboard ${DASHBOARD}
    time cargo zigbuild --profile ${PROFILE} --bin convd --target x86_64-unknown-linux-musl

# 交叉编译 (dev|prod|alpine)
cross profile="dev":
    #!/bin/sh
    PROFILE="dev"
    DASHBOARD="development"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
        DASHBOARD="production"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
        DASHBOARD="production"
    fi

    just dashboard ${DASHBOARD}
    time cross build  --profile ${PROFILE} --bin convd --target x86_64-unknown-linux-gnu

#╭──────────────────────────────────────────────╮
#│                 前端构建                      │
#╰──────────────────────────────────────────────╯

# 构建前端界面 (dev|prod)
dashboard profile="dev":
    #!/bin/sh
    cd dashboard
    pnpm install
    PROFILE="development"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="production"
    fi
    pnpm ng build --configuration ${PROFILE}
    cd ..
    rm -rf ./crates/convd/assets/${PROFILE}
    cp -rf ./dashboard/dist/dashboard/${PROFILE}/browser ./crates/convd/assets/${PROFILE}

#╭──────────────────────────────────────────────╮
#│                 Docker                       │
#╰──────────────────────────────────────────────╯

# 构建镜像 (dev|prod|alpine)
image profile="dev":
    #!/bin/sh
    TARGET_TRIPLE="x86_64-unknown-linux-musl"
    BIN_NAME="convd"
    PROFILE="debug"
    REGISTRY="local"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
        REGISTRY="ghcr.io/bppleman/convertor"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
        REGISTRY="ghcr.io/bppleman/convertor"
    fi

    VERSION=$(docker run --rm -v ./target/$TARGET_TRIPLE/$PROFILE/$BIN_NAME:/app/$BIN_NAME alpine:3.20 /app/$BIN_NAME tag)
    BUILD_DATE=$(date +%Y-%m-%dT%H:%M:%S%z)
    echo TARGET_TRIPLE=$TARGET_TRIPLE
    echo BIN_NAME=$BIN_NAME
    echo PROFILE=$PROFILE
    echo VERSION=$VERSION
    echo REGISTRY=$REGISTRY
    docker build -f Dockerfile \
        --build-arg TARGET_TRIPLE=$TARGET_TRIPLE \
        --build-arg BIN_NAME=$BIN_NAME \
        --build-arg PROFILE=$PROFILE \
        --build-arg VERSION=$VERSION \
        --build-arg BUILD_DATE=$BUILD_DATE \
        -t $REGISTRY/$BIN_NAME:$VERSION .
    docker tag $REGISTRY/$BIN_NAME:$VERSION $REGISTRY/$BIN_NAME:latest

# 运行镜像 (dev|prod|alpine)
run profile="dev":
    #!/bin/sh
    PROFILE="debug"
    REGISTRY="local"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
        REGISTRY="ghcr.io/bppleman/convertor"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
        REGISTRY="ghcr.io/bppleman/convertor"
    fi

    VERSION=$(docker run --rm -v ./target/x86_64-unknown-linux-musl/$PROFILE/convd:/app/convd alpine:3.20 /app/convd tag)
    echo VERSION=$VERSION
    docker run --rm -it \
        -v ~/.convertor/convertor.toml:/app/.convertor/convertor.toml \
        -e REDIS_ENDPOINT \
        -e REDIS_SCHEME \
        -e REDIS_CONVERTOR_USERNAME \
        -e REDIS_CONVERTOR_PASSWORD \
        -e REDIS_CA_CERT \
        --name convd-temp \
        $REGISTRY/convd:$VERSION

# 发布到 GHCR (dev|prod|alpine)
publish-ghcr profile="dev" dry_run="false":
    #!/bin/sh
    PROFILE="{{ profile }}"
    DRY_RUN="{{ dry_run }}"

    TARGET_TRIPLE="x86_64-unknown-linux-musl"
    BIN_NAME="convd"
    REGISTRY="local"

    if [ "$PROFILE" = "prod" ]; then
        PROFILE="release"
        REGISTRY="ghcr.io/bppleman/convertor"
    elif [ "$PROFILE" = "alpine" ]; then
        PROFILE="alpine"
        REGISTRY="ghcr.io/bppleman/convertor"
    fi

    if [ "$REGISTRY" = "local" ]; then
        echo "profile '$PROFILE' uses local registry, skipping GHCR push"
        exit 0
    fi

    just image $PROFILE

    VERSION=$(docker run --rm -v ./target/$TARGET_TRIPLE/$PROFILE/$BIN_NAME:/app/$BIN_NAME alpine:3.20 /app/$BIN_NAME tag)

    echo "TARGET_TRIPLE=$TARGET_TRIPLE"
    echo "BIN_NAME=$BIN_NAME"
    echo "PROFILE=$PROFILE"
    echo "VERSION=$VERSION"
    echo "REGISTRY=$REGISTRY"

    if [ -z "$CR_PAT" ]; then
        echo "错误: 请设置环境变量 CR_PAT (GitHub Personal Access Token)"
        exit 1
    fi

    IMAGE_WITH_VERSION="$REGISTRY/$BIN_NAME:$VERSION"
    IMAGE_LATEST="$REGISTRY/$BIN_NAME:latest"

    if [ "$DRY_RUN" = "true" ]; then
        echo "[预览模式] 将执行以下操作:"
        echo "[预览模式] echo \$CR_PAT | docker login ghcr.io -u TOKEN --password-stdin"
        echo "[预览模式] docker tag $REGISTRY/$BIN_NAME:$VERSION $IMAGE_WITH_VERSION"
        echo "[预览模式] docker push $IMAGE_WITH_VERSION"
        echo "[预览模式] docker tag $IMAGE_WITH_VERSION $IMAGE_LATEST"
        echo "[预览模式] docker push $IMAGE_LATEST"
        echo "[预览模式] docker logout ghcr.io"
        exit 0
    fi

    echo "$CR_PAT" | docker login ghcr.io -u TOKEN --password-stdin

    docker tag $REGISTRY/$BIN_NAME:$VERSION $IMAGE_WITH_VERSION
    docker push $IMAGE_WITH_VERSION

    docker tag $IMAGE_WITH_VERSION $IMAGE_LATEST
    docker push $IMAGE_LATEST

    docker logout ghcr.io

    echo "镜像发布完成:"
    echo "  $IMAGE_WITH_VERSION"
    echo "  $IMAGE_LATEST"

# 使用 GitHub CLI 发布到 GHCR (dev|prod|alpine)
publish-ghcr-gh profile="dev" dry_run="false":
    #!/bin/sh
    PROFILE="{{ profile }}"
    DRY_RUN="{{ dry_run }}"
    TARGET_TRIPLE="x86_64-unknown-linux-musl"
    BIN_NAME="convd"
    REGISTRY="local"

    if [ "$PROFILE" = "prod" ]; then
        PROFILE="release"
        REGISTRY="ghcr.io/bppleman/convertor"
    elif [ "$PROFILE" = "alpine" ]; then
        PROFILE="alpine"
        REGISTRY="ghcr.io/bppleman/convertor"
    fi

    if [ "$REGISTRY" = "local" ]; then
        echo "profile '$PROFILE' uses local registry, skipping GHCR push"
        exit 0
    fi

    if ! command -v gh >/dev/null 2>&1; then
        echo "错误: 请安装 GitHub CLI (gh)"
        echo "macOS: brew install gh"
        echo "或访问: https://cli.github.com/"
        exit 1
    fi

    if ! gh auth status >/dev/null 2>&1; then
        echo "错误: 请先使用 GitHub CLI 登录"
        echo "运行: gh auth login"
        exit 1
    fi

    just image $PROFILE

    VERSION=$(docker run --rm -v ./target/$TARGET_TRIPLE/$PROFILE/$BIN_NAME:/app/$BIN_NAME alpine:3.20 /app/$BIN_NAME tag)

    echo "TARGET_TRIPLE=$TARGET_TRIPLE"
    echo "BIN_NAME=$BIN_NAME"
    echo "PROFILE=$PROFILE"
    echo "VERSION=$VERSION"
    echo "REGISTRY=$REGISTRY"

    IMAGE_WITH_VERSION="$REGISTRY/$BIN_NAME:$VERSION"
    IMAGE_LATEST="$REGISTRY/$BIN_NAME:latest"

    if [ "$DRY_RUN" = "true" ]; then
        echo "[预览模式] 将执行以下操作:"
        echo "[预览模式] gh auth token | docker login ghcr.io -u \$(gh api user --jq .login) --password-stdin"
        echo "[预览模式] docker tag $REGISTRY/$BIN_NAME:$VERSION $IMAGE_WITH_VERSION"
        echo "[预览模式] docker push $IMAGE_WITH_VERSION"
        echo "[预览模式] docker tag $IMAGE_WITH_VERSION $IMAGE_LATEST"
        echo "[预览模式] docker push $IMAGE_LATEST"
        echo "[预览模式] docker logout ghcr.io"
        exit 0
    fi

    GH_USERNAME=$(gh api user --jq .login)
    echo "使用 GitHub 用户: $GH_USERNAME"

    gh auth token | docker login ghcr.io -u "$GH_USERNAME" --password-stdin

    docker tag $REGISTRY/$BIN_NAME:$VERSION $IMAGE_WITH_VERSION
    docker push $IMAGE_WITH_VERSION

    docker tag $IMAGE_WITH_VERSION $IMAGE_LATEST
    docker push $IMAGE_LATEST

    docker logout ghcr.io

    echo "镜像发布完成 (via GitHub CLI):"
    echo "  $IMAGE_WITH_VERSION"
    echo "  $IMAGE_LATEST"

