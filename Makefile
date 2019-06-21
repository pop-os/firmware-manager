prefix ?= /usr/local
bindir = $(prefix)/bin
includedir = $(prefix)/include
libdir = $(prefix)/lib

export CARGO_C_PREFIX = $(prefix)
export CARGO_C_LIBDIR = $(libdir)

TARGET = debug
DEBUG ?= 0

ifeq ($(DEBUG),0)
	TARGET = release
	ARGS += --release
endif

VENDORED ?= 0
ifeq ($(VENDORED),1)
	ARGS += --frozen
endif

APPID = com.system76.FirmwareManager
PACKAGE = firmware_manager
BINARY = target/$(TARGET)/firmware-manager
LIBRARY = target/$(TARGET)/lib$(PACKAGE).so
PKGCONFIG = target/$(PACKAGE).pc
HEADER = ffi/$(PACKAGE).h
VERSION = $(shell grep version Cargo.toml | head -1 | awk '{print $$3}' | tail -c +2 | head -c -2)

SOURCES = $(shell find src -type f -name '*.rs') Cargo.toml Cargo.lock
FFI_SOURCES = $(shell find ffi/src -type f -name '*.rs') \
	ffi/Cargo.toml ffi/build.rs ffi/$(PACKAGE).h

all: $(BINARY) $(LIBRARY) $(PKGCONFIG)

clean:
	cargo clean

distclean: clean
	rm -rf .cargo vendor $(PKGCONFIG) $(PKGCONFIG).stub

bin $(BINARY):
	cargo build $(ARGS)

$(LIBRARY): $(SOURCES) $(FFI_SOURCES)
	cargo build $(ARGS) -p firmware-manager-ffi

ffi: $(LIBRARY) $(PKGCONFIG)

install: install-bin install-ffi

install-bin:
	install -Dm0755 "$(BINARY)" "$(DESTDIR)$(bindir)/$(APPID)"

install-ffi:
	install -Dm0644 "$(HEADER)"    "$(DESTDIR)$(includedir)/$(PACKAGE).h"
	install -Dm0644 "$(LIBRARY)"   "$(DESTDIR)$(libdir)/lib$(PACKAGE).so"
	install -Dm0644 "$(PKGCONFIG)" "$(DESTDIR)$(libdir)/pkgconfig/$(PACKAGE).pc"

uninstall: uninstall-bin uninstall-ffi

uninstall-bin:
	rm "$(DESTDIR)$(bindir)/$(APPID)"

uninstall-ffi:
	rm "$(DESTDIR)$(includedir)/$(PACKAGE).h"
	rm "$(DESTDIR)$(libdir)/lib$(PACKAGE).so"
	rm "$(DESTDIR)$(libdir)/pkgconfig/$(PACKAGE).pc"

vendor:
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	tar pcfJ vendor.tar.xz vendor
	rm -rf vendor

$(PKGCONFIG):
	echo "libdir=$(libdir)" > "$@.partial"
	echo "includedir=$(includedir)" >> "$@.partial"
	cat "$(PKGCONFIG).stub" >> "$@.partial"
	mv "$@.partial" "$@"
