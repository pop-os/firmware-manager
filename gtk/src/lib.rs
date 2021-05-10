#![deny(missing_docs)]

//! # Firmware Manager GTK
//!
//! This crate is a frontend for the firmware manager to be used with GTK-based desktop
//! environments. Only GTK-specific application logic is contained here. All application logic is
//! delegated to the core which this wraps.
//!
//! See the [application crate] for an example of how this can be integrated into an application.
//!
//! [application crate]: ./main.rs

#[macro_use]
extern crate cascade;
#[macro_use]
extern crate log;
#[macro_use]
extern crate shrinkwraprs;

mod changelog;
mod dialogs;
mod localize;
mod state;
mod traits;
mod views;
mod widgets;

pub use self::localize::localizer;

use self::{state::State, views::*};
use firmware_manager::*;
use gtk::{self, prelude::*};
use slotmap::DefaultKey as Entity;
use std::{
    collections::HashSet,
    error::Error as _,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc,
    },
    thread::{self, JoinHandle},
};
use yansi::Paint;

/// Activates, or deactivates, the movement of progress bars.
/// TODO: As soon as glib::WeakRef supports Eq/Hash derives, use WeakRef instead.
pub(crate) enum ActivateEvent {
    Activate(gtk::ProgressBar),
    Deactivate(gtk::ProgressBar),
    Clear,
}

/// The complete firmware manager, as a widget structure
pub struct FirmwareWidget {
    container:  gtk::Container,
    sender:     Sender<FirmwareEvent>,
    background: Option<JoinHandle<()>>,
    is_admin:   bool,
}

/// An event which the GTK UI may propagate to the event loop in the main context.
#[derive(Debug)]
enum UiEvent {
    /// It was requested to hide the upgrade stack of an entity
    HideStack(Entity),
    /// An entity is scheduled to be revealed
    Reveal(Entity),
    /// An entity has been revealed
    Revealed(Entity, bool),
    /// The update button of an entity was triggered
    Update(Entity),
}

/// An event that requests for the UI to perform a specific action.
///
/// This are sent to the glib channel receiver attached to the main context.
/// All events are managed in a central location with shared state.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum Event {
    Firmware(FirmwareSignal),
    Ui(UiEvent),
    Stop,
}

impl FirmwareWidget {
    /// Create a new firmware manager widget.
    ///
    /// # Notes
    /// - This will spawn a background thread to handle non-UI events.
    /// - On drop, the background thread will exit
    pub fn new() -> Self {
        let (sender, rx) = channel();

        let view_devices = DevicesView::new();
        let view_empty = EmptyView::new();

        let info_bar_label = cascade! {
            gtk::Label::new(None);
            ..set_line_wrap(true);
            ..set_selectable(true);
            ..show();
        };

        let sender1 = sender.clone();
        let sender2 = sender.clone();
        let info_bar = cascade! {
            gtk::InfoBar::new();
            ..set_message_type(gtk::MessageType::Error);
            ..set_show_close_button(true);
            ..set_valign(gtk::Align::Start);
            ..connect_close(move |info_bar| {
                info_bar.set_visible(false);
                let _ = sender1.send(FirmwareEvent::Scan);
            });
            ..connect_response(move |info_bar, _| {
                info_bar.set_visible(false);
                let _ = sender2.send(FirmwareEvent::Scan);
            });
            ..set_no_show_all(true);
        };

        let area = info_bar.get_content_area();
        area.add(&info_bar_label);

        let stack = cascade! {
            gtk::Stack::new();
            ..add(view_empty.as_ref());
            ..add(view_devices.as_ref());
            ..set_no_show_all(true);
        };

        let is_admin = user_is_admin();

        if is_admin {
            stack.set_visible_child(view_empty.as_ref());
        } else {
            let view = PermissionView::new();
            stack.add(view.as_ref());
            stack.set_visible_child(view.as_ref());
            stack.show();
        }

        let container = {
            let sender = sender.clone();
            let container = cascade! {
                gtk::Overlay::new();
                ..add_overlay(&info_bar);
                ..add(&stack);
                ..set_can_default(true);
                ..connect_key_press_event(move |_, event| {
                    gtk::Inhibit(if event.get_keyval() == gdk::keys::constants::F5 {
                        let _ = sender.send(FirmwareEvent::Scan);
                        true
                    } else {
                        false
                    })
                });
                ..show_all();
            };

            container
        };

        info_bar.hide();

        let (tx_progress, rx_progress) = channel();
        let (tx_events, rx_events) = glib::MainContext::channel::<Event>(glib::PRIORITY_DEFAULT);

        // Spawns a background thread to handle all background events.
        let background = Self::background(rx, tx_events.clone());

        let state = State::new(
            sender.clone(),
            tx_events,
            tx_progress,
            stack.clone(),
            info_bar,
            info_bar_label,
            view_devices,
            view_empty,
        );

        Self::attach_main_event_loop(state, rx_events);
        Self::connect_progress_events(rx_progress);

        Self {
            background: Some(background),
            container: container.upcast::<gtk::Container>(),
            is_admin,
            sender,
        }
    }

    /// Sends a signal to the background thread to scan for available firmware.
    ///
    /// This clears any devices that have been previously discovered, and repopulates the
    /// devices view with new devices, if found. If devices are not found, the empty view
    /// will be displayed instead.
    pub fn scan(&self) {
        if self.is_admin {
            let _ = self.sender.send(FirmwareEvent::Scan);
        }
    }

    /// Returns the primary container widget of this structure.
    pub fn container(&self) -> &gtk::Container { self.container.upcast_ref::<gtk::Container>() }

    /// The main event loop for this widget.
    ///
    /// Manages all `FirmwareSignal` events received on the receiver from the background thread.
    /// The `State` input is captured by the receiver's move closure, and therefore retains its
    /// state between executions of the receiver's event loop.
    fn attach_main_event_loop(mut state: State, receiver: glib::Receiver<Event>) {
        use crate::{Event::*, FirmwareSignal::*, UiEvent::*};
        let mut last_active_revealer = None;

        // TODO: Use a better approach than an Arc<AtomicBool>
        let firmware_flashing = Arc::new(AtomicBool::new(false));
        let firmware_flashing_ = firmware_flashing.clone();
        let tx_udev = state.sender.clone();
        let usb_trigger = usb_hotplug_event_loop(move || {
            if !firmware_flashing_.load(Ordering::SeqCst) {
                let _ = tx_udev.send(FirmwareEvent::Scan);
            }
        });

        receiver.attach(None, move |event| {
            // Capture the USB trigger in the lifetime of the attached receiver.
            let _ = usb_trigger;

            trace!("received UI event: {:#?}", Paint::yellow(&event));
            match event {
                // When a device begins flashing, we can begin moving the progress bar based on
                // its duration.
                Firmware(DeviceFlashing(entity)) => {
                    firmware_flashing.store(true, Ordering::SeqCst);
                    let widget = &state.components.device_widgets[entity];
                    let message = if state.entities.is_system(entity) {
                        fl!("action-scheduling")
                    } else {
                        fl!("action-flashing")
                    };

                    widget.stack.switch_to_progress(&message);
                    state.progress_activate(&widget.stack.progress);
                }
                // An event that occurs when firmware has successfully updated.
                Firmware(DeviceUpdated(entity)) => {
                    firmware_flashing.store(false, Ordering::SeqCst);
                    let latest = state.components.latest.remove(entity);
                    state.device_updated(entity, latest.expect("updated device without version"))
                }
                // Firmware for a device has begun downloading.
                Firmware(DownloadBegin(entity, size)) => {
                    let widget = &state.components.device_widgets[entity];
                    state.components.firmware_download.insert(entity, (0, size));
                    widget.stack.switch_to_progress(&fl!("action-downloading"));
                }
                // Firmware for a device has finished downloading.
                Firmware(DownloadComplete(entity)) => {
                    state.components.firmware_download.remove(entity);
                    let widget = &state.components.device_widgets[entity];
                    widget.stack.progress.set_fraction(1.0);
                }
                // Update the progress for the firmware being downloaded.
                Firmware(DownloadUpdate(entity, downloaded)) => {
                    let widget = &state.components.device_widgets[entity];
                    let progress = &mut state.components.firmware_download[entity];
                    progress.0 += downloaded as u64;
                    widget.stack.progress.set_fraction(progress.0 as f64 / progress.1 as f64);
                }
                // An error occurred in the background thread, which we shall display in the UI.
                Firmware(Error(entity, why)) => {
                    firmware_flashing.store(false, Ordering::SeqCst);
                    // Convert the error and its causes into a string.
                    let mut error_message = format!("{}", why);
                    let mut cause = why.source();
                    while let Some(error) = cause {
                        error_message.push_str(format!(": {}", error).as_str());
                        cause = error.source();
                    }

                    error!("firmware widget error: {}", error_message);

                    state.widgets.info_bar.set_visible(true);
                    state.widgets.info_bar_label.set_text(error_message.as_str());

                    if let Some(entity) = entity {
                        let widget = &state.components.device_widgets[entity];
                        widget.stack.set_visible_child(&widget.stack.button);
                        state.components.firmware_download.remove(entity);
                        state.progress_deactivate(&widget.stack.progress);
                    }
                }
                // An event that occurs when fwupd firmware is found.
                Firmware(Fwupd(signal)) => state.fwupd(signal),
                // Begins searching for devices that have firmware upgrade support
                Firmware(Scanning) => {
                    state.widgets.view_devices.clear();
                    last_active_revealer = None;
                    state.entities.clear();

                    let _ = state.progress_sender.send(ActivateEvent::Clear);

                    state.widgets.stack.hide();
                    state.widgets.view_devices.hide_systems();
                    state.widgets.view_devices.hide_devices();
                }
                // Signal is received when scanning has completed.
                Firmware(ScanningComplete) => {
                    info!("scanning for firmware is complete");
                    if state.entities.entities.is_empty() {
                        state.widgets.stack.show();
                        state.widgets.view_empty.show_all();
                        state.widgets.stack.set_visible_child(state.widgets.view_empty.as_ref());
                    }
                }
                // When system firmwmare is successfully scheduled, reboot the system.
                Firmware(SystemScheduled) => reboot(),
                // An event that occurs when System76 system firmware has been found.
                Firmware(S76System(info, data)) => state.system76_system(info, data),
                // An event that occurs when a Thelio I/O board was discovered.
                Firmware(ThelioIo(info, digest)) => state.thelio_io(info, digest),
                // Schedules the given firmware for an update, and show a dialog if it requires a
                // reboot.
                Ui(Update(entity)) => state.update(entity),
                // Hides the entity's stack.
                Ui(HideStack(entity)) => {
                    if let Some(widget) = state.components.device_widgets.get(entity) {
                        widget.stack.hide();
                    }
                }
                // Reveals a widget's changelog in a revealer, and generate that changelog if it has
                // not been revealed yet.
                Ui(Reveal(entity)) => state.reveal(entity),
                // Signals that an entity's revealer has been revealed, and so we should hide the
                // last-active revealer.
                Ui(Revealed(entity, revealed)) => {
                    if revealed {
                        if let Some(previous) = last_active_revealer {
                            let widgets = &state.components.device_widgets[previous];
                            widgets.revealer.set_reveal_child(false);
                        }

                        last_active_revealer = Some(entity);
                    } else {
                        last_active_revealer = None;
                    }
                }
                // This is the last message sent before the background thread exits.
                Stop => {
                    trace!("glib channel receiver closed");
                    return glib::Continue(false);
                }
            }

            glib::Continue(true)
        });
    }

    /// Manages all firmware client interactions from a background thread.
    fn background(
        receiver: Receiver<FirmwareEvent>,
        sender: glib::Sender<Event>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            firmware_manager::event_loop(receiver, |event| {
                let _ = sender.send(Event::Firmware(event));
            });

            info!("firmware manager event loop stopped");
            let _ = sender.send(Event::Stop);
        })
    }

    /// Actively moves available progress bars on the device view.
    ///
    /// When a device is to be flashed, it will be submitted to this signal, and actively
    /// stepped at regular intervals. Each device will move their progress bar based on the
    /// value of the `pulse_step` defined in the progress bar widget. This value is based on
    /// the amount of time that is required to flash the device.
    ///
    /// On completion, devices will be removed from this signal.
    fn connect_progress_events(rx_progress: Receiver<ActivateEvent>) {
        let mut active_widgets: HashSet<gtk::ProgressBar> = HashSet::new();
        let mut remove = Vec::new();
        glib::timeout_add_local(100, move || {
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
                        return glib::Continue(true);
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        trace!("disconnecting progress event loop");
                        return glib::Continue(false);
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

            glib::Continue(true)
        });
    }
}

impl Default for FirmwareWidget {
    fn default() -> Self { Self::new() }
}

impl Drop for FirmwareWidget {
    fn drop(&mut self) {
        trace!("firmware widget dropped: sending stop signal to background thread");
        let _ = self.sender.send(FirmwareEvent::Stop);

        if let Some(handle) = self.background.take() {
            let _ = handle.join();
        }
    }
}

/// Convenience function for rebooting the system.
///
/// Currently only supports rebooting via `systemctl`. Feature flags could use other init system
/// facilities to do the same.
fn reboot() {
    if let Err(why) = Command::new("systemctl").arg("reboot").status() {
        error!("failed to reboot: {}", why);
    }
}
