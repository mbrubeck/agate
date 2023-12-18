#!/bin/sh

# $FreeBSD$
#
# PROVIDE: agate
# REQUIRE: LOGIN
# KEYWORD: shutdown
#
# Add these lines to /etc/rc.conf.local or /etc/rc.conf
# to enable this service:
#
# agate_enable (bool):  Set to NO by default.
#                       Set it to YES to enable agate.
# agate_user:           default www
# agate_content:        default /usr/local/www/gemini
# agate_key:            default /usr/local/etc/gemini/ssl/key.der
# agate_cert:           default /usr/local/etc/gemini/ssl/cert.der
# agate_hostname:       e.g., gemini.example.tld, default hostname
# agate_addr:           default [::], listen on IPV4 and IPV6
# agate_port:           default 1965
# agate_lang:           default en_US
# agate_logfile:        default /var/log/gemini/agate.log

. /etc/rc.subr

desc="Agate Gemini server"
name=agate
rcvar=$name_enable

load_rc_config $name

: ${agate_enable:="NO"}
: ${agate_user:="www"}
: ${agate_content:="/usr/local/www/gemini/"}
: ${agate_key:="/usr/local/etc/gemini/ssl/key.der"}
: ${agate_cert:="/usr/local/etc/gemini/ssl/cert.der"}
: ${agate_hostname:=`uname -n`}
: ${agate_addr:="[::]"}
: ${agate_port:="1965"}
: ${agate_lang:="en-US"}
: ${agate_logfile:="/var/log/gemini/agate.log"}

agate_user=${agate_user}

command="/usr/local/bin/agate"
command_args="--content ${agate_content} \
       --key ${agate_key} \
       --cert ${agate_cert} \
       --addr ${agate_addr}:${agate_port} \
       --hostname ${agate_hostname} \
       --lang ${agate_lang} >> ${agate_logfile} 2>&1 &"

run_rc_command "$1"
