use firmware_manager::FirmwareInfo;
use gtk::prelude::*;

#[derive(Shrinkwrap)]
pub struct DeviceWidget {
    #[shrinkwrap(main_field)]
    pub container: gtk::EventBox,
    pub button: gtk::Button,
    pub label: gtk::Label,
    pub progress: gtk::ProgressBar,
    pub stack: gtk::Stack,
}

impl DeviceWidget {
    pub fn new(info: &FirmwareInfo) -> Self {
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
            gtk::ProgressBarBuilder::new()
                .show_text(true)
                .pulse_step(1.0 / f64::from(info.install_duration))
                .build();
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

        DeviceWidget { container, button, label, progress, stack }
    }

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
