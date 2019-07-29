#[macro_use]
extern crate cascade;

use firmware_manager_gtk::FirmwareWidget;
use gio::prelude::*;
use gtk::prelude::*;
use std::rc::Rc;

pub const APP_ID: &str = "com.system76.FirmwareManager";

fn main() {
    glib::set_program_name(APP_ID.into());
    gtk::init().expect("failed to init GTK");

    let application = gtk::ApplicationBuilder::new().application_id(APP_ID).build();

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
            gtk::HeaderBarBuilder::new()
                .title("System76 Firmware Manager")
                .show_close_button(true)
                .build();
            ..pack_end(&cascade! {
                gtk::ButtonBuilder::new()
                    .image(gtk::ImageBuilder::new()
                        .icon_name("view-refresh-symbolic")
                        .icon_size(gtk::IconSize::SmallToolbar.into())
                        .build()
                        .upcast_ref::<gtk::Widget>()
                    )
                    .build();
                ..connect_clicked(move |_| {
                    if let Some(widget) = weak_widget.upgrade() {
                        widget.scan();
                    }
                });
            });
        };

        let _window = cascade! {
            gtk::ApplicationWindowBuilder::new()
                .application(app)
                .icon_name("firmware-manager")
                .window_position(gtk::WindowPosition::Center)
                .default_width(768)
                .default_height(576)
                .build();
            ..set_keep_above(true);
            ..set_titlebar(Some(&headerbar));
            ..add(widget.container());
            ..show_all();
            ..connect_delete_event(move |window, _| {
                window.destroy();

                // Allow this closure to attain ownership of our firmware widget,
                // so that this widget will exist for as long as the window exists.
                let _widget = &widget;

                Inhibit(false)
            });
            ..connect_key_press_event(move |window, event| {
                use gdk::enums::key;
                gtk::Inhibit(match event.get_keyval() {
                    key::q if event.get_state().contains(gdk::ModifierType::CONTROL_MASK) => {
                        let _ = window.emit("delete-event", &[&gdk::Event::new(gdk::EventType::Delete)]);
                        true
                    }
                    _ => false
                })
            });
        };
    });

    application.run(&[]);
}
