//! Views are entire sections of a UI in an application consisting of a collection of widgets.

mod devices;
mod error;

pub use self::{
    devices::DevicesView,
    error::{EmptyView, PermissionView},
};
