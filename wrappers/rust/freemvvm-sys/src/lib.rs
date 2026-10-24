//! Raw FFI boundary for the shared dynamic native runtime.
//!
//! This crate deliberately has no dependency on the internal Core crate. The
//! declarations mirror the public versioned C ABI header.

pub type Handle = u64;
pub type Status = u32;
pub type Bool = u8;

pub const OK: Status = 0;
pub const INVALID_ARGUMENT: Status = 1;
pub const INVALID_HANDLE: Status = 2;
pub const WRONG_TYPE: Status = 3;
pub const WRONG_RUNTIME: Status = 4;
pub const WRONG_THREAD: Status = 5;
pub const CLOSED: Status = 6;
pub const BUFFER_TOO_SMALL: Status = 7;
pub const OUT_OF_MEMORY: Status = 8;
pub const INTERNAL_ERROR: Status = 9;
pub const UNSUPPORTED: Status = 10;
pub const PENDING: Status = 14;

pub const SHUTDOWN_ASYNC: u32 = 0;
pub const SHUTDOWN_SYNC: u32 = 1;
pub const SHUTDOWN_REQUEST_PENDING: u32 = 0;
pub const SHUTDOWN_REQUEST_COMPLETED: u32 = 1;
pub const SHUTDOWN_REQUEST_HOST_UNAVAILABLE: u32 = 2;

#[cfg_attr(target_os = "windows", link(name = "freemvvm_abi_1.dll"))]
#[cfg_attr(not(target_os = "windows"), link(name = "freemvvm_abi_1"))]
unsafe extern "C" {
    pub fn ui_abi_1_get_version(
        out_major: *mut u32,
        out_minor: *mut u32,
        out_patch: *mut u32,
    ) -> Status;

    pub fn ui_abi_1_runtime_create(out_runtime: *mut Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_runtime_is_ui_thread(runtime: Handle, out_yes: *mut Bool) -> Status;
    pub fn ui_abi_1_property_create_bool(
        runtime: Handle,
        initial: Bool,
        out_property: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_create_i64(
        runtime: Handle,
        initial: i64,
        out_property: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_create_utf8(
        runtime: Handle,
        bytes: *const u8,
        byte_length: u64,
        out_property: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_create_f64(
        runtime: Handle,
        initial: f64,
        out_property: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_create_decimal(
        runtime: Handle,
        initial_bits: u64,
        out_property: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_get_bool(property: Handle, out_value: *mut Bool) -> Status;
    pub fn ui_abi_1_property_get_i64(property: Handle, out_value: *mut i64) -> Status;
    pub fn ui_abi_1_property_get_utf8(
        property: Handle,
        buffer: *mut u8,
        capacity: u64,
        out_length: *mut u64,
    ) -> Status;
    pub fn ui_abi_1_property_get_f64(property: Handle, out_value: *mut f64) -> Status;
    pub fn ui_abi_1_property_get_decimal(property: Handle, out_bits: *mut u64) -> Status;
    pub fn ui_abi_1_property_set_bool(
        property: Handle,
        value: Bool,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_set_i64(
        property: Handle,
        value: i64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_set_utf8(
        property: Handle,
        bytes: *const u8,
        byte_length: u64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_set_f64(
        property: Handle,
        value: f64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_set_decimal(
        property: Handle,
        value_bits: u64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_runtime_shutdown(runtime: Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_runtime_request_shutdown(
        runtime: Handle,
        mode: u32,
        out_request: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_shutdown_request_query(
        request: Handle,
        out_state: *mut u32,
        out_reason: *mut u32,
        out_error: *mut Handle,
    ) -> Status;

    pub fn ui_abi_1_handle_retain(borrowed: Handle, out_owned: *mut Handle) -> Status;
    pub fn ui_abi_1_handle_release(owned: Handle) -> Status;
    pub fn ui_abi_1_error_status(error: Handle, out_status: *mut Status) -> Status;
    pub fn ui_abi_1_error_code(error: Handle, out_domain: *mut u32, out_code: *mut u32) -> Status;
    pub fn ui_abi_1_error_message_utf8(
        error: Handle,
        buffer: *mut u8,
        capacity: u64,
        out_length: *mut u64,
    ) -> Status;
}
