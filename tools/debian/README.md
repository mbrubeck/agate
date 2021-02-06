If you want to run agate on a pretty much standard Debian install, this
directory contains some additional materials that may help you.

Please keep in mind that there is no warranty whatsoever provided for this
software as specified in the disclaimer in the MIT license or section 7 of
the Apache license respectively.

To run Agate as a service with systemd, put the `gemini.service` file
in the directory `/etc/systemd/system/` (copy or move it there).

This service file has some comments you may want to look at before using it!

If you use the service file and want the agate logs in a separate file,
using the gemini.conf file and putting it in the directory
`/etc/rsyslog.d/` will make the agate log messages appear in a file
called `/var/log/gemini.log`.

If you use Debians `logrotate` and want to automatically rotate these log files,
you can use the `geminilogs` file and put it in `/etc/logrotate.d/`.

You can also use the `install.sh` file which will check if these systems
are installed (but not if they are running) and copy the files to their
described locations. Please ensure your systems hostname is set correctly
(i.e. `uname -n` should give your domain name).

You will have to run this with elevated privileges, i.e. `sudo ./install.sh`
to work correctly. This install script will also create the necessary content
directories and the certificate and private key in the `/srv/gemini/`
directory. After the script is done sucessfully, you can start by putting
content in `/srv/gemini/content/`, the server is running already!
