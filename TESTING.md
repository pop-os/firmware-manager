# Testing

This document provides a guideline for testing and verifying the expected behaviors of the project. When a patch is ready for testing, the checklists may be copied and marked as they are proven to be working.

## Checklists

Tasks for a tester to verify when approving a patch.

### Stand-alone Application

The application can be used to test that

- [ ] fwupd-managed device firmware is upgradeable
- [ ] On a system with downgraded fwupd-managed system firmware, verify that it can be upgraded
    - [ ] A dialog should appear with changelog details
- [ ] On a downgraded system76 system, verify that the it can be upgraded
    - [ ] A dialog should appear with changelog details
- [ ] On a Thelio with a downgraded I/O board, verify that it can be upgraded
    - [ ] A system with multiple Thelio I/O boards should only display one upgrade button
- [ ] UI
    - [ ] An error callback should display errors in an info bar
    - [ ] Progress bars should appear on devices being upgraded
    - [ ] On success, the firmware widget is displayed with the new version, and no button
    - [ ] On failure, an info bar details the cause of the error, and the button is shown again
    - [ ] Test that the refresh button works when new compatible devices are plugged in
    - [ ] Test hotplugging (the UI should refresh when new compatible devices are plugged in).

### GNOME Settings Integration

The GNOME Settings integration calls the exact same code as the application, with the exception to how the widget is added to a new Firmware panel in the Devices category, with a C-based callback for displaying the error messages in an info bar.

- [ ] The firmware panel loads, and can be interacted with
- [ ] The user can navigate out of the panel and back again, sigsegv-free
- [ ] An error callback should display an info bar in the panel on an error

## How To

Instructions for interacting with features for first-time testers.

### Downgrading fwupd-managed firmware

1. First, you need the `DeviceId` of the device to downgrade.
    ```sh
    ❯ fwupdmgr get-devices
    Unifying Receiver
      DeviceId:             61c9f6f93fb9b2def1a081d1e296cf72e59b8ae2
      Guid:                 9d131a0c-a606-580f-8eda-80587250b8d6
      Guid:                 279ed287-3607-549e-bacc-f873bb9838c4
      Summary:              A miniaturised USB wireless receiver
      Plugin:               unifying
      Flags:                updatable|supported|registered
      Vendor:               Logitech
      VendorId:             USB:0x046D
      Version:              RQR12.08_B0030
      VersionBootloader:    BOT01.02_B0015
      VersionFormat:        plain
      Icon:                 preferences-desktop-keyboard
      InstallDuration:      7
      Created:              2019-07-16
     ```
2. Then, copy the ID into the `downgrade` subcommand
    ```sh
    ❯ fwupdmgr downgrade 61c9f6f93fb9b2def1a081d1e296cf72e59b8ae2
    Downloading RQR12.07_B0029 for Unifying Receiver...
    Decompressing…           [***************************************]
    Authenticating…          [***************************************]
    Downgrading Unifying Receiver…
    Idle…                    [***************************************]
    Downgrading Unifying Receiver…***********************************]
    Writing…                 [***************************************]
    ```
