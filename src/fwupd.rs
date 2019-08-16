//! Functions specific to working with fwupd firmware.

use crate::{FirmwareInfo, FirmwareSignal};
use fwupd_dbus::{Client as FwupdClient, Device as FwupdDevice, Release as FwupdRelease};
use std::{error::Error as _, io, process::Command};

/// A signal sent when a fwupd-compatible device has been discovered.
#[derive(Debug)]
pub struct FwupdSignal {
    pub info:        FirmwareInfo,
    pub device:      FwupdDevice,
    pub upgradeable: bool,
    pub releases:    Vec<FwupdRelease>,
}

/// Scan for supported devices from the fwupd DBus daemon.
pub fn fwupd_scan<F: Fn(FirmwareSignal)>(fwupd: &FwupdClient, sender: F) {
    eprintln!("scanning fwupd devices");

    let devices = match fwupd.devices() {
        Ok(devices) => devices,
        Err(why) => {
            sender(FirmwareSignal::Error(None, why.into()));
            return;
        }
    };

    for device in devices {
        if device.is_supported() {
            if let Ok(mut releases) = fwupd.releases(&device) {
                crate::sort_versions(&mut releases);

                let latest = releases.iter().last().expect("no releases");
                let upgradeable = latest.version != device.version;

                sender(FirmwareSignal::Fwupd(FwupdSignal {
                    info: FirmwareInfo {
                        name:             [&device.vendor, " ", &device.name].concat().into(),
                        current:          device.version.clone(),
                        latest:           Some(latest.version.clone()),
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

#[cfg(feature = "fwupd")]
pub fn fwupdmgr_refresh() -> io::Result<()> {
    Command::new("fwupdmgr").arg("refresh").status().map(|_| ())
}

/// Update the fwupd remotes
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
                    let mut error_message = format!("{}", why);
                    let mut cause = why.source();
                    while let Some(error) = cause {
                        error_message.push_str(format!(": {}", error).as_str());
                        cause = error.source();
                    }
                    eprintln!(
                        "failed to fetch updates from {}: {:?}",
                        remote.filename_cache, error_message
                    );
                }
            }
        }
    }

    Ok(())
}
