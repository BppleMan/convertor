#!/usr/bin/env just --justfile

build-dev:
    cd dashboard && just dashboard dev
    cd convertor && just musl dev
    cd convertor && just image dev

build-prod:
    cd dashboard && just dashboard prod
    cd convertor && just musl alpine
    cd convertor && just image alpine

prepare:
    cargo install cargo-zigbuild
    brew install zig

#╭──────────────────────────────────────────────╮
#│                   release                    │
#╰──────────────────────────────────────────────╯

install bin="convd":
    cargo install --bin {{ bin }} --path .

publish:
    just publish-convertor
    just publish-convd
    just publish-confly

publish-convertor:
    cargo publish -p convertor

publish-convd:
    just dashboard dev
    just dashboard prod
    cargo publish -p convd

publish-confly:
    cargo publish -p confly

#╭──────────────────────────────────────────────╮
#│                    build                     │
#╰──────────────────────────────────────────────╯

# profile: dev | prod | alpine
all profile="dev":
    #!/bin/sh
    PROFILE="dev"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
    fi

    time cargo build --workspace --all-targets --profile ${PROFILE}

# profile: dev | prod | alpine
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

# profile: dev | prod | alpine
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
#│                     test                     │
#╰──────────────────────────────────────────────╯

test-convertor:
    cargo insta test -p convertor --features=testkit

test-convd:
    cargo insta test -p convd

test-confly:
    cargo insta test -p confly

#╭──────────────────────────────────────────────╮
#│                    linux                     │
#╰──────────────────────────────────────────────╯

# profile: dev | prod | alpine
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

# profile: dev | prod | alpine
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

# profile: dev | prod | alpine
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
#│                  dashboard                   │
#╰──────────────────────────────────────────────╯

# profile: dev | prod
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
#│                    docker                    │
#╰──────────────────────────────────────────────╯

# profile: dev | prod | alpine
image profile="dev":
    #!/bin/sh
    TARGET_TRIPLE="x86_64-unknown-linux-musl"
    BIN_NAME="convd"
    PROFILE_PATH="debug"
    REGISTRY="local"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE_PATH="release"
        REGISTRY="ghcr.io/bppleman/convertor"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE_PATH="alpine"
        REGISTRY="ghcr.io/bppleman/convertor"
    fi

    VERSION=$(docker run --rm -v ./target/$TARGET_TRIPLE/$PROFILE_PATH/$BIN_NAME:/app/$BIN_NAME alpine:3.20 /app/$BIN_NAME tag)
    # VERSION=${VERSION//[^A-Za-z0-9_.-]/_}
    BUILD_DATE=$(date +%Y-%m-%dT%H:%M:%S%z)
    echo TARGET_TRIPLE=$TARGET_TRIPLE
    echo BIN_NAME=$BIN_NAME
    echo PROFILE_PATH=$PROFILE_PATH
    echo VERSION=$VERSION
    echo REGISTRY=$REGISTRY
    docker build -f Dockerfile \
        --build-arg TARGET_TRIPLE=$TARGET_TRIPLE \
        --build-arg BIN_NAME=$BIN_NAME \
        --build-arg PROFILE_PATH=$PROFILE_PATH \
        --build-arg VERSION=$VERSION \
        --build-arg BUILD_DATE=$BUILD_DATE \
        -t $REGISTRY/$BIN_NAME:$VERSION .
    docker tag $REGISTRY/$BIN_NAME:$VERSION $REGISTRY/$BIN_NAME:latest

# profile: dev | prod | alpine
run profile="dev":
    #!/bin/sh
    PROFILE_PATH="debug"
    REGISTRY="local"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE_PATH="release"
        REGISTRY="ghcr.io/bppleman/convertor"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE_PATH="alpine"
        REGISTRY="ghcr.io/bppleman/convertor"
    fi

    VERSION=$(docker run --rm -v ./target/x86_64-unknown-linux-musl/$PROFILE_PATH/convd:/app/convd alpine:3.20 /app/convd tag)
    # VERSION=${VERSION//[^A-Za-z0-9_.-]/_}
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

# profile: dev | prod | alpine, dry_run: true | false
publish-ghcr profile="dev" dry_run="false":
    #!/bin/sh
    # 发布镜像到 GHCR (需要环境变量 CR_PAT)

    # 将just参数转换为shell变量
    PROFILE="{{ profile }}"
    DRY_RUN="{{ dry_run }}"

    TARGET_TRIPLE="x86_64-unknown-linux-musl"
    BIN_NAME="convd"
    PROFILE_PATH="debug"
    REGISTRY="local"

    if [ "$PROFILE" = "prod" ]; then
        PROFILE_PATH="release"
        REGISTRY="ghcr.io/bppleman/convertor"
    elif [ "$PROFILE" = "alpine" ]; then
        PROFILE_PATH="alpine"
        REGISTRY="ghcr.io/bppleman/convertor"
    fi

    if [ "$REGISTRY" = "local" ]; then
        echo "profile '$PROFILE' uses local registry, skipping GHCR push"
        exit 0
    fi

    # 先构建镜像确保存在
    just image $PROFILE

    VERSION=$(docker run --rm -v ./target/$TARGET_TRIPLE/$PROFILE_PATH/$BIN_NAME:/app/$BIN_NAME alpine:3.20 /app/$BIN_NAME tag)
    # VERSION=${VERSION//[^A-Za-z0-9_.-]/_}

    echo "TARGET_TRIPLE=$TARGET_TRIPLE"
    echo "BIN_NAME=$BIN_NAME"
    echo "PROFILE_PATH=$PROFILE_PATH"
    echo "VERSION=$VERSION"
    echo "REGISTRY=$REGISTRY"

    if [ -z "$CR_PAT" ]; then
        echo "错误: 请设置环境变量 CR_PAT (GitHub Personal Access Token)"
        exit 1
    fi

    IMAGE_WITH_VERSION="$REGISTRY/$BIN_NAME:$VERSION"
    IMAGE_LATEST="$REGISTRY/$BIN_NAME:latest"

    if [ "$DRY_RUN" = "true" ]; then
        echo "[DRY RUN] 将执行以下操作:"
        echo "[DRY RUN] echo \$CR_PAT | docker login ghcr.io -u TOKEN --password-stdin"
        echo "[DRY RUN] docker tag $REGISTRY/$BIN_NAME:$VERSION $IMAGE_WITH_VERSION"
        echo "[DRY RUN] docker push $IMAGE_WITH_VERSION"
        echo "[DRY RUN] docker tag $IMAGE_WITH_VERSION $IMAGE_LATEST"
        echo "[DRY RUN] docker push $IMAGE_LATEST"
        echo "[DRY RUN] docker logout ghcr.io"
        exit 0
    fi

    # 使用 CR_PAT 登录 GHCR (username 可以是任意值，这里使用 TOKEN)
    echo "$CR_PAT" | docker login ghcr.io -u TOKEN --password-stdin

    # 标记并推送带版本的镜像
    docker tag $REGISTRY/$BIN_NAME:$VERSION $IMAGE_WITH_VERSION
    docker push $IMAGE_WITH_VERSION

    # 标记并推送 latest 镜像
    docker tag $IMAGE_WITH_VERSION $IMAGE_LATEST
    docker push $IMAGE_LATEST

    # 登出
    docker logout ghcr.io

    echo "镜像发布完成:"
    echo "  $IMAGE_WITH_VERSION"
    echo "  $IMAGE_LATEST"

# profile: dev | prod | alpine, dry_run: true | false
publish-ghcr-gh profile="dev" dry_run="false":
    #!/bin/sh
    # 使用 GitHub CLI 发布镜像到 GHCR (需要先 gh auth login)

    # 将just参数转换为shell变量
    PROFILE="{{ profile }}"
    DRY_RUN="{{ dry_run }}"

    TARGET_TRIPLE="x86_64-unknown-linux-musl"
    BIN_NAME="convd"
    PROFILE_PATH="debug"
    REGISTRY="local"

    if [ "$PROFILE" = "prod" ]; then
        PROFILE_PATH="release"
        REGISTRY="ghcr.io/bppleman/convertor"
    elif [ "$PROFILE" = "alpine" ]; then
        PROFILE_PATH="alpine"
        REGISTRY="ghcr.io/bppleman/convertor"
    fi

    if [ "$REGISTRY" = "local" ]; then
        echo "profile '$PROFILE' uses local registry, skipping GHCR push"
        exit 0
    fi

    # 检查 gh CLI 是否安装和认证
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

    # 先构建镜像确保存在
    just image $PROFILE

    VERSION=$(docker run --rm -v ./target/$TARGET_TRIPLE/$PROFILE_PATH/$BIN_NAME:/app/$BIN_NAME alpine:3.20 /app/$BIN_NAME tag)
    # VERSION=${VERSION//[^A-Za-z0-9_.-]/_}

    echo "TARGET_TRIPLE=$TARGET_TRIPLE"
    echo "BIN_NAME=$BIN_NAME"
    echo "PROFILE_PATH=$PROFILE_PATH"
    echo "VERSION=$VERSION"
    echo "REGISTRY=$REGISTRY"

    IMAGE_WITH_VERSION="$REGISTRY/$BIN_NAME:$VERSION"
    IMAGE_LATEST="$REGISTRY/$BIN_NAME:latest"

    if [ "$DRY_RUN" = "true" ]; then
        echo "[DRY RUN] 将执行以下操作:"
        echo "[DRY RUN] gh auth token | docker login ghcr.io -u \$(gh api user --jq .login) --password-stdin"
        echo "[DRY RUN] docker tag $REGISTRY/$BIN_NAME:$VERSION $IMAGE_WITH_VERSION"
        echo "[DRY RUN] docker push $IMAGE_WITH_VERSION"
        echo "[DRY RUN] docker tag $IMAGE_WITH_VERSION $IMAGE_LATEST"
        echo "[DRY RUN] docker push $IMAGE_LATEST"
        echo "[DRY RUN] docker logout ghcr.io"
        exit 0
    fi

    # 使用 gh CLI 获取认证信息并登录 Docker
    GH_USERNAME=$(gh api user --jq .login)
    echo "使用 GitHub 用户: $GH_USERNAME"

    gh auth token | docker login ghcr.io -u "$GH_USERNAME" --password-stdin

    # 标记并推送带版本的镜像
    docker tag $REGISTRY/$BIN_NAME:$VERSION $IMAGE_WITH_VERSION
    docker push $IMAGE_WITH_VERSION

    # 标记并推送 latest 镜像
    docker tag $IMAGE_WITH_VERSION $IMAGE_LATEST
    docker push $IMAGE_LATEST

    # 登出
    docker logout ghcr.io

    echo "镜像发布完成 (via GitHub CLI):"
    echo "  $IMAGE_WITH_VERSION"
    echo "  $IMAGE_LATEST"
