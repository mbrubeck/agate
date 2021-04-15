#!/bin/sh

exec agate --content /gmi/ \
	--hostname ${HOSTNAME} \
	--lang ${LANG}

