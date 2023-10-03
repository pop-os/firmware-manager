use crate::fl;
use gtk::prelude::*;
use chrono::{LocalResult, TimeZone, Utc};

pub fn generate_widget_none() -> gtk::Box {
    gtk::Box::builder()
        .margin_start(48)
        .margin_end(48)
        .child(
            &gtk::Label::builder()
                .label(&fl!("changelog-unavailable"))
                .build()
                .upcast::<gtk::Widget>(),
        )
        .build()
}

pub fn generate_widget<I, S>(changelog: I) -> gtk::Box
where
    S: AsRef<str>,
    I: Iterator<Item = (S, u64, S)>,
{
    let changelog_entries = cascade! {
        gtk::Box::new(gtk::Orientation::Vertical, 12);
        ..show_all();
    };

    let mut initiated = false;
    changelog.for_each(|(version, date, entry)| {
        let markdown = if entry.as_ref().is_empty() {
            fl!("changelog-unavailable")
        } else {
            html2md::parse_html(entry.as_ref()).trim().to_string()
        };

        // NOTE: If we don't set a max width in chars, the label resizes its parent.
        // Even though we set a max width of chars, this will be ignored by GTK as the
        // parent is resized.

        const PADDING: i32 = 48;

        let version_label = match Utc.timestamp_opt(date as i64, 0) {
            LocalResult::Single(dt) => {
                format!("<b>{}</b> ({})", version.as_ref(), dt.format("%Y-%m-%d"))
            }
            _ => format!("<b>{}</b>", version.as_ref()),
        };

        let version = gtk::Label::builder()
            .label(&version_label)
            .use_markup(true)
            .xalign(0.0)
            .max_width_chars(40)
            .margin_start(PADDING)
            .margin_end(PADDING)
            .build();

        let changelog = gtk::Label::builder()
            .label(&*markdown)
            .wrap(true)
            .xalign(0.0)
            .max_width_chars(40)
            .margin_start(PADDING)
            .margin_end(PADDING)
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
