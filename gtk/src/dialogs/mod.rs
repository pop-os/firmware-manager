mod fwupd;
mod system76;

pub use self::{fwupd::FwupdDialog, system76::System76Dialog};

use crate::fl;
use gtk::prelude::*;

/// A generic GTK dialog which is displayed for firmware which requires a system reboot.
///
/// This dialog displays a changelog covering the details of the updates, and all prior updates, as
/// well as a confirmation button that will initiate configuring the system to be rebooted into the
/// firmware upgrade environment.
#[derive(Shrinkwrap)]
pub struct FirmwareUpdateDialog(gtk::Dialog);

impl FirmwareUpdateDialog {
    pub fn new<S: AsRef<str>, I: Iterator<Item = (S, u64, S)>>(
        version: &str,
        changelog: I,
        has_battery: bool,
    ) -> Self {
        let changelog_entries = crate::changelog::generate_widget(changelog);

        let mut header = fl!("update-available", version = version);
        header.push(' ');

        if has_battery {
            header.push_str(&fl!("update-connect-to-ac"));
            header.push_str("\n\n")
        }

        header.push_str(&fl!(
            "update-guide",
            url_tag_start = "<a href=\"https://support.system76.com/articles/system-firmware/\">",
            url_tag_end = "</a>"
        ));

        let changelog_text = format!("<b>{}</b>", fl!("changelog"));

        let changelog_container = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 12);
            ..set_vexpand(true);
            ..add(&gtk::Label::builder().label(&*header).wrap(true).xalign(0.0).use_markup(true).build());
            ..add(&gtk::Label::builder().label(&*changelog_text).use_markup(true).xalign(0.0).build());
            ..add(&changelog_entries);
            ..show_all();
        };

        let cancel = gtk::Button::with_label(&fl!("button-cancel"));

        let reboot = cascade! {
            gtk::Button::builder()
                .label(&fl!("button-reboot-and-install"))
                .build();
            ..style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        };

        let dialog = gtk::Dialog::builder()
            .accept_focus(true)
            .use_header_bar(1)
            .deletable(true)
            .destroy_with_parent(true)
            .width_request(600)
            .height_request(500)
            .build();

        let headerbar = dialog
            .header_bar()
            .expect("dialog generated without header bar")
            .downcast::<gtk::HeaderBar>()
            .expect("dialog header bar is not a header bar");

        cascade! {
            &headerbar;
            ..set_custom_title(
                Some(&gtk::Label::builder()
                    .label(&format!("<b>{}</b>", fl!("header-firmware-update")))
                    .use_markup(true)
                    .build())
            );
            ..set_show_close_button(false);
            ..pack_start(&cancel);
            ..pack_end(&reboot);
        };

        cascade! {
            dialog.content_area();
            ..set_orientation(gtk::Orientation::Horizontal);
            ..set_border_width(12);
            ..set_spacing(12);
            ..add(
                &gtk::Image::builder()
                    .icon_name("application-x-firmware")
                    .icon_size(gtk::IconSize::Dialog.into())
                    .valign(gtk::Align::Start)
                    .build()
            );
            ..add(&cascade! {
                gtk::ScrolledWindow::builder()
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
