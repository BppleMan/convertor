FROM ghcr.io/bppleman/convertor/base:alpine3.20

# buildx 构建参数
ARG TARGETARCH
# 容器元信息（通过构建参数注入）
ARG TITLE=convertor
ARG NAME=convd
ARG DESCRIPTION="A profile converter for Surge/Clash."
ARG URL="https://github.com/BppleMan/convertor"
ARG SOURCE="${URL}"
ARG DOCUMENTATION="${URL}#readme"
ARG VENDOR=BppleMan
ARG LICENSE=Apache-2.0
ARG VERSION=0.0.1
ARG BUILD_DATE=1970-01-01T00:00:00Z
ARG VCS_REF=unknown

LABEL org.opencontainers.image.title="${TITLE}" \
    org.opencontainers.image.description="${DESCRIPTION}" \
    org.opencontainers.image.url="${URL}" \
    org.opencontainers.image.source="${SOURCE}" \
    org.opencontainers.image.documentation="${DOCUMENTATION}" \
    org.opencontainers.image.vendor="${VENDOR}" \
    org.opencontainers.image.licenses=$LICENSE \
    org.opencontainers.image.version="${VERSION}" \
    org.opencontainers.image.revision="${VCS_REF}" \
    org.opencontainers.image.created="${BUILD_DATE}"

# 复制编译好的二进制文件
COPY --chown=app:app dist/${TARGETARCH}/${NAME} /app/convd
RUN mkdir -p /app/.convertor

EXPOSE 8080
STOPSIGNAL SIGTERM
ENTRYPOINT ["/app/convd"]
CMD ["0.0.0.0:8080"]
