FROM alpine:3.20

# 容器元信息（通过构建参数注入）
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

# 安装 HTTPS 证书
RUN apk add --no-cache ca-certificates && update-ca-certificates

# 构建参数
ARG TARGET_TRIPLE=x86_64-unknown-linux-musl
ARG PROFILE=prod
ARG BIN_NAME=convd
ARG BIN_PATH=target/${TARGET_TRIPLE}/${PROFILE}/${BIN_NAME}

# 创建非特权用户
WORKDIR /app
RUN addgroup -S -g 10001 app \
    && adduser  -S -u 10001 -G app app \
    && chown -R app:app /app

# 复制编译好的二进制文件
COPY --chown=app:app ${BIN_PATH} /app/convd

# 切换到非特权用户
ENV HOME=/app
USER app:app

RUN mkdir -p /app/.convertor

# 环境变量
ENV RUST_LOG=info \
    SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
    TZ=UTC

EXPOSE 8080

STOPSIGNAL SIGTERM

ENTRYPOINT ["/app/convd"]
CMD ["0.0.0.0:8080"]
