// TODO: Further abstract state from the UI, removing the need for Rc, with new component storages.

use crate::{dialogs::*, views::*, widgets::*, ActivateEvent, Event, UiEvent};
use firmware_manager::*;

use gtk::prelude::*;
use slotmap::{DefaultKey as Entity, SecondaryMap};
use std::{
    cell::Cell,
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
    /// If this system has a battery.
    pub(crate) has_battery: bool,
    /// Sends events to the progress signal
    pub(crate) progress_sender: Sender<ActivateEvent>,
    /// A sender to send firmware requests to the background thread
    pub(crate) sender: Sender<FirmwareEvent>,
    /// Events to be processed by the main event loop
    pub(crate) ui_sender: glib::Sender<Event>,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sender: Sender<FirmwareEvent>,
        ui_sender: glib::Sender<Event>,
        progress_sender: Sender<ActivateEvent>,
        stack: gtk::Stack,
        info_bar: gtk::InfoBar,
        info_bar_label: gtk::Label,
        view_devices: DevicesView,
        view_empty: EmptyView,
    ) -> Self {
        let has_battery =
            upower_dbus::UPower::new(-1).and_then(|upower| upower.on_battery()).unwrap_or(false);

        Self {
            entities: Entities::default(),
            components: Components::default(),
            has_battery,
            progress_sender,
            sender,
            widgets: Widgets { info_bar, info_bar_label, stack, view_devices, view_empty },
            ui_sender,
        }
    }

    /// An event that occurs when firmware has successfully updated.
    pub fn device_updated(&self, entity: Entity, latest: Box<str>) {
        if let Some(widget) = self.components.device_widgets.get(entity) {
            widget.stack.progress.set_fraction(1.0);
            widget.label.set_text(latest.as_ref());

            if let Some(upgradeable) = self.components.upgradeable.get(entity) {
                upgradeable.set(false);
            }

            let _ =
                self.progress_sender.send(ActivateEvent::Deactivate(widget.stack.progress.clone()));

            if self.entities.is_system(entity) {
                crate::reboot();
            }

            // Wait 1 second before changing the visibility of the stack.
            let sender = self.ui_sender.clone();
            gtk::timeout_add_seconds(1, move || {
                let _ = sender.send(Event::Ui(UiEvent::HideStack(entity)));

                gtk::Continue(false)
            });
        }
    }

    /// An event that occurs when fwupd firmware is found.
    #[cfg(feature = "fwupd")]
    pub fn fwupd(&mut self, signal: FwupdSignal) {
        let FwupdSignal { info, device, upgradeable, releases } = signal;
        let upgradeable = Rc::new(Cell::new(upgradeable));
        let entity = self.entities.create();
        let has_battery = self.has_battery;

        let widget = if device.needs_reboot() {
            self.entities.associate_system(entity);
            self.widgets.view_devices.system(&info)
        } else {
            self.widgets.view_devices.device(&info)
        };

        widget.stack.hide();

        let data = match info.latest.clone() {
            Some(latest) => {
                let data = Rc::new(FwupdDialogData {
                    device: Arc::new(device),
                    releases,
                    entity,
                    latest,
                    shared: DialogData {
                        sender: self.sender.clone(),
                        stack: Rc::downgrade(&widget.stack),
                        info,
                    },
                });

                if upgradeable.get() {
                    let data = Rc::clone(&data);
                    let upgradeable = Rc::clone(&upgradeable);
                    widget.stack.show();
                    widget.connect_upgrade_clicked(move || {
                        fwupd_dialog(&data, upgradeable.get(), has_battery, true)
                    });
                }

                Some(data)
            }
            None => None,
        };

        let sender = self.ui_sender.clone();
        widget.connect_clicked(move |revealer| {
            let data = &data;
            reveal(&revealer, &sender, entity, move || {
                let widget = data.as_ref().map_or_else(
                    crate::changelog::generate_widget_none,
                    |data| {
                        let releases = &data.releases;
                        let log_entries = releases
                            .iter()
                            .rev()
                            .map(|release| (release.version.as_ref(), release.description.as_ref()));

                        crate::changelog::generate_widget(log_entries, true)
                    }
                );

                widget.upcast::<gtk::Container>()
            });
        });

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
        downloaded: Option<(System76Digest, System76Changelog)>,
    ) {
        let widget = self.widgets.view_devices.system(&info);
        widget.stack.hide();
        let entity = self.entities.create();
        self.entities.associate_system(entity);

        let upgradeable = Rc::new(Cell::new(info.latest.as_ref().map_or(false, |latest| latest.as_ref() != info.current.as_ref())));
        let has_battery = self.has_battery;

        let latest = info.latest.clone();
        let data = latest
            .and_then(|latest| downloaded.map(|d| (d, latest)))
            .map(|((digest, changelog), latest)| {
                let data = Rc::new(System76DialogData {
                    entity,
                    digest,
                    latest,
                    changelog,
                    shared: DialogData {
                        sender: self.sender.clone(),
                        stack: Rc::downgrade(&widget.stack),
                        info,
                    },
                });

                if upgradeable.get() {
                    widget.stack.show();
                    let data = Rc::clone(&data);
                    let upgradeable = Rc::clone(&upgradeable);
                    widget.connect_upgrade_clicked(move || {
                        s76_system_dialog(&data, upgradeable.get(), has_battery);
                    });
                }

                data
            });

        let sender = self.ui_sender.clone();
        widget.connect_clicked(move |revealer| {
            reveal(&revealer, &sender, entity, || {
                let widget = if let Some(data) = data.as_ref() {
                    let changelog = &data.changelog;

                    let log_entries = changelog.versions.iter().map(|version| {
                        (
                            version.bios.as_ref(),
                            version.description.as_ref().map_or("N/A", |desc| desc.as_ref()),
                        )
                    });

                    crate::changelog::generate_widget(log_entries, true)
                } else {
                    crate::changelog::generate_widget_none()
                };

                widget.upcast::<gtk::Container>()
            })
        });

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

        let upgradeable = info.latest.as_ref().map_or(false, |latest| latest.as_ref() != info.current.as_ref());

        let sender = self.sender.clone();
        let tx_progress = self.progress_sender.clone();
        let stack = Rc::downgrade(&widget.stack);
        let latest: Option<Rc<str>> = info.latest.map(Rc::from);

        if let (Some(digest), Some(latest)) = (digest, latest.as_ref()) {
            let latest = Rc::clone(latest);
            widget.connect_upgrade_clicked(move || {
                // Exchange the button for a progress bar.
                if let Some(stack) = stack.upgrade() {
                    stack.switch_to_waiting();
                    let _ = tx_progress.send(ActivateEvent::Activate(stack.progress.clone()));
                }

                let _ =
                    sender.send(FirmwareEvent::ThelioIo(entity, digest.clone(), (&*latest).into()));
            });
        }

        {
            // When the device's widget is clicked.
            let sender = self.ui_sender.clone();
            widget.connect_clicked(move |revealer| {
                reveal(&revealer, &sender, entity, || {
                    let widget = latest.as_ref().map_or_else(
                        || crate::changelog::generate_widget_none(),
                        |latest| crate::changelog::generate_widget(iter::once((latest.as_ref(), "")), true)
                    );
                    
                    widget.upcast::<gtk::Container>()
                });
            });
        }

        if upgradeable {
            widget.stack.show();
        } else {
            widget.stack.hide();
        }
        self.components.device_widgets.insert(entity, widget);

        self.widgets.stack.show();
        self.widgets.stack.set_visible_child(self.widgets.view_devices.as_ref());
    }
}

fn reveal<F: FnMut() -> gtk::Container>(
    revealer: &gtk::Revealer,
    sender: &glib::Sender<Event>,
    entity: Entity,
    mut func: F,
) {
    let reveal = if revealer.get_reveal_child() {
        false
    } else {
        // If the content to be revealed has not been generated yet, do so.
        if revealer.get_child().is_none() {
            let widget = func();

            let container = cascade! {
                gtk::Box::new(gtk::Orientation::Vertical, 12);
                ..set_vexpand(true);
                ..add(&gtk::Separator::new(gtk::Orientation::Horizontal));
                ..add(&gtk::LabelBuilder::new().label("<b>Changelog</b>").use_markup(true).xalign(0.0).build());
                ..add(&widget);
                ..show_all();
            };

            revealer.add(&container);
            revealer.show_all();
        }

        true
    };

    let _ = sender.send(Event::Ui(UiEvent::Revealed(entity, reveal)));
    revealer.set_reveal_child(reveal);
}
