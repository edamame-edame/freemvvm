//! Internal Rust Core crate. Product implementation starts in subsequent units.
//!
//! Language wrappers must use the versioned C ABI rather than depend on this crate.

mod dispatcher;
mod handle_registry;
pub mod numeric;
mod runtime;
mod runtime_handles;

/// Narrow support surface used by the C ABI crate while the Core remains private to wrappers.
#[doc(hidden)]
pub mod abi_support {
    use crate::handle_registry::HandleToken;
    use crate::runtime::RuntimeCore;
    use crate::runtime_handles::{RuntimeHandleError, process_runtime_handles};
    use std::sync::Arc;

    /// Failure categories needed by the versioned C ABI Runtime creation entrypoint.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum RuntimeCreateError {
        OutOfMemory,
        Internal,
    }

    /// Creates one Runtime and registers its owning opaque handle.
    pub fn runtime_create_handle() -> Result<u64, RuntimeCreateError> {
        let runtime = Arc::new(RuntimeCore::new().map_err(|_| RuntimeCreateError::Internal)?);
        process_runtime_handles()
            .insert(&runtime)
            .map(HandleToken::into_abi)
            .map_err(|error| match error {
                RuntimeHandleError::OutOfMemory => RuntimeCreateError::OutOfMemory,
                RuntimeHandleError::InvalidToken
                | RuntimeHandleError::TokenUnavailable
                | RuntimeHandleError::SynchronizationPoisoned => RuntimeCreateError::Internal,
            })
    }
}
