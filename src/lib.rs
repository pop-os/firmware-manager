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

use fwupd_dbus::{Client as FwupdClient, Device as FwupdDevice, Release as FwupdRelease};
use gtk::{self, prelude::*};
use slotmap::DefaultKey as Entity;
use slotmap::{SecondaryMap as SM, SlotMap, SparseSecondaryMap as SSM};
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
use system76_firmware_daemon::{
    Changelog, Client as System76Client, Digest, Error as System76Error, ThelioInfo, ThelioIoInfo,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "error in fwupd client")]
    Fwupd(#[error(cause)] fwupd_dbus::Error),
    #[error(display = "error in system76-firmware client")]
    System76(#[error(cause)] System76Error),
}

impl From<fwupd_dbus::Error> for Error {
    fn from(error: fwupd_dbus::Error) -> Self {
        Error::Fwupd(error)
    }
}

impl From<System76Error> for Error {
    fn from(error: System76Error) -> Self {
        Error::System76(error)
    }
}

enum FirmwareEvent {
    Scan,
    Fwupd(Entity, Arc<FwupdDevice>, Arc<FwupdRelease>),
    Thelio(Entity, Digest, Box<str>),
    ThelioIo(Entity, Digest, Box<str>),
    Quit,
}

#[derive(Debug)]
enum WidgetEvent {
    Clear,
    Fwupd(FwupdDevice, Option<Box<[FwupdRelease]>>),
    Thelio(FirmwareInfo, Digest, Changelog),
    ThelioIo(FirmwareInfo, Option<Digest>),
    DeviceUpdated(Entity, Box<str>),
    Error(Option<Entity>, Error),
}

pub struct FirmwareWidget {
    container: gtk::Container,
    sender: Sender<FirmwareEvent>,
}

impl FirmwareWidget {
    pub fn new() -> Self {
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

            let mut entities: SlotMap<Entity, bool> = SlotMap::new();
            let mut system_entity: Option<Entity> = None;
            let mut thelio_io_entities: SM<Entity, ()> = SM::new();
            let mut device_widgets: SSM<Entity, DeviceWidget> = SSM::new();

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
                    WidgetEvent::Clear => {
                        view_devices.clear();
                        entities.clear();
                        system_entity = None;

                        let _ = tx_progress.send(ActivateEvent::Clear);

                        stack.set_visible_child(view_empty.as_ref());
                    }
                    WidgetEvent::DeviceUpdated(entity, latest) => {
                        if thelio_io_entities.contains_key(entity) {
                            for entity in thelio_io_entities.keys() {
                                let widget = &device_widgets[entity];
                                widget.stack.set_visible(false);
                                widget.label.set_text(latest.as_ref());
                                let _ = tx_progress
                                    .send(ActivateEvent::Deactivate(widget.progress.clone()));
                            }
                        } else if let Some(widget) = device_widgets.get(entity) {
                            widget.stack.set_visible(false);
                            widget.label.set_text(latest.as_ref());
                            let _ = tx_progress
                                .send(ActivateEvent::Deactivate(widget.progress.clone()));

                            if entities[entity] {
                                reboot();
                            }
                        }
                    }
                    WidgetEvent::Error(entity, why) => {
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
                    WidgetEvent::Fwupd(device, releases) => {
                        let info = FirmwareInfo {
                            name: [&device.vendor, " ", &device.name].concat().into(),
                            current: device.version.clone(),
                            latest: match releases.as_ref() {
                                Some(releases) => releases[releases.len() - 1].version.clone(),
                                None => device.version.clone(),
                            },
                        };

                        let entity = entities.insert(device.needs_reboot());

                        let widget = if device.needs_reboot() && device.plugin.as_ref() == "Uefi" {
                            system_entity = Some(entity);
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
                                let expected: i32 = gtk::ResponseType::Accept.into();
                                if expected == response {
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
                    WidgetEvent::Thelio(info, digest, changelog) => {
                        let widget = view_devices.system(&info);
                        let entity = entities.insert(true);
                        system_entity = Some(entity);

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

                                let expected: i32 = gtk::ResponseType::Accept.into();
                                if expected == dialog.run() {
                                    // Exchange the button for a progress bar.
                                    if let (Some(stack), Some(progress)) =
                                        (stack.upgrade(), progress.upgrade())
                                    {
                                        stack.set_visible_child(&progress);
                                        let _ = tx_progress.send(ActivateEvent::Activate(progress));
                                    }

                                    let event = FirmwareEvent::Thelio(
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
                    WidgetEvent::ThelioIo(info, digest) => {
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
                        thelio_io_entities.insert(entity, ());

                        // If any Thelio I/O device requires an update, then enable the
                        // update button on the first Thelio I/O device widget.
                        if requires_update {
                            let entity = thelio_io_entities
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

    pub fn scan(&self) {
        let _ = self.sender.send(FirmwareEvent::Scan);
    }

    pub fn container(&self) -> &gtk::Container {
        self.container.upcast_ref::<gtk::Container>()
    }

    /// Manages all firmware client interactions from a background thread.
    fn background(receiver: Receiver<FirmwareEvent>, sender: glib::Sender<Option<WidgetEvent>>) {
        thread::spawn(move || {
            let s76 = Self::get_client("system76", s76_firmware_is_active, System76Client::new);
            let fwupd = Self::get_client("fwupd", fwupd_is_active, FwupdClient::new);
            let http_client = &reqwest::Client::new();

            while let Ok(event) = receiver.recv() {
                match event {
                    FirmwareEvent::Scan => scan(s76.as_ref(), fwupd.as_ref(), &sender),
                    FirmwareEvent::Fwupd(entity, device, release) => {
                        let flags = fwupd_dbus::InstallFlags::empty();
                        let event = match fwupd.as_ref().map(|fwupd| {
                            fwupd.update_device_with_release(http_client, &device, &release, flags)
                        }) {
                            Some(Ok(_)) => {
                                WidgetEvent::DeviceUpdated(entity, release.version.clone())
                            }
                            Some(Err(why)) => WidgetEvent::Error(Some(entity), why.into()),
                            None => panic!("fwupd event assigned to non-fwupd button"),
                        };

                        let _ = sender.send(Some(event));
                    }
                    FirmwareEvent::Thelio(entity, digest, _latest) => {
                        match s76.as_ref().map(|client| client.schedule(&digest)) {
                            Some(Ok(_)) => {
                                reboot();
                            }
                            Some(Err(why)) => {
                                let _ =
                                    sender.send(Some(WidgetEvent::Error(Some(entity), why.into())));
                            }
                            None => panic!("thelio event assigned to non-thelio button"),
                        }
                    }
                    FirmwareEvent::ThelioIo(entity, digest, latest) => {
                        eprintln!("updating thelio io");
                        let event =
                            match s76.as_ref().map(|client| client.thelio_io_update(&digest)) {
                                Some(Ok(_)) => WidgetEvent::DeviceUpdated(entity, latest),
                                Some(Err(why)) => WidgetEvent::Error(Some(entity), why.into()),
                                None => panic!("thelio event assigned to non-thelio button"),
                            };

                        let _ = sender.send(Some(event));
                    }
                    FirmwareEvent::Quit => {
                        eprintln!("received quit signal");
                        break;
                    }
                }
            }

            eprintln!("stopping firmware client connection");
        });
    }

    fn get_client<F, T, E>(name: &str, is_active: fn() -> bool, connect: F) -> Option<T>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        if is_active() {
            connect().map_err(|why| eprintln!("{} client error: {}", name, why)).ok()
        } else {
            None
        }
    }
}

impl Drop for FirmwareWidget {
    fn drop(&mut self) {
        let _ = self.sender.send(FirmwareEvent::Quit);
    }
}

fn scan(
    s76_client: Option<&System76Client>,
    fwupd_client: Option<&FwupdClient>,
    sender: &glib::Sender<Option<WidgetEvent>>,
) {
    let _ = sender.send(Some(WidgetEvent::Clear));

    if let Some(ref client) = s76_client {
        s76_scan(client, sender);
    }

    if let Some(client) = fwupd_client {
        fwupd_scan(client, sender);
    }
}

fn fwupd_scan(fwupd: &FwupdClient, sender: &glib::Sender<Option<WidgetEvent>>) {
    eprintln!("scanning fwupd devices");
    let devices = match fwupd.devices() {
        Ok(devices) => devices,
        Err(why) => {
            eprintln!("errored");
            let _ = sender.send(Some(WidgetEvent::Error(None, why.into())));
            return;
        }
    };

    for device in devices {
        if device.is_supported() {
            if let Ok(upgrades) = fwupd.upgrades(&device) {
                let releases: Box<[FwupdRelease]> = if let Some(current) =
                    upgrades.iter().position(|v| v.version == device.version)
                {
                    Box::from(Vec::from(&upgrades[current..]))
                } else if let Some(upgrade) = upgrades.into_iter().last() {
                    Box::from([upgrade])
                } else {
                    continue;
                };

                let _ = sender.send(Some(WidgetEvent::Fwupd(device, Some(releases))));
            } else {
                let _ = sender.send(Some(WidgetEvent::Fwupd(device, None)));
            }
        }
    }
}

fn s76_scan(client: &System76Client, sender: &glib::Sender<Option<WidgetEvent>>) {
    // Thelio system firmware check.
    let event = match client.bios() {
        Ok(current) => match client.download() {
            Ok(ThelioInfo { digest, changelog }) => {
                let fw = FirmwareInfo {
                    name: current.model,
                    current: current.version,
                    latest: changelog.versions.iter().last().expect("empty changelog").bios.clone(),
                };

                WidgetEvent::Thelio(fw, digest, changelog)
            }
            Err(why) => WidgetEvent::Error(None, why.into()),
        },
        Err(why) => WidgetEvent::Error(None, why.into()),
    };

    let _ = sender.send(Some(event));

    // Thelio I/O system firmware check.
    let event = match client.thelio_io_list() {
        Ok(list) => match client.thelio_io_download() {
            Ok(info) => {
                let ThelioIoInfo { digest, .. } = info;
                let digest = &mut Some(digest);
                for (num, (_, revision)) in list.iter().enumerate() {
                    let fw = FirmwareInfo {
                        name: format!("Thelio I/O #{}", num).into(),
                        current: Box::from(if revision.is_empty() {
                            "N/A"
                        } else {
                            revision.as_str()
                        }),
                        latest: Box::from(revision.as_str()),
                    };

                    let event = WidgetEvent::ThelioIo(fw, digest.take());
                    let _ = sender.send(Some(event));
                }

                None
            }
            Err(why) => Some(WidgetEvent::Error(None, why.into())),
        },
        Err(why) => Some(WidgetEvent::Error(None, why.into())),
    };

    if let Some(event) = event {
        let _ = sender.send(Some(event));
    }
}

fn fwupd_is_active() -> bool {
    systemd_service_is_active("fwupd")
}

fn s76_firmware_is_active() -> bool {
    systemd_service_is_active("system76-firmware-daemon")
}

fn systemd_service_is_active(name: &str) -> bool {
    Command::new("systemctl")
        .args(&["-q", "is-active", name])
        .status()
        .map_err(|why| eprintln!("{}", why))
        .ok()
        .map_or(false, |status| status.success())
}

fn reboot() {
    eprintln!("rebooting");
    if let Err(why) = Command::new("systemctl").arg("reboot").status() {
        eprintln!("failed to reboot: {}", why);
    }
}
