use gtk::prelude::*;

#[derive(Shrinkwrap)]
pub struct EmptyView {
    #[shrinkwrap(main_field)]
    pub container: gtk::Container,
}

impl EmptyView {
    pub fn new() -> Self {
        let container = cascade! {
            gtk::Box::new(gtk::Orientation::Horizontal, 24);
            ..set_halign(gtk::Align::Center);
            ..set_valign(gtk::Align::Center);
            ..add(&cascade! {
                gtk::Image::new_from_icon_name(
                    "firmware-manager-symbolic".into(),
                    gtk::IconSize::Dialog
                );
                ..set_pixel_size(64);
            });
            ..add(&cascade! {
                gtk::Label::new("Managed Firmware Unavailable\n\nNo devices supporting \
                    automatic firmware updates detected".into());
                ..set_line_wrap(true);
                ..set_xalign(0.0);
                ..set_yalign(0.0);
                ..get_style_context().add_class(&gtk::STYLE_CLASS_DIM_LABEL);
            });
        };

        Self { container: container.upcast::<gtk::Container>() }
    }
}
