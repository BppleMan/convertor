FROM alpine:3.20

# buildx 构建参数
ARG TARGETARCH
# 容器元信息（通过构建参数注入）
ARG TITLE=convertor
ARG NAME=base
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

RUN apk add --no-cache ca-certificates tzdata && update-ca-certificates


RUN addgroup -S -g 10001 app \
    && adduser  -S -u 10001 -G app app \
    && mkdir -p /app/.convertor \
    && chown -R app:app /app


USER app:app
WORKDIR /app
ENV HOME=/app TZ=UTC
