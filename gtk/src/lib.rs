#[macro_use]
extern crate cascade;
#[macro_use]
extern crate shrinkwraprs;

mod dialogs;
mod traits;
mod views;
mod widgets;

use self::{dialogs::*, views::*, widgets::*};
use firmware_manager::*;

use gtk::{self, prelude::*};
use slotmap::{DefaultKey as Entity, SecondaryMap};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    error::Error as ErrorTrait,
    iter,
    process::Command,
    rc::Rc,
    sync::{
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc,
    },
    thread::{self, JoinHandle},
};
/// The complete firmware manager, as a widget structure
pub struct FirmwareWidget {
    container:  gtk::Container,
    sender:     Sender<FirmwareEvent>,
    background: Option<JoinHandle<()>>,
}

impl FirmwareWidget {
    /// Create a new firmware manager widget.
    ///
    /// # Notes
    /// - This will spawn a background thread to handle non-UI events.
    /// - On drop, the background thread will exit
    pub fn new() -> Self {
        #[cfg(all(not(feature = "fwupd"), not(feature = "system76")))]
        compile_error!("must enable one or more of [fwupd system76]");

        let (sender, rx) = channel();
        let (tx, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let background = Self::background(rx, tx);

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
            ..set_valign(gtk::Align::Start);
            ..connect_close(|info_bar| {
                info_bar.set_visible(false);
            });
            ..connect_response(|info_bar, _| {
                info_bar.set_visible(false);
            });
        };

        if let Some(area) = info_bar.get_content_area() {
            if let Ok(area) = area.downcast::<gtk::Container>() {
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

        let (tx_progress, rx_progress) = channel();
        progress_handler(rx_progress);

        {
            let sender = sender.clone();
            let stack = stack.clone();

            // All devices in the system will be stored as entities here.
            let mut entities = Entities::default();

            // Extra component storages used by our entities, which are specific to the GTK
            // implementation.
            let mut device_widget_storage: SecondaryMap<Entity, DeviceWidget> = SecondaryMap::new();
            let mut upgradeable_storage: SecondaryMap<Entity, Rc<Cell<bool>>> = SecondaryMap::new();
            let mut firmware_download_storage: SecondaryMap<Entity, (u64, u64)> =
                SecondaryMap::new();

            // Miscellaneous state that will be captured by the main event loop's move closure.
            let mut devices_found = false;
            let thelio_io_upgradeable =
                Rc::new(RefCell::new(ThelioData { digest: None, upgradeable: false }));

            receiver.attach(None, move |event| {
                match event {
                    // When a device begins flashing, we can begin moving the progress bar based on
                    // its duration.
                    FirmwareSignal::DeviceFlashing(entity) => {
                        let widget = &device_widget_storage[entity];
                        let message =
                            if entities.is_system(entity) { "Scheduling" } else { "Flashing" };
                        widget.progress.set_text(message.into());
                        widget.progress.set_fraction(0.0);
                        let _ = tx_progress.send(ActivateEvent::Activate(widget.progress.clone()));
                    }
                    // An event that occurs when firmware has successfully updated.
                    FirmwareSignal::DeviceUpdated(entity, latest) => {
                        let mut device_continue = true;

                        #[cfg(feature = "system76")]
                        {
                            if entities.is_thelio_io(entity) {
                                for entity in entities.thelio_io.keys() {
                                    let widget = &device_widget_storage[entity];
                                    widget.stack.set_visible(false);
                                    widget.label.set_text(latest.as_ref());
                                    let _ = tx_progress
                                        .send(ActivateEvent::Deactivate(widget.progress.clone()));
                                }

                                device_continue = false;
                            }
                        }

                        if device_continue {
                            if let Some(widget) = device_widget_storage.get(entity) {
                                widget.stack.set_visible(false);
                                widget.label.set_text(latest.as_ref());

                                if let Some(upgradeable) = upgradeable_storage.get(entity) {
                                    upgradeable.set(false);
                                }

                                let _ = tx_progress
                                    .send(ActivateEvent::Deactivate(widget.progress.clone()));

                                if entities.is_system(entity) {
                                    reboot();
                                }
                            }
                        }
                    }
                    // Firmware for a device has begun downloading.
                    FirmwareSignal::DownloadBegin(entity, size) => {
                        let widget = &device_widget_storage[entity];
                        firmware_download_storage.insert(entity, (0, size));
                        widget.progress.set_text("Downloading".into());
                        widget.progress.set_fraction(0.0);
                    }
                    // Firmware for a device has finished downloading.
                    FirmwareSignal::DownloadComplete(entity) => {
                        firmware_download_storage.remove(entity);
                        let widget = &device_widget_storage[entity];
                        widget.progress.set_fraction(1.0);
                    }
                    // Update the progress for the firmware being downloaded.
                    FirmwareSignal::DownloadUpdate(entity, downloaded) => {
                        let widget = &device_widget_storage[entity];
                        let progress = &mut firmware_download_storage[entity];
                        progress.0 += downloaded as u64;
                        widget.progress.set_fraction(progress.0 as f64 / progress.1 as f64);
                    }
                    // An error occurred in the background thread, which we shall display in the UI.
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
                        info_bar_label.set_text(error_message.as_str());

                        if let Some(entity) = entity {
                            let widget = &device_widget_storage[entity];
                            widget.stack.set_visible_child(&widget.button);
                            let _ = tx_progress
                                .send(ActivateEvent::Deactivate(widget.progress.clone()));
                        }
                    }
                    // An event that occurs when fwupd firmware is found.
                    #[cfg(feature = "fwupd")]
                    FirmwareSignal::Fwupd(FwupdSignal { info, device, upgradeable, releases }) => {
                        devices_found = true;
                        let entity = entities.create();

                        let widget = if device.needs_reboot() {
                            entities.associate_system(entity);
                            view_devices.system(&info)
                        } else {
                            view_devices.device(&info)
                        };

                        let data = Rc::new(FwupdDialogData {
                            device: Arc::new(device),
                            releases,
                            entity,
                            shared: DialogData {
                                sender: sender.clone(),
                                stack: widget.stack.downgrade(),
                                progress: widget.progress.downgrade(),
                                info,
                            },
                        });

                        let upgradeable = Rc::new(Cell::new(upgradeable));

                        if upgradeable.get() {
                            let data = data.clone();
                            let upgradeable = upgradeable.clone();
                            widget.connect_upgrade_clicked(move || {
                                fwupd_dialog(&data, upgradeable.get(), true)
                            });
                        } else {
                            widget.stack.set_visible(false);
                        }

                        {
                            let upgradeable = upgradeable.clone();
                            widget.connect_clicked(move || {
                                fwupd_dialog(&data, upgradeable.get(), false)
                            });
                        }

                        device_widget_storage.insert(entity, widget);
                        upgradeable_storage.insert(entity, upgradeable);
                        stack.show();
                        stack.set_visible_child(view_devices.as_ref());
                    }
                    // Begins searching for devices that have firmware upgrade support
                    FirmwareSignal::Scanning => {
                        view_devices.clear();
                        entities.clear();
                        devices_found = false;

                        let _ = tx_progress.send(ActivateEvent::Clear);

                        stack.hide();
                    }
                    // Signal is received when scanning has completed.
                    FirmwareSignal::ScanningComplete => {
                        if !devices_found {
                            stack.show();
                            stack.set_visible_child(view_empty.as_ref());
                        }
                    }
                    // When system firmwmare is successfully scheduled, reboot the system.
                    FirmwareSignal::SystemScheduled => {
                        reboot();
                    }
                    // An event that occurs when System76 system firmware has been found.
                    #[cfg(feature = "system76")]
                    FirmwareSignal::S76System(info, digest, changelog) => {
                        devices_found = true;
                        let widget = view_devices.system(&info);
                        let entity = entities.create();
                        entities.associate_system(entity);
                        let upgradeable = info.current != info.latest;

                        let data = Rc::new(System76DialogData {
                            entity,
                            digest,
                            changelog,
                            shared: DialogData {
                                sender: sender.clone(),
                                stack: widget.stack.downgrade(),
                                progress: widget.progress.downgrade(),
                                info,
                            },
                        });

                        let upgradeable = Rc::new(Cell::new(upgradeable));

                        if upgradeable.get() {
                            let data = data.clone();
                            let upgradeable = upgradeable.clone();
                            widget.connect_upgrade_clicked(move || {
                                s76_system_dialog(&data, upgradeable.get());
                            });
                        } else {
                            widget.stack.set_visible(false);
                        }

                        {
                            let upgradeable = upgradeable.clone();
                            widget.connect_clicked(move || {
                                s76_system_dialog(&data, upgradeable.get());
                            });
                        }

                        device_widget_storage.insert(entity, widget);
                        upgradeable_storage.insert(entity, upgradeable);
                        stack.show();
                        stack.set_visible_child(view_devices.as_ref());
                    }
                    // An event that occurs when a Thelio I/O board was discovered.
                    #[cfg(feature = "system76")]
                    FirmwareSignal::ThelioIo(info, digest) => {
                        devices_found = true;
                        let widget = view_devices.device(&info);
                        let entity = entities.create();
                        let info = Rc::new(info);

                        if info.current != info.latest {
                            thelio_io_upgradeable.borrow_mut().upgradeable = true;
                        }

                        if let Some(digest) = digest {
                            thelio_io_upgradeable.borrow_mut().digest = Some(digest.clone());

                            let sender = sender.clone();
                            let tx_progress = tx_progress.clone();
                            let stack = widget.stack.downgrade();
                            let progress = widget.progress.downgrade();
                            let info = info.clone();

                            widget.connect_upgrade_clicked(move || {
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
                                    info.latest.clone(),
                                ));
                            });
                        }

                        {
                            let sender = sender.clone();
                            let tx_progress = tx_progress.clone();
                            let stack = widget.stack.downgrade();
                            let progress = widget.progress.downgrade();
                            let upgradeable = thelio_io_upgradeable.clone();
                            let data = thelio_io_upgradeable.clone();
                            let info = info.clone();
                            widget.connect_clicked(move || {
                                let dialog = FirmwareUpdateDialog::new(
                                    info.latest.as_ref(),
                                    iter::once((info.latest.as_ref(), "")),
                                    upgradeable.borrow().upgradeable,
                                    false,
                                );

                                let sender = sender.clone();
                                let tx_progress = tx_progress.clone();

                                if gtk::ResponseType::Accept == dialog.run() {
                                    if let Some(ref digest) = data.borrow().digest {
                                        if let (Some(stack), Some(progress)) =
                                            (stack.upgrade(), progress.upgrade())
                                        {
                                            stack.set_visible_child(&progress);
                                            let _ =
                                                tx_progress.send(ActivateEvent::Activate(progress));
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
                        device_widget_storage.insert(entity, widget);
                        entities.associate_thelio_io(entity);

                        // If any Thelio I/O device requires an update, then enable the
                        // update button on the first Thelio I/O device widget.
                        if thelio_io_upgradeable.borrow_mut().upgradeable {
                            let entity = entities
                                .thelio_io
                                .keys()
                                .next()
                                .expect("missing thelio I/O widgets");
                            device_widget_storage[entity].stack.set_visible(true);
                        }

                        stack.show();
                        stack.set_visible_child(view_devices.as_ref());
                    }
                    // This is the last message sent before the background thread exits.
                    FirmwareSignal::Stop => {
                        return glib::Continue(false);
                    }
                }

                glib::Continue(true)
            });
        }

        Self {
            background: Some(background),
            container: container.upcast::<gtk::Container>(),
            sender,
        }
    }

    /// Sends a signal to the background thread to scan for available firmware.
    pub fn scan(&self) { let _ = self.sender.send(FirmwareEvent::Scan); }

    /// Returns the primary container widget of this structure.
    pub fn container(&self) -> &gtk::Container { self.container.upcast_ref::<gtk::Container>() }

    /// Manages all firmware client interactions from a background thread.
    fn background(
        receiver: Receiver<FirmwareEvent>,
        sender: glib::Sender<FirmwareSignal>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            firmware_manager::event_loop(receiver, |event| {
                let _ = sender.send(event);
            });

            let _ = sender.send(FirmwareSignal::Stop);

            eprintln!("stopping firmware client connection");
        })
    }
}

impl Drop for FirmwareWidget {
    fn drop(&mut self) {
        let _ = self.sender.send(FirmwareEvent::Stop);

        if let Some(handle) = self.background.take() {
            let _ = handle.join();
        }
    }
}

fn reboot() {
    if let Err(why) = Command::new("systemctl").arg("reboot").status() {
        eprintln!("failed to reboot: {}", why);
    }
}

/// Activates, or deactivates, the movement of progress bars.
/// TODO: As soon as glib::WeakRef supports Eq/Hash derives, use WeakRef instead.
enum ActivateEvent {
    Activate(gtk::ProgressBar),
    Deactivate(gtk::ProgressBar),
    Clear,
}

/// Actively moves available progress bars on the device view.
fn progress_handler(rx_progress: Receiver<ActivateEvent>) {
    let mut active_widgets: HashSet<gtk::ProgressBar> = HashSet::new();
    let mut remove = Vec::new();
    gtk::timeout_add(1000, move || {
        loop {
            match rx_progress.try_recv() {
                Ok(ActivateEvent::Activate(widget)) => {
                    active_widgets.insert(widget);
                }
                Ok(ActivateEvent::Deactivate(widget)) => {
                    active_widgets.remove(&widget);
                }
                Ok(ActivateEvent::Clear) => {
                    active_widgets.clear();
                    return gtk::Continue(true);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return gtk::Continue(false);
                }
            }
        }

        for widget in remove.drain(..) {
            active_widgets.remove(&widget);
        }

        for widget in &active_widgets {
            let new_value = widget.get_fraction() + widget.get_pulse_step();
            widget.set_fraction(if new_value > 1.0 { 1.0 } else { new_value });
        }

        gtk::Continue(true)
    });
}

struct ThelioData {
    digest:      Option<System76Digest>,
    upgradeable: bool,
}
