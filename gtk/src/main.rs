#[macro_use]
extern crate cascade;

use firmware_manager_gtk::FirmwareWidget;
use gio::{prelude::*, ApplicationFlags};
use gtk::{prelude::*, Application};
use std::rc::Rc;

pub const APP_ID: &str = "com.system76.FirmwareManager";

fn main() {
    glib::set_program_name(APP_ID.into());

    let application =
        Application::new(APP_ID, ApplicationFlags::empty()).expect("GTK initialization failed");

    application.connect_activate(|app| {
        if let Some(window) = app.get_window_by_id(0) {
            window.present();
        }
    });

    application.connect_startup(|app| {
        let widget = Rc::new(FirmwareWidget::new());
        widget.scan();

        let weak_widget = Rc::downgrade(&widget);
        let headerbar = cascade! {
            gtk::HeaderBar::new();
            ..set_title("System76 Firmware Manager");
            ..set_show_close_button(true);
            ..show();
            ..pack_end(&cascade! {
                gtk::Button::new_from_icon_name(
                    "view-refresh-symbolic",
                    gtk::IconSize::SmallToolbar
                );
                ..connect_clicked(move |_| {
                    if let Some(widget) = weak_widget.upgrade() {
                        widget.scan();
                    }
                });
                ..show();
            });
        };

        let _window = cascade! {
            gtk::ApplicationWindow::new(app);
            ..set_titlebar(Some(&headerbar));
            ..set_icon_name("firmware-manager");
            ..set_keep_above(true);
            ..set_property_window_position(gtk::WindowPosition::Center);
            ..set_default_size(768, 576);
            ..add(widget.container());
            ..show();
            ..connect_delete_event(move |window, _| {
                window.destroy();

                // Allow this closure to attain ownership of our firmware widget,
                // so that this widget will exist for as long as the window exists.
                let _widget = &widget;

                Inhibit(false)
            });
        };
    });

    application.run(&[]);
}
