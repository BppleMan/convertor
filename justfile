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
