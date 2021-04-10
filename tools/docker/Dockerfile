FROM alpine:latest
RUN wget https://github.com/mbrubeck/agate/releases/download/v3.0.2/agate.$(uname -m)-unknown-linux-gnu.gz && gunzip agate.$(uname -m)-unknown-linux-gnu.gz && mv agate.$(uname -m)-unknown-linux-gnu /agate && ls && chmod +x /agate
CMD /agate --content /gmi/ \
	--hostname $GEMINI_HOST \
	--lang $GEMINI_LANG
