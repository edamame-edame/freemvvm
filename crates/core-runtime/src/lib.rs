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

    /// Failure categories needed by the versioned C ABI Runtime query entrypoint.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum RuntimeQueryError {
        InvalidHandle,
        Internal,
    }

    /// Failure categories needed by the versioned C ABI handle retain entrypoint.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum RuntimeRetainError {
        InvalidHandle,
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

    /// Reports whether the current thread is the one captured by this Runtime.
    pub fn runtime_is_ui_thread(handle: u64) -> Result<bool, RuntimeQueryError> {
        let token = HandleToken::from_abi(handle).ok_or(RuntimeQueryError::InvalidHandle)?;
        let runtime = process_runtime_handles()
            .resolve(token)
            .map_err(|error| match error {
                RuntimeHandleError::InvalidToken => RuntimeQueryError::InvalidHandle,
                RuntimeHandleError::TokenUnavailable
                | RuntimeHandleError::OutOfMemory
                | RuntimeHandleError::SynchronizationPoisoned => RuntimeQueryError::Internal,
            })?;
        Ok(runtime.is_ui_thread())
    }

    /// Creates an independent owner token for an existing Runtime handle.
    pub fn runtime_retain_handle(handle: u64) -> Result<u64, RuntimeRetainError> {
        let token = HandleToken::from_abi(handle).ok_or(RuntimeRetainError::InvalidHandle)?;
        process_runtime_handles()
            .retain(token)
            .map(HandleToken::into_abi)
            .map_err(|error| match error {
                RuntimeHandleError::InvalidToken => RuntimeRetainError::InvalidHandle,
                RuntimeHandleError::OutOfMemory => RuntimeRetainError::OutOfMemory,
                RuntimeHandleError::TokenUnavailable
                | RuntimeHandleError::SynchronizationPoisoned => RuntimeRetainError::Internal,
            })
    }
}
