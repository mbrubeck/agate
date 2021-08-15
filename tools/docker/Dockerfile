FROM alpine:latest
RUN apk add cargo jq && wget -O source.tar.gz https://github.com/mbrubeck/agate/archive/refs/tags/$(wget -qO- https://api.github.com/repos/mbrubeck/agate/releases/latest | jq -r ".name").tar.gz && tar xzf source.tar.gz && mv /agate-* /agate && cd agate && cargo build --release
RUN cp /agate/target/release/agate /usr/bin/agate
WORKDIR /app
COPY . /app
ADD . .
ENTRYPOINT ["/bin/sh", "start.sh"]

