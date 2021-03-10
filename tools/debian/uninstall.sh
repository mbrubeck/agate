#!/bin/bash
# This file is part of the Agate software and licensed under either the
# MIT license or Apache license at your option.
#
# Please keep in mind that there is not warranty whatsoever provided for this
# software as specified in the disclaimer in the MIT license or section 7 of
# the Apache license respectively.

echo "stopping and disabling service..."
systemctl stop gemini
systemctl disable gemini

echo "removing config files..."
rm -f /etc/systemd/system/gemini.service /etc/rsyslog.d/gemini.conf /etc/logrotate.d/geminilogs

echo "deleting certificates..."
rm -rf /srv/gemini/.certificates
# do not delete content files, user might want to use them still or can delete them manually
echo "NOTE: content files at /srv/gemini/content not deleted"
# cannot uninstall executable since we did not install it
echo "NOTE: agate executable at $(which agate) not uninstalled"
