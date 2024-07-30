use super::FirmwareUpdateDialog;
use crate::widgets::DeviceWidget;
use firmware_manager::{Entity, FirmwareEvent, FwupdDevice, FwupdRelease};
use gtk::prelude::*;
use std::sync::{mpsc::Sender, Arc};

/// An instance of the firmware update dialog specific to fwupd-managed system devices.
pub struct FwupdDialog<'a> {
    pub device: &'a FwupdDevice,
    pub entity: Entity,
    pub has_battery: bool,
    pub latest: &'a str,
    pub needs_reboot: bool,
    pub releases: &'a [FwupdRelease],
    pub sender: &'a Sender<FirmwareEvent>,
    pub widgets: &'a DeviceWidget,
}

impl<'a> FwupdDialog<'a> {
    pub fn run(self) {
        let log_entries = self
            .releases
            .iter()
            .rev()
            .map(|release| (release.version.as_ref(), release.created, release.description.as_ref()));

        let response = if self.needs_reboot {
            let dialog = FirmwareUpdateDialog::new(self.latest, log_entries, self.has_battery);

            let response = dialog.run();
            dialog.close();
            response
        } else {
            gtk::ResponseType::Accept
        };

        if gtk::ResponseType::Accept == response {
            // Exchange the button for a progress bar.
            self.widgets.stack.switch_to_waiting();

            let _ = self.sender.send(FirmwareEvent::Fwupd(
                self.entity,
                Arc::new(self.device.clone()),
                Arc::new(self.releases.iter().last().expect("no release found").clone()),
            ));
        }
    }
}
