#[macro_use]
extern crate cascade;
#[macro_use]
extern crate err_derive;
#[macro_use]
extern crate shrinkwraprs;

mod dialogs;
mod traits;
mod views;

use self::{dialogs::*, views::*};
use firmware_manager::*;

use gtk::{self, prelude::*};
use slotmap::{DefaultKey as Entity, SecondaryMap};
use std::{
    collections::HashSet,
    error::Error as ErrorTrait,
    process::Command,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
};

pub struct FirmwareWidget {
    container: gtk::Container,
    sender:    Sender<FirmwareEvent>,
}

impl FirmwareWidget {
    pub fn new() -> Self {
        #[cfg(all(not(feature = "fwupd"), not(feature = "system76")))]
        compile_error!("must enable one or more of [fwupd system76]");

        let (sender, rx) = channel();
        let (tx, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        Self::background(rx, tx);

        let view_devices = DevicesView::new();
        let view_empty = EmptyView::new();

        let info_bar_label = cascade! {
            gtk::Label::new(None);
            ..set_line_wrap(true);
        };

        let info_bar = cascade! {
            gtk::InfoBar::new();
            ..set_message_type(gtk::MessageType::Error);
            ..set_show_close_button(true);
            // ..set_revealed(false);
            ..set_valign(gtk::Align::Start);
            ..connect_close(|info_bar| {
                // info_bar.set_revealed(false);
                info_bar.set_visible(false);
            });
            ..connect_response(|info_bar, _| {
                // info_bar.set_revealed(false);
                info_bar.set_visible(false);
            });
        };

        if let Some(area) = info_bar.get_content_area() {
            if let Some(area) = area.downcast::<gtk::Container>().ok() {
                area.add(&info_bar_label);
            }
        }

        let stack = cascade! {
            gtk::Stack::new();
            ..add(view_empty.as_ref());
            ..add(view_devices.as_ref());
            ..set_visible_child(view_empty.as_ref());
        };

        let container = cascade! {
            gtk::Overlay::new();
            ..add_overlay(&info_bar);
            ..add(&stack);
            ..show_all();
        };

        info_bar.hide();

        {
            let sender = sender.clone();
            let stack = stack.clone();

            let mut entities = Entities::default();
            let mut device_widgets: SecondaryMap<Entity, DeviceWidget> = SecondaryMap::new();

            /// Activates, or deactivates, the movement of progress bars.
            /// TODO: As soon as glib::WeakRef supports Eq/Hash derives, use WeakRef instead.
            enum ActivateEvent {
                Activate(gtk::ProgressBar),
                Deactivate(gtk::ProgressBar),
                Clear,
            }

            let (tx_progress, rx_progress) = channel();

            {
                // Keeps the progress bars moving.
                let mut active_widgets: HashSet<gtk::ProgressBar> = HashSet::new();
                let mut remove = Vec::new();
                gtk::timeout_add(100, move || {
                    while let Ok(event) = rx_progress.try_recv() {
                        match event {
                            ActivateEvent::Activate(widget) => {
                                active_widgets.insert(widget);
                            }
                            ActivateEvent::Deactivate(widget) => {
                                active_widgets.remove(&widget);
                            }
                            ActivateEvent::Clear => {
                                active_widgets.clear();
                                return gtk::Continue(true);
                            }
                        }
                    }

                    for widget in remove.drain(..) {
                        active_widgets.remove(&widget);
                    }

                    for widget in &active_widgets {
                        widget.pulse();
                    }

                    gtk::Continue(true)
                });
            }

            receiver.attach(None, move |event| {
                let event = match event {
                    Some(event) => event,
                    None => return glib::Continue(false),
                };

                match event {
                    FirmwareSignal::DeviceUpdated(entity, latest) => {
                        let mut device_continue = true;

                        #[cfg(feature = "system76")]
                        {
                            if entities.thelio_io.contains_key(entity) {
                                for entity in entities.thelio_io.keys() {
                                    let widget = &device_widgets[entity];
                                    widget.stack.set_visible(false);
                                    widget.label.set_text(latest.as_ref());
                                    let _ = tx_progress
                                        .send(ActivateEvent::Deactivate(widget.progress.clone()));
                                }

                                device_continue = false;
                            }
                        }

                        if device_continue {
                            if let Some(widget) = device_widgets.get(entity) {
                                widget.stack.set_visible(false);
                                widget.label.set_text(latest.as_ref());
                                let _ = tx_progress
                                    .send(ActivateEvent::Deactivate(widget.progress.clone()));

                                if entities[entity] {
                                    reboot();
                                }
                            }
                        }
                    }
                    FirmwareSignal::Error(entity, why) => {
                        // Convert the error and its causes into a string.
                        let mut error_message = format!("{}", why);
                        let mut cause = why.source();
                        while let Some(error) = cause {
                            error_message.push_str(format!(": {}", error).as_str());
                            cause = error.source();
                        }

                        eprintln!("firmware widget error: {}", error_message);

                        info_bar.set_visible(true);
                        // info_bar.set_revealed(true);
                        info_bar_label.set_text(error_message.as_str().into());

                        if let Some(entity) = entity {
                            let widget = &device_widgets[entity];
                            widget.stack.set_visible_child(&widget.button);
                        }
                    }
                    #[cfg(feature = "fwupd")]
                    FirmwareSignal::Fwupd(device, releases) => {
                        let info = FirmwareInfo {
                            name:    [&device.vendor, " ", &device.name].concat().into(),
                            current: device.version.clone(),
                            latest:  match releases.as_ref() {
                                Some(releases) => releases[releases.len() - 1].version.clone(),
                                None => device.version.clone(),
                            },
                        };

                        let entity = entities.insert(device.needs_reboot());

                        let widget = if device.needs_reboot() && device.plugin.as_ref() == "Uefi" {
                            entities.system = Some(entity);
                            view_devices.system(&info)
                        } else {
                            view_devices.device(&info)
                        };

                        if let Some(releases) = releases {
                            let sender = sender.clone();
                            let stack = widget.stack.downgrade();
                            let progress = widget.progress.downgrade();
                            let device = Arc::new(device);
                            let release = Arc::new(releases[releases.len() - 1].clone());
                            let tx_progress = tx_progress.clone();
                            widget.button.connect_clicked(move |_| {
                                let response = if device.needs_reboot() {
                                    let &FirmwareInfo { ref latest, .. } = &info;

                                    let log_entries = releases.iter().map(|release| {
                                        (release.version.as_ref(), release.description.as_ref())
                                    });

                                    let dialog = FirmwareUpdateDialog::new(latest, log_entries);
                                    dialog.show_all();

                                    let value = dialog.run();
                                    dialog.destroy();
                                    value
                                } else {
                                    gtk::ResponseType::Accept.into()
                                };

                                eprintln!("received response");
                                if gtk::ResponseType::Accept == response {
                                    // Exchange the button for a progress bar.
                                    if let (Some(stack), Some(progress)) =
                                        (stack.upgrade(), progress.upgrade())
                                    {
                                        stack.set_visible_child(&progress);
                                        let _ = tx_progress.send(ActivateEvent::Activate(progress));
                                    }

                                    let _ = sender.send(FirmwareEvent::Fwupd(
                                        entity,
                                        device.clone(),
                                        release.clone(),
                                    ));
                                }
                            });
                        } else {
                            widget.stack.set_visible(false);
                        }

                        device_widgets.insert(entity, widget);
                        stack.set_visible_child(view_devices.as_ref());
                    }
                    FirmwareSignal::Scanning => {
                        view_devices.clear();
                        entities.entities.clear();
                        entities.system = None;

                        let _ = tx_progress.send(ActivateEvent::Clear);

                        stack.set_visible_child(view_empty.as_ref());
                    }
                    FirmwareSignal::SystemScheduled => {
                        reboot();
                    }
                    #[cfg(feature = "system76")]
                    FirmwareSignal::S76System(info, digest, changelog) => {
                        let widget = view_devices.system(&info);
                        let entity = entities.insert(true);
                        entities.system = Some(entity);

                        if info.current == info.latest {
                            widget.stack.set_visible(false);
                        } else {
                            let sender = sender.clone();
                            let stack = widget.stack.downgrade();
                            let progress = widget.progress.downgrade();
                            let tx_progress = tx_progress.clone();
                            widget.button.connect_clicked(move |_| {
                                let &FirmwareInfo { ref current, ref latest, .. } = &info;
                                let log_entries = changelog
                                    .versions
                                    .iter()
                                    .skip_while(|version| version.bios.as_ref() != current.as_ref())
                                    .map(|version| {
                                        (version.bios.as_ref(), version.description.as_ref())
                                    });

                                let dialog = FirmwareUpdateDialog::new(latest, log_entries);
                                dialog.show_all();

                                if gtk::ResponseType::Accept == dialog.run() {
                                    // Exchange the button for a progress bar.
                                    if let (Some(stack), Some(progress)) =
                                        (stack.upgrade(), progress.upgrade())
                                    {
                                        stack.set_visible_child(&progress);
                                        let _ = tx_progress.send(ActivateEvent::Activate(progress));
                                    }

                                    let event = FirmwareEvent::S76System(
                                        entity,
                                        digest.clone(),
                                        latest.clone(),
                                    );
                                    let _ = sender.send(event);
                                }

                                dialog.destroy();
                            });
                        }

                        device_widgets.insert(entity, widget);
                        stack.set_visible_child(view_devices.as_ref());
                    }
                    #[cfg(feature = "system76")]
                    FirmwareSignal::ThelioIo(info, digest) => {
                        let widget = view_devices.device(&info);
                        let requires_update = info.current != info.latest;
                        let entity = entities.insert(false);

                        // Only the first Thelio I/O device will have a connected button.
                        if let Some(digest) = digest {
                            let sender = sender.clone();
                            let latest = info.latest;
                            let stack = widget.stack.downgrade();
                            let progress = widget.progress.downgrade();
                            let tx_progress = tx_progress.clone();
                            widget.button.connect_clicked(move |_| {
                                // Exchange the button for a progress bar.
                                if let (Some(stack), Some(progress)) =
                                    (stack.upgrade(), progress.upgrade())
                                {
                                    stack.set_visible_child(&progress);
                                    let _ = tx_progress.send(ActivateEvent::Activate(progress));
                                }

                                let _ = sender.send(FirmwareEvent::ThelioIo(
                                    entity,
                                    digest.clone(),
                                    latest.clone(),
                                ));
                            });
                        }

                        widget.stack.set_visible(false);
                        device_widgets.insert(entity, widget);
                        entities.thelio_io.insert(entity, ());

                        // If any Thelio I/O device requires an update, then enable the
                        // update button on the first Thelio I/O device widget.
                        if requires_update {
                            let entity = entities
                                .thelio_io
                                .keys()
                                .next()
                                .expect("missing thelio I/O widgets");
                            device_widgets[entity].stack.set_visible(true);
                        }

                        stack.set_visible_child(view_devices.as_ref());
                    }
                }

                glib::Continue(true)
            });
        }

        Self { container: container.upcast::<gtk::Container>(), sender }
    }

    pub fn scan(&self) { let _ = self.sender.send(FirmwareEvent::Scan); }

    pub fn container(&self) -> &gtk::Container { self.container.upcast_ref::<gtk::Container>() }

    /// Manages all firmware client interactions from a background thread.
    fn background(receiver: Receiver<FirmwareEvent>, sender: glib::Sender<Option<FirmwareSignal>>) {
        thread::spawn(move || {
            firmware_manager::event_loop(receiver, |event| {
                let _ = sender.send(event);
            });

            eprintln!("stopping firmware client connection");
        });
    }
}

impl Drop for FirmwareWidget {
    fn drop(&mut self) { let _ = self.sender.send(FirmwareEvent::Quit); }
}

fn reboot() {
    eprintln!("rebooting");
    if let Err(why) = Command::new("systemctl").arg("reboot").status() {
        eprintln!("failed to reboot: {}", why);
    }
}
