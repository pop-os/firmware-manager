use firmware_manager_gtk::FirmwareWidget;
use glib::object::ObjectType;
use std::ptr;

#[no_mangle]
pub struct S76FirmwareWidget;

#[no_mangle]
pub extern "C" fn s76_firmware_widget_new() -> *mut S76FirmwareWidget {
    // When used from C, assume that GTK has been initialized.
    unsafe {
        gtk::set_initialized();
    }

    Box::into_raw(Box::new(FirmwareWidget::new())) as *mut S76FirmwareWidget
}

#[no_mangle]
pub extern "C" fn s76_firmware_widget_container(
    ptr: *const S76FirmwareWidget,
) -> *mut gtk_sys::GtkContainer {
    let value = unsafe { (ptr as *const FirmwareWidget).as_ref() };
    value.map_or(ptr::null_mut(), |widget| widget.container().as_ptr())
}

#[no_mangle]
pub extern "C" fn s76_firmware_widget_free(widget: *mut S76FirmwareWidget) {
    unsafe { Box::from_raw(widget as *mut FirmwareWidget) };
}

#[no_mangle]
pub extern "C" fn s76_firmware_widget_scan(ptr: *mut S76FirmwareWidget) -> i32 {
    let value = unsafe { (ptr as *mut FirmwareWidget).as_mut() };

    value.map_or(-1, |widget| {
        widget.scan();
        0
    })
}
