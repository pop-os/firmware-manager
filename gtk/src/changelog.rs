use crate::fl;
use gtk::prelude::*;

pub fn generate_widget_none() -> gtk::Box {
    gtk::BoxBuilder::new()
        .margin_start(48)
        .margin_end(48)
        .child(
            &gtk::LabelBuilder::new()
                .label(&fl!("changelog-unavailable"))
                .build()
                .upcast::<gtk::Widget>(),
        )
        .build()
}

pub fn generate_widget<I, S, X>(changelog: I) -> gtk::Box
where
    S: AsRef<str>,
    X: AsRef<str>,
    I: Iterator<Item = (S, X)>,
{
    let changelog_entries = cascade! {
        gtk::Box::new(gtk::Orientation::Vertical, 12);
        ..show_all();
    };

    let mut initiated = false;
    changelog.for_each(|(version, entry)| {
        let markdown = if entry.as_ref().is_empty() {
            fl!("changelog-unavailable")
        } else {
            html2runes::markdown::convert_string(entry.as_ref())
        };

        // NOTE: If we don't set a max width in chars, the label resizes its parent.
        // Even though we set a max width of chars, this will be ignored by GTK as the
        // parent is resized.

        const PADDING: i32 = 48;

        let version = gtk::LabelBuilder::new()
            .label(&*format!("<b>{}</b>", version.as_ref()))
            .use_markup(true)
            .xalign(0.0)
            .max_width_chars(40)
            .margin_start(PADDING)
            .margin_end(PADDING)
            .build();

        let changelog = gtk::LabelBuilder::new()
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
