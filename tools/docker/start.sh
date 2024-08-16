#!/bin/sh

echo "Using hostname ${HOSTNAME:-localhost}"
echo "Using lang ${LANG:-en-US}"

exec agate --content /gmi/ \
	--hostname "${HOSTNAME:-localhost}" \
	--lang "${LANG:-en-US}"
