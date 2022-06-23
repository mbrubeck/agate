FROM rust:alpine as builder

RUN apk add --no-cache cargo git
RUN git clone https://github.com/mbrubeck/agate.git /usr/src/agate
RUN cargo install --path /usr/src/agate

FROM alpine
RUN apk add --no-cache bash tini sudo
COPY --from=builder /usr/local/cargo/bin/agate /usr/bin/agate
COPY --from=builder /usr/src/agate/content /content

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 1965
VOLUME /content
ENTRYPOINT ["/sbin/tini", "-g", "--", "/entrypoint.sh"]
