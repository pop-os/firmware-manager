use gtk::prelude::*;

#[derive(Shrinkwrap)]
pub struct FirmwareUpdateDialog(gtk::Dialog);

impl FirmwareUpdateDialog {
    pub fn new<S: AsRef<str>, I: Iterator<Item = S>>(version: &str, changelog: I) -> Self {
        let mut text_content =
            ["Firmware version ", version.trim(), " is available. Fixes and features include:\n\n"]
                .concat();

        {
            let text_content = &mut text_content;

            changelog.for_each(|entry| {
                text_content.push_str("  * ");
                text_content.push_str(entry.as_ref());
                text_content.push_str("\n");
            });

            text_content
                .push_str("\nIf you're on a laptop, <b>plug into power</b> before you begin");
        }

        let image = cascade! {
            gtk::Image::new_from_icon_name(
                "application-x-firmware",
                gtk::IconSize::Dialog.into(),
            );
            ..set_valign(gtk::Align::Start);
        };

        let text = cascade! {
            gtk::Label::new(text_content.as_str());
            ..set_line_wrap(true);
            ..set_use_markup(true);
            ..set_xalign(0.0);
            ..set_yalign(0.0);
        };

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
        };

        let headerbar = dialog
            .get_header_bar()
            .expect("dialog generated without header bar")
            .downcast::<gtk::HeaderBar>()
            .expect("dialog header bar is not a header bar");

        cascade! {
            &headerbar;
            ..set_custom_title(&cascade! {
                gtk::Label::new("<b>Firmware Update</b>");
                ..set_use_markup(true);
            });
            ..set_show_close_button(false);
            ..pack_start(&cancel);
            ..pack_end(&reboot);
        };

        cascade! {
            dialog.get_content_area();
            ..set_orientation(gtk::Orientation::Horizontal);
            ..set_border_width(12);
            ..set_spacing(12);
            ..add(&image);
            ..add(&text);
        };

        {
            let dialog = dialog.clone();
            cancel.connect_clicked(move |_| {
                dialog.response(gtk::ResponseType::Cancel);
            });
        }

        {
            let dialog = dialog.clone();
            reboot.connect_clicked(move |_| {
                dialog.response(gtk::ResponseType::Accept);
            });
        }

        Self(dialog)
    }
}
