//! Safe facade over the shared C ABI.

mod binding;
mod decimal;
mod error;
mod handle;
mod native_api;
mod property;
mod runtime;
mod subscription;

pub use binding::{Binding, BindingDirection, BindingError, BindingMode};
pub use decimal::Decimal64;
pub use error::UiError;
pub use handle::{Borrowed, Owned};
pub use property::{Property, PropertyValue};
pub use runtime::{
    ShutdownMode, ShutdownRequest, ShutdownRequestState, ShutdownSnapshot, UiContext, UiRuntime,
};
pub use subscription::Subscription;

pub const STATUS_UNSUPPORTED: u32 = freemvvm_sys::UNSUPPORTED;
