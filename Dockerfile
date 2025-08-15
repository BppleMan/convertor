FROM alpine:latest

ARG VERSION=0.0.1
LABEL org.opencontainers.image.title="convertor" \
    org.opencontainers.image.version="${VERSION}" \
    org.opencontainers.image.source="https://github.com/BppleMan/convertor" \
    org.opencontainers.image.description="A profile converter for surge/clash." \
    org.opencontainers.image.licenses="Apache-2.0"

COPY target/x86_64-unknown-linux-musl/debug/convertor /convertor

EXPOSE 80

ENTRYPOINT ["/convertor"]
CMD ["0.0.0.0:80"]
