#!/usr/bin/env just --justfile

set shell := ["zsh", "-uc"]

prepare:
    cargo install cargo-zigbuild
    brew install zig

install:
    cargo install --bin convertor --path .

linux:
    time CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    cargo build --release --bin convertor --target x86_64-unknown-linux-gnu

musl:
    time cargo zigbuild --release --bin convertor --target x86_64-unknown-linux-musl

cross-linux:
    time cross build --release --bin convertor --target x86_64-unknown-linux-gnu

zig-linux:
    time CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=./zig-cc \
    cargo build --release --bin convertor --target x86_64-unknown-linux-gnu

deploy:
    echo "Stopping remote service..."
    ssh convertor "rc-service convertor stop"

    echo "Uploading file..."
    scp target/x86_64-unknown-linux-musl/release/convertor convertor:/root/convertor
    scp ~/.convertor/convertor.toml convertor:/root/.convertor/convertor.toml

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

push_config alias:
    echo "Uploading file..."
    scp ~/.convertor/convertor.toml ubuntu:/root/.convertor/convertor.toml

    echo "Restarting remote service..."
    ssh {{ alias }} "systemctl restart convertor"
    ssh {{ alias }} "systemctl status convertor"

pull_config alias:
    echo "Downloading file..."
    scp ubuntu:/root/.convertor/convertor.toml ~/.convertor/convertor.toml

cert:
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
      -keyout cert/ip-key.pem \
      -out cert/ip-cert.pem \
      -config cert/ip-cert.cnf

container-name := "convertor-dev"

start:
    docker run -dit \
      -v "$(pwd):/usr/src/app" \
      -v "$(pwd)/target:/usr/src/app/target" \
      -v "$(pwd)/.convertor.dev:/usr/src/app/.convertor.dev" \
      -w /usr/src/app \
      --name {{ container-name }} \
      rust:1.87.0 \
      bash

cargo-run *args:
    docker exec -it {{ container-name }} bash -c "cargo run --target aarch64-unknown-linux-gnu --bin convertor -- {{ args }}"

bash *args:
    docker exec -it {{ container-name }} bash -c "bash {{ args }}"

stop:
    docker rm -f {{ container-name }}
