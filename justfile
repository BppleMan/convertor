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

#cert:
#    # 生成服务器私钥 + CSR
#    openssl req -new -nodes -newkey rsa:2048 \
#        -keyout cert/ip-key.pem \
#        -out cert/ip.csr \
#        -config cert/ip-cert.cnf
#
#    # 用 CA 签服务器证书（非自签）
#    openssl x509 -req -in cert/ip.csr \
#        -CA cert/ca-cert.pem -CAkey cert/ca-key.pem -CAcreateserial \
#        -out cert/ip-cert.pem -days 825 -sha256 \
#        -extfile cert/ip-cert.cnf -extensions v3_req

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

dlogin:
    echo "Logging in to GitHub Container Registry..."
    echo $CR_PAT | docker login ghcr.io -u bppleman --password-stdin

dbuild version:
    docker build --platform linux/amd64 -t ghcr.io/bppleman/convertor:{{ version }} --push .

drun version:
    docker run --rm \
        -e REDIS_ENDPOINT \
        -e REDIS_CONVERTOR_USERNAME \
        -e REDIS_CONVERTOR_PASSWORD \
        -e REDIS_CA_CERT \
        -p 8080:80 \
        ghcr.io/bppleman/convertor:{{ version }} 0.0.0.0:80

dpush version:
    docker push ghcr.io/bppleman/convertor:{{ version }}

dinspect version:
    docker buildx imagetools inspect ghcr.io/bppleman/convertor:{{ version }}

dall version profile="release":
    just musl {{ profile }}
    just dbuild {{ version }}
    just dinspect {{ version }}

dbuildx version:
    docker buildx build \
      --platform linux/amd64 \
      -t ghcr.io/bppleman/convertor:{{ version }} \
      --push .
