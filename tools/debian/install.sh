#!/bin/bash
# This file is part of the Agate software and licensed under either the
# MIT license or Apache license at your option.
#
# Please keep in mind that there is not warranty whatsoever provided for this
# software as specified in the disclaimer in the MIT license or section 7 of
# the Apache license respectively.

echo -n "checking:agate......."
if command -v agate >/dev/null
then
	echo "found"
else
	echo "FAILED"
	echo "Agate is probably not in your PATH variable."
	echo "If you installed it with cargo, try linking the binary to /usr/local/bin with something like this:"
	echo "    ln -s $HOME/.cargo/bin/agate /usr/local/bin/agate"
	echo "or what seems reasonable to you."
	exit 1
fi

echo -n "checking:systemd....."
if [[ "$(cat /proc/1/comm)" != "systemd" ]]
then
	echo "NOT THE INIT PROCESS"
	echo "Your system seems to not use systemd, sorry. Aborting."
	exit 1
else
	echo "installed and running"
fi

echo -n "checking:rsyslogd...."
if command -v rsyslogd >/dev/null
then
	echo -n "installed"
	if ps cax | grep -q "rsyslogd"
	then
		echo " and running"
	else
		echo " but not running!"
		echo "You should enable rsyslogd to use this functionality."
	fi
else
	echo "NOT INSTALLED!"
	echo "Aborting."
	exit 1
fi

echo -n "checking:logrotate..."
if type logrotate >/dev/null 2>&1
then
	echo "installed, but I cannot check if it is enabled"
else
	echo "NOT INSTALLED!"
	echo "Aborting."
	exit 1
fi

# immediately exit if one of the following commands fails
set -e

echo "copying config files..."
cp gemini.service /etc/systemd/system/
cp gemini.conf /etc/rsyslog.d/
cp geminilogs /etc/logrotate.d/

echo "setting up content files..."
mkdir -p /srv/gemini/content
mkdir -p /srv/gemini/.certificates
# agate will generate certificates on first run

echo "starting service..."
systemctl daemon-reload
systemctl restart rsyslog
systemctl enable gemini
systemctl start gemini

echo "setup done, checking..."
# wait until the restarts would have timed out
sleep 10
systemctl status gemini
