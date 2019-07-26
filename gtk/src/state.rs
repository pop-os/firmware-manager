use crate::{dialogs::*, views::*, widgets::*, ActivateEvent};
use firmware_manager::*;

use gtk::prelude::*;
use slotmap::{DefaultKey as Entity, SecondaryMap};
use std::{
    cell::{Cell, RefCell},
    iter,
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};

/// Manages all state and state interactions with the UI.
pub(crate) struct State {
    /// Components that have been associated with entities.
    pub(crate) components: Components,
    /// All devices will be created as an entity here
    pub(crate) entities: Entities,
    /// Sends events to the progress signal
    pub(crate) progress_sender: Sender<ActivateEvent>,
    /// A sender to send firmware requests to the background thread
    pub(crate) sender: Sender<FirmwareEvent>,
    #[cfg(feature = "system76")]
    /// Stores information about Thelio I/O boards
    pub(crate) thelio_io_data: Rc<RefCell<ThelioData>>,
    /// Widgets that will be actively managed.
    pub(crate) widgets: Widgets,
}

/// GTK widgets that are interacted with throughout the lifetime of the firmware widget.
pub(crate) struct Widgets {
    /// Controls the display of error messages.
    pub(crate) info_bar: gtk::InfoBar,
    /// Error messages will be set in this label.
    pub(crate) info_bar_label: gtk::Label,
    /// Controls which view to display in the UI
    pub(crate) stack: gtk::Stack,
    /// The devices view shows a list of all supported devices.
    pub(crate) view_devices: DevicesView,
    /// The empty view is displayed when a scan found no devices.
    pub(crate) view_empty: EmptyView,
}

/// Components are optional pieces of data that are assigned to entities
#[derive(Default)]
pub(crate) struct Components {
    /// The GTK widgets associated with a device are stored here.
    pub(crate) device_widgets: SecondaryMap<Entity, DeviceWidget>,
    /// Remembers if a device is upgradeable or not
    pub(crate) upgradeable: SecondaryMap<Entity, Rc<Cell<bool>>>,
    /// Tracks progress of a firmware download.
    pub(crate) firmware_download: SecondaryMap<Entity, (u64, u64)>,
}

impl State {
    pub fn new(
        sender: Sender<FirmwareEvent>,
        progress_sender: Sender<ActivateEvent>,
        stack: gtk::Stack,
        info_bar: gtk::InfoBar,
        info_bar_label: gtk::Label,
        view_devices: DevicesView,
        view_empty: EmptyView,
    ) -> Self {
        Self {
            entities: Entities::default(),
            components: Components::default(),
            progress_sender,
            sender,
            #[cfg(feature = "system76")]
            thelio_io_data: Rc::new(RefCell::new(ThelioData {
                digest:      None,
                upgradeable: false,
            })),
            widgets: Widgets { info_bar, info_bar_label, stack, view_devices, view_empty },
        }
    }

    /// An event that occurs when firmware has successfully updated.
    pub fn device_updated(&self, entity: Entity, latest: Box<str>) {
        let mut device_continue = true;

        #[cfg(feature = "system76")]
        {
            if self.entities.is_thelio_io(entity) {
                for entity in self.entities.thelio_io.keys() {
                    let widget = &self.components.device_widgets[entity];
                    widget.stack.set_visible(false);
                    widget.label.set_text(latest.as_ref());
                    let _ = self
                        .progress_sender
                        .send(ActivateEvent::Deactivate(widget.progress.clone()));
                }

                device_continue = false;
            }
        }

        if device_continue {
            if let Some(widget) = self.components.device_widgets.get(entity) {
                widget.stack.set_visible(false);
                widget.label.set_text(latest.as_ref());

                if let Some(upgradeable) = self.components.upgradeable.get(entity) {
                    upgradeable.set(false);
                }

                let _ =
                    self.progress_sender.send(ActivateEvent::Deactivate(widget.progress.clone()));

                if self.entities.is_system(entity) {
                    crate::reboot();
                }
            }
        }
    }

    /// An event that occurs when fwupd firmware is found.
    #[cfg(feature = "fwupd")]
    pub fn fwupd(&mut self, signal: FwupdSignal) {
        let FwupdSignal { info, device, upgradeable, releases } = signal;
        let entity = self.entities.create();

        let widget = if device.needs_reboot() {
            self.entities.associate_system(entity);
            self.widgets.view_devices.system(&info)
        } else {
            self.widgets.view_devices.device(&info)
        };

        let data = Rc::new(FwupdDialogData {
            device: Arc::new(device),
            releases,
            entity,
            shared: DialogData {
                sender: self.sender.clone(),
                stack: widget.stack.downgrade(),
                progress: widget.progress.downgrade(),
                info,
            },
        });

        let upgradeable = Rc::new(Cell::new(upgradeable));

        if upgradeable.get() {
            let data = Rc::clone(&data);
            let upgradeable = Rc::clone(&upgradeable);
            widget.connect_upgrade_clicked(move || fwupd_dialog(&data, upgradeable.get(), true));
        } else {
            widget.stack.set_visible(false);
        }

        {
            let upgradeable = Rc::clone(&upgradeable);
            widget.connect_clicked(move || fwupd_dialog(&data, upgradeable.get(), false));
        }

        self.components.device_widgets.insert(entity, widget);
        self.components.upgradeable.insert(entity, upgradeable);
        self.widgets.stack.show();
        self.widgets.stack.set_visible_child(self.widgets.view_devices.as_ref());
    }

    /// An event that occurs when System76 system firmware has been found.
    #[cfg(feature = "system76")]
    pub fn system76_system(
        &mut self,
        info: FirmwareInfo,
        digest: System76Digest,
        changelog: System76Changelog,
    ) {
        let widget = self.widgets.view_devices.system(&info);
        let entity = self.entities.create();
        self.entities.associate_system(entity);
        let upgradeable = info.current != info.latest;

        let data = Rc::new(System76DialogData {
            entity,
            digest,
            changelog,
            shared: DialogData {
                sender: self.sender.clone(),
                stack: widget.stack.downgrade(),
                progress: widget.progress.downgrade(),
                info,
            },
        });

        let upgradeable = Rc::new(Cell::new(upgradeable));

        if upgradeable.get() {
            let data = Rc::clone(&data);
            let upgradeable = Rc::clone(&upgradeable);
            widget.connect_upgrade_clicked(move || {
                s76_system_dialog(&data, upgradeable.get());
            });
        } else {
            widget.stack.set_visible(false);
        }

        {
            let upgradeable = Rc::clone(&upgradeable);
            widget.connect_clicked(move || {
                s76_system_dialog(&data, upgradeable.get());
            });
        }

        self.components.device_widgets.insert(entity, widget);
        self.components.upgradeable.insert(entity, upgradeable);
        self.widgets.stack.show();
        self.widgets.stack.set_visible_child(self.widgets.view_devices.as_ref());
    }

    /// An event that occurs when a Thelio I/O board was discovered.
    #[cfg(feature = "system76")]
    pub fn thelio_io(&mut self, info: FirmwareInfo, digest: Option<System76Digest>) {
        let widget = self.widgets.view_devices.device(&info);
        let entity = self.entities.create();
        let info = Rc::new(info);

        if info.current != info.latest {
            self.thelio_io_data.borrow_mut().upgradeable = true;
        }

        if let Some(digest) = digest {
            self.thelio_io_data.borrow_mut().digest = Some(digest.clone());

            let sender = self.sender.clone();
            let tx_progress = self.progress_sender.clone();
            let stack = widget.stack.downgrade();
            let progress = widget.progress.downgrade();
            let info = Rc::clone(&info);

            widget.connect_upgrade_clicked(move || {
                // Exchange the button for a progress bar.
                if let (Some(stack), Some(progress)) = (stack.upgrade(), progress.upgrade()) {
                    stack.set_visible_child(&progress);
                    let _ = tx_progress.send(ActivateEvent::Activate(progress));
                }

                let _ = sender.send(FirmwareEvent::ThelioIo(
                    entity,
                    digest.clone(),
                    info.latest.clone(),
                ));
            });
        }

        {
            // When the device's widget is clicked.
            let sender = self.sender.clone();
            let tx_progress = self.progress_sender.clone();
            let stack = widget.stack.downgrade();
            let progress = widget.progress.downgrade();
            let data = Rc::clone(&self.thelio_io_data);
            let info = Rc::clone(&info);
            widget.connect_clicked(move || {
                let dialog = FirmwareUpdateDialog::new(
                    info.latest.as_ref(),
                    iter::once((info.latest.as_ref(), "")),
                    data.borrow().upgradeable,
                    false,
                );

                if gtk::ResponseType::Accept == dialog.run() {
                    if let Some(ref digest) = data.borrow().digest {
                        if let (Some(stack), Some(progress)) = (stack.upgrade(), progress.upgrade())
                        {
                            stack.set_visible_child(&progress);
                            let _ = tx_progress.send(ActivateEvent::Activate(progress));
                        }

                        let _ = sender.send(FirmwareEvent::ThelioIo(
                            entity,
                            digest.clone(),
                            info.latest.clone(),
                        ));
                    }
                }

                dialog.destroy();
            });
        }

        widget.stack.set_visible(false);
        self.components.device_widgets.insert(entity, widget);
        self.entities.associate_thelio_io(entity);

        // If any Thelio I/O device requires an update, then enable the
        // update button on the first Thelio I/O device widget.
        if self.thelio_io_data.borrow_mut().upgradeable {
            let entity = self.entities.thelio_io.keys().next().expect("missing thelio I/O widgets");
            self.components.device_widgets[entity].stack.set_visible(true);
        }

        self.widgets.stack.show();
        self.widgets.stack.set_visible_child(self.widgets.view_devices.as_ref());
    }
}

#[cfg(feature = "system76")]
pub(crate) struct ThelioData {
    digest:      Option<System76Digest>,
    upgradeable: bool,
}
