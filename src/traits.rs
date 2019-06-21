use core::num::NonZeroU8;

pub trait DynamicGtkResize
where
    Self: gtk::WidgetExt,
{
    /// When this widget is resized, the `other` widget will also be resized to the given width and
    /// height percent (1-100);
    ///
    /// This is most useful for dynamically resizing the child of a container to be a certain % of
    /// the parent's dimensions.
    fn dynamic_resize<W: gtk::WidgetExt + 'static>(
        &self,
        other: W,
        width_percent: Option<NonZeroU8>,
        height_percent: Option<NonZeroU8>,
    ) {
        self.connect_size_allocate(move |_, allocation| {
            let width = width_percent.map_or(-1, |percent| calc_side(allocation.width, percent));
            let height = height_percent.map_or(-1, |percent| calc_side(allocation.height, percent));
            other.set_size_request(width, height);
        });
    }
}

fn calc_side(measurement: i32, percent: NonZeroU8) -> i32 {
    measurement * i32::from(percent.get()) / 100
}

impl<T: gtk::WidgetExt> DynamicGtkResize for T {}
