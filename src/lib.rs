#![deny(missing_docs)]

//! # Firmware Manager Core
//!
//! The firmware manager core manages all firmware tasks in an event loop, and provides a handful of
//! useful capabilities that are useful to frontends of the firmware manager.

#[macro_use]
extern crate err_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate shrinkwraprs;

mod cache;
mod timestamp;
mod udev;
mod users;
mod version_sorting;

#[cfg(feature = "fwupd")]
mod fwupd;
#[cfg(feature = "system76")]
mod system76;

pub use self::users::user_is_admin;

#[cfg(feature = "fwupd")]
pub use fwupd_dbus::{
    Client as FwupdClient, Device as FwupdDevice, Error as FwupdError, Release as FwupdRelease,
};

#[cfg(feature = "system76")]
pub use system76_firmware_daemon::{
    Changelog as System76Changelog, Digest as System76Digest, Error as System76Error,
    SystemInfo as S76SystemInfo, ThelioIoInfo,
};

#[cfg(feature = "fwupd")]
pub use self::fwupd::*;
#[cfg(feature = "system76")]
pub use self::system76::*;

#[cfg(feature = "system76")]
pub use system76_firmware_daemon::Client as System76Client;

pub use self::udev::usb_hotplug_event_loop;
use self::version_sorting::sort_versions;
pub use slotmap::DefaultKey as Entity;
use slotmap::{SlotMap, SparseSecondaryMap};
use std::{
    io,
    process::Command,
    sync::{mpsc::Receiver, Arc},
};

/// Errors that may occur in the firmware manager core.
#[derive(Debug, Error)]
pub enum Error {
    /// Errors specific to fwupd devices.
    #[cfg(feature = "fwupd")]
    #[error(display = "error in fwupd client")]
    Fwupd(#[error(cause)] fwupd_dbus::Error),
    /// Errors specific to system76 devices.
    #[cfg(feature = "system76")]
    #[error(display = "error in system76-firmware client")]
    System76(#[error(cause)] System76Error),
}

/// A request for the background event loop to perform.
#[derive(Debug)]
pub enum FirmwareEvent {
    /// Upgrade the firmware of a fwupd-compatible device.
    #[cfg(feature = "fwupd")]
    Fwupd(Entity, Arc<FwupdDevice>, Arc<FwupdRelease>),

    /// Stop processing events.
    Stop,

    /// Upgrade system firmware for System76 systems.
    #[cfg(feature = "system76")]
    S76System(Entity, System76Digest),

    /// Search for available firmware devices.
    Scan,

    /// Upgrade the firmware of Thelio I/O boarods.
    #[cfg(feature = "system76")]
    ThelioIo(Entity, System76Digest),
}

/// Information about a device and its current and latest firmware.
#[derive(Debug)]
pub struct FirmwareInfo {
    /// The name of this device.
    pub name: Box<str>,

    /// The currently-installed version.
    pub current: Box<str>,

    /// The latest version of firmware for this device.
    pub latest: Option<Box<str>>,

    /// The time required for this firmware to be flashed, in seconds.
    pub install_duration: u32,
}

/// A collection of all firmware device entities that a frontend is managing.
///
/// This only contains the entity keys, and whether that entity is system firmware or not.
/// The frontend is responsible for creating secondary maps that will store data specific to
/// the entities contained within this map.
#[derive(Debug, Default, Shrinkwrap)]
pub struct Entities {
    /// The primary storage to record all device entities.
    #[shrinkwrap(main_field)]
    pub entities: SlotMap<Entity, ()>,

    /// Secondary storage to keep record of all system devices.
    pub system: SparseSecondaryMap<Entity, ()>,
}

impl Entities {
    /// Associate this entity as a system device
    pub fn associate_system(&mut self, entity: Entity) { self.system.insert(entity, ()); }

    /// Clear all entities from the world
    ///
    /// Entities are automatically erased from secondary storages on lookup
    pub fn clear(&mut self) { self.entities.clear(); }

    /// Create a new device entity.
    pub fn create(&mut self) -> Entity { self.entities.insert(()) }

    /// Check if an entity is a system device
    pub fn is_system(&self, entity: Entity) -> bool { self.system.contains_key(entity) }
}

/// Signals that the firmware manager core will send to a frontend.
///
/// This will keep a frontend informed on the current progress of an action, or events which have
/// been triggered. Entity keys are assigned with most types of events to associate the events with
/// the firmware devices which initiated the event.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum FirmwareSignal {
    /// A device has initiated the flashing process.
    DeviceFlashing(Entity),

    /// A device was updated
    DeviceUpdated(Entity),

    /// Signals that the entity's firmware is being downloaded.
    DownloadBegin(Entity, u64),

    /// Signals completion of an entity's firmware download.
    DownloadComplete(Entity),

    /// Progress updates on firmware downloads.
    DownloadUpdate(Entity, usize),

    /// An error occurred
    Error(Option<Entity>, Error),

    /// Fwupd firmware was discovered.
    #[cfg(feature = "fwupd")]
    Fwupd(FwupdSignal),

    /// Devices are being scanned
    Scanning,

    /// Signals when scanning has completed.
    ScanningComplete,

    /// System firmware was scheduled for installation.
    SystemScheduled,

    /// System76 system firmware was discovered.
    #[cfg(feature = "system76")]
    S76System(FirmwareInfo, Option<(System76Digest, System76Changelog)>),

    /// Thelio I/O firmware was discovered.
    #[cfg(feature = "system76")]
    ThelioIo(FirmwareInfo, Option<System76Digest>),
}

/// An event loop that should be run in the background, as this function will block until
/// the stop signal is received.
pub fn event_loop<F: Fn(FirmwareSignal)>(receiver: Receiver<FirmwareEvent>, sender: F) {
    #[cfg(feature = "system76")]
    let s76 = get_client("system76", s76_firmware_is_active, System76Client::new);

    #[cfg(feature = "fwupd")]
    let fwupd = {
        // Use Ping() to wake up fwupd, and to check if it exists.
        let fwupd_connect = || {
            let client = FwupdClient::new()?;
            client.ping()?;
            Ok(client)
        };

        get_client::<_, _, fwupd_dbus::Error>("fwupd", || true, fwupd_connect)
    };

    #[cfg(feature = "fwupd")]
    let http_client = &reqwest::blocking::Client::new();

    while let Ok(event) = receiver.recv() {
        trace!("event loop received firmware event: {:?}", event);
        match event {
            FirmwareEvent::Scan => {
                let sender = &sender;
                sender(FirmwareSignal::Scanning);

                #[cfg(feature = "system76")]
                {
                    if let Some(ref client) = s76 {
                        s76_scan(client, sender);
                    }
                }

                #[cfg(feature = "fwupd")]
                {
                    if let Some(ref client) = fwupd {
                        if let Err(why) = fwupd_updates(client, http_client) {
                            eprintln!("failed to update fwupd remotes: {}", why);
                        }
                        fwupd_scan(client, sender);
                    }
                }

                let _ = sender(FirmwareSignal::ScanningComplete);
            }
            #[cfg(feature = "fwupd")]
            FirmwareEvent::Fwupd(entity, device, release) => {
                let flags = fwupd_dbus::InstallFlags::empty();
                let event = match fwupd.as_ref().map(|fwupd| {
                    fwupd.update_device_with_release(
                        http_client,
                        &device,
                        &release,
                        flags,
                        Some(|download_event| {
                            use fwupd_dbus::FlashEvent::*;
                            let event = match download_event {
                                DownloadUpdate(progress) => {
                                    FirmwareSignal::DownloadUpdate(entity, progress)
                                }
                                DownloadInitiate(size) => {
                                    FirmwareSignal::DownloadBegin(entity, size)
                                }
                                DownloadComplete => FirmwareSignal::DownloadComplete(entity),
                                FlashInProgress => FirmwareSignal::DeviceFlashing(entity),
                                VerifyingChecksum => return,
                            };

                            sender(event);
                        }),
                    )
                }) {
                    Some(Ok(_)) => FirmwareSignal::DeviceUpdated(entity),
                    Some(Err(why)) => FirmwareSignal::Error(Some(entity), why.into()),
                    None => panic!("fwupd event assigned to non-fwupd button"),
                };

                sender(event);
            }
            #[cfg(feature = "system76")]
            FirmwareEvent::S76System(entity, digest) => {
                match s76.as_ref().map(|client| client.schedule(&digest)) {
                    Some(Ok(_)) => sender(FirmwareSignal::SystemScheduled),
                    Some(Err(why)) => sender(FirmwareSignal::Error(Some(entity), why.into())),
                    None => panic!("thelio event assigned to non-thelio button"),
                }
            }
            #[cfg(feature = "system76")]
            FirmwareEvent::ThelioIo(entity, digest) => {
                sender(FirmwareSignal::DeviceFlashing(entity));
                let event = match s76.as_ref().map(|client| client.thelio_io_update(&digest)) {
                    Some(Ok(_)) => FirmwareSignal::DeviceUpdated(entity),
                    Some(Err(why)) => FirmwareSignal::Error(Some(entity), why.into()),
                    None => panic!("thelio event assigned to non-thelio button"),
                };

                sender(event);
            }
            FirmwareEvent::Stop => {
                trace!("received quit signal");
                break;
            }
        }
    }
}

/// Function for getting a timmed string from a file.
fn read_trimmed(path: &str) -> io::Result<String> {
    let mut vendor = std::fs::read_to_string(path)?;
    vendor.truncate(vendor.trim_end().len());
    Ok(vendor)
}

/// Convenience function for reading the product name from the DMI ID information.
fn product_name() -> io::Result<String> { read_trimmed("/sys/class/dmi/id/product_name") }

/// Convenience function for reading the product version from the DMI ID information.
fn product_version() -> io::Result<String> { read_trimmed("/sys/class/dmi/id/product_version") }

/// Convenience function for reading the system vendor from the DMI ID information.
fn sys_vendor() -> io::Result<String> { read_trimmed("/sys/class/dmi/id/sys_vendor") }

/// Creates a string identifying system firmware by the board vendor and name.
pub(crate) fn system_board_identity() -> io::Result<String> {
    Ok([
        &*sys_vendor()?,
        " ",
        &*product_name()?,
        " (",
        &*product_version()?,
        ")"
    ].concat())
}

/// Generic function for attaining a DBus client connection to a firmware service.
pub fn get_client<F, T, E>(name: &str, is_active: fn() -> bool, connect: F) -> Option<T>
where
    F: FnOnce() -> Result<T, E>,
    E: std::fmt::Display,
{
    if is_active() {
        connect().map_err(|why| error!("{} client error: {}", name, why)).ok()
    } else {
        None
    }
}

/// Checks if a systemd service is active.
fn systemd_service_is_active(name: &str) -> bool {
    Command::new("systemctl")
        .args(&["-q", "is-active", name])
        .status()
        .map_err(|why| error!("{}", why))
        .ok()
        .map_or(false, |status| status.success())
}

/// Finds the lowest revision from anything that is or may become an `Iterator` of strings.
fn lowest_revision<'a, I: IntoIterator<Item = &'a str>>(list: I) -> &'a str {
    use std::cmp::Ordering;
    let mut list = list.into_iter();
    match list.next() {
        Some(mut lowest_revision) => {
            for rev in list {
                if human_sort::compare(lowest_revision, &rev) == Ordering::Greater {
                    lowest_revision = &rev;
                }
            }
            lowest_revision
        }
        None => "",
    }
}

/// Helper for formatting errors for logs.
fn format_error<E: std::error::Error>(why: E) -> String {
    let mut error_message = format!("{}", why);
    let mut cause = why.source();
    while let Some(error) = cause {
        error_message.push_str(format!(": {}", error).as_str());
        cause = error.source();
    }

    error_message
}

#[cfg(test)]
mod tests {
    #[test]
    fn lowest_revision() {
        let input = vec!["", "F10", "F5"];
        let rev = super::lowest_revision(input.iter().cloned());
        assert_eq!(rev, "");

        let input = vec!["F3", "F10", "F5"];
        let rev = super::lowest_revision(input.iter().cloned());
        assert_eq!(rev, "F3");

        let input = vec!["F10", "F3", "F5"];
        let rev = super::lowest_revision(input.iter().cloned());
        assert_eq!(rev, "F3");
    }
}
