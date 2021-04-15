FROM alpine:latest
RUN apk add cargo && wget https://github.com/mbrubeck/agate/archive/refs/tags/v3.0.2.tar.gz && tar xzf v3.0.2.tar.gz && mv /agate-3.0.2 /agate && cd agate && cargo build --release
RUN cp /agate/target/release/agate /usr/bin/agate
WORKDIR /app
COPY . /app
ADD . .
ENTRYPOINT ["/bin/sh", "start.sh"]

