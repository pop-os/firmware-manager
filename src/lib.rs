#[macro_use]
extern crate err_derive;

#[macro_use]
extern crate shrinkwraprs;

#[cfg(feature = "fwupd")]
pub use fwupd_dbus::{Client as FwupdClient, Device as FwupdDevice, Release as FwupdRelease};

#[cfg(feature = "system76")]
pub use system76_firmware_daemon::{
    Changelog as System76Changelog, Digest as System76Digest, Error as System76Error,
    SystemInfo as S76SystemInfo, ThelioIoInfo,
};

#[cfg(feature = "system76")]
pub use system76_firmware_daemon::Client as System76Client;

use slotmap::{DefaultKey as Entity, SecondaryMap, SlotMap};
use std::{
    collections::BTreeSet,
    process::Command,
    sync::{mpsc::Receiver, Arc},
};

#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "fwupd")]
    #[error(display = "error in fwupd client")]
    Fwupd(#[error(cause)] fwupd_dbus::Error),
    #[cfg(feature = "system76")]
    #[error(display = "error in system76-firmware client")]
    System76(#[error(cause)] System76Error),
}

#[cfg(feature = "fwupd")]
impl From<fwupd_dbus::Error> for Error {
    fn from(error: fwupd_dbus::Error) -> Self { Error::Fwupd(error) }
}

#[cfg(feature = "system76")]
impl From<System76Error> for Error {
    fn from(error: System76Error) -> Self { Error::System76(error) }
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
    S76System(Entity, System76Digest, Box<str>),

    /// Search for available firmware devices.
    Scan,

    /// Upgrade the firmware of Thelio I/O boarods.
    #[cfg(feature = "system76")]
    ThelioIo(Entity, System76Digest, Box<str>),
}

/// Information about a device and its current and latest firmware.
#[derive(Debug)]
pub struct FirmwareInfo {
    /// The name of this device.
    pub name: Box<str>,

    /// The currently-installed version.
    pub current: Box<str>,

    /// The latest version of firmware for this device.
    pub latest: Box<str>,

    // The time required for this firmware to be flashed, in seconds.
    pub install_duration: u32,
}

#[derive(Debug, Default, Shrinkwrap)]
pub struct Entities {
    /// The primary storage to record all device entities.
    #[shrinkwrap(main_field)]
    pub entities: SlotMap<Entity, ()>,

    /// Secondary storage to keep record of all system devices.
    pub system: SecondaryMap<Entity, ()>,
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

/// A signal sent when a fwupd-compatible device has been discovered.
#[cfg(feature = "fwupd")]
#[derive(Debug)]
pub struct FwupdSignal {
    pub info:        FirmwareInfo,
    pub device:      FwupdDevice,
    pub upgradeable: bool,
    pub releases:    BTreeSet<FwupdRelease>,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum FirmwareSignal {
    /// A device has initiated the flashing process.
    DeviceFlashing(Entity),

    /// A device was updated
    DeviceUpdated(Entity, Box<str>),

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
    S76System(FirmwareInfo, System76Digest, System76Changelog),

    /// Thelio I/O firmware was discovered.
    #[cfg(feature = "system76")]
    ThelioIo(FirmwareInfo, System76Digest),
}

/// An event loop that should be run in the background, as this function will block until
/// the stop signal is received.
pub fn event_loop<F: Fn(FirmwareSignal)>(receiver: Receiver<FirmwareEvent>, sender: F) {
    #[cfg(feature = "system76")]
    let s76 = get_client("system76", s76_firmware_is_active, System76Client::new);
    #[cfg(feature = "fwupd")]
    let fwupd = get_client("fwupd", fwupd_is_active, FwupdClient::new);
    #[cfg(feature = "fwupd")]
    let http_client = &reqwest::Client::new();

    while let Ok(event) = receiver.recv() {
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
                    Some(Ok(_)) => FirmwareSignal::DeviceUpdated(entity, release.version.clone()),
                    Some(Err(why)) => FirmwareSignal::Error(Some(entity), why.into()),
                    None => panic!("fwupd event assigned to non-fwupd button"),
                };

                sender(event);
            }
            #[cfg(feature = "system76")]
            FirmwareEvent::S76System(entity, digest, _latest) => {
                match s76.as_ref().map(|client| client.schedule(&digest)) {
                    Some(Ok(_)) => sender(FirmwareSignal::SystemScheduled),
                    Some(Err(why)) => sender(FirmwareSignal::Error(Some(entity), why.into())),
                    None => panic!("thelio event assigned to non-thelio button"),
                }
            }
            #[cfg(feature = "system76")]
            FirmwareEvent::ThelioIo(entity, digest, latest) => {
                eprintln!("updating thelio I/O to {}", latest);
                sender(FirmwareSignal::DeviceFlashing(entity));
                let event = match s76.as_ref().map(|client| client.thelio_io_update(&digest)) {
                    Some(Ok(_)) => FirmwareSignal::DeviceUpdated(entity, latest),
                    Some(Err(why)) => FirmwareSignal::Error(Some(entity), why.into()),
                    None => panic!("thelio event assigned to non-thelio button"),
                };

                sender(event);
            }
            FirmwareEvent::Stop => {
                eprintln!("received quit signal");
                break;
            }
        }
    }
}

/// Scan for supported devices from the fwupd DBus daemon.
#[cfg(feature = "fwupd")]
pub fn fwupd_scan<F: Fn(FirmwareSignal)>(fwupd: &FwupdClient, sender: F) {
    eprintln!("scanning fwupd devices");

    let devices = match fwupd.devices() {
        Ok(devices) => devices,
        Err(why) => {
            eprintln!("errored");
            sender(FirmwareSignal::Error(None, why.into()));
            return;
        }
    };

    for device in devices {
        if device.is_supported() {
            if let Ok(releases) = fwupd.releases(&device) {
                let upgradeable =
                    releases.iter().rev().next().map_or(false, |v| v.version != device.version);

                let latest = releases.iter().last().expect("no releases");

                sender(FirmwareSignal::Fwupd(FwupdSignal {
                    info: FirmwareInfo {
                        name:             [&device.vendor, " ", &device.name].concat().into(),
                        current:          device.version.clone(),
                        latest:           latest.version.clone(),
                        install_duration: latest.install_duration,
                    },
                    device,
                    upgradeable,
                    releases,
                }));
            }
        }
    }
}

/// Update the fwupd remotes
#[cfg(feature = "fwupd")]
pub fn fwupd_updates(
    client: &FwupdClient,
    http: &reqwest::Client,
) -> Result<(), fwupd_dbus::Error> {
    use std::time::Duration;

    const SECONDS_IN_DAY: u64 = 60 * 60 * 24;

    // NOTE: This attribute is required due to a clippy bug.
    #[allow(clippy::identity_conversion)]
    for remote in client.remotes()? {
        if !remote.enabled {
            continue;
        }

        if let fwupd_dbus::RemoteKind::Download = remote.kind {
            let update = remote
                .time_since_last_update()
                .map_or(true, |since| since > Duration::from_secs(14 * SECONDS_IN_DAY));

            if update {
                eprintln!("Updating {:?} metadata from {:?}", remote.remote_id, remote.uri);
                if let Err(why) = remote.update_metadata(client, http) {
                    eprintln!("failed to fetch updates from {}: {}", remote.filename_cache, why);
                }
            }
        }
    }

    Ok(())
}

/// Scan for available System76 firmware
#[cfg(feature = "system76")]
pub fn s76_scan<F: Fn(FirmwareSignal)>(client: &System76Client, sender: F) {
    // Thelio system firmware check.
    let event = match client.bios() {
        Ok(current) => match client.download() {
            Ok(S76SystemInfo { digest, changelog }) => {
                let fw = FirmwareInfo {
                    name:             current.model,
                    current:          current.version,
                    latest:           changelog
                        .versions
                        .iter()
                        .next()
                        .expect("empty changelog")
                        .bios
                        .clone(),
                    install_duration: 1,
                };

                FirmwareSignal::S76System(fw, digest, changelog)
            }
            Err(why) => FirmwareSignal::Error(None, why.into()),
        },
        Err(why) => FirmwareSignal::Error(None, why.into()),
    };

    sender(event);

    // Thelio I/O system firmware check.
    let event = match client.thelio_io_list() {
        Ok(list) => {
            if list.is_empty() {
                None
            } else {
                match client.thelio_io_download() {
                    Ok(info) => {
                        let ThelioIoInfo { digest, revision } = info;
                        let lowest_revision =
                            lowest_revision(list.iter().map(|(_, rev)| rev.as_ref()));

                        let fw = FirmwareInfo {
                            name:             "Thelio I/O".into(),
                            current:          Box::from(if lowest_revision.is_empty() {
                                "N/A"
                            } else {
                                lowest_revision
                            }),
                            latest:           revision,
                            install_duration: 15,
                        };

                        Some(FirmwareSignal::ThelioIo(fw, digest))
                    }
                    Err(why) => Some(FirmwareSignal::Error(None, why.into())),
                }
            }
        }
        Err(why) => Some(FirmwareSignal::Error(None, why.into())),
    };

    if let Some(event) = event {
        sender(event);
    }
}

/// Check if the fwupd service is active.
#[cfg(feature = "fwupd")]
pub fn fwupd_is_active() -> bool { systemd_service_is_active("fwupd") }

/// Check if the system76-firmware-daemon service is active.
#[cfg(feature = "system76")]
pub fn s76_firmware_is_active() -> bool { systemd_service_is_active("system76-firmware-daemon") }

/// Generic function for attaining a DBus client connection to a firmware service.
pub fn get_client<F, T, E>(name: &str, is_active: fn() -> bool, connect: F) -> Option<T>
where
    F: FnOnce() -> Result<T, E>,
    E: std::fmt::Display,
{
    if is_active() {
        connect().map_err(|why| eprintln!("{} client error: {}", name, why)).ok()
    } else {
        None
    }
}

fn systemd_service_is_active(name: &str) -> bool {
    Command::new("systemctl")
        .args(&["-q", "is-active", name])
        .status()
        .map_err(|why| eprintln!("{}", why))
        .ok()
        .map_or(false, |status| status.success())
}

fn lowest_revision<'a, I: Iterator<Item = &'a str>>(mut list: I) -> &'a str {
    use std::cmp::Ordering;
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
