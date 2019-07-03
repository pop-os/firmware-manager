use crate::traits::DynamicGtkResize;
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
                gtk::Label::new("<b>System Firmware</b>");
                ..set_use_markup(true);
                ..set_xalign(0.0);
            });
            ..add(&system_firmware);
            ..add(&cascade! {
                gtk::Label::new("<b>Device Firmware</b>");
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

        // Enable w/ Rust 1.34.0 and gtk-rs 0.8
        // device_firmware.set_header_func(Some(Box::new(separator_header)));
        // system_firmware.set_header_func(Some(Box::new(separator_header)));

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

    fn append(container: &impl gtk::ContainerExt, info: &FirmwareInfo) -> DeviceWidget {
        let device = cascade! {
            gtk::Label::new(info.name.as_ref());
            ..set_xalign(0.0);
        };

        let label = cascade! {
            gtk::Label::new(info.current.as_ref());
            ..set_xalign(0.0);
            ..get_style_context().add_class(&gtk::STYLE_CLASS_DIM_LABEL);
        };

        let button = cascade! {
            gtk::Button::new_with_label("Update");
            ..set_halign(gtk::Align::End);
            ..set_hexpand(true);
            ..set_visible(info.current != info.latest);
            ..get_style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        };

        let progress = cascade! {
            gtk::ProgressBar::new();
            ..pulse();
            ..set_pulse_step(0.33);
            ..show();
        };

        let stack = cascade! {
            gtk::Stack::new();
            ..add(&button);
            ..add(&progress);
            ..set_visible_child(&button);
            ..show();
        };

        container.add(&cascade! {
            grid: gtk::Grid::new();
            ..set_border_width(12);
            ..set_column_spacing(12);
            ..attach(&device, 0, 0, 1, 1);
            ..attach(&label, 0, 1, 1, 1);
            ..attach(&stack, 1, 0, 1, 2);
            ..show_all();
        });

        DeviceWidget { button, label, progress, stack }
    }
}

pub struct DeviceWidget {
    pub button: gtk::Button,
    pub label: gtk::Label,
    pub progress: gtk::ProgressBar,
    pub stack: gtk::Stack,
}

#[derive(Debug)]
pub struct FirmwareInfo {
    pub name: Box<str>,
    pub current: Box<str>,
    pub latest: Box<str>,
}

// fn separator_header(current: &gtk::ListBoxRow, before: Option<&gtk::ListBoxRow>) {
//     current.set_header(Some(&gtk::Separator::new(gtk::Orientation::Horizontal)));
// }
