use gtk::prelude::*;

#[derive(Shrinkwrap)]
pub struct FirmwareUpdateDialog(gtk::Dialog);

impl FirmwareUpdateDialog {
    pub fn new<S: AsRef<str>, I: Iterator<Item = (S, S)>>(
        version: &str,
        changelog: I,
        upgradeable: bool,
        needs_reboot: bool,
    ) -> Self {
        let changelog_entries = crate::changelog::generate_widget(changelog, false);

        let header_text =
            ["Firmware version ", version.trim(), " is available. Fixes and features include:"]
                .concat();

        let cancel = gtk::Button::new_with_label("Cancel");

        let reboot = cascade! {
            gtk::ButtonBuilder::new()
                .label(if needs_reboot { "Reboot and Install" } else { "Install" })
                .build();
            ..get_style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        };

        let dialog = gtk::DialogBuilder::new()
            .accept_focus(true)
            .use_header_bar(1)
            .deletable(true)
            .destroy_with_parent(true)
            .width_request(400)
            .height_request(300)
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
                gtk::Box::new(gtk::Orientation::Vertical, 6);
                ..add(
                    &gtk::LabelBuilder::new()
                        .label(&*header_text)
                        .wrap(true)
                        .use_markup(true)
                        .xalign(0.0)
                        .build()
                );
                ..add(&cascade! {
                    gtk::ScrolledWindowBuilder::new()
                        .hexpand(true)
                        .vexpand(true)
                        .build();
                    ..add(&changelog_entries);
                });
                ..add(
                    &gtk::LabelBuilder::new()
                        .label("If you're on a laptop, <b>plug into power</b> before you begin.")
                        .use_markup(true)
                        .xalign(0.0)
                        .build()
                );
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

        if upgradeable {
            let dialog = dialog.downgrade();
            reboot.connect_clicked(move |_| {
                if let Some(dialog) = dialog.upgrade() {
                    dialog.response(gtk::ResponseType::Accept);
                }
            });
        } else {
            reboot.hide();
        }

        Self(dialog)
    }
}
