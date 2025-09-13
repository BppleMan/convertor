#!/usr/bin/env just --justfile

build-dev:
    cd dashboard && just build-dev
    cd convertor && just convd-musl-dev
