#[macro_use]
extern crate cascade;

mod logging;

use firmware_manager_gtk::FirmwareWidget;
use gio::prelude::*;
use gtk::prelude::*;
use i18n_embed::DesktopLanguageRequester;
use std::rc::Rc;

pub const APP_ID: &str = "com.system76.FirmwareManager";

fn main() {
    translate();
    argument_parsing();

    better_panic::install();
    glib::set_program_name(APP_ID.into());
    gtk::init().expect("failed to init GTK");

    let application = gtk::Application::builder().application_id(APP_ID).build();

    application.connect_activate(|app| {
        if let Some(window) = app.window_by_id(0) {
            window.present();
        }
    });

    application.connect_startup(|app| {
        let widget = Rc::new(FirmwareWidget::new());
        widget.scan();

        let weak_widget = Rc::downgrade(&widget);
        let headerbar = cascade! {
            gtk::HeaderBar::builder()
                .title("Firmware Manager")
                .show_close_button(true)
                .build();
            ..pack_end(&cascade! {
                gtk::Button::builder()
                    .image(gtk::Image::builder()
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
            gtk::ApplicationWindow::builder()
                .application(app)
                .icon_name("firmware-manager")
                .window_position(gtk::WindowPosition::Center)
                .default_width(768)
                .default_height(576)
                .build();
            ..set_titlebar(Some(&headerbar));
            ..add(widget.container());
            ..show_all();
            ..connect_delete_event(move |window, _| {
                window.close();

                // Allow this closure to attain ownership of our firmware widget,
                // so that this widget will exist for as long as the window exists.
                let _widget = &widget;

                Inhibit(false)
            });
            ..connect_key_press_event(move |window, event| {
                use gdk::keys::constants as key;
                gtk::Inhibit(match event.keyval() {
                    key::q if event.state().contains(gdk::ModifierType::CONTROL_MASK) => {
                        let _ = window.emit_by_name::<()>("delete-event", &[&gdk::Event::new(gdk::EventType::Delete)]);
                        true
                    }
                    _ => false
                })
            });
        };
    });

    application.run();
}

/// Manages argument parsing for the GTK application via clap.
///
/// Currently the primary purpose is to determine the logging level.
fn argument_parsing() {
    use clap::{Command, Arg, ArgAction};
    use log::LevelFilter;

    let matches = Command::new("com.system76.FirmwareManager")
        .arg(
            Arg::new("verbose")
                .short('v')
                .action(ArgAction::Count)
                .help("define the logging level; multiple occurrences increases the logging level"),
        )
        .get_matches();

    let logging_level = match matches.get_count("verbose") {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    if let Err(why) = logging::install(logging_level) {
        eprintln!("failed to initiate logging: {}", why);
    }
}

fn translate() {
    let localizer = firmware_manager_gtk::localizer();
    let requested_languages = DesktopLanguageRequester::requested_languages();

    if let Err(error) = localizer.select(&requested_languages) {
        eprintln!("Error while loading languages for firmware-manager-gtk {}", error);
    }
}
