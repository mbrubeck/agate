target/agate.rc.d: tools/freebsd/rc.d.sh target/stamp.config
	@mkdir -p "$(dir $@)"
	sed -e 's|@BINDIR@|$(BINDIR)|g' \
	    -e 's|@PREFIX@|$(PREFIX)|g' \
	    tools/freebsd/rc.d.sh >$@
