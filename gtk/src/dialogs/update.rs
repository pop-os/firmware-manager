use gtk::prelude::*;

#[derive(Shrinkwrap)]
pub struct FirmwareUpdateDialog(gtk::Dialog);

impl FirmwareUpdateDialog {
    pub fn new<S: AsRef<str>, I: Iterator<Item = (S, S)>>(version: &str, changelog: I) -> Self {
        let changelog_entries = &cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 12);
        };

        changelog.for_each(|(version, entry)| {
            let markdown = html2runes::markdown::convert_string(entry.as_ref());

            changelog_entries.add(&cascade! {
                gtk::Box::new(gtk::Orientation::Vertical, 12);
                ..add(&cascade! {
                    gtk::Label::new(Some(&*format!("<b>{}</b>", version.as_ref())));
                    ..set_use_markup(true);
                    ..set_xalign(0.0);
                });
                ..add(&gtk::Separator::new(gtk::Orientation::Horizontal));
                ..add(&gtk::LabelBuilder::new().label(&*markdown).wrap(true).xalign(0.0).build());
            });
        });

        let header_text =
            ["Firmware version ", version.trim(), " is available. Fixes and features include:"]
                .concat();

        let cancel = gtk::Button::new_with_label("Cancel".into());

        let reboot = cascade! {
            gtk::Button::new_with_label("Reboot and Install".into());
            ..get_style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        };

        let dialog =
            gtk::Object::new(gtk::Dialog::static_type(), &[("use-header-bar", &true)]).unwrap();

        let dialog = cascade! {
            unsafe { dialog.unsafe_cast::<gtk::Dialog>() };
            ..set_accept_focus(true);
            ..set_deletable(true);
            ..set_destroy_with_parent(true);
            ..set_size_request(400, 300);
        };

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
            ..add(&cascade! {
                gtk::Image::new_from_icon_name("application-x-firmware".into(), gtk::IconSize::Dialog.into());
                ..set_valign(gtk::Align::Start);
            });
            ..add(&cascade! {
                gtk::Box::new(gtk::Orientation::Vertical, 12);
                ..add(&cascade! {
                    gtk::Label::new(Some(&*header_text));
                    ..set_line_wrap(true);
                    ..set_use_markup(true);
                    ..set_xalign(0.0);
                });
                ..add(&cascade! {
                    gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
                    ..set_hexpand(true);
                    ..set_vexpand(true);
                    ..add(changelog_entries);
                });
                ..add(&cascade! {
                    gtk::Label::new("If you're on a laptop, <b>plug into power</b> before you begin.".into());
                    ..set_use_markup(true);
                    ..set_xalign(0.0);
                });
            });
        };

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

    // pub fn run(&self) -> gtk::ResponseType {
    //     self.set_window_size()
    // }
}
