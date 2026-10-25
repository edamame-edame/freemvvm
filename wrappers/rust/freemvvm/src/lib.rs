//! Safe facade over the shared C ABI.

mod binding;
mod command;
mod decimal;
mod error;
mod handle;
mod native_api;
mod property;
mod runtime;
mod subscription;
mod ui;

pub use binding::{Binding, BindingDirection, BindingError, BindingMode};
pub use command::{AsyncCommand, CancellationToken, CommandOperation, CommandState, OperationId};
pub use decimal::Decimal64;
pub use error::UiError;
pub use handle::{Borrowed, Owned};
pub use property::{Property, PropertyValue};
pub use runtime::{
    ShutdownMode, ShutdownRequest, ShutdownRequestState, ShutdownSnapshot, UiContext, UiRuntime,
};
pub use subscription::Subscription;
pub use ui::{
    Button, ChildAlign, CloseDecision, CloseRequest, CloseRequestView, Orientation, StackPanel,
    Text, TextBox, Widget, WidgetRef, Window,
};

pub const STATUS_UNSUPPORTED: u32 = freemvvm_sys::UNSUPPORTED;
