#!/bin/sh

# PROVIDE: agate
# REQUIRE: NETWORKING LOGIN
# KEYWORD: shutdown
#
# agate_enable (bool):  Set to NO by default.
#                       Set it to YES to enable agate.
# agate_username:       default www
# agate_content:        default @PREFIX@/www/gemini
# agate_certs:          default @PREFIX@/etc/agate/
# agate_hostname:       e.g., gemini.example.tld, default hostname
# agate_addr:           default [::], listen on IPV4 and IPV6
# agate_port:           default 1965
# agate_lang:           default en-US
# agate_flags:          extra flags

. /etc/rc.subr

desc="Agate Gemini server"
name=agate
rcvar="${name}_enable"
command="/usr/sbin/daemon"
pidfile="/var/run/agate.pid"
start_cmd="${name}_start"
required_files="@BINDIR@/agate"

load_rc_config $name

: "${agate_enable:="NO"}"
: "${agate_username:="www"}"
: "${agate_content:="@PREFIX@/www/gemini/"}"
: "${agate_certs:="@PREFIX@/etc/agate/"}"
: "${agate_hostname:=$(uname -n)}"
: "${agate_addr:="[::]"}"
: "${agate_port:="1965"}"
: "${agate_lang:="en-US"}"
: "${agate_flags:=}"

agate_start()
{
    if [ ! -d "${agate_certs}" ]; then
        mkdir -p "${agate_certs}"
        chmod 700 "${agate_certs}"
        chown "${agate_username}" "${agate_certs}"
    fi

    if [ ! -f /var/log/agate.log ]; then
        touch /var/log/agate.log
        chmod 600 /var/log/agate.log
        chown "${agate_username}" /var/log/agate.log
    fi

    echo "Starting ${name}."
    /usr/sbin/daemon -P "${pidfile}" -o /var/log/agate.log -H \
        -u "${agate_username}" -t agate @BINDIR@/agate \
        --content "${agate_content}" \
        --certs "${agate_certs}" \
        --addr "${agate_addr}:${agate_port}" \
        --hostname "${agate_hostname}" \
        --lang "${agate_lang}" \
        ${agate_flags}
}

run_rc_command "$1"
