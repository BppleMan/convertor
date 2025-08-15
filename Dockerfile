###############################################
# Ultra minimal runtime Dockerfile (first quick version)
# 假设你已经在宿主机执行过：
#   cargo build --target x86_64-unknown-linux-musl --release
# 并生成了: target/x86_64-unknown-linux-musl/release/convertor (静态或接近静态)
#
# 目的：快速验证，可直接监听 80 端口。
# 说明：若需要系统 CA 根证书 (reqwest/rustls) 而又未静态内嵌，将 scratch 换成 distroless 或 debian-slim。
###############################################

FROM alpine:latest

ARG VERSION=0.0.1
LABEL org.opencontainers.image.title="convertor" \
    org.opencontainers.image.version="${VERSION}" \
    org.opencontainers.image.source="https://github.com/BppleMan/convertor" \
    org.opencontainers.image.description="A profile converter for surge/clash." \
    org.opencontainers.image.licenses="Apache-2.0"

# 拷贝已编译好的 musl 二进制
COPY target/x86_64-unknown-linux-musl/release/convertor /convertor

EXPOSE 80

# 注意：程序的第一个位置参数是 listen (SocketAddrV4)，必须包含端口。
# 你的需求 “/path/to/convertor 0.0.0.0” 在当前实现会解析失败，需要写成 0.0.0.0:80。
# 若想仅写 IP 就默认 80，需要修改源码解析逻辑；当前先按合法形式提供。
ENTRYPOINT ["/convertor"]
CMD ["0.0.0.0:80"]

###############################################
# 如果运行后报 TLS / 证书相关错误：
# 说明二进制在解析 HTTPS 需要系统 CA，但 scratch 没有；可以改用：
#   FROM debian:bookworm-slim AS base
#   RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
#   COPY target/x86_64-unknown-linux-musl/release/convertor /convertor
#   ... 保持其它相同
###############################################
