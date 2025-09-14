#!/usr/bin/env just --justfile

dev-build:
    cd dashboard && just dev-dashboard
    cd convertor && just dev-musl && just image dev

prod-build:
    cd dashboard && just prod-dashboard
    cd convertor && just alpine-musl && just image alpine
