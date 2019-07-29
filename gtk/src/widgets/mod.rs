use firmware_manager::FirmwareInfo;
use gtk::prelude::*;

#[derive(Shrinkwrap)]
pub struct DeviceWidget {
    #[shrinkwrap(main_field)]
    pub container: gtk::Container,
    pub event_box: gtk::EventBox,
    pub revealer: gtk::Revealer,
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
                .pulse_step(0.1 / f64::from(info.install_duration + 1))
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

        let revealer = gtk::Revealer::new();

        let event_box = cascade! {
            gtk::EventBoxBuilder::new()
                .can_focus(false)
                .events(gdk::EventMask::BUTTON_PRESS_MASK)
                .build();
            ..add(&cascade! {
                gtk::GridBuilder::new()
                    .column_spacing(12)
                    .build();
                ..attach(&device, 0, 0, 1, 1);
                ..attach(&label, 0, 1, 1, 1);
                ..attach(&stack, 1, 0, 1, 2);
            });
        };

        let container = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 6);
            ..set_border_width(12);
            ..set_can_focus(false);
            ..add(&event_box);
            ..add(&revealer);
            ..show_all();
        };

        DeviceWidget {
            container: container.upcast::<gtk::Container>(),
            button,
            event_box,
            label,
            progress,
            revealer,
            stack,
        }
    }

    /// Activates when the widget's container is clicked.
    pub fn connect_clicked<F: Fn(gtk::Revealer) + 'static>(&self, func: F) {
        let revealer = self.revealer.downgrade();
        self.event_box.connect_button_press_event(move |_, _| {
            func(revealer.upgrade().expect("revealer for device did not exist"));
            gtk::Inhibit(true)
        });
    }

    /// Activates when the widget's container's button is clicked.
    pub fn connect_upgrade_clicked<F: Fn() + 'static>(&self, func: F) {
        self.button.connect_clicked(move |_| func());
    }
}
