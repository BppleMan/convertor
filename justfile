#!/usr/bin/env just --justfile

release:
    cargo build --release

linux:
    cross build --release --target x86_64-unknown-linux-gnu

convertor:
    cargo run --release --bin convertor

refresh-token:
    cargo run --release --bin convertor -- refresh-token
