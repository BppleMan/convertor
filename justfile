#!/usr/bin/env just --justfile

release:
    cargo build --release

linux:
    cross build --release --target x86_64-unknown-linux-gnu

convertor:
    cargo run --release --bin convertor

refresh-token:
    cargo run --release --bin convertor -- refresh-token

cert:
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
      -keyout cert/ip-key.pem \
      -out cert/ip-cert.pem \
      -config cert/ip-cert.cnf

sync_toml:
    scp ~/.convertor/convertor.toml ubuntu:/root/.convertor/convertor.toml

sync_bin:
    scp target/x86_64-unknown-linux-gnu/release/convertor ubuntu:/root

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
