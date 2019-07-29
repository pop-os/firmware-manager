use crate::{traits::DynamicGtkResize, widgets::DeviceWidget};
use firmware_manager::FirmwareInfo;
use gtk::prelude::*;
use std::num::NonZeroU8;

#[derive(Shrinkwrap)]
pub struct DevicesView {
    #[shrinkwrap(main_field)]
    container: gtk::Container,
    device_firmware: gtk::ListBox,
    device_header: gtk::Label,
    system_firmware: gtk::ListBox,
    system_header: gtk::Label,
}

impl DevicesView {
    pub fn new() -> Self {
        let system_firmware = cascade! {
            gtk::ListBox::new();
            ..set_no_show_all(true);
            ..set_margin_bottom(12);
            ..set_selection_mode(gtk::SelectionMode::None);
            ..connect_row_activated(move |_, row| {
                let widget = row.get_child()
                    .and_then(|w| w.downcast::<gtk::Box>().ok())
                    .and_then(|w| w.get_children().into_iter().next());

                if let Some(widget) = widget {
                    let _ = widget.emit("button_press_event", &[&gdk::Event::new(gdk::EventType::ButtonPress)]);
                }
            });
        };

        let upper = system_firmware.downgrade();
        let device_firmware = cascade! {
            gtk::ListBox::new();
            ..set_no_show_all(true);
            ..set_selection_mode(gtk::SelectionMode::None);
            ..connect_row_activated(move |_, row| {
                let widget = row.get_child()
                    .and_then(|w| w.downcast::<gtk::Box>().ok())
                    .and_then(|w| w.get_children().into_iter().next());

                if let Some(widget) = widget {
                    let _ = widget.emit("button_press_event", &[&gdk::Event::new(gdk::EventType::ButtonPress)]);
                }
            });
            ..connect_key_press_event(move |listbox, event| {
                gtk::Inhibit(
                    if event.get_keyval() == gdk::enums::key::Up {
                        listbox.get_children()
                            .into_iter()
                            .filter_map(|widget| widget.downcast::<gtk::ListBoxRow>().ok())
                            .next()
                            .and_then(|row| if row.has_focus() { upper.upgrade() } else { None })
                            .and_then(|upper| upper.get_children().into_iter().last())
                            .map_or(false, |child| {
                                child.grab_focus();
                                true
                            })
                    } else {
                        false
                    }
                )
            });
        };

        let lower = device_firmware.downgrade();
        system_firmware.connect_key_press_event(move |listbox, event| {
            gtk::Inhibit(if event.get_keyval() == gdk::enums::key::Down {
                listbox
                    .get_children()
                    .into_iter()
                    .filter_map(|widget| widget.downcast::<gtk::ListBoxRow>().ok())
                    .last()
                    .and_then(|row| if row.has_focus() { lower.upgrade() } else { None })
                    .and_then(|lower| lower.get_children().into_iter().next())
                    .map_or(false, |child| {
                        child.grab_focus();
                        true
                    })
            } else {
                false
            })
        });

        let system_header = cascade! {
            gtk::Label::new("<b>System Firmware</b>".into());
            ..set_no_show_all(true);
            ..set_use_markup(true);
            ..set_xalign(0.0);
        };

        let device_header = cascade! {
            gtk::Label::new("<b>Device Firmware</b>".into());
            ..set_no_show_all(true);
            ..set_use_markup(true);
            ..set_xalign(0.0);
        };

        let layout: gtk::Box = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 12);
            ..set_halign(gtk::Align::Center);
            ..set_margin_top(24);
            ..add(&system_header);
            ..add(&system_firmware);
            ..add(&device_header);
            ..add(&device_firmware);
        };

        cascade! {
            gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
            ..add_widget(&system_firmware);
            ..add_widget(&device_firmware);
        };

        device_firmware.set_header_func(Some(Box::new(separator_header)));
        system_firmware.set_header_func(Some(Box::new(separator_header)));

        let container = cascade! {
            gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
            ..add(&layout);
            ..show_all();
            ..dynamic_resize(layout, NonZeroU8::new(66), None);
        };

        Self {
            container: container.upcast(),
            device_firmware,
            device_header,
            system_firmware,
            system_header,
        }
    }

    pub fn clear(&self) {
        self.system_firmware.foreach(WidgetExt::destroy);
        self.device_firmware.foreach(WidgetExt::destroy);
    }

    pub fn device(&self, info: &FirmwareInfo) -> DeviceWidget {
        self.show_devices();
        Self::append(&self.device_firmware, info)
    }

    pub fn system(&self, info: &FirmwareInfo) -> DeviceWidget {
        self.show_systems();
        Self::append(&self.system_firmware, info)
    }

    pub fn hide_devices(&self) {
        self.device_firmware.hide();
        self.device_header.hide();
    }

    pub fn hide_systems(&self) {
        self.system_firmware.hide();
        self.system_header.hide();
    }

    fn show_devices(&self) {
        self.device_firmware.show();
        self.device_header.show();
    }

    fn show_systems(&self) {
        self.system_firmware.show();
        self.system_header.show();
    }

    fn append(parent: &impl gtk::ContainerExt, info: &FirmwareInfo) -> DeviceWidget {
        let widget = DeviceWidget::new(info);
        parent.add(widget.as_ref());
        widget
    }
}

fn separator_header(current: &gtk::ListBoxRow, before: Option<&gtk::ListBoxRow>) {
    if before.is_some() {
        current.set_header(Some(&gtk::Separator::new(gtk::Orientation::Horizontal)));
    }
}
