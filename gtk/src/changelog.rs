use gtk::prelude::*;
use std::borrow::Cow;

pub fn generate_widget<I, S>(changelog: I, pad: bool) -> gtk::Box
where
    S: AsRef<str>,
    I: Iterator<Item = (S, S)>,
{
    let changelog_entries = cascade! {
        gtk::Box::new(gtk::Orientation::Vertical, 12);
        ..show_all();
    };

    let mut initiated = false;
    changelog.for_each(|(version, entry)| {
        let markdown = if entry.as_ref().is_empty() {
            Cow::Borrowed("No changelog available")
        } else {
            Cow::Owned(html2runes::markdown::convert_string(entry.as_ref()))
        };

        // NOTE: If we don't set a max width in chars, the label resizes its parent.
        // Even though we set a max width of chars, this will be ignored by GTK as the
        // parent is resized.

        let margin = if pad { 48 } else { 0 };

        let version = gtk::LabelBuilder::new()
            .label(&*format!("<b>{}</b>", version.as_ref()))
            .use_markup(true)
            .xalign(0.0)
            .max_width_chars(40)
            .margin_start(margin)
            .margin_end(margin)
            .build();

        let changelog = gtk::LabelBuilder::new()
            .label(&*markdown)
            .wrap(true)
            .xalign(0.0)
            .max_width_chars(40)
            .margin_start(margin)
            .margin_end(margin)
            .build();

        if initiated {
            changelog_entries.add(&gtk::Separator::new(gtk::Orientation::Horizontal));
        }

        initiated = true;
        changelog_entries.add(&version);
        changelog_entries.add(&changelog);
    });

    changelog_entries
}
