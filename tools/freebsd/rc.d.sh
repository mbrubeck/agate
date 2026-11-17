#!/bin/sh

# PROVIDE: agate
# REQUIRE: LOGIN
# KEYWORD: shutdown
#
# agate_enable (bool):  Set to NO by default.
#                       Set it to YES to enable agate.
# agate_user:           default www
# agate_content:        default @PREFIX@/www/gemini
# agate_certs:          default @PREFIX@/etc/agate/
# agate_hostname:       e.g., gemini.example.tld, default hostname
# agate_addr:           default [::], listen on IPV4 and IPV6
# agate_port:           default 1965
# agate_lang:           default en-US
# agate_logfile:        default /var/log/gemini/agate.log

. /etc/rc.subr

desc="Agate Gemini server"
name=agate
rcvar=$name_enable

load_rc_config $name

: "${agate_enable:="NO"}"
: "${agate_user:="www"}"
: "${agate_content:="@PREFIX@/www/gemini/"}"
: "${agate_certs:="@PREFIX@/etc/agate/"}"
: "${agate_hostname:=$(uname -n)}"
: "${agate_addr:="[::]"}"
: "${agate_port:="1965"}"
: "${agate_lang:="en-US"}"
: "${agate_logfile:="/var/log/gemini/agate.log"}"

command="@BINDIR@/agate"
command_args="--content ${agate_content} \
       --certs ${agate_certs} \
       --addr ${agate_addr}:${agate_port} \
       --hostname ${agate_hostname} \
       --lang ${agate_lang} >> ${agate_logfile} 2>&1 &"

run_rc_command "$1"
