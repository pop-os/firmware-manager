# Firmware Manager

A GTK firmware management widget from System76, written in Rust, with optional C FFI support.

## Rust API

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

An example of the Rust API in practice in a GTK application can be found [here](./src/main.rs).

## C API

The Rust library also supports C interface with FFI rules in the Makefile for gnerating a dynamic C library with `pkg-config` support. This is integrated in GNOME Settings on Pop!_OS.

```sh
make ffi prefix=/usr
sudo make install-ffi prefix=/usr
```

Which can then be imported into a C code base with:

```c
#include <s76_firmware.h>

// Create a new firmware widget
S76FirmwareWidget *firmware =
    s76_firmware_widget_new ();

// Signal the widget's background thread
// to begin scanning for firmware.
s76_firmware_widget_scan (firmware);

// Get the GTK widget from the firmware widget
// to attach it to a container.
GtkWidget *firmware_widget =
    s76_firmware_widget_container (firmware);

// Destroy the widget and signal its
// background thread to quit.
s76_firmware_widget_destroy (firmware);
```

The C implementation of the Rust application is [here](./ffi/examples/c), demonstrated with the Meson build system.