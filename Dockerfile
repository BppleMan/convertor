FROM alpine:3.20

# ====== OCI 元信息（通过 --build-arg 注入）======
ARG VERSION=0.0.1
ARG VCS_REF=unknown
ARG BUILD_DATE=1970-01-01T00:00:00Z

LABEL org.opencontainers.image.title="convertor" \
      org.opencontainers.image.description="A profile converter for Surge/Clash." \
      org.opencontainers.image.url="https://github.com/BppleMan/convertor" \
      org.opencontainers.image.source="https://github.com/BppleMan/convertor" \
      org.opencontainers.image.documentation="https://github.com/BppleMan/convertor#readme" \
      org.opencontainers.image.vendor="BppleMan" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.created="${BUILD_DATE}"

# ====== 运行期依赖（HTTPS 出站需要 CA 证书）======
RUN apk add --no-cache ca-certificates && update-ca-certificates

# ====== 运行时参数（由 CI/compose 传入）======
ARG TARGET_TRIPLE=x86_64-unknown-linux-musl
ARG PROFILE_PATH=alpine
ARG BIN_NAME=convd
ARG BIN_PATH=target/${TARGET_TRIPLE}/${PROFILE_PATH}/${BIN_NAME}

# ====== 非 root 用户与工作目录 ======
# 说明：使用固定 UID/GID，方便与宿主机/卷权限配合
WORKDIR /app
RUN addgroup -S -g 10001 app \
    && adduser  -S -u 10001 -G app app \
    && chown -R app:app /app

# 复制本机已编译产物（用 --chown 直接赋权，避免额外层）
COPY --chown=app:app ${BIN_PATH} /app/convd

# 3) 运行期可写目录
ENV HOME=/app
USER app:app

RUN mkdir -p /app/.convertor

# ====== 缺省环境变量（可被外部覆盖）======
ENV RUST_LOG=info \
    SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
    TZ=UTC

# 文档化端口（容器间通信不依赖它，但利于可读性）
EXPOSE 8080

# 优雅退出（配合 docker --init 更佳）
STOPSIGNAL SIGTERM

# 如你的二进制以“监听地址”作第一个参数，这里保留
ENTRYPOINT ["/app/convd"]
CMD ["0.0.0.0:8080"]
