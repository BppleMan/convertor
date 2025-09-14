#!/usr/bin/env just --justfile

dev-build:
    cd dashboard && just dev-dashboard
    cd convertor && just dev-musl
    cd convertor && just image dev

prod-build:
    cd dashboard && just prod-dashboard
    cd convertor && just alpine-musl
    cd convertor && just image prod
