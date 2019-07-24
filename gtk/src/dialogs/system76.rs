use crate::{ActivateEvent, Entity, FirmwareEvent, FirmwareInfo, System76Changelog, System76Digest};
use super::{DialogData, FirmwareUpdateDialog};
use std::{collections::BTreeSet, sync::Arc};
use gtk::{self, prelude::*};

#[cfg(feature = "system76")]
pub(crate) struct System76DialogData {
    pub entity:    Entity,
    pub digest:    System76Digest,
    pub changelog: System76Changelog,
    pub shared:    DialogData,
}

#[cfg(feature = "system76")]
pub(crate) fn s76_system_dialog(data: &System76DialogData, upgradeable: bool) {
    let &System76DialogData { entity, digest, changelog, shared } = &data;
    let &DialogData { sender, tx_progress, stack, progress, info } = &shared;
    let &FirmwareInfo { latest, .. } = &info;

    let log_entries = changelog.versions.iter().map(|version| {
        (version.bios.as_ref(), version.description.as_ref().map_or("N/A", |desc| desc.as_ref()))
    });

    let dialog = FirmwareUpdateDialog::new(latest, log_entries, upgradeable, true);

    if gtk::ResponseType::Accept == dialog.run() {
        // Exchange the button for a progress bar.
        if let (Some(stack), Some(progress)) = (stack.upgrade(), progress.upgrade()) {
            stack.set_visible_child(&progress);
            let _ = tx_progress.send(ActivateEvent::Activate(progress));
        }

        let event = FirmwareEvent::S76System(*entity, digest.clone(), latest.clone());
        let _ = sender.send(event);
    }

    dialog.destroy();
}
