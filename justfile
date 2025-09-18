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
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
    fi

    just dashboard {{ profile }}
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
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
    fi
    just dashboard {{ profile }}

    time CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    cargo build  --profile ${PROFILE} --bin convd --target x86_64-unknown-linux-gnu

# profile: dev | prod | alpine
musl profile="dev":
    #!/bin/sh
    PROFILE="dev"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
    fi
    just dashboard {{ profile }}

    cp -rf ./dashboard/dist/dashboard/${DASHBOARD_DIST}/browser ./crates/convd/assets/${DASHBOARD_DIST}
    time cargo zigbuild --profile ${PROFILE} --bin convd --target x86_64-unknown-linux-musl

# profile: dev | prod | alpine
cross profile="dev":
    #!/bin/sh
    PROFILE="dev"
    DASHBOARD_DIST="development"
    if [ "{{ profile }}" = "prod" ]; then
        PROFILE="release"
        DASHBOARD_DIST="production"
    elif [ "{{ profile }}" = "alpine" ]; then
        PROFILE="alpine"
        DASHBOARD_DIST="production"
    fi

    cp -rf ./dashboard/dist/dashboard/${DASHBOARD_DIST}/browser ./crates/convd/assets/${DASHBOARD_DIST}
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

    VERSION=$(docker run --rm -v ./target/$TARGET_TRIPLE/$PROFILE_PATH/$BIN_NAME:/app/$BIN_NAME alpine:3.20 /app/$BIN_NAME -V)
    VERSION=${VERSION//[^A-Za-z0-9_.-]/_}
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

    VERSION=$(docker run --rm -v ./target/x86_64-unknown-linux-musl/$PROFILE_PATH/convd:/app/convd alpine:3.20 /app/convd -V)
    VERSION=${VERSION//[^A-Za-z0-9_.-]/_}
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
