FROM docker.io/library/rust:alpine AS builder
# RUN wget -O source.tar.gz $(wget -qO- https://api.github.com/repos/mbrubeck/agate/releases/latest | sed -nE 's/^.*"tarball_url"\s*:\s*"([^"]+)".*$/\1/p')&& tar xzf source.tar.gz && \
#     mv mbrubeck-agate-* /agate

WORKDIR /agate
COPY src src
COPY Cargo.toml .
COPY Cargo.lock .
COPY Cross.toml .
RUN apk --no-cache add libc-dev && \
    cargo install --target x86_64-unknown-linux-musl --path .

FROM docker.io/library/alpine:latest
COPY --from=builder /usr/local/cargo/bin/agate /usr/bin/agate
WORKDIR /app
COPY tools/docker/start.sh /app
ENTRYPOINT ["/bin/sh", "start.sh"]

