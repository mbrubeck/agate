# User-settable settings.
DESTDIR ?=
FLAVOR ?= $(shell uname -s | tr A-Z a-z)
PREFIX ?= /usr/local

# Derived directories, mimicking autoconf/automake names.
# Can also be overridden for weird installation layouts, but this should be rare.
# If you override these, installcheck won't work though.
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
DOCDIR ?= $(DATADIR)/doc/agate
EXAMPLESDIR ?= $(DATADIR)/examples/agate

# The default build target. This MUST come first.
.PHONY: all
all: release

# Config change detection: list of vars that influence outputs.
# Can be extended by platform-specific code, so keep this before the include.
CONFIG_VARS = \
	BINDIR=$(BINDIR) \
	DATADIR=$(DATADIR) \
	DOCDIR=$(DOCDIR) \
	EXAMPLESDIR=$(EXAMPLESDIR) \
	FLAVOR=$(FLAVOR) \
	PREFIX=$(PREFIX)

ifeq ($(wildcard tools/$(FLAVOR)/Makefile.inc),)
FLAVOR = generic
endif
include tools/$(FLAVOR)/Makefile.inc

# Config change detection, stage 1: unconditionally dump settings to a file.
.PHONY: target/stamp.config.new
target/stamp.config.new:
	@mkdir -p "$(dir $@)"
	@printf '%s\n' '$(CONFIG_VARS)' >"$@"

# Config change detection, stage 2: update timestamps only if stage 1 changed.
target/stamp.config: target/stamp.config.new
	@mkdir -p "$(dir $@)"
	@cmp -s "$<" "$@" || cp "$<" "$@"

.PHONY: release
release: target/release/agate

SRCS := Cargo.toml Cargo.lock \
    $(shell find "src" "tests" \( -name "*.rs" -o -name "Cargo.*" \) -and -not -path "./target/*")

target/release/agate: $(SRCS)
	cargo build --release
	@touch "$@"

.PHONY: install-common
install-common: target/release/agate
	install -d "$(DESTDIR)$(BINDIR)"
	install -m 755 "target/release/agate" "$(DESTDIR)$(BINDIR)/agate"
	install -d "$(DESTDIR)$(DOCDIR)"
	install -m 644 README.md CHANGELOG.md LICENSE-APACHE LICENSE-MIT "$(DESTDIR)$(DOCDIR)/"
	install -d "$(DESTDIR)$(EXAMPLESDIR)/content"
	install -m 644 content/index.gmi "$(DESTDIR)$(EXAMPLESDIR)/content/index.gmi"

.PHONY: install
install: install-common install-tools

target/all.mtree: tests/common.mtree $(wildcard tools/$(FLAVOR)/tools.mtree) target/stamp.config
	@mkdir -p "$(dir $@)"
	cat $(filter %.mtree,$^) >$@

.PHONY: installcheck
installcheck: install target/all.mtree
	mtree -f target/all.mtree -p "$(DESTDIR)$(PREFIX)"
	"$(DESTDIR)$(PREFIX)/bin/agate" --help >/dev/null

.PHONY: clean
clean:
	cargo clean
