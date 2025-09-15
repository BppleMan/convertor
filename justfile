#!/usr/bin/env just --justfile

build-dev:
    cd dashboard && just dashboard dev
    cd convertor && just musl dev
    cd convertor && just image dev

build-prod:
    cd dashboard && just dashboard prod
    cd convertor && just musl alpine
    cd convertor && just image alpine
