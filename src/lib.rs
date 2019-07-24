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

#[derive(Debug)]
pub enum FirmwareEvent {
    #[cfg(feature = "fwupd")]
    Fwupd(Entity, Arc<FwupdDevice>, Arc<FwupdRelease>),
    Quit,
    #[cfg(feature = "system76")]
    S76System(Entity, System76Digest, Box<str>),
    Scan,
    #[cfg(feature = "system76")]
    ThelioIo(Entity, System76Digest, Box<str>),
}

#[derive(Debug)]
pub struct FirmwareInfo {
    pub name:    Box<str>,
    pub current: Box<str>,
    pub latest:  Box<str>,
}

#[derive(Debug, Default, Shrinkwrap)]
pub struct Entities {
    #[shrinkwrap(main_field)]
    pub entities: SlotMap<Entity, ()>,
    pub system: SecondaryMap<Entity, ()>,

    #[cfg(feature = "system76")]
    pub thelio_io: SecondaryMap<Entity, ()>,
}

impl Entities {
    pub fn associate_system(&mut self, entity: Entity) { self.system.insert(entity, ()); }

    pub fn insert(&mut self) -> Entity { self.entities.insert(()) }

    pub fn is_system(&self, entity: Entity) -> bool { self.system.contains_key(entity) }
}

#[derive(Debug)]
pub enum FirmwareSignal {
    /// A device was updated
    DeviceUpdated(Entity, Box<str>),

    /// An error occurred
    Error(Option<Entity>, Error),

    /// Fwupd firmware was discovered.
    #[cfg(feature = "fwupd")]
    Fwupd(FwupdDevice, bool, BTreeSet<FwupdRelease>),

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
    ThelioIo(FirmwareInfo, Option<System76Digest>),

    /// Stops listening
    Stop,
}

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
                    fwupd.update_device_with_release(http_client, &device, &release, flags)
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
                eprintln!("updating thelio io");
                let event = match s76.as_ref().map(|client| client.thelio_io_update(&digest)) {
                    Some(Ok(_)) => FirmwareSignal::DeviceUpdated(entity, latest),
                    Some(Err(why)) => FirmwareSignal::Error(Some(entity), why.into()),
                    None => panic!("thelio event assigned to non-thelio button"),
                };

                sender(event);
            }
            FirmwareEvent::Quit => {
                eprintln!("received quit signal");
                break;
            }
        }
    }
}

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
                sender(FirmwareSignal::Fwupd(device, upgradeable, releases.into()));
            }
        }
    }
}

#[cfg(feature = "fwupd")]
pub fn fwupd_updates(
    client: &FwupdClient,
    http: &reqwest::Client,
) -> Result<(), fwupd_dbus::Error> {
    use std::time::Duration;

    const SECONDS_IN_DAY: u64 = 60 * 60 * 24;

    for remote in client.remotes()? {
        if let fwupd_dbus::RemoteKind::Download = remote.kind {
            let update = remote
                .time_since_last_update()
                .map_or(true, |since| since > Duration::from_secs(14 * SECONDS_IN_DAY));

            if update {
                eprintln!("updating remote cache for {}", remote.filename_cache);
                if let Err(why) = remote.update_metadata(client, http) {
                    eprintln!("failed to fetch updates from {}: {}", remote.filename_cache, why);
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "system76")]
pub fn s76_scan<F: Fn(FirmwareSignal)>(client: &System76Client, sender: F) {
    // Thelio system firmware check.
    let event = match client.bios() {
        Ok(current) => match client.download() {
            Ok(S76SystemInfo { digest, changelog }) => {
                let fw = FirmwareInfo {
                    name:    current.model,
                    current: current.version,
                    latest:  changelog
                        .versions
                        .iter()
                        .last()
                        .expect("empty changelog")
                        .bios
                        .clone(),
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
        Ok(list) => match client.thelio_io_download() {
            Ok(info) => {
                let ThelioIoInfo { digest, .. } = info;
                let digest = &mut Some(digest);
                for (num, (_, revision)) in list.iter().enumerate() {
                    let fw = FirmwareInfo {
                        name:    format!("Thelio I/O #{}", num + 1).into(),
                        current: Box::from(if revision.is_empty() {
                            "N/A"
                        } else {
                            revision.as_str()
                        }),
                        latest:  Box::from(revision.as_str()),
                    };

                    let event = FirmwareSignal::ThelioIo(fw, digest.take());
                    sender(event);
                }

                None
            }
            Err(why) => Some(FirmwareSignal::Error(None, why.into())),
        },
        Err(why) => Some(FirmwareSignal::Error(None, why.into())),
    };

    if let Some(event) = event {
        sender(event);
    }
}

#[cfg(feature = "fwupd")]
pub fn fwupd_is_active() -> bool { systemd_service_is_active("fwupd") }

#[cfg(feature = "system76")]
pub fn s76_firmware_is_active() -> bool { systemd_service_is_active("system76-firmware-daemon") }

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
