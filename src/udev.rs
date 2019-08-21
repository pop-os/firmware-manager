use futures::{stream::Stream, Future};
use std::thread;
use stream_cancel::{Trigger, Valved};
use tokio_udev::{Context, Event, EventType, MonitorBuilder};

macro_rules! ok_or_return {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(_) => return None,
        }
    };
}

/// Convenience function for an event loop which reacts to USB hotplug events.
pub fn usb_hotplug_event_loop<F: Fn() + Send + 'static>(func: F) -> Option<Trigger> {
    trace!("initiating USB hotplug event loop thread");

    let context = ok_or_return!(Context::new());
    let mut builder = ok_or_return!(MonitorBuilder::new(&context));
    ok_or_return!(builder.match_subsystem_devtype("usb", "usb_device"));
    let monitor = ok_or_return!(builder.listen());

    let handler = move |e: Event| {
        match e.event_type() {
            EventType::Add | EventType::Remove => func(),
            _ => (),
        }
        Ok(())
    };

    let (triggered, stream) = Valved::new(monitor);

    thread::spawn(move || {
        trace!("USB hotplug events now being processed");
        tokio::run(stream.for_each(handler).map_err(|_| ()));
        trace!("usb hotplug thread stopped");
    });

    Some(triggered)
}
