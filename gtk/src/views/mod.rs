//! Views are entire sections of a UI in an application consisting of a collection of widgets.

mod devices;
mod empty;

pub use self::{devices::DevicesView, empty::EmptyView};
