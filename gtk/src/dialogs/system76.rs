use super::{DialogData, FirmwareUpdateDialog};
use crate::{Entity, FirmwareEvent, FirmwareInfo, System76Changelog, System76Digest};
use gtk::{self, prelude::*};

#[cfg(feature = "system76")]
pub(crate) struct System76DialogData {
    pub entity:    Entity,
    pub digest:    System76Digest,
    pub changelog: System76Changelog,
    pub shared:    DialogData,
}

#[cfg(feature = "system76")]
pub(crate) fn s76_system_dialog(data: &System76DialogData, upgradeable: bool, has_battery: bool) {
    let &System76DialogData { entity, digest, changelog, shared } = &data;
    let &DialogData { sender, stack, info } = &shared;
    let &FirmwareInfo { latest, .. } = &info;

    let log_entries = changelog.versions.iter().map(|version| {
        (version.bios.as_ref(), version.description.as_ref().map_or("", |desc| desc.as_ref()))
    });

    let dialog = FirmwareUpdateDialog::new(latest, log_entries, upgradeable, true, has_battery);

    if gtk::ResponseType::Accept == dialog.run() {
        // Exchange the button for a progress bar.
        if let Some(stack) = stack.upgrade() {
            stack.switch_to_waiting();
        }

        let event = FirmwareEvent::S76System(*entity, digest.clone(), latest.clone());
        let _ = sender.send(event);
    }

    dialog.destroy();
}
