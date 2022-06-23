#!/bin/bash

PUID="${PUID:-0}"
PGID="${PGID:-0}"

CONTENT="${CONTENT:-/content}"
CERTIFICATES="${CERTIFICATES:-/content/.certificates}"

HOSTNAME="${HOSTNAME:-localhost}"
LANG="${LANG:-en-US}"

[[ -d "$CONTENT" ]] || mkdir "$CONTENT"
[[ -d "$CERTIFICATES" ]] || mkdir "$CERTIFICATES"

if ! (getent group "$PGID" 1>/dev/null 2>/dev/null); then
	addgroup -g "$PGID" agate
fi

if ! (getent passwd "$PUID" 1>/dev/null 2>/dev/null); then
	adduser -G agate -u 1000 -H -D -h "$CONTENT" agate
fi

sudo chown -R "$PUID:$PGID" "$CONTENT" "$CERTIFICATES"

sudo \
	--user="$(getent passwd "$PUID" | cut -d : -f 1)" \
	--group="$(getent group "$PUID" | cut -d : -f 1)" \
	/usr/bin/agate \
	--addr '0.0.0.0:1965' \
	--addr '[::]:1965' \
	--content "$CONTENT" \
	--certs "$CERTIFICATES" \
	--hostname "$HOSTNAME" \
	--lang "$LANG"
