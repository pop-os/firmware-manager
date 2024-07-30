use super::FirmwareUpdateDialog;
use crate::widgets::DeviceWidget;
use firmware_manager::{Entity, FirmwareEvent, System76Changelog, System76Digest};
use gtk::prelude::*;
use std::sync::mpsc::Sender;
use chrono::NaiveDateTime;

/// An instance of the firmware update dialog specific to system76-managed system devices.
pub struct System76Dialog<'a> {
    pub changelog: &'a System76Changelog,
    pub digest: &'a System76Digest,
    pub entity: Entity,
    pub has_battery: bool,
    pub latest: &'a str,
    pub sender: &'a Sender<FirmwareEvent>,
    pub widgets: &'a DeviceWidget,
}

impl<'a> System76Dialog<'a> {
    pub fn run(self) {
        let log_entries = self.changelog.versions.iter().map(|version| {
            let date = NaiveDateTime::parse_from_str(&version.date, "%Y-%m-%d")
                .unwrap_or_default()
                .timestamp();

            (version.bios.as_ref(), date as u64, version.description.as_ref())
        });

        let dialog = FirmwareUpdateDialog::new(self.latest, log_entries, self.has_battery);

        if gtk::ResponseType::Accept == dialog.run() {
            // Exchange the button for a progress bar.
            self.widgets.stack.switch_to_waiting();

            let event = FirmwareEvent::S76System(self.entity, self.digest.clone());
            let _ = self.sender.send(event);
        }

        dialog.close();
    }
}
