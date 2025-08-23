#!/usr/bin/env just --justfile

set shell := ["zsh", "-uc"]

prepare:
    cargo install cargo-zigbuild
    brew install zig

install:
    cargo install --bin convertor --path .

release:
    cargo build --release --bin convertor

linux:
    time CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    cargo build --release --bin convertor --target x86_64-unknown-linux-gnu

musl profile="release":
    time cargo zigbuild --profile {{ profile }} --bin convertor --target x86_64-unknown-linux-musl

cross-linux:
    time cross build --release --bin convertor --target x86_64-unknown-linux-gnu

zig-linux:
    time CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=./zig-cc \
    cargo build --release --bin convertor --target x86_64-unknown-linux-gnu

deploy profile="release":
    echo "Stopping remote service..."
    ssh convertor "rc-service convertor stop"

    echo "Uploading file..."
    scp target/x86_64-unknown-linux-musl/{{ profile }}/convertor convertor:/root/convertor
    # scp ~/.convertor/convertor.toml convertor:/root/.convertor/convertor.toml

    ssh convertor "rc-service convertor restart"
    ssh convertor "rc-service convertor status"

# 用法: just deploy user@host path/to/local/file /remote/path your-service-name
deploy_ubuntu:
    echo "Stopping remote service..."
    ssh ubuntu "systemctl stop convertor"
    ssh ubuntu "systemctl daemon-reload"

    echo "Uploading file..."
    scp target/x86_64-unknown-linux-gnu/release/convertor ubuntu:/root/.cargo/bin/convertor
    scp ~/.convertor/convertor.toml ubuntu:/root/.convertor/convertor.toml

    echo "Restarting remote service..."
    ssh ubuntu "systemctl restart convertor"
    ssh ubuntu "systemctl status convertor"

# ===== 基础参数，可改 =====

TARGET := "x86_64-unknown-linux-musl"
BIN := "convertor"
CONTAINER_NAME := "convertor-dev"

# 1) 打本地镜像（只打包宿主构建产物）
image PROFILE="release":
    docker build -f docker-service/Dockerfile \
      --build-arg TARGET_TRIPLE={{ TARGET }} \
      --build-arg PROFILE={{ PROFILE }} \
      --build-arg BIN_NAME={{ BIN }} \
      --build-arg BIN_PATH=target/{{ TARGET }}/{{ PROFILE }}/{{ BIN }} \
      --build-arg VERSION=dev-`date +%Y%m%d%H%M%S` \
      -t local/{{ BIN }}:dev .
