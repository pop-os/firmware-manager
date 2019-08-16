//! Functions specific to working with system76 firmware.

use crate::{lowest_revision, FirmwareInfo, FirmwareSignal};
use std::error::Error as _;
use system76_firmware_daemon::{
    Client as System76Client, SystemInfo as S76SystemInfo, ThelioIoInfo,
};

/// Scan for available System76 firmware
pub fn s76_scan<F: Fn(FirmwareSignal)>(client: &System76Client, sender: F) {
    // Thelio system firmware check.
    if let Ok(current) = client.bios() {
        let info = match client.download() {
            Ok(S76SystemInfo { digest, changelog }) => Some((digest, changelog)),
            Err(why) => {
                let mut error_message = format!("{}", why);
                let mut cause = why.source();
                while let Some(error) = cause {
                    error_message.push_str(format!(": {}", error).as_str());
                    cause = error.source();
                }
                eprintln!("failed to download system76 changelog: {}", error_message);
                None
            }
        };

        let name: Box<str> = crate::system_board_identity().map(Box::from).unwrap_or(current.model);

        let fw = FirmwareInfo {
            name,
            current: current.version,
            latest: info.as_ref().map(|(_, changelog)| {
                changelog.versions.iter().next().expect("empty changelog").bios.clone()
            }),
            install_duration: 1,
        };

        sender(FirmwareSignal::S76System(fw, info));
    }

    // Thelio I/O system firmware check.
    let event = match client.thelio_io_list() {
        Ok(list) => {
            if list.is_empty() {
                None
            } else {
                let lowest_revision = lowest_revision(list.iter().map(|(_, rev)| rev.as_ref()));

                let current =
                    Box::from(if lowest_revision.is_empty() { "N/A" } else { lowest_revision });

                let (latest, digest) = match client.thelio_io_download() {
                    Ok(info) => {
                        let ThelioIoInfo { digest, revision } = info;
                        (Some(revision), Some(digest))
                    }
                    Err(why) => {
                        eprintln!("failed to download Thelio I/O digest: {:?}", why);
                        (None, None)
                    }
                };

                let fw = FirmwareInfo {
                    name: "Thelio I/O".into(),
                    current,
                    latest,
                    install_duration: 15,
                };

                Some(FirmwareSignal::ThelioIo(fw, digest))
            }
        }
        Err(why) => Some(FirmwareSignal::Error(None, why.into())),
    };

    if let Some(event) = event {
        sender(event);
    }
}

/// Check if the system76-firmware-daemon service is active.
pub fn s76_firmware_is_active() -> bool {
    crate::systemd_service_is_active("system76-firmware-daemon")
}
