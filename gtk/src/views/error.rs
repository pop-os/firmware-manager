use crate::fl;
use gtk::prelude::*;

/// View displayed when scanning has completed, but no firmware was found.
#[derive(Shrinkwrap)]
pub struct EmptyView(gtk::Container);

impl EmptyView {
    pub fn new() -> Self {
        Self(error_view("firmware-manager-symbolic", &fl!("view-empty")))
    }
}

/// View displayed to users who lack administrative permissions.
#[derive(Shrinkwrap)]
pub struct PermissionView(gtk::Container);

impl PermissionView {
    pub fn new() -> Self {
        Self(error_view("system-lock-screen-symbolic", &fl!("view-permission")))
    }
}

/// Template for creating new error views.
fn error_view(icon: &str, reason: &str) -> gtk::Container {
    let container = cascade! {
        gtk::Box::new(gtk::Orientation::Horizontal, 24);
        ..set_halign(gtk::Align::Center);
        ..set_valign(gtk::Align::Center);
        ..add(
            &gtk::ImageBuilder::new()
                .icon_name(icon)
                .icon_size(gtk::IconSize::Dialog.into())
                .pixel_size(64)
                .build()
        );
        ..add(&cascade! {
            gtk::LabelBuilder::new()
                .label(reason)
                .wrap(true)
                .xalign(0.0)
                .yalign(0.0)
                .build();
            ..style_context().add_class(&gtk::STYLE_CLASS_DIM_LABEL);
        });
        ..show_all();
    };

    container.upcast::<gtk::Container>()
}
