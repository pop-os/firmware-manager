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

VENDOR ?= 0
ifneq ($(VENDOR),0)
	ARGS += --frozen
	DESKTOP_ARGS += --frozen
endif

features ?= fwupd system76

APPID = com.system76.FirmwareManager

GTKPROJ = gtk/Cargo.toml
GTKFFIPROJ = gtk/ffi/Cargo.toml
NOTPROJ = notify/Cargo.toml
PACKAGE = firmware_manager

DESKTOP = target/$(APPID).desktop
STARTUP_DESKTOP = target/$(APPID).Notify.desktop
GTKBINARY = target/$(TARGET)/firmware-manager-gtk
NOTBINARY = target/$(TARGET)/firmware-manager-notify
LIBRARY = target/$(TARGET)/lib$(PACKAGE).so
PKGCONFIG = target/$(PACKAGE).pc
HEADER = gtk/ffi/$(PACKAGE).h

VERSION = $(shell grep version Cargo.toml | head -1 | awk '{print $$3}' | tail -c +2 | head -c -2)

SOURCES = $(shell find src -type f -name '*.rs') Cargo.toml Cargo.lock \
	$(shell find tools/src -type f -name '*.rs') tools/Cargo.toml
FFI_SOURCES = $(shell find gtk/ffi/src -type f -name '*.rs') \
	gtk/ffi/Cargo.toml gtk/ffi/build.rs gtk/ffi/$(PACKAGE).h

all: $(GTKBINARY) $(NOTBINARY) $(LIBRARY) $(PKGCONFIG)

clean:
	cargo clean

distclean: clean
	rm -rf .cargo vendor vendor.tar.xz target


## Developer tools

clippy:
	cargo clippy --manifest-path $(GTKPROJ) $(ARGS) --features '$(features)'
	cargo clippy --manifest-path $(NOTPROJ) $(ARGS) --features '$(features)'

## Building the binaries

bin $(GTKBINARY): $(DESKTOP) vendor-check
	cargo build --manifest-path $(GTKPROJ) $(ARGS) --features '$(features)'

bin-notify $(NOTBINARY): $(STARTUP_DESKTOP) vendor-check
	cargo build --manifest-path $(NOTPROJ) $(ARGS) --features '$(features)'

## Builds the desktop entry in the target directory.

desktop $(DESKTOP): vendor-check
	cargo run -p tools --bin desktop-entry $(DESKTOP_ARGS) -- \
		--appid $(APPID) \
		--name "System76 Firmware" \
		--icon firmware-manager \
		--comment "Manage system and device firmware" \
		--keywords firmware \
		--keywords system76 \
		--keywords fwupd \
		--keywords lvfs \
		--categories System \
		--categories GTK \
		--binary $(APPID) \
		--prefix $(prefix) \
		--startup-notify

notify $(STARTUP_DESKTOP): vendor-check
	cargo run -p tools --bin desktop-entry $(DESKTOP_ARGS) -- \
		--appid $(APPID).Notify \
		--name "System76 Firmware Check" \
		--icon firmware-manager \
		--comment "Check for firmware updates, and display notification if found" \
		--categories System \
		--binary $(APPID).Notify \
		--prefix $(prefix) \

## Building the library

ffi: $(LIBRARY) $(PKGCONFIG)

$(LIBRARY): $(SOURCES) $(FFI_SOURCES) vendor-check
	cargo build --manifest-path $(GTKFFIPROJ) $(ARGS) --features '$(features)'

## Builds the pkg-config file necessary to locate the library.

$(PKGCONFIG): tools/src/pkgconfig.rs
	cargo run -p tools --bin pkgconfig $(DESKTOP_ARGS) -- \
		$(PACKAGE) $(libdir) $(includedir)

## Install commands

install: install-bin install-ffi install-notify

install-bin:
	install -Dm0755 "$(GTKBINARY)"  "$(DESTDIR)$(bindir)/$(APPID)"
	install -Dm0644 "$(DESKTOP)" "$(DESTDIR)$(prefix)/share/applications/$(APPID).desktop"

install-ffi:
	install -Dm0644 "$(HEADER)"    "$(DESTDIR)$(includedir)/$(PACKAGE).h"
	install -Dm0644 "$(LIBRARY)"   "$(DESTDIR)$(libdir)/lib$(PACKAGE).so"
	install -Dm0644 "$(PKGCONFIG)" "$(DESTDIR)$(libdir)/pkgconfig/$(PACKAGE).pc"

install-notify:
	install -Dm0755 "$(NOTBINARY)"  "$(DESTDIR)$(bindir)/$(APPID).Notify"
	install -Dm0644 "$(STARTUP_DESKTOP)"  "$(DESTDIR)/etc/xdg/autostart/$(APPID).Notify.desktop"

## Uninstall Commands

uninstall: uninstall-bin uninstall-ffi

uninstall-bin:
	rm "$(DESTDIR)$(bindir)/$(APPID)"

uninstall-ffi:
	rm "$(DESTDIR)$(includedir)/$(PACKAGE).h"
	rm "$(DESTDIR)$(libdir)/lib$(PACKAGE).so"
	rm "$(DESTDIR)$(libdir)/pkgconfig/$(PACKAGE).pc"

## Cargo Vendoring

vendor:
	rm .cargo -rf
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	tar pcfJ vendor.tar.xz vendor
	rm -rf vendor

vendor-check:
ifeq ($(VENDOR),1)
	test -e vendor || tar pxf vendor.tar.xz
endif
