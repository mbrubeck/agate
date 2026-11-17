RCDIR ?= $(PREFIX)/etc/rc.d

CONFIG_VARS += RCDIR=$(RCDIR)

target/agate.rc.d: tools/freebsd/rc.d.sh target/stamp.config
	@mkdir -p "$(dir $@)"
	sed -e 's|@BINDIR@|$(BINDIR)|g' \
	    -e 's|@PREFIX@|$(PREFIX)|g' \
	    tools/freebsd/rc.d.sh >$@

.PHONY: install-tools
install-tools: target/agate.rc.d
	install -d "$(DESTDIR)$(DOCDIR)"
	install -m 755 tools/freebsd/README.md "$(DESTDIR)$(DOCDIR)/README.freebsd.md"
	install -d "$(DESTDIR)$(RCDIR)"
	install -m 755 target/agate.rc.d "$(DESTDIR)$(RCDIR)/agate"
	install -d "$(DESTDIR)$(PREFIX)/etc/newsyslog.conf.d"
	install -m 644 tools/freebsd/newsyslog.conf "$(DESTDIR)$(PREFIX)/etc/newsyslog.conf.d/agate.conf"
