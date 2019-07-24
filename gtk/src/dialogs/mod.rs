use crate::{ActivateEvent, FirmwareEvent, FirmwareInfo};
use std::sync::mpsc::Sender;

#[cfg(feature = "fwupd")]
mod fwupd;

#[cfg(feature = "system76")]
mod system76;

mod update;

#[cfg(feature = "fwupd")]
pub(crate) use self::fwupd::*;

#[cfg(feature = "system76")]
pub(crate) use self::system76::*;

pub use self::update::*;

/// Senders and widgets shared by all device dialogs.
pub(crate) struct DialogData {
    pub sender:      Sender<FirmwareEvent>,
    pub tx_progress: Sender<ActivateEvent>,
    pub stack:       glib::WeakRef<gtk::Stack>,
    pub progress:    glib::WeakRef<gtk::ProgressBar>,
    pub info:        FirmwareInfo,
}
