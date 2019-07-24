use crate::traits::DynamicGtkResize;
use firmware_manager::FirmwareInfo;
use gtk::prelude::*;
use std::num::NonZeroU8;

#[derive(Shrinkwrap)]
pub struct DevicesView {
    #[shrinkwrap(main_field)]
    container: gtk::Container,
    device_firmware: gtk::ListBox,
    system_firmware: gtk::ListBox,
}

impl DevicesView {
    pub fn new() -> Self {
        let system_firmware = cascade! {
            gtk::ListBox::new();
            ..set_margin_bottom(12);
            ..set_selection_mode(gtk::SelectionMode::None);
        };

        let device_firmware = cascade! {
            gtk::ListBox::new();
            ..set_selection_mode(gtk::SelectionMode::None);
        };

        let layout: gtk::Box = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 12);
            ..set_halign(gtk::Align::Center);
            ..set_margin_top(24);
            ..add(&cascade! {
                gtk::Label::new("<b>System Firmware</b>".into());
                ..set_use_markup(true);
                ..set_xalign(0.0);
            });
            ..add(&system_firmware);
            ..add(&cascade! {
                gtk::Label::new("<b>Device Firmware</b>".into());
                ..set_use_markup(true);
                ..set_xalign(0.0);
            });
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

        Self { container: container.upcast(), device_firmware, system_firmware }
    }

    pub fn clear(&self) {
        self.system_firmware.foreach(WidgetExt::destroy);
        self.device_firmware.foreach(WidgetExt::destroy);
    }

    pub fn device(&self, info: &FirmwareInfo) -> DeviceWidget {
        Self::append(&self.device_firmware, info)
    }

    pub fn system(&self, info: &FirmwareInfo) -> DeviceWidget {
        Self::append(&self.system_firmware, info)
    }

    fn append(parent: &impl gtk::ContainerExt, info: &FirmwareInfo) -> DeviceWidget {
        let device = gtk::LabelBuilder::new().label(info.name.as_ref()).xalign(0.0).build();

        let label = cascade! {
            gtk::LabelBuilder::new()
                .label(info.current.as_ref())
                .xalign(0.0)
                .build();
            ..get_style_context().add_class(&gtk::STYLE_CLASS_DIM_LABEL);
        };

        let button = cascade! {
            gtk::ButtonBuilder::new()
                .label("Update")
                .halign(gtk::Align::End)
                .hexpand(true)
                .visible(info.current != info.latest)
                .build();
            ..get_style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        };

        let progress = cascade! {
            gtk::ProgressBarBuilder::new().pulse_step(0.33).build();
            ..pulse();
            ..show();
        };

        let stack = cascade! {
            gtk::Stack::new();
            ..add(&button);
            ..add(&progress);
            ..set_visible_child(&button);
            ..show();
        };

        let container = cascade! {
            gtk::EventBox::new();
            ..add(&cascade! {
                gtk::GridBuilder::new()
                    .border_width(12)
                    .column_spacing(12)
                    .build();
                ..attach(&device, 0, 0, 1, 1);
                ..attach(&label, 0, 1, 1, 1);
                ..attach(&stack, 1, 0, 1, 2);
            });
            ..show_all();
            ..set_events(gdk::EventMask::BUTTON_PRESS_MASK);
        };

        parent.add(&container);

        DeviceWidget { container, button, label, progress, stack }
    }
}

pub struct DeviceWidget {
    pub container: gtk::EventBox,
    pub button:    gtk::Button,
    pub label:     gtk::Label,
    pub progress:  gtk::ProgressBar,
    pub stack:     gtk::Stack,
}

impl DeviceWidget {
    pub fn connect_clicked<F: Fn() + 'static>(&self, func: F) {
        self.container.connect_button_press_event(move |_, _| {
            func();
            gtk::Inhibit(true)
        });
    }

    pub fn connect_upgrade_clicked<F: Fn() + 'static>(&self, func: F) {
        self.button.connect_clicked(move |_| func());
    }
}

fn separator_header(current: &gtk::ListBoxRow, before: Option<&gtk::ListBoxRow>) {
    if before.is_some() {
        current.set_header(Some(&gtk::Separator::new(gtk::Orientation::Horizontal)));
    }
}
