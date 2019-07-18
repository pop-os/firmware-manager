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
            ..add(
                &gtk::ImageBuilder::new()
                    .icon_name("firmware-manager-symbolic")
                    .icon_size(gtk::IconSize::Dialog.into())
                    .pixel_size(64)
                    .build()
            );
            ..add(&cascade! {
                gtk::LabelBuilder::new()
                    .label("Managed Firmware Unavailable\n\nNo devices supporting \
                        automatic firmware updates detected")
                    .wrap(true)
                    .xalign(0.0)
                    .yalign(0.0)
                    .build();
                ..get_style_context().add_class(&gtk::STYLE_CLASS_DIM_LABEL);
            });
        };

        Self { container: container.upcast::<gtk::Container>() }
    }
}
