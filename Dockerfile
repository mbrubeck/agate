FROM docker.io/library/rust:alpine AS builder

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
EXPOSE 1965
VOLUME /gmi/
VOLUME /certs/

ENTRYPOINT ["agate", "--addr", "0.0.0.0:1965", "--content", "/gmi/", "--certs", "/certs/"]

