use firmware_manager::FirmwareInfo;
use gtk::prelude::*;
use std::rc::Rc;

#[derive(Shrinkwrap)]
pub struct DeviceWidgetStack {
    #[shrinkwrap(main_field)]
    pub stack: gtk::Stack,
    pub button: gtk::Button,
    pub progress: gtk::ProgressBar,
    pub waiting: gtk::Label,
}

impl DeviceWidgetStack {
    pub fn switch_to_waiting(&self) {
        self.stack.set_visible_child(&self.waiting);
        self.progress.set_fraction(0.0);
    }

    pub fn switch_to_progress(&self, message: &str) {
        self.stack.set_visible_child(&self.progress);
        self.progress.set_text(message.into());
        self.progress.set_fraction(0.0);
    }
}

#[derive(Shrinkwrap)]
pub struct DeviceWidget {
    #[shrinkwrap(main_field)]
    pub container: gtk::Container,
    pub event_box: gtk::EventBox,
    pub revealer: gtk::Revealer,
    pub label: gtk::Label,
    pub stack: Rc<DeviceWidgetStack>,
}

impl DeviceWidget {
    pub fn new(info: &FirmwareInfo) -> Self {
        let device = gtk::LabelBuilder::new()
            .label(info.name.as_ref())
            .xalign(0.0)
            .valign(gtk::Align::End)
            .build();

        let label = cascade! {
            gtk::LabelBuilder::new()
                .label(info.current.as_ref())
                .xalign(0.0)
                .valign(gtk::Align::Start)
                .build();
            ..get_style_context().add_class(&gtk::STYLE_CLASS_DIM_LABEL);
        };

        let button = cascade! {
            gtk::ButtonBuilder::new()
                .label("Update")
                .halign(gtk::Align::End)
                .hexpand(true)
                .vexpand(true)
                .build();
            ..get_style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        };

        let progress = cascade! {
            gtk::ProgressBarBuilder::new()
                .show_text(true)
                .pulse_step(0.1 / f64::from(info.install_duration + 1))
                .valign(gtk::Align::Center)
                .height_request(30)
                .build();
            ..pulse();
        };

        let waiting = gtk::LabelBuilder::new().label("Waiting").build();

        let stack = cascade! {
            gtk::Stack::new();
            ..add(&button);
            ..add(&progress);
            ..add(&waiting);
            ..set_visible_child(&button);
        };

        let dropdown_image = gtk::ImageBuilder::new()
            .icon_name("pan-end-symbolic")
            .icon_size(gtk::IconSize::Menu.into())
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Center)
            .build();

        let dropdown_image_ = dropdown_image.downgrade();
        let revealer = cascade! {
            gtk::Revealer::new();
            ..connect_property_reveal_child_notify(move |revealer| {
                dropdown_image_.upgrade()
                    .expect("dropdown image did not exist")
                    .set_from_icon_name(
                        Some(if revealer.get_reveal_child() {
                            "pan-down-symbolic"
                        } else {
                            "pan-end-symbolic"
                        }),
                        gtk::IconSize::Menu
                    );
            });
        };

        let event_box = cascade! {
            gtk::EventBoxBuilder::new()
                .can_focus(false)
                .hexpand(true)
                .events(gdk::EventMask::BUTTON_PRESS_MASK)
                .build();
            ..add(&cascade! {
                gtk::GridBuilder::new()
                    .column_spacing(12)
                    .row_spacing(3)
                    .build();
                ..attach(&dropdown_image, 0, 0, 1, 2);
                ..attach(&device, 1, 0, 1, 1);
                ..attach(&label, 1, 1, 1, 1);
                ..attach(&stack, 2, 0, 1, 2);
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
            event_box,
            label,
            revealer,
            stack: Rc::new(DeviceWidgetStack { button, stack, progress, waiting }),
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
        self.stack.button.connect_clicked(move |_| func());
    }
}
