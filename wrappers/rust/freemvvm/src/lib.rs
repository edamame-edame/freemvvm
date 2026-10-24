//! Safe facade over the shared C ABI.

mod error;
mod handle;
mod native_api;
mod runtime;

pub use error::UiError;
pub use handle::{Borrowed, Owned};
pub use runtime::{
    ShutdownMode, ShutdownRequest, ShutdownRequestState, ShutdownSnapshot, UiContext, UiRuntime,
};

pub const STATUS_UNSUPPORTED: u32 = freemvvm_sys::UNSUPPORTED;
