FROM alpine:3.20
RUN apk add --no-cache ca-certificates tzdata && update-ca-certificates
RUN addgroup -S -g 10001 app \
    && adduser  -S -u 10001 -G app app \
    && mkdir -p /app/.convertor \
    && chown -R app:app /app
USER app:app
WORKDIR /app
ENV HOME=/app TZ=UTC
