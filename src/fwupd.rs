//! Functions specific to working with fwupd firmware.

use crate::{FirmwareInfo, FirmwareSignal};
use fwupd_dbus::{Client as FwupdClient, Device as FwupdDevice, Release as FwupdRelease};
use std::cmp::Ordering;

/// A signal sent when a fwupd-compatible device has been discovered.
#[derive(Debug)]
pub struct FwupdSignal {
    /// Generic information about the firmware.
    pub info: FirmwareInfo,
    /// Information specific to fwupd devices.
    pub device: FwupdDevice,
    /// Tracks whether the firmware is upgradeable or not.
    pub upgradeable: bool,
    /// All releases that were found for the firmware.
    pub releases: Vec<FwupdRelease>,
}

/// Scan for supported devices from the fwupd DBus daemon.
pub fn fwupd_scan<F: Fn(FirmwareSignal)>(fwupd: &FwupdClient, sender: F) {
    info!("scanning fwupd devices");

    let devices = match fwupd.devices() {
        Ok(devices) => devices,
        Err(why) => {
            sender(FirmwareSignal::Error(None, why.into()));
            return;
        }
    };

    for device in devices {
        if device.is_supported() {
            match fwupd.releases(&device) {
                Ok(mut releases) => {
                    crate::sort_versions(&mut releases);

                    let latest = releases.iter().last().expect("no releases");
                    let upgradeable = is_newer(&device.version, &latest.version);

                    sender(FirmwareSignal::Fwupd(FwupdSignal {
                        info: FirmwareInfo {
                            name:             [&device.vendor, " ", &device.name].concat().into(),
                            current:          device.version.clone(),
                            latest:           Some(latest.version.clone()),
                            install_duration: latest.install_duration,
                            update_message:   latest.update_message.clone(),
                        },
                        device,
                        upgradeable,
                        releases,
                    }));
                }
                Err(why) => {
                    error!(
                        "failure to get fwupd releases for {}: {}",
                        device.name,
                        super::format_error(why)
                    );
                }
            }
        }
    }

    info!("fwupd scanning complete");
}

/// Update the fwupd remotes
pub fn fwupd_updates(
    client: &FwupdClient,
    http: &reqwest::blocking::Client,
) -> Result<(), fwupd_dbus::Error> {
    const SECONDS_IN_DAY: u64 = 60 * 60 * 24;

    if crate::timestamp::exceeded(SECONDS_IN_DAY).ok().unwrap_or(true) {
        info!("refreshing remotes");

        if let Err(why) = crate::timestamp::refresh() {
            error!("failed to update timestamp: {}", why);
        }

        // NOTE: This attribute is required due to a clippy bug.
        #[allow(clippy::identity_conversion)]
        for remote in client.remotes()? {
            if !remote.enabled {
                continue;
            }

            if let fwupd_dbus::RemoteKind::Download = remote.kind {
                info!("Updating {:?} metadata from {:?}", remote.remote_id, remote.uri);
                if let Err(why) = remote.update_metadata(client, http) {
                    error!(
                        "failed to fetch updates from {}: {:?}",
                        remote.filename_cache,
                        super::format_error(why)
                    );
                }
            }
        }
    }

    Ok(())
}

// Returns `true` if the `latest` string is a newer version than the `current` string.
fn is_newer(current: &str, latest: &str) -> bool {
    human_sort::compare(current, latest) == Ordering::Less
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn is_newer() {
        assert!(super::is_newer("0.2.8", "0.2.11"));
        assert!(!super::is_newer("0.2.11", "0.2.8"));
        assert!(super::is_newer("0.2.7", "0.2.8"));
        assert!(!super::is_newer("0.2.8", "0.2.7"));
    }
}
