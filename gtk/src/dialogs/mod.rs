use gtk::prelude::*;

#[cfg(feature = "fwupd")]
mod fwupd;

#[cfg(feature = "system76")]
mod system76;

#[cfg(feature = "fwupd")]
pub use self::fwupd::FwupdDialog;

#[cfg(feature = "system76")]
pub use self::system76::System76Dialog;

/// A generic GTK dialog which is displayed for firmware which requires a system reboot.
///
/// This dialog displays a changelog covering the details of the updates, and all prior updates, as
/// well as a confirmation button that will initiate configuring the system to be rebooted into the
/// firmware upgrade environment.
#[derive(Shrinkwrap)]
pub struct FirmwareUpdateDialog(gtk::Dialog);

impl FirmwareUpdateDialog {
    pub fn new<S: AsRef<str>, I: Iterator<Item = (S, S)>>(
        version: &str,
        changelog: I,
        has_battery: bool,
    ) -> Self {
        let changelog_entries = crate::changelog::generate_widget(changelog);

        let mut header = ["Firmware version ", version, " is available."].concat();

        if has_battery {
            header.push_str(
                " Connect your computer to power. <b>USB Type-C</b> charging is not \
                supported for firmware updates.",
            );
        }

        header.push_str(
            " After the system powers off, press the \
            power button to turn it back on. It may be necessary to power on more \
            than once after a firmware update. On machines running Open Firmware, \
            the system should then boot normally.",
        );

        let changelog_container = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 12);
            ..set_vexpand(true);
            ..add(&gtk::LabelBuilder::new().label(&*header).wrap(true).xalign(0.0).use_markup(true).build());
            ..add(&gtk::LabelBuilder::new().label("<b>Changelog</b>").use_markup(true).xalign(0.0).build());
            ..add(&changelog_entries);
            ..show_all();
        };

        let cancel = gtk::Button::new_with_label("Cancel");

        let reboot = cascade! {
            gtk::ButtonBuilder::new()
                .label("Reboot and Install")
                .build();
            ..get_style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        };

        let dialog = gtk::DialogBuilder::new()
            .accept_focus(true)
            .use_header_bar(1)
            .deletable(true)
            .destroy_with_parent(true)
            .width_request(600)
            .height_request(500)
            .build();

        let headerbar = dialog
            .get_header_bar()
            .expect("dialog generated without header bar")
            .downcast::<gtk::HeaderBar>()
            .expect("dialog header bar is not a header bar");

        cascade! {
            &headerbar;
            ..set_custom_title(
                Some(&gtk::LabelBuilder::new()
                    .label("<b>Firmware Update</b>")
                    .use_markup(true)
                    .build())
            );
            ..set_show_close_button(false);
            ..pack_start(&cancel);
            ..pack_end(&reboot);
        };

        cascade! {
            dialog.get_content_area();
            ..set_orientation(gtk::Orientation::Horizontal);
            ..set_border_width(12);
            ..set_spacing(12);
            ..add(
                &gtk::ImageBuilder::new()
                    .icon_name("application-x-firmware")
                    .icon_size(gtk::IconSize::Dialog.into())
                    .valign(gtk::Align::Start)
                    .build()
            );
            ..add(&cascade! {
                gtk::ScrolledWindowBuilder::new()
                    .hexpand(true)
                    .vexpand(true)
                    .build();
                ..add(&changelog_container);
            });
        };

        dialog.show_all();

        {
            let dialog = dialog.downgrade();
            cancel.connect_clicked(move |_| {
                if let Some(dialog) = dialog.upgrade() {
                    dialog.response(gtk::ResponseType::Cancel);
                }
            });
        }

        {
            let dialog = dialog.downgrade();
            reboot.connect_clicked(move |_| {
                if let Some(dialog) = dialog.upgrade() {
                    dialog.response(gtk::ResponseType::Accept);
                }
            });
        }

        Self(dialog)
    }
}
