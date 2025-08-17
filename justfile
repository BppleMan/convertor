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

push_config alias:
    echo "Uploading file..."
    scp ~/.convertor/convertor.toml ubuntu:/root/.convertor/convertor.toml

    echo "Restarting remote service..."
    ssh {{ alias }} "systemctl restart convertor"
    ssh {{ alias }} "systemctl status convertor"

pull_config alias:
    echo "Downloading file..."
    scp ubuntu:/root/.convertor/convertor.toml ~/.convertor/convertor.toml

ca:
    mkdir -p $ICLOUD/cert/ca
    openssl req -x509 -nodes -days 3650 -newkey rsa:4096 \
            -keyout $ICLOUD/cert/ca/ca-key.pem \
            -out $ICLOUD/cert/ca/ca-cert.pem \
            -config cert/ca.cnf
    # 校验是 CA
    openssl x509 -in $ICLOUD/cert/ca/ca-cert.pem -noout -text | grep -E "CA:TRUE|KeyCertSign"

cert IP="127.0.0.1":
    echo {{ IP }}
    mkdir -p "$ICLOUD/cert/redis"
    # 生成带指定 IP 的临时 cnf（替换 [alt_names] 段内的 IP.* 行）
    awk -v ip="{{ IP }}" 'BEGIN{inside=0} /^\[alt_names\]$/ {print; print "IP.1 = " ip; inside=1; next} inside && /^\[.*\]$/ {inside=0; print; next} inside && /^IP\.[0-9]+ *=/ {next} {print}' cert/ip-cert.cnf > "$ICLOUD/cert/redis/_gen.cnf"

    # 生成服务器私钥 + CSR
    openssl req -new -nodes -newkey rsa:2048 \
        -keyout $ICLOUD/cert/redis/ip-key.pem \
        -out $ICLOUD/cert/redis/ip.csr \
        -config $ICLOUD/cert/redis/_gen.cnf

    # 用 CA 签服务器证书
    openssl x509 -req -in $ICLOUD/cert/redis/ip.csr \
        -CA $ICLOUD/cert/ca/ca-cert.pem -CAkey $ICLOUD/cert/ca/ca-key.pem -CAcreateserial \
        -out $ICLOUD/cert/redis/ip-cert.pem -days 365 -sha256 \
        -extfile $ICLOUD/cert/redis/_gen.cnf -extensions v3_req

    # 校验证书链 & SAN
    openssl verify -CAfile $ICLOUD/cert/ca/ca-cert.pem $ICLOUD/cert/redis/ip-cert.pem
    openssl x509 -in $ICLOUD/cert/redis/ip-cert.pem -noout -text | grep -A3 "Subject Alternative Name"

install-cert ecs:
    # 确保 /etc/redis/cert 目录存在
    ssh {{ ecs }} "mkdir -p /etc/redis/cert"
    # 上传 CA 证书和 Redis 证书
    scp $ICLOUD/cert/ca/ca-cert.pem {{ ecs }}:/etc/redis/cert/ca-cert.pem
    scp $ICLOUD/cert/redis/ip-key.pem {{ ecs }}:/etc/redis/cert/ip-key.pem
    scp $ICLOUD/cert/redis/ip-cert.pem {{ ecs }}:/etc/redis/cert/ip-cert.pem

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
