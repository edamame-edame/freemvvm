//! Internal Rust Core crate. Product implementation starts in subsequent units.
//!
//! Language wrappers must use the versioned C ABI rather than depend on this crate.

mod dispatcher;
mod handle_registry;
mod runtime;
