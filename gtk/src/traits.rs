use core::num::NonZeroU8;

pub trait DynamicGtkResize
where
    Self: gtk::traits::WidgetExt,
{
    /// When this widget is resized, the `other` widget will also be resized to the given width and
    /// height percent (1-100);
    ///
    /// This is most useful for dynamically resizing the child of a container to be a certain % of
    /// the parent's dimensions.
    fn dynamic_resize<W: gtk::traits::WidgetExt + 'static>(
        &self,
        other: W,
        width_percent: Option<NonZeroU8>,
        height_percent: Option<NonZeroU8>,
    ) where
        Self: Clone,
    {
        self.connect_size_allocate(move |_, allocation| {
            // The parent widget has not been realized if this value is less than 2.
            // Keep the child hidden until this value is not zero.
            if allocation.width < 2 {
                other.hide();
                return;
            }

            // Calculate the size of the child based on the given percentages.
            let width = width_percent.map_or(-1, |percent| calc_side(allocation.width, percent));
            let height = height_percent.map_or(-1, |percent| calc_side(allocation.height, percent));

            other.show();
            other.set_size_request(width, height);
        });

        // The first invocation of size_allocate will fail, because it has not been realized
        // yet, so this will ensure that we get the correct value after init.
        let parent = self.clone();
        glib::idle_add_local(move || {
            parent.size_allocate(&mut parent.allocation());
            glib::Continue(false)
        });
    }
}

fn calc_side(measurement: i32, percent: NonZeroU8) -> i32 {
    measurement * i32::from(percent.get()) / 100
}

impl<T: gtk::traits::WidgetExt> DynamicGtkResize for T {}
