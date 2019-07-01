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

use gtk::{self, prelude::*};
use slotmap::DefaultKey as Entity;
use slotmap::{SecondaryMap as SM, SlotMap, SparseSecondaryMap as SSM};
use std::{
    error::Error as ErrorTrait,
    process::Command,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread,
};
use system76_firmware_daemon::{
    Changelog, Client as System76Client, Digest, Error as System76Error, ThelioInfo, ThelioIoInfo,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "error in system76-firmware client")]
    System76(#[error(cause)] System76Error),
}

impl From<System76Error> for Error {
    fn from(error: System76Error) -> Self {
        Error::System76(error)
    }
}

enum FirmwareEvent {
    Scan,
    Thelio(Entity, Digest, Box<str>),
    ThelioIo(Entity, Digest, Box<str>),
    Quit,
}

#[derive(Debug)]
enum WidgetEvent {
    Clear,
    Thelio(FirmwareInfo, Digest, Changelog),
    ThelioIo(FirmwareInfo, Option<Digest>),
    ThelioIoUpdated(Box<str>),
    DeviceUpdated(Entity, Box<str>),
    Error(Option<Entity>, Error),
}

pub struct FirmwareWidget {
    container: gtk::Container,
    sender: SyncSender<FirmwareEvent>,
}

impl FirmwareWidget {
    pub fn new() -> Self {
        let (sender, rx) = sync_channel(0);
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

            let mut entities: SlotMap<Entity, ()> = SlotMap::new();

            let mut system_widget: Option<Entity> = None;
            let mut device_widgets: SSM<Entity, DeviceWidget> = SSM::new();
            let mut thelio_io_widgets: SM<Entity, ()> = SM::new();

            receiver.attach(None, move |event| {
                let event = match event {
                    Some(event) => event,
                    None => return glib::Continue(false),
                };

                match event {
                    WidgetEvent::Clear => {
                        view_devices.clear();
                        stack.set_visible_child(view_empty.as_ref());
                    }
                    WidgetEvent::DeviceUpdated(entity, latest) => {
                        if let Some(widget) = device_widgets.get(entity) {
                            widget.stack.set_visible(false);
                            widget.label.set_text(latest.as_ref());
                        }
                    }
                    WidgetEvent::ThelioIoUpdated(latest) => {
                        for entity in thelio_io_widgets.keys() {
                            let widget = &device_widgets[entity];
                            widget.stack.set_visible(false);
                            widget.label.set_text(latest.as_ref());
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
                    WidgetEvent::Thelio(info, digest, changelog) => {
                        let widget = view_devices.system(&info);
                        let entity = entities.insert(());

                        if info.current == info.latest {
                            widget.stack.set_visible(false);
                        } else {
                            let sender = sender.clone();
                            let stack = widget.stack.downgrade();
                            let progress = widget.progress.downgrade();
                            widget.button.connect_clicked(move |_| {
                                let &FirmwareInfo { ref current, ref latest, .. } = &info;
                                let log_entries = changelog
                                    .versions
                                    .iter()
                                    .skip_while(|version| version.bios.as_ref() != current.as_ref())
                                    .map(|version| version.description.as_ref());

                                let dialog = FirmwareUpdateDialog::new(latest, log_entries);
                                dialog.show_all();

                                let expected: i32 = gtk::ResponseType::Accept.into();
                                if expected == dialog.run() {
                                    // Exchange the button for a progress bar.
                                    if let (Some(stack), Some(progress)) =
                                        (stack.upgrade(), progress.upgrade())
                                    {
                                        stack.set_visible_child(&progress);
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
                        system_widget = Some(entity);

                        stack.set_visible_child(view_devices.as_ref());
                    }
                    WidgetEvent::ThelioIo(info, digest) => {
                        let widget = view_devices.device(&info);
                        let requires_update = info.current != info.latest;
                        let entity = entities.insert(());

                        // Only the first Thelio I/O device will have a connected button.
                        if let Some(digest) = digest {
                            let sender = sender.clone();
                            let latest = info.latest;
                            let stack = widget.stack.downgrade();
                            let progress = widget.progress.downgrade();
                            widget.button.connect_clicked(move |_| {
                                // Exchange the button for a progress bar.
                                if let (Some(stack), Some(progress)) =
                                    (stack.upgrade(), progress.upgrade())
                                {
                                    stack.set_visible_child(&progress);
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
                        thelio_io_widgets.insert(entity, ());

                        // If any Thelio I/O device requires an update, then enable the
                        // update button on the first Thelio I/O device widget.
                        if requires_update {
                            let entity = thelio_io_widgets
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
            let client = if system76_firmware_is_active() {
                System76Client::new()
                    .map_err(|why| eprintln!("firmware client error: {}", why))
                    .ok()
            } else {
                None
            };

            while let Ok(event) = receiver.recv() {
                match event {
                    FirmwareEvent::Scan => scan(client.as_ref(), &sender),
                    FirmwareEvent::Thelio(entity, digest, _latest) => {
                        match client.as_ref().map(|client| client.schedule(&digest)) {
                            Some(Ok(_)) => {
                                let _ = Command::new("systemctl").arg("reboot").status();
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
                            match client.as_ref().map(|client| client.thelio_io_update(&digest)) {
                                Some(Ok(_)) => WidgetEvent::ThelioIoUpdated(latest),
                                Some(Err(why)) => WidgetEvent::Error(Some(entity), why.into()),
                                None => panic!("thelio event assigned to non-thelio button"),
                            };

                        eprintln!("replying with {:?}", event);

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
}

impl Drop for FirmwareWidget {
    fn drop(&mut self) {
        let _ = self.sender.send(FirmwareEvent::Quit);
    }
}

fn scan(client: Option<&System76Client>, sender: &glib::Sender<Option<WidgetEvent>>) {
    let _ = sender.send(Some(WidgetEvent::Clear));

    if let Some(ref client) = client {
        // Thelio system firmware check.
        let event = match client.bios() {
            Ok(current) => match client.download() {
                Ok(ThelioInfo { digest, changelog }) => {
                    let fw = FirmwareInfo {
                        name: current.model,
                        current: current.version,
                        latest: changelog
                            .versions
                            .iter()
                            .last()
                            .expect("empty changelog")
                            .bios
                            .clone(),
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
}

fn system76_firmware_is_active() -> bool {
    Command::new("systemctl")
        .args(&["-q", "is-active", "system76-firmware-daemon"])
        .status()
        .map_err(|why| eprintln!("{}", why))
        .ok()
        .map_or(false, |status| status.success())
}
