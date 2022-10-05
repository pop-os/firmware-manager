use apply::Apply;
use futures::{
    future::{AbortHandle, Abortable},
    stream::StreamExt,
};
use std::thread;
use tokio_udev::{EventType, MonitorBuilder};

/// Convenience function for an event loop which reacts to USB hotplug events.
pub fn usb_hotplug_event_loop<F: Fn() + Send + 'static>(func: F) -> Option<AbortHandle> {
    trace!("initiating USB hotplug event loop thread");

    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    thread::spawn(move || {
        trace!("USB hotplug events now being processed");

        let _ =
            tokio::runtime::Builder::new_current_thread().enable_io().build().unwrap().block_on(
                async move {
                    let _res = MonitorBuilder::new()
                        .expect("couldn't create monitor builder")
                        .match_subsystem_devtype("usb", "usb_device")
                        .expect("failed to add filter for USB devices")
                        .listen()
                        .expect("couldn't create MonitorSocket")
                        .apply(tokio_udev::AsyncMonitorSocket::new)
                        .expect("couldn't create AsyncMonitorSocket")
                        .for_each(move |event| {
                            if let Ok(event) = event {
                                if let EventType::Add | EventType::Remove = event.event_type() {
                                    func();
                                }
                            }

                            futures::future::ready(())
                        })
                        .apply(|future| Abortable::new(future, abort_registration))
                        .await;
                },
            );

        trace!("usb hotplug thread stopped");
    });

    Some(abort_handle)
}
