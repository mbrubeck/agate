FROM alpine:latest
RUN apk update
RUN apk add cargo
RUN wget https://github.com/mbrubeck/agate/archive/refs/tags/v3.0.2.tar.gz
RUN tar xzf v3.0.2.tar.gz
RUN ls /
RUN mv /agate-3.0.2 /agate
WORKDIR "/agate"
RUN ls
RUN cargo build --release
RUN mkdir /gmi/
CMD /agate/target/release/agate --content /gmi/ \
	--addr [::]:1965 \
	--addr 0.0.0.0:1965 \
	--hostname $GEMINI_HOST \
	--lang $GEMINI_LANG
