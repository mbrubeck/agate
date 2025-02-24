FROM docker.io/library/rust:alpine AS builder

WORKDIR /agate
RUN apk --no-cache add libc-dev

COPY src src
COPY Cargo.toml .
COPY Cargo.lock .
ARG TARGETARCH
ARG TARGETARCH
RUN if [ "$TARGETARCH" = "amd64" ]; then \
      cargo install --target x86_64-unknown-linux-musl --path . ; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
      cargo install --target aarch64-unknown-linux-musl --path . ; \
    else \
      echo "The architecture $TARGETARCH isn't unsupported." && exit 1; \
    fi

FROM docker.io/library/alpine:latest
COPY --from=builder /usr/local/cargo/bin/agate /usr/bin/agate
WORKDIR /app
EXPOSE 1965
VOLUME /gmi/
VOLUME /certs/

ENTRYPOINT ["agate", "--addr", "0.0.0.0:1965", "--content", "/gmi/", "--certs", "/certs/"]

