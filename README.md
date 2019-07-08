# Firmware Manager

A GTK firmware management widget from System76, written in Rust, with optional C FFI support. It supports fetching and installing firmware from `system76-firmware` and `fwupd`.

## Build Instructions

This project uses a Makefile. When building the application, the `prefix` flag must be provided, so that the desktop entry file is generated to point to the correct path of the target binary after installation.

```sh
make prefix=/usr features='fwupd system76'
sudo make install prefix=/usr
```

Note that the generated desktop entry is stored in the `target` directory, where the `pkgconfig` file is also stored after it is generated. If you need to regenerate the desktop entry with a different prefix, you can manually call the `desktop` rule.

```
make desktop prefix=/usr
```

### Conditional Features

There are also two conditional features of the crate:

- `system76`: enables support for the system76-firmware daemon
- `fwupd`: enables support for the fwupd DBus daemon

These must be passed into the makefile with the `features` flag. At least one feature must be specified, otherwise a compiler error will occur.

### Debug Binaries

To build a debug binary, pass `DEBUG=1` into make.

```sh
make prefix=/usr features='fwupd system76' DEBUG=1
sudo make install DEBUG=1
```

### Vendoring

To vendor the project for packaging, call `make vendor`. To build a project that has been vendored, pass `VENDOR=1` to the makefile.

```sh
make vendor
make prefix=/usr features='fwupd system76' VENDOR=1
```

## API Overview

This section provides details about the API and how to call it from Rust or C.

### Rust API

The primary API, which the C API is based upon. An example of the Rust API in practice in a GTK application can be found [here](./src/main.rs).

```rust
use system76_firmware_manager::FirmwareWidget;

// Create a new firmware widget
//
// This spawns a background thread which listens for widget events until
// the `Quit` signal is received, which occurs when the firmware widget
// is dropped.
let mut firmware = FirmwareWidget::new();

// Signal the widget's background thread to begin scanning for firmware.
firmware.scan();

// Get the GTK widget from the firmware widget to add into a window.
let widget = firmware.container();
```

### C API

The Rust library also supports C interface with FFI rules in the Makefile for gnerating a dynamic C library with `pkg-config` support. This is integrated in GNOME Settings on Pop!_OS.

```sh
make ffi prefix=/usr features='system76 fwupd'
sudo make install-ffi prefix=/usr
```

Which can then be imported into a C code base with:

```c
#include <s76_firmware.h>

// Create a new firmware widget
S76FirmwareWidget *firmware =
    s76_firmware_widget_new ();

// Signal the widget's background thread to begin scanning for firmware.
s76_firmware_widget_scan (firmware);

// Get the GTK widget from the firmware widget to attach it to a container.
GtkWidget *firmware_widget =
    s76_firmware_widget_container (firmware);

// Destroy the widget and signal its background thread to quit.
s76_firmware_widget_destroy (firmware);
```

The C implementation of the Rust application is [here](./ffi/examples/c), demonstrated with the Meson build system.
