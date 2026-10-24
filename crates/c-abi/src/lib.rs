//! Versioned C ABI boundary over the internal Core.
//!
//! The public API is under construction and is not distributable yet.

use freemvvm_core::abi_support::{
    BindingApiError, CommandApiError, CommandStartCallback, DestroyCallback, ErrorApiError,
    PostCallback, PropertyApiError, RuntimeCreateError, RuntimeQueryError, RuntimeRetainError,
    ShutdownApiError, ShutdownApiState, ValidateBoolCallback, ValidateI64Callback,
    ValidateUtf8Callback, WakeCallback, binding_cancel as core_binding_cancel,
    binding_create as core_binding_create, binding_error_snapshot as core_binding_error_snapshot,
    binding_subscribe_error as core_binding_subscribe_error,
    command_can_execute_property as core_command_can_execute_property,
    command_cancel as core_command_cancel,
    command_cancel_token_requested as core_command_cancel_token_requested,
    command_close as core_command_close,
    command_complete_cancelled as core_command_complete_cancelled,
    command_complete_failure as core_command_complete_failure,
    command_complete_success as core_command_complete_success,
    command_create_async as core_command_create_async, command_execute as core_command_execute,
    command_failure_snapshot as core_command_failure_snapshot,
    command_state_property as core_command_state_property, error_code as core_error_code,
    error_message_utf8 as core_error_message_utf8, error_status as core_error_status,
    handle_release as core_handle_release,
    issue_validation_error_snapshot as core_issue_validation_error_snapshot,
    property_create_bool as core_property_create_bool,
    property_create_decimal as core_property_create_decimal,
    property_create_f64 as core_property_create_f64,
    property_create_i64 as core_property_create_i64,
    property_create_utf8 as core_property_create_utf8, property_get_bool as core_property_get_bool,
    property_get_decimal as core_property_get_decimal, property_get_f64 as core_property_get_f64,
    property_get_i64 as core_property_get_i64, property_get_utf8 as core_property_get_utf8,
    property_kind as core_property_kind, property_set_bool as core_property_set_bool,
    property_set_decimal as core_property_set_decimal, property_set_f64 as core_property_set_f64,
    property_set_i64 as core_property_set_i64, property_set_utf8 as core_property_set_utf8,
    property_subscribe as core_property_subscribe,
    property_validator_cancel as core_property_validator_cancel,
    property_validator_register_bool as core_property_validator_register_bool,
    property_validator_register_i64 as core_property_validator_register_i64,
    property_validator_register_utf8 as core_property_validator_register_utf8,
    runtime_create_handle, runtime_dispatch_turn, runtime_host_begin_event, runtime_host_end_event,
    runtime_host_stopped, runtime_is_ui_thread as core_runtime_is_ui_thread, runtime_post,
    runtime_request_shutdown, runtime_retain_handle as core_runtime_retain_handle,
    runtime_set_ui_loop_bridge, runtime_shutdown, shutdown_request_query,
    subscription_cancel as core_subscription_cancel,
    validation_error_create as core_validation_error_create,
};
use freemvvm_core::numeric::{NarrowMode, expand_f32_to_f64, narrow_f64_to_f32};
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

const UI_OK: u32 = 0;
const UI_INVALID_ARGUMENT: u32 = 1;
const UI_INVALID_HANDLE: u32 = 2;
const UI_WRONG_TYPE: u32 = 3;
const UI_WRONG_RUNTIME: u32 = 4;
const UI_OUT_OF_MEMORY: u32 = 8;
const UI_INTERNAL_ERROR: u32 = 9;
const UI_WRONG_THREAD: u32 = 5;
const UI_CLOSED: u32 = 6;
const UI_UNSUPPORTED: u32 = 10;
const UI_VALIDATION_FAILED: u32 = 11;
const UI_BUFFER_TOO_SMALL: u32 = 7;
const UI_PENDING: u32 = 14;
const NUMERIC_NARROW_EXACT: u32 = 0;
const NUMERIC_NARROW_NEAREST_EVEN: u32 = 1;
const RUNTIME_VERSION_MAJOR: u32 = 1;
const RUNTIME_VERSION_MINOR: u32 = 0;
const RUNTIME_VERSION_PATCH: u32 = 0;

fn map_shutdown_error(error: ShutdownApiError) -> u32 {
    match error {
        ShutdownApiError::InvalidHandle => UI_INVALID_HANDLE,
        ShutdownApiError::InvalidArgument => UI_INVALID_ARGUMENT,
        ShutdownApiError::WrongThread => UI_WRONG_THREAD,
        ShutdownApiError::Closed => UI_CLOSED,
        ShutdownApiError::HostUnavailable { .. } | ShutdownApiError::Unsupported => UI_UNSUPPORTED,
        ShutdownApiError::OutOfMemory => UI_OUT_OF_MEMORY,
        ShutdownApiError::Internal => UI_INTERNAL_ERROR,
    }
}

fn map_shutdown_state(state: ShutdownApiState) -> (u32, u32) {
    match state {
        ShutdownApiState::Pending => (0, 0),
        ShutdownApiState::Completed => (1, 0),
        ShutdownApiState::HostUnavailable { reason } => (2, reason),
    }
}

fn catch_runtime_creation(
    operation: impl FnOnce() -> Result<u64, RuntimeCreateError>,
) -> Result<u64, u32> {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(handle)) => Ok(handle),
        Ok(Err(RuntimeCreateError::OutOfMemory)) => Err(UI_OUT_OF_MEMORY),
        Ok(Err(RuntimeCreateError::Internal)) | Err(_) => Err(UI_INTERNAL_ERROR),
    }
}

fn catch_runtime_query(
    operation: impl FnOnce() -> Result<bool, RuntimeQueryError>,
) -> Result<bool, u32> {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(is_ui_thread)) => Ok(is_ui_thread),
        Ok(Err(RuntimeQueryError::InvalidHandle)) => Err(UI_INVALID_HANDLE),
        Ok(Err(RuntimeQueryError::Internal)) | Err(_) => Err(UI_INTERNAL_ERROR),
    }
}

fn catch_runtime_retain(
    operation: impl FnOnce() -> Result<u64, RuntimeRetainError>,
) -> Result<u64, u32> {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(handle)) => Ok(handle),
        Ok(Err(RuntimeRetainError::InvalidHandle)) => Err(UI_INVALID_HANDLE),
        Ok(Err(RuntimeRetainError::OutOfMemory)) => Err(UI_OUT_OF_MEMORY),
        Ok(Err(RuntimeRetainError::Internal)) | Err(_) => Err(UI_INTERNAL_ERROR),
    }
}

/// Create a Runtime and return its owned opaque handle.
///
/// # Safety
/// `out_runtime` must be null or point to writable storage for one `u64`. A non-null
/// `out_error` must point to writable storage for one ABI handle (`u64`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_runtime_create(
    out_runtime: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if !out_runtime.is_null() {
        // SAFETY: the caller guarantees that a non-null runtime output is writable.
        unsafe { out_runtime.write(0) };
    }
    if out_runtime.is_null() {
        return UI_INVALID_ARGUMENT;
    }

    match catch_runtime_creation(runtime_create_handle) {
        Ok(handle) => {
            // SAFETY: the required output pointer was checked non-null above.
            unsafe { out_runtime.write(handle) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Report whether the caller is the UI thread captured when this Runtime was created.
///
/// # Safety
/// `out_yes` must be null or point to writable storage for one `u8`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_runtime_is_ui_thread(runtime: u64, out_yes: *mut u8) -> u32 {
    if !out_yes.is_null() {
        // SAFETY: the caller guarantees that a non-null output is writable.
        unsafe { out_yes.write(0) };
    }
    if out_yes.is_null() {
        return UI_INVALID_ARGUMENT;
    }

    match catch_runtime_query(|| core_runtime_is_ui_thread(runtime)) {
        Ok(is_ui_thread) => {
            // SAFETY: the required output pointer was checked non-null above.
            unsafe { out_yes.write(u8::from(is_ui_thread)) };
            UI_OK
        }
        Err(status) => status,
    }
}

fn catch_property_creation(
    operation: impl FnOnce() -> Result<u64, ShutdownApiError>,
) -> Result<u64, u32> {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(handle)) => Ok(handle),
        Ok(Err(error)) => Err(map_shutdown_error(error)),
        Err(_) => Err(UI_INTERNAL_ERROR),
    }
}

fn map_property_error(error: PropertyApiError) -> u32 {
    match error {
        PropertyApiError::InvalidHandle => UI_INVALID_HANDLE,
        PropertyApiError::InvalidArgument => UI_INVALID_ARGUMENT,
        PropertyApiError::WrongType => UI_WRONG_TYPE,
        PropertyApiError::WrongThread => UI_WRONG_THREAD,
        PropertyApiError::Closed => UI_CLOSED,
        PropertyApiError::Unsupported => UI_UNSUPPORTED,
        PropertyApiError::ValidationFailed(_) => UI_VALIDATION_FAILED,
        PropertyApiError::OutOfMemory => UI_OUT_OF_MEMORY,
        PropertyApiError::BufferTooSmall { .. } => UI_BUFFER_TOO_SMALL,
        PropertyApiError::Internal => UI_INTERNAL_ERROR,
    }
}

fn map_binding_error(error: BindingApiError) -> u32 {
    match error {
        BindingApiError::InvalidHandle => UI_INVALID_HANDLE,
        BindingApiError::InvalidArgument => UI_INVALID_ARGUMENT,
        BindingApiError::WrongType => UI_WRONG_TYPE,
        BindingApiError::WrongRuntime => UI_WRONG_RUNTIME,
        BindingApiError::WrongThread => UI_WRONG_THREAD,
        BindingApiError::Closed => UI_CLOSED,
        BindingApiError::Unsupported => UI_UNSUPPORTED,
        BindingApiError::OutOfMemory => UI_OUT_OF_MEMORY,
        BindingApiError::ValidationFailed(_) => UI_VALIDATION_FAILED,
        BindingApiError::Internal => UI_INTERNAL_ERROR,
    }
}

fn map_command_error(error: CommandApiError) -> u32 {
    match error {
        CommandApiError::InvalidHandle => UI_INVALID_HANDLE,
        CommandApiError::InvalidArgument | CommandApiError::InvalidOperation => UI_INVALID_ARGUMENT,
        CommandApiError::WrongType => UI_WRONG_TYPE,
        CommandApiError::WrongRuntime => UI_WRONG_RUNTIME,
        CommandApiError::WrongThread => UI_WRONG_THREAD,
        CommandApiError::Closed => UI_CLOSED,
        CommandApiError::NotExecutable => UI_UNSUPPORTED,
        CommandApiError::OutOfMemory => UI_OUT_OF_MEMORY,
        CommandApiError::Internal => UI_INTERNAL_ERROR,
    }
}

fn map_error_api_error(error: ErrorApiError) -> u32 {
    match error {
        ErrorApiError::InvalidHandle => UI_INVALID_HANDLE,
        ErrorApiError::InvalidArgument => UI_INVALID_ARGUMENT,
        ErrorApiError::WrongThread => UI_WRONG_THREAD,
        ErrorApiError::OutOfMemory => UI_OUT_OF_MEMORY,
        ErrorApiError::BufferTooSmall { .. } => UI_BUFFER_TOO_SMALL,
        ErrorApiError::Internal => UI_INTERNAL_ERROR,
    }
}

fn catch_property_get<T>(
    operation: impl FnOnce() -> Result<T, PropertyApiError>,
) -> Result<T, u32> {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(map_property_error(error)),
        Err(_) => Err(UI_INTERNAL_ERROR),
    }
}

fn catch_property_set(operation: impl FnOnce() -> Result<(), PropertyApiError>) -> (u32, u64) {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => (UI_OK, 0),
        Ok(Err(PropertyApiError::ValidationFailed(snapshot))) => {
            let error_handle = snapshot
                .and_then(|snapshot| core_issue_validation_error_snapshot(snapshot).ok())
                .unwrap_or(0);
            (UI_VALIDATION_FAILED, error_handle)
        }
        Ok(Err(error)) => (map_property_error(error), 0),
        Err(_) => (UI_INTERNAL_ERROR, 0),
    }
}

unsafe fn finish_property_set(result: (u32, u64), out_error: *mut u64) -> u32 {
    let (status, error_handle) = result;
    if !out_error.is_null() {
        // SAFETY: the exported setter's contract requires a writable optional output.
        unsafe { out_error.write(error_handle) };
    } else if error_handle != 0 {
        let _ = core_handle_release(error_handle);
    }
    status
}

/// Create an owned BOOL Property with initial value 0 or 1.
///
/// # Safety
/// `out_property` must be null or writable; a non-null `out_error` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_create_bool(
    runtime: u64,
    initial: u8,
    out_property: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if !out_property.is_null() {
        // SAFETY: the caller guarantees that a non-null property output is writable.
        unsafe { out_property.write(0) };
    }
    if out_property.is_null() || initial > 1 {
        return UI_INVALID_ARGUMENT;
    }
    match catch_property_creation(|| core_property_create_bool(runtime, initial != 0)) {
        Ok(property) => {
            // SAFETY: required output is non-null and writable per the caller contract.
            unsafe { out_property.write(property) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Create an owned I64 Property.
///
/// # Safety
/// `out_property` must be null or writable; a non-null `out_error` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_create_i64(
    runtime: u64,
    initial: i64,
    out_property: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if !out_property.is_null() {
        // SAFETY: the caller guarantees that a non-null property output is writable.
        unsafe { out_property.write(0) };
    }
    if out_property.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_property_creation(|| core_property_create_i64(runtime, initial)) {
        Ok(property) => {
            // SAFETY: required output is non-null and writable per the caller contract.
            unsafe { out_property.write(property) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Create an owned UTF-8 Property by copying its input byte sequence.
///
/// `bytes == NULL` is accepted only for `byte_length == 0`; embedded NUL bytes are preserved.
///
/// # Safety
/// For a non-null `bytes` pointer and positive length, it must reference that many readable
/// bytes. `out_property` must be null or writable; a non-null `out_error` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_create_utf8(
    runtime: u64,
    bytes: *const u8,
    byte_length: u64,
    out_property: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if !out_property.is_null() {
        // SAFETY: the caller guarantees that a non-null property output is writable.
        unsafe { out_property.write(0) };
    }
    if out_property.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let Ok(length) = usize::try_from(byte_length) else {
        return UI_INVALID_ARGUMENT;
    };
    if bytes.is_null() && length != 0 {
        return UI_INVALID_ARGUMENT;
    }
    let initial = if length == 0 {
        &[]
    } else {
        // SAFETY: the caller guarantees that `bytes` references `length` readable bytes.
        unsafe { std::slice::from_raw_parts(bytes, length) }
    };
    match catch_property_creation(|| core_property_create_utf8(runtime, initial)) {
        Ok(property) => {
            // SAFETY: required output is non-null and writable per the caller contract.
            unsafe { out_property.write(property) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Create an owned F64 Property.
///
/// # Safety
/// `out_property` must be null or writable; a non-null `out_error` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_create_f64(
    runtime: u64,
    initial: f64,
    out_property: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if !out_property.is_null() {
        // SAFETY: the caller guarantees that a non-null property output is writable.
        unsafe { out_property.write(0) };
    }
    if out_property.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_property_creation(|| core_property_create_f64(runtime, initial)) {
        Ok(property) => {
            // SAFETY: required output is non-null and writable per the caller contract.
            unsafe { out_property.write(property) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Create an owned DECIMAL Property from its canonical 64-bit representation.
///
/// # Safety
/// `out_property` must be null or writable; a non-null `out_error` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_create_decimal(
    runtime: u64,
    initial_bits: u64,
    out_property: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if !out_property.is_null() {
        // SAFETY: the caller guarantees that a non-null property output is writable.
        unsafe { out_property.write(0) };
    }
    if out_property.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_property_creation(|| core_property_create_decimal(runtime, initial_bits)) {
        Ok(property) => {
            // SAFETY: required output is non-null and writable per the caller contract.
            unsafe { out_property.write(property) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Query the fixed kind of an owned Property. Kind IDs are BOOL=1, I64=2, UTF8=3, F64=5,
/// DECIMAL=6; OBJECT=4 remains reserved.
///
/// # Safety
/// `out_kind` must be null or point to writable storage for one `u32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_kind(property: u64, out_kind: *mut u32) -> u32 {
    if !out_kind.is_null() {
        // SAFETY: the caller guarantees that a non-null kind output is writable.
        unsafe { out_kind.write(0) };
    }
    if out_kind.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| core_property_kind(property))) {
        Ok(Ok(kind)) => {
            // SAFETY: required output is non-null and writable per the caller contract.
            unsafe { out_kind.write(kind) };
            UI_OK
        }
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Read a BOOL Property's current value.
///
/// # Safety
/// `out_value` must be null or writable storage for one `u8`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_get_bool(property: u64, out_value: *mut u8) -> u32 {
    if !out_value.is_null() {
        // SAFETY: the caller guarantees that a non-null scalar output is writable.
        unsafe { out_value.write(0) };
    }
    if out_value.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_property_get(|| core_property_get_bool(property)) {
        Ok(value) => {
            // SAFETY: required output was checked above.
            unsafe { out_value.write(u8::from(value)) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Read an I64 Property's current value.
///
/// # Safety
/// `out_value` must be null or writable storage for one `i64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_get_i64(property: u64, out_value: *mut i64) -> u32 {
    if !out_value.is_null() {
        // SAFETY: the caller guarantees that a non-null scalar output is writable.
        unsafe { out_value.write(0) };
    }
    if out_value.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_property_get(|| core_property_get_i64(property)) {
        Ok(value) => {
            // SAFETY: required output was checked above.
            unsafe { out_value.write(value) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Read UTF-8 bytes from a Property into caller-owned storage.
///
/// `(buffer == NULL, capacity == 0)` queries the required byte count. The result is not
/// NUL-terminated. If the buffer is too small, it remains unchanged and `out_length` receives
/// the required byte count.
///
/// # Safety
/// `out_length` must be null or writable storage for one `u64`. For a non-null `buffer` and
/// positive capacity, `buffer` must be writable for `capacity` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_get_utf8(
    property: u64,
    buffer: *mut u8,
    capacity: u64,
    out_length: *mut u64,
) -> u32 {
    if !out_length.is_null() {
        // SAFETY: the caller guarantees that a non-null length output is writable.
        unsafe { out_length.write(0) };
    }
    if out_length.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let Ok(capacity) = usize::try_from(capacity) else {
        return UI_INVALID_ARGUMENT;
    };
    if buffer.is_null() && capacity != 0 {
        return UI_INVALID_ARGUMENT;
    }
    let mut empty_buffer = [];
    let buffer: Option<&mut [u8]> = if buffer.is_null() {
        None
    } else if capacity == 0 {
        Some(&mut empty_buffer)
    } else {
        // SAFETY: the caller guarantees writable storage for the declared capacity.
        Some(unsafe { std::slice::from_raw_parts_mut(buffer, capacity) })
    };
    match catch_unwind(AssertUnwindSafe(|| {
        core_property_get_utf8(property, buffer)
    })) {
        Ok(Ok(length)) => match u64::try_from(length) {
            Ok(length) => {
                // SAFETY: the required output was checked above.
                unsafe { out_length.write(length) };
                UI_OK
            }
            Err(_) => UI_INTERNAL_ERROR,
        },
        Ok(Err(PropertyApiError::BufferTooSmall { required })) => {
            match u64::try_from(required) {
                Ok(required) => {
                    // SAFETY: the required output was checked above; short-buffer contract
                    // updates only the required length and does not touch caller buffer bytes.
                    unsafe { out_length.write(required) };
                    UI_BUFFER_TOO_SMALL
                }
                Err(_) => UI_INTERNAL_ERROR,
            }
        }
        Ok(Err(error)) => map_property_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Read an F64 Property's current value.
///
/// # Safety
/// `out_value` must be null or writable storage for one `f64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_get_f64(property: u64, out_value: *mut f64) -> u32 {
    if !out_value.is_null() {
        // SAFETY: the caller guarantees that a non-null scalar output is writable.
        unsafe { out_value.write(0.0) };
    }
    if out_value.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_property_get(|| core_property_get_f64(property)) {
        Ok(value) => {
            // SAFETY: required output was checked above.
            unsafe { out_value.write(value) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Read a DECIMAL Property's canonical `uint64_t` representation.
///
/// # Safety
/// `out_bits` must be null or writable storage for one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_get_decimal(property: u64, out_bits: *mut u64) -> u32 {
    if !out_bits.is_null() {
        // SAFETY: the caller guarantees that a non-null scalar output is writable.
        unsafe { out_bits.write(0) };
    }
    if out_bits.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_property_get(|| core_property_get_decimal(property)) {
        Ok(bits) => {
            // SAFETY: required output was checked above.
            unsafe { out_bits.write(bits) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Set a BOOL Property to 0 or 1.
///
/// # Safety
/// A non-null `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_set_bool(
    property: u64,
    value: u8,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller promises that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if value > 1 {
        return UI_INVALID_ARGUMENT;
    }
    // SAFETY: this function forwards the caller's optional writable error output to the helper.
    unsafe {
        finish_property_set(
            catch_property_set(|| core_property_set_bool(property, value != 0)),
            out_error,
        )
    }
}

/// Set an I64 Property.
///
/// # Safety
/// A non-null `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_set_i64(
    property: u64,
    value: i64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller promises that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    // SAFETY: this function forwards the caller's optional writable error output to the helper.
    unsafe {
        finish_property_set(
            catch_property_set(|| core_property_set_i64(property, value)),
            out_error,
        )
    }
}

/// Set a UTF-8 Property by copying the specified bytes.
///
/// # Safety
/// For positive `byte_length`, `bytes` must point to readable storage for that many bytes. A
/// non-null `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_set_utf8(
    property: u64,
    bytes: *const u8,
    byte_length: u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller promises that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    let Ok(length) = usize::try_from(byte_length) else {
        return UI_INVALID_ARGUMENT;
    };
    if length > 0 && bytes.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let input = if length == 0 {
        &[][..]
    } else {
        // SAFETY: the caller promises readable storage for byte_length bytes; conversion above
        // guarantees the length fits this platform's slice representation.
        unsafe { std::slice::from_raw_parts(bytes, length) }
    };
    // SAFETY: the input slice is valid for this call and the optional error output is writable.
    unsafe {
        finish_property_set(
            catch_property_set(|| core_property_set_utf8(property, input)),
            out_error,
        )
    }
}

/// Set an F64 Property.
///
/// # Safety
/// A non-null `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_set_f64(
    property: u64,
    value: f64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller promises that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    // SAFETY: this function forwards the caller's optional writable error output to the helper.
    unsafe {
        finish_property_set(
            catch_property_set(|| core_property_set_f64(property, value)),
            out_error,
        )
    }
}

/// Set a DECIMAL Property from its canonical `uint64_t` representation.
///
/// # Safety
/// A non-null `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_set_decimal(
    property: u64,
    bits: u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller promises that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    // SAFETY: this function forwards the caller's optional writable error output to the helper.
    unsafe {
        finish_property_set(
            catch_property_set(|| core_property_set_decimal(property, bits)),
            out_error,
        )
    }
}

/// Create an immutable validation Error from inside a Property Validator callback.
///
/// # Safety
/// For positive length, `message` must point to readable UTF-8 bytes. `out_error` must be
/// writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_validation_error_create(
    domain: u32,
    code: u32,
    message: *const u8,
    message_byte_length: u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees a non-null output is writable.
        unsafe { out_error.write(0) };
    }
    if out_error.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let Ok(length) = usize::try_from(message_byte_length) else {
        return UI_INVALID_ARGUMENT;
    };
    if length > 0 && message.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let bytes = if length == 0 {
        &[][..]
    } else {
        // SAFETY: caller promises `length` readable bytes; conversion above fits usize.
        unsafe { std::slice::from_raw_parts(message, length) }
    };
    match catch_unwind(AssertUnwindSafe(|| {
        core_validation_error_create(domain, code, bytes)
    })) {
        Ok(Ok(error)) => {
            // SAFETY: required output was checked above.
            unsafe { out_error.write(error) };
            UI_OK
        }
        Ok(Err(error)) => map_error_api_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Return the stable status stored in an immutable Error.
///
/// # Safety
/// `out_status` must be null or writable storage for one `u32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_error_status(error: u64, out_status: *mut u32) -> u32 {
    if !out_status.is_null() {
        // SAFETY: caller guarantees non-null outputs are writable.
        unsafe { out_status.write(0) };
    }
    if out_status.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| core_error_status(error))) {
        Ok(Ok(status)) => {
            // SAFETY: required output was checked above.
            unsafe { out_status.write(status) };
            UI_OK
        }
        Ok(Err(error)) => map_error_api_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Return the domain and code stored in an immutable Error.
///
/// # Safety
/// Each output must be null or writable storage for one `u32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_error_code(
    error: u64,
    out_domain: *mut u32,
    out_code: *mut u32,
) -> u32 {
    if !out_domain.is_null() {
        // SAFETY: caller guarantees non-null outputs are writable.
        unsafe { out_domain.write(0) };
    }
    if !out_code.is_null() {
        // SAFETY: caller guarantees non-null outputs are writable.
        unsafe { out_code.write(0) };
    }
    if out_domain.is_null() || out_code.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| core_error_code(error))) {
        Ok(Ok((domain, code))) => {
            // SAFETY: required outputs were checked above.
            unsafe {
                out_domain.write(domain);
                out_code.write(code);
            }
            UI_OK
        }
        Ok(Err(error)) => map_error_api_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Copy the immutable Error message into a caller-owned UTF-8 buffer.
///
/// # Safety
/// For positive capacity, `buffer` must point to writable storage for that many bytes.
/// `out_length` must be writable storage for one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_error_message_utf8(
    error: u64,
    buffer: *mut u8,
    capacity: u64,
    out_length: *mut u64,
) -> u32 {
    if !out_length.is_null() {
        // SAFETY: caller guarantees non-null output is writable.
        unsafe { out_length.write(0) };
    }
    if out_length.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let Ok(capacity) = usize::try_from(capacity) else {
        return UI_INVALID_ARGUMENT;
    };
    if capacity > 0 && buffer.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let output = if capacity == 0 {
        None
    } else {
        // SAFETY: caller promises writable storage for capacity bytes.
        Some(unsafe { std::slice::from_raw_parts_mut(buffer, capacity) })
    };
    match catch_unwind(AssertUnwindSafe(|| core_error_message_utf8(error, output))) {
        Ok(Ok(length)) => {
            // SAFETY: required output was checked above.
            unsafe { out_length.write(length as u64) };
            UI_OK
        }
        Ok(Err(ErrorApiError::BufferTooSmall { required })) => {
            // SAFETY: required output was checked above.
            unsafe { out_length.write(required as u64) };
            UI_BUFFER_TOO_SMALL
        }
        Ok(Err(error)) => map_error_api_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

fn catch_validator_register(
    operation: impl FnOnce() -> Result<u64, PropertyApiError>,
) -> Result<u64, u32> {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(handle)) => Ok(handle),
        Ok(Err(error)) => Err(map_property_error(error)),
        Err(_) => Err(UI_INTERNAL_ERROR),
    }
}

/// Register a synchronous BOOL Property Validator.
///
/// # Safety
/// `out_registration` must be writable. A non-null `out_error` must be writable. Callback and
/// destroy must follow the ABI lifetime contract; ownership transfers only on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_validator_register_bool(
    property: u64,
    callback: Option<ValidateBoolCallback>,
    userdata: *mut c_void,
    destroy: Option<DestroyCallback>,
    out_registration: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_registration.is_null() {
        // SAFETY: caller guarantees non-null output is writable.
        unsafe { out_registration.write(0) };
    }
    if !out_error.is_null() {
        // SAFETY: caller guarantees non-null output is writable.
        unsafe { out_error.write(0) };
    }
    if out_registration.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let (Some(callback), Some(destroy)) = (callback, destroy) else {
        return UI_INVALID_ARGUMENT;
    };
    match catch_validator_register(|| {
        core_property_validator_register_bool(property, callback, userdata as usize, destroy)
    }) {
        Ok(registration) => {
            // SAFETY: required output was checked above.
            unsafe { out_registration.write(registration) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Register a synchronous I64 Property Validator.
///
/// # Safety
/// See `ui_abi_1_property_validator_register_bool`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_validator_register_i64(
    property: u64,
    callback: Option<ValidateI64Callback>,
    userdata: *mut c_void,
    destroy: Option<DestroyCallback>,
    out_registration: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_registration.is_null() {
        // SAFETY: caller guarantees non-null output is writable.
        unsafe { out_registration.write(0) };
    }
    if !out_error.is_null() {
        // SAFETY: caller guarantees non-null output is writable.
        unsafe { out_error.write(0) };
    }
    if out_registration.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let (Some(callback), Some(destroy)) = (callback, destroy) else {
        return UI_INVALID_ARGUMENT;
    };
    match catch_validator_register(|| {
        core_property_validator_register_i64(property, callback, userdata as usize, destroy)
    }) {
        Ok(registration) => {
            // SAFETY: required output was checked above.
            unsafe { out_registration.write(registration) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Register a synchronous UTF8 Property Validator.
///
/// # Safety
/// See `ui_abi_1_property_validator_register_bool`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_validator_register_utf8(
    property: u64,
    callback: Option<ValidateUtf8Callback>,
    userdata: *mut c_void,
    destroy: Option<DestroyCallback>,
    out_registration: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_registration.is_null() {
        // SAFETY: caller guarantees non-null output is writable.
        unsafe { out_registration.write(0) };
    }
    if !out_error.is_null() {
        // SAFETY: caller guarantees non-null output is writable.
        unsafe { out_error.write(0) };
    }
    if out_registration.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let (Some(callback), Some(destroy)) = (callback, destroy) else {
        return UI_INVALID_ARGUMENT;
    };
    match catch_validator_register(|| {
        core_property_validator_register_utf8(property, callback, userdata as usize, destroy)
    }) {
        Ok(registration) => {
            // SAFETY: required output was checked above.
            unsafe { out_registration.write(registration) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Explicitly cancel a Property Validator registration without releasing its handle.
///
/// # Safety
/// A non-null `out_error` must be writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_validator_cancel(
    registration: u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: caller guarantees non-null output is writable.
        unsafe { out_error.write(0) };
    }
    match catch_unwind(AssertUnwindSafe(|| {
        core_property_validator_cancel(registration)
    })) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_property_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Subscribe to changes of one Property.
///
/// Ownership of `userdata` and `destroy` transfers only when this function succeeds.
///
/// # Safety
/// `out_subscription` must be null or writable storage for one `u64`. A non-null
/// `out_error` must be writable storage for one ABI handle. Callback and destroy must obey the
/// C ABI and remain valid until destroy runs; userdata must remain valid for that duration.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_property_subscribe(
    property: u64,
    callback: Option<PostCallback>,
    userdata: *mut c_void,
    destroy: Option<DestroyCallback>,
    out_subscription: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if !out_subscription.is_null() {
        // SAFETY: the caller guarantees a non-null subscription output is writable.
        unsafe { out_subscription.write(0) };
    }
    if out_subscription.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let Some(callback) = callback else {
        return UI_INVALID_ARGUMENT;
    };
    let Some(destroy) = destroy else {
        return UI_INVALID_ARGUMENT;
    };

    match catch_unwind(AssertUnwindSafe(|| {
        core_property_subscribe(property, callback, userdata as usize, destroy)
    })) {
        Ok(Ok(subscription)) => {
            // SAFETY: the required output pointer was checked non-null above.
            unsafe { out_subscription.write(subscription) };
            UI_OK
        }
        Ok(Err(error)) => map_property_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Explicitly cancel a Property Subscription without releasing its owned handle.
///
/// # Safety
/// A non-null `out_error` must be writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_subscription_cancel(
    subscription: u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    match catch_unwind(AssertUnwindSafe(|| core_subscription_cancel(subscription))) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_property_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Create a one-time, one-way, or two-way Binding between same-kind Properties.
///
/// Mode values are 1 (one-time), 2 (one-way), and 3 (two-way). One-time success returns a zero
/// Binding handle; persistent modes return an owned handle. Initial validation failures return an
/// owned immutable Error through `out_error` when that output is provided.
///
/// # Safety
/// `out_binding` must point to writable storage for one `u64`; a non-null `out_error` must be
/// writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_binding_create(
    source_property: u64,
    target_property: u64,
    mode: u32,
    out_binding: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if !out_binding.is_null() {
        // SAFETY: the caller guarantees a non-null binding output is writable.
        unsafe { out_binding.write(0) };
    }
    if out_binding.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        core_binding_create(source_property, target_property, mode)
    })) {
        Ok(Ok(binding)) => {
            // SAFETY: required output is non-null and writable per the caller contract.
            unsafe { out_binding.write(binding) };
            UI_OK
        }
        Ok(Err(BindingApiError::ValidationFailed(snapshot))) => {
            let error_handle = snapshot
                .and_then(|snapshot| core_issue_validation_error_snapshot(snapshot).ok())
                .unwrap_or(0);
            if !out_error.is_null() {
                // SAFETY: the caller guarantees the non-null optional error output is writable.
                unsafe { out_error.write(error_handle) };
            } else {
                let _ = core_handle_release(error_handle);
            }
            UI_VALIDATION_FAILED
        }
        Ok(Err(error)) => map_binding_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Explicitly cancel a Binding without releasing its owned handle.
///
/// # Safety
/// A non-null `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_binding_cancel(binding: u64, out_error: *mut u64) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    match catch_unwind(AssertUnwindSafe(|| core_binding_cancel(binding))) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_binding_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Reads the current immutable Binding validation error and returns newly owned Error and
/// rejected-Property handles. A successful call with no current error returns three zero values.
///
/// # Safety
/// All three output pointers are required and must point to writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_binding_error_snapshot(
    binding: u64,
    out_owned_error: *mut u64,
    out_owned_rejected_property: *mut u64,
    out_direction: *mut u32,
) -> u32 {
    if !out_owned_error.is_null() {
        // SAFETY: the ABI caller supplies writable storage for every non-null output pointer.
        unsafe { out_owned_error.write(0) };
    }
    if !out_owned_rejected_property.is_null() {
        // SAFETY: the ABI caller supplies writable storage for every non-null output pointer.
        unsafe { out_owned_rejected_property.write(0) };
    }
    if !out_direction.is_null() {
        // SAFETY: the ABI caller supplies writable storage for every non-null output pointer.
        unsafe { out_direction.write(0) };
    }
    if out_owned_error.is_null() || out_owned_rejected_property.is_null() || out_direction.is_null()
    {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| core_binding_error_snapshot(binding))) {
        Ok(Ok((error, property, direction))) => {
            // SAFETY: all required output pointers were checked non-null above.
            unsafe {
                out_owned_error.write(error);
                out_owned_rejected_property.write(property);
                out_direction.write(direction);
            }
            UI_OK
        }
        Ok(Err(error)) => map_binding_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Subscribes to changes in the current Binding validation error. No initial callback is made.
///
/// # Safety
/// Callback and destroy must be valid function pointers. `userdata` remains owned by the
/// registration until `destroy` runs. `out_error` is optional; `out_subscription` is required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_binding_subscribe_error(
    binding: u64,
    callback: Option<PostCallback>,
    userdata: *mut c_void,
    destroy: Option<DestroyCallback>,
    out_subscription: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_subscription.is_null() {
        // SAFETY: the ABI caller supplies writable storage for every non-null output pointer.
        unsafe { out_subscription.write(0) };
    }
    if !out_error.is_null() {
        // SAFETY: the ABI caller supplies writable storage for every non-null output pointer.
        unsafe { out_error.write(0) };
    }
    if out_subscription.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    let (Some(callback), Some(destroy)) = (callback, destroy) else {
        return UI_INVALID_ARGUMENT;
    };
    match catch_unwind(AssertUnwindSafe(|| {
        core_binding_subscribe_error(binding, callback, userdata as usize, destroy)
    })) {
        Ok(Ok(subscription)) => {
            // SAFETY: the required output pointer was checked non-null above.
            unsafe { out_subscription.write(subscription) };
            UI_OK
        }
        Ok(Err(error)) => map_binding_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Register the Runtime's one-time UI-loop wake bridge.
///
/// # Safety
/// `wake` must be a valid callback. `userdata` must remain valid until `destroy` runs
/// (or until the caller handles a failed registration). The callbacks must be thread-safe;
/// `wake` may be called from any thread and must only schedule a future UI-loop turn.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_runtime_set_ui_loop_bridge(
    runtime: u64,
    wake: Option<WakeCallback>,
    userdata: *mut c_void,
    destroy: Option<DestroyCallback>,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    let Some(wake) = wake else {
        return UI_INVALID_ARGUMENT;
    };
    match catch_unwind(AssertUnwindSafe(|| {
        runtime_set_ui_loop_bridge(runtime, wake, userdata as usize, destroy)
    })) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Request Runtime shutdown and return an owned receipt for its completion.
///
/// `mode` is 0 for ASYNC or 1 for SYNC. `UI_PENDING` means accepted but not complete.
///
/// # Safety
/// `out_request` must point to writable storage for one ABI handle. A non-null
/// `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_runtime_request_shutdown(
    runtime: u64,
    mode: u32,
    out_request: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if !out_request.is_null() {
        // SAFETY: the caller guarantees that a non-null request output is writable.
        unsafe { out_request.write(0) };
    }
    if out_request.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| runtime_request_shutdown(runtime, mode))) {
        Ok(Ok((request, state))) => {
            // SAFETY: required output was checked above.
            unsafe { out_request.write(request) };
            match state {
                ShutdownApiState::Pending | ShutdownApiState::HostUnavailable { .. } => UI_PENDING,
                ShutdownApiState::Completed => UI_OK,
            }
        }
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Query an owned shutdown receipt from any thread.
///
/// State is 0 Pending, 1 Completed, or 2 HostUnavailable. `out_reason` is nonzero
/// only for HostUnavailable.
///
/// # Safety
/// `out_state` and `out_reason` must point to writable storage for one `u32` each.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_shutdown_request_query(
    request: u64,
    out_state: *mut u32,
    out_reason: *mut u32,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if !out_state.is_null() {
        // SAFETY: the caller guarantees that a non-null state output is writable.
        unsafe { out_state.write(0) };
    }
    if !out_reason.is_null() {
        // SAFETY: the caller guarantees that a non-null reason output is writable.
        unsafe { out_reason.write(0) };
    }
    if out_state.is_null() || out_reason.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| shutdown_request_query(request))) {
        Ok(Ok(state)) => {
            let (state, reason) = map_shutdown_state(state);
            // SAFETY: required outputs were checked above.
            unsafe {
                out_state.write(state);
                out_reason.write(reason);
            }
            UI_OK
        }
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Begin a host event scope. Pair the returned scope ID with `runtime_host_end_event`.
///
/// # Safety
/// `out_scope_id` must point to writable storage for one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_runtime_host_begin_event(
    runtime: u64,
    out_scope_id: *mut u64,
) -> u32 {
    if !out_scope_id.is_null() {
        // SAFETY: the caller guarantees that a non-null scope output is writable.
        unsafe { out_scope_id.write(0) };
    }
    if out_scope_id.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| runtime_host_begin_event(runtime))) {
        Ok(Ok(scope_id)) => {
            // SAFETY: required output was checked above.
            unsafe { out_scope_id.write(scope_id) };
            UI_OK
        }
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// End a host event scope and service accepted shutdown at its outer boundary.
///
/// # Safety
/// The Runtime handle and scope ID are opaque scalar values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_runtime_host_end_event(runtime: u64, scope_id: u64) -> u32 {
    match catch_unwind(AssertUnwindSafe(|| {
        runtime_host_end_event(runtime, scope_id)
    })) {
        Ok(Ok(_)) => UI_OK,
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Notify Runtime that its host loop has stopped; this does not mark cleanup complete.
///
/// # Safety
/// A non-null `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_runtime_host_stopped(
    runtime: u64,
    reason: u32,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    match catch_unwind(AssertUnwindSafe(|| runtime_host_stopped(runtime, reason))) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Enqueue one callback for a future UI-thread dispatch turn.
///
/// # Safety
/// `callback` and `destroy` must be valid non-unwinding callbacks. On success, `userdata`
/// remains valid until `destroy` is called. On failure, ownership remains with the caller.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_runtime_post(
    runtime: u64,
    callback: Option<PostCallback>,
    userdata: *mut c_void,
    destroy: Option<DestroyCallback>,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    let (Some(callback), Some(destroy)) = (callback, destroy) else {
        return UI_INVALID_ARGUMENT;
    };
    match catch_unwind(AssertUnwindSafe(|| {
        runtime_post(runtime, callback, userdata as usize, destroy)
    })) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Run at most one queued callback on the Runtime UI thread.
///
/// # Safety
/// The Runtime handle is an opaque scalar. The host must call this only from a future
/// UI-loop turn after its wake callback has returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_runtime_dispatch_turn(runtime: u64) -> u32 {
    match catch_unwind(AssertUnwindSafe(|| runtime_dispatch_turn(runtime))) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Complete shutdown synchronously when called from a worker, or return UI_PENDING
/// when called from the active UI event.
///
/// # Safety
/// A non-null `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_runtime_shutdown(runtime: u64, out_error: *mut u64) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    match catch_unwind(AssertUnwindSafe(|| runtime_shutdown(runtime))) {
        Ok(Ok(ShutdownApiState::Pending)) => UI_PENDING,
        Ok(Ok(ShutdownApiState::Completed)) => UI_OK,
        Ok(Ok(ShutdownApiState::HostUnavailable { .. })) => UI_UNSUPPORTED,
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Create one asynchronous Command. Callback ownership transfers only on success.
///
/// # Safety
/// `start` and `destroy` must be non-null valid function pointers, `userdata` must remain valid
/// until destroy, `out_command` must be writable, and a non-null `out_error` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_command_create_async(
    runtime: u64,
    eligibility_bool_or_zero: u64,
    start: Option<CommandStartCallback>,
    userdata: *mut c_void,
    destroy: Option<DestroyCallback>,
    out_command: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_command.is_null() {
        // SAFETY: a non-null output points to writable storage by the ABI contract.
        unsafe { out_command.write(0) };
    }
    if !out_error.is_null() {
        // SAFETY: a non-null output points to writable storage by the ABI contract.
        unsafe { out_error.write(0) };
    }
    if out_command.is_null() || start.is_none() || destroy.is_none() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        core_command_create_async(
            runtime,
            eligibility_bool_or_zero,
            start.expect("validated callback"),
            userdata as usize,
            destroy.expect("validated destroy"),
        )
    })) {
        Ok(Ok(command)) => {
            // SAFETY: required output was checked non-null above.
            unsafe { out_command.write(command) };
            UI_OK
        }
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Execute one Command operation and return its Runtime-unique operation ID.
///
/// # Safety
/// `out_operation_id` must point to writable storage for one `u64`; a non-null `out_error` must
/// point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_command_execute(
    command: u64,
    out_operation_id: *mut u64,
    out_error: *mut u64,
) -> u32 {
    if !out_operation_id.is_null() {
        // SAFETY: a non-null output points to writable storage by the ABI contract.
        unsafe { out_operation_id.write(0) };
    }
    if !out_error.is_null() {
        // SAFETY: a non-null output points to writable storage by the ABI contract.
        unsafe { out_error.write(0) };
    }
    if out_operation_id.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| core_command_execute(command))) {
        Ok(Ok(operation_id)) => {
            // SAFETY: required output was checked non-null above.
            unsafe { out_operation_id.write(operation_id) };
            UI_OK
        }
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Request cooperative cancellation for the identified operation.
///
/// # Safety
/// A non-null `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_command_cancel(
    command: u64,
    operation_id: u64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: a non-null output points to writable storage by the ABI contract.
        unsafe { out_error.write(0) };
    }
    match catch_unwind(AssertUnwindSafe(|| {
        core_command_cancel(command, operation_id)
    })) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Close a Command without waiting for its worker.
///
/// # Safety
/// A non-null `out_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_command_close(command: u64, out_error: *mut u64) -> u32 {
    if !out_error.is_null() {
        // SAFETY: a non-null output points to writable storage by the ABI contract.
        unsafe { out_error.write(0) };
    }
    match catch_unwind(AssertUnwindSafe(|| core_command_close(command))) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Return an independently owned read-only I64 state Property.
///
/// # Safety
/// `out_owned_property` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_command_state_property(
    command: u64,
    out_owned_property: *mut u64,
) -> u32 {
    if !out_owned_property.is_null() {
        // SAFETY: a non-null output points to writable storage by the ABI contract.
        unsafe { out_owned_property.write(0) };
    }
    if out_owned_property.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| core_command_state_property(command))) {
        Ok(Ok(property)) => {
            // SAFETY: required output was checked non-null above.
            unsafe { out_owned_property.write(property) };
            UI_OK
        }
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Return an independently owned read-only BOOL can-execute Property.
///
/// # Safety
/// `out_owned_property` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_command_can_execute_property(
    command: u64,
    out_owned_property: *mut u64,
) -> u32 {
    if !out_owned_property.is_null() {
        // SAFETY: a non-null output points to writable storage by the ABI contract.
        unsafe { out_owned_property.write(0) };
    }
    if out_owned_property.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        core_command_can_execute_property(command)
    })) {
        Ok(Ok(property)) => {
            // SAFETY: required output was checked non-null above.
            unsafe { out_owned_property.write(property) };
            UI_OK
        }
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Return the current immutable Command failure snapshot or zero when none is available.
///
/// # Safety
/// `out_owned_error` must point to writable storage for one ABI handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_command_failure_snapshot(
    command: u64,
    out_owned_error: *mut u64,
) -> u32 {
    if !out_owned_error.is_null() {
        // SAFETY: a non-null output points to writable storage by the ABI contract.
        unsafe { out_owned_error.write(0) };
    }
    if out_owned_error.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| core_command_failure_snapshot(command))) {
        Ok(Ok(error)) => {
            // SAFETY: required output was checked non-null above.
            unsafe { out_owned_error.write(error) };
            UI_OK
        }
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Read the thread-safe cancellation request flag from any worker thread.
///
/// # Safety
/// `out_yes` must point to writable storage for one `ui_bool`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_command_cancel_token_requested(
    token: u64,
    out_yes: *mut u8,
) -> u32 {
    if !out_yes.is_null() {
        // SAFETY: a non-null output points to writable storage by the ABI contract.
        unsafe { out_yes.write(0) };
    }
    if out_yes.is_null() {
        return UI_INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        core_command_cancel_token_requested(token)
    })) {
        Ok(Ok(requested)) => {
            // SAFETY: required output was checked non-null above.
            unsafe { out_yes.write(u8::from(requested)) };
            UI_OK
        }
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Report successful completion from a UI-thread post callback.
#[unsafe(no_mangle)]
pub extern "C" fn ui_abi_1_command_complete_success(command: u64, operation_id: u64) -> u32 {
    match catch_unwind(AssertUnwindSafe(|| {
        core_command_complete_success(command, operation_id)
    })) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Report a business failure from a UI-thread post callback.
///
/// # Safety
/// When `message_byte_length` is non-zero, `message` must point to readable storage of that size.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_command_complete_failure_utf8(
    command: u64,
    operation_id: u64,
    domain: u32,
    code: u32,
    message: *const u8,
    message_byte_length: u64,
) -> u32 {
    if message_byte_length > usize::MAX as u64 || (message_byte_length != 0 && message.is_null()) {
        return UI_INVALID_ARGUMENT;
    }
    // SAFETY: the ABI caller supplies readable bytes for the declared length.
    let message = if message_byte_length == 0 {
        &[]
    } else {
        // SAFETY: validated non-null and usize-sized above; caller guarantees the declared bytes
        // are readable for this call.
        unsafe { std::slice::from_raw_parts(message, message_byte_length as usize) }
    };
    match catch_unwind(AssertUnwindSafe(|| {
        core_command_complete_failure(command, operation_id, domain, code, message)
    })) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Report cooperative cancellation after the worker confirms it stopped.
#[unsafe(no_mangle)]
pub extern "C" fn ui_abi_1_command_complete_cancelled(command: u64, operation_id: u64) -> u32 {
    match catch_unwind(AssertUnwindSafe(|| {
        core_command_complete_cancelled(command, operation_id)
    })) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_command_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Release one owned Runtime, Command, Property, or shutdown-request handle.
#[unsafe(no_mangle)]
pub extern "C" fn ui_abi_1_handle_release(owned: u64) -> u32 {
    match catch_unwind(AssertUnwindSafe(|| core_handle_release(owned))) {
        Ok(Ok(())) => UI_OK,
        Ok(Err(error)) => map_shutdown_error(error),
        Err(_) => UI_INTERNAL_ERROR,
    }
}

/// Create a new owned token for the same object while keeping the borrowed token valid.
///
/// # Safety
/// `out_owned` must be null or point to writable storage for one ABI handle (`u64`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_handle_retain(borrowed: u64, out_owned: *mut u64) -> u32 {
    if !out_owned.is_null() {
        // SAFETY: the caller guarantees that a non-null owned output is writable.
        unsafe { out_owned.write(0) };
    }
    if out_owned.is_null() {
        return UI_INVALID_ARGUMENT;
    }

    match catch_runtime_retain(|| core_runtime_retain_handle(borrowed)) {
        Ok(handle) => {
            // SAFETY: the required output pointer was checked non-null above.
            unsafe { out_owned.write(handle) };
            UI_OK
        }
        Err(status) => status,
    }
}

/// Report the stable native runtime version supported by this ABI.
///
/// # Safety
/// Each non-null output pointer must point to writable storage for one `u32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_get_version(
    out_major: *mut u32,
    out_minor: *mut u32,
    out_patch: *mut u32,
) -> u32 {
    if !out_major.is_null() {
        // SAFETY: the caller guarantees that every non-null output points to writable `u32` storage.
        unsafe { out_major.write(0) };
    }
    if !out_minor.is_null() {
        // SAFETY: the caller guarantees that every non-null output points to writable `u32` storage.
        unsafe { out_minor.write(0) };
    }
    if !out_patch.is_null() {
        // SAFETY: the caller guarantees that every non-null output points to writable `u32` storage.
        unsafe { out_patch.write(0) };
    }

    if out_major.is_null() || out_minor.is_null() || out_patch.is_null() {
        return UI_INVALID_ARGUMENT;
    }

    // SAFETY: all required outputs were checked non-null above and the caller guarantees writable storage.
    unsafe {
        out_major.write(RUNTIME_VERSION_MAJOR);
        out_minor.write(RUNTIME_VERSION_MINOR);
        out_patch.write(RUNTIME_VERSION_PATCH);
    }

    UI_OK
}

/// Expand an F32 value to F64 without losing finite precision.
///
/// # Safety
/// `out_value` must be null or point to writable storage for one `f64`. A non-null
/// `out_error` must point to writable storage for one ABI handle (`u64`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_numeric_expand_f32(
    value: f32,
    out_value: *mut f64,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if out_value.is_null() {
        return UI_INVALID_ARGUMENT;
    }

    // SAFETY: the caller guarantees that a non-null required output is writable.
    unsafe { out_value.write(0.0) };
    let converted = expand_f32_to_f64(value);
    // SAFETY: the required output pointer was checked non-null above.
    unsafe { out_value.write(converted) };
    UI_OK
}

/// Narrow an F64 value to F32 using exact or nearest-even precision.
///
/// `mode` is 0 for exact conversion or 1 for nearest-even rounding.
///
/// # Safety
/// `out_value` must be null or point to writable storage for one `f32`. A non-null
/// `out_error` must point to writable storage for one ABI handle (`u64`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_numeric_narrow_f64(
    value: f64,
    mode: u32,
    out_value: *mut f32,
    out_error: *mut u64,
) -> u32 {
    if !out_error.is_null() {
        // SAFETY: the caller guarantees that a non-null optional error output is writable.
        unsafe { out_error.write(0) };
    }
    if out_value.is_null() {
        return UI_INVALID_ARGUMENT;
    }

    // SAFETY: the caller guarantees that the required output pointer is writable.
    unsafe { out_value.write(0.0) };
    let mode = match mode {
        NUMERIC_NARROW_EXACT => NarrowMode::Exact,
        NUMERIC_NARROW_NEAREST_EVEN => NarrowMode::NearestEven,
        _ => return UI_INVALID_ARGUMENT,
    };

    let Ok(converted) = narrow_f64_to_f32(value, mode) else {
        return UI_INVALID_ARGUMENT;
    };
    // SAFETY: the required output pointer was checked non-null above.
    unsafe { out_value.write(converted) };
    UI_OK
}

/// Report whether an ABI feature is available in this library build.
///
/// No feature is advertised until its complete public contract is implemented.
///
/// # Safety
/// `out_supported` must be null or point to writable storage for one `u8`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ui_abi_1_has_feature(feature_id: u32, out_supported: *mut u8) -> u32 {
    if out_supported.is_null() {
        return UI_INVALID_ARGUMENT;
    }

    // SAFETY: the caller guarantees that a non-null output points to writable `u8` storage.
    unsafe { out_supported.write(0) };

    if feature_id == 0 {
        return UI_INVALID_ARGUMENT;
    }

    if feature_id == 3 || feature_id == 7 {
        // Feature 3 is numeric conversion; feature 7 is Runtime post/dispatch.
        // SAFETY: the caller guarantees that the non-null output points to writable `u8` storage.
        unsafe { out_supported.write(1) };
    }

    UI_OK
}

#[cfg(test)]
mod tests {
    use super::{
        RuntimeRetainError, UI_BUFFER_TOO_SMALL, UI_INTERNAL_ERROR, UI_INVALID_ARGUMENT,
        UI_INVALID_HANDLE, UI_OK, UI_OUT_OF_MEMORY, UI_PENDING, UI_UNSUPPORTED,
        UI_VALIDATION_FAILED, UI_WRONG_RUNTIME, UI_WRONG_THREAD, UI_WRONG_TYPE,
        catch_runtime_creation, catch_runtime_query, catch_runtime_retain, ui_abi_1_binding_cancel,
        ui_abi_1_binding_create, ui_abi_1_binding_error_snapshot, ui_abi_1_binding_subscribe_error,
        ui_abi_1_command_can_execute_property, ui_abi_1_command_cancel,
        ui_abi_1_command_cancel_token_requested, ui_abi_1_command_close,
        ui_abi_1_command_complete_cancelled, ui_abi_1_command_create_async,
        ui_abi_1_command_execute, ui_abi_1_command_failure_snapshot,
        ui_abi_1_command_state_property, ui_abi_1_error_code, ui_abi_1_error_message_utf8,
        ui_abi_1_error_status, ui_abi_1_get_version, ui_abi_1_handle_release,
        ui_abi_1_handle_retain, ui_abi_1_has_feature, ui_abi_1_numeric_expand_f32,
        ui_abi_1_numeric_narrow_f64, ui_abi_1_property_create_bool,
        ui_abi_1_property_create_decimal, ui_abi_1_property_create_f64,
        ui_abi_1_property_create_i64, ui_abi_1_property_create_utf8, ui_abi_1_property_get_bool,
        ui_abi_1_property_get_decimal, ui_abi_1_property_get_f64, ui_abi_1_property_get_i64,
        ui_abi_1_property_get_utf8, ui_abi_1_property_kind, ui_abi_1_property_set_bool,
        ui_abi_1_property_set_decimal, ui_abi_1_property_set_f64, ui_abi_1_property_set_i64,
        ui_abi_1_property_set_utf8, ui_abi_1_property_subscribe,
        ui_abi_1_property_validator_cancel, ui_abi_1_property_validator_register_bool,
        ui_abi_1_property_validator_register_i64, ui_abi_1_property_validator_register_utf8,
        ui_abi_1_runtime_create, ui_abi_1_runtime_dispatch_turn, ui_abi_1_runtime_host_begin_event,
        ui_abi_1_runtime_host_end_event, ui_abi_1_runtime_host_stopped,
        ui_abi_1_runtime_is_ui_thread, ui_abi_1_runtime_post, ui_abi_1_runtime_request_shutdown,
        ui_abi_1_runtime_set_ui_loop_bridge, ui_abi_1_runtime_shutdown,
        ui_abi_1_shutdown_request_query, ui_abi_1_subscription_cancel,
        ui_abi_1_validation_error_create,
    };
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use std::sync::{Mutex, mpsc};

    struct BridgeProbe {
        priorities: Mutex<Vec<u32>>,
        destroyed: AtomicUsize,
    }

    #[derive(Default)]
    struct ValidatorProbe {
        calls: AtomicUsize,
        destroys: AtomicUsize,
    }

    #[derive(Default)]
    struct CommandProbe {
        starts: AtomicUsize,
        destroys: AtomicUsize,
        operation_id: AtomicU64,
        retained_command: AtomicU64,
        retained_cancel: AtomicU64,
    }

    unsafe extern "C" fn command_start(
        userdata: *mut c_void,
        borrowed_command: u64,
        operation_id: u64,
        borrowed_cancel: u64,
    ) -> u32 {
        // SAFETY: test keeps the probe alive until Command shutdown and handle release.
        let probe = unsafe { &*userdata.cast::<CommandProbe>() };
        let mut retained_command = 0;
        // SAFETY: both output addresses are writable and callback handles are borrowed/live here.
        let command_status =
            unsafe { ui_abi_1_handle_retain(borrowed_command, &mut retained_command) };
        if command_status != UI_OK {
            return command_status;
        }
        let mut retained_cancel = 0;
        // SAFETY: both output addresses are writable and callback handles are borrowed/live here.
        let cancel_status =
            unsafe { ui_abi_1_handle_retain(borrowed_cancel, &mut retained_cancel) };
        if cancel_status != UI_OK {
            let _ = ui_abi_1_handle_release(retained_command);
            return cancel_status;
        }
        probe.operation_id.store(operation_id, Ordering::SeqCst);
        probe
            .retained_command
            .store(retained_command, Ordering::SeqCst);
        probe
            .retained_cancel
            .store(retained_cancel, Ordering::SeqCst);
        probe.starts.fetch_add(1, Ordering::SeqCst);
        UI_OK
    }

    unsafe extern "C" fn command_destroy(userdata: *mut c_void) {
        // SAFETY: test keeps the probe alive through Runtime shutdown.
        unsafe { &*userdata.cast::<CommandProbe>() }
            .destroys
            .fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn command_start_failure(
        userdata: *mut c_void,
        _command: u64,
        operation_id: u64,
        _cancel_token: u64,
    ) -> u32 {
        // SAFETY: test keeps the probe alive until Command shutdown and handle release.
        let probe = unsafe { &*userdata.cast::<CommandProbe>() };
        probe.operation_id.store(operation_id, Ordering::SeqCst);
        if probe.starts.fetch_add(1, Ordering::SeqCst) == 0 {
            UI_INTERNAL_ERROR
        } else {
            UI_OK
        }
    }

    unsafe extern "C" fn reject_negative_i64(
        userdata: *mut c_void,
        candidate: i64,
        out_error: *mut u64,
    ) -> u32 {
        // SAFETY: test keeps this probe alive until after validator cancellation.
        let probe = unsafe { &*userdata.cast::<ValidatorProbe>() };
        probe.calls.fetch_add(1, Ordering::SeqCst);
        if candidate >= 0 {
            return UI_OK;
        }
        let message = b"negative values are rejected";
        // SAFETY: callback scope permits validation error creation and output is writable.
        let status = unsafe {
            ui_abi_1_validation_error_create(
                41,
                9,
                message.as_ptr(),
                message.len() as u64,
                out_error,
            )
        };
        if status == UI_OK {
            UI_VALIDATION_FAILED
        } else {
            UI_INTERNAL_ERROR
        }
    }

    unsafe extern "C" fn validator_destroy(userdata: *mut c_void) {
        // SAFETY: test keeps this probe alive until after validator cancellation.
        let probe = unsafe { &*userdata.cast::<ValidatorProbe>() };
        probe.destroys.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn accept_bool(
        userdata: *mut c_void,
        _candidate: u8,
        _out_error: *mut u64,
    ) -> u32 {
        // SAFETY: the test keeps its probe alive until cancellation.
        unsafe { &*userdata.cast::<ValidatorProbe>() }
            .calls
            .fetch_add(1, Ordering::SeqCst);
        UI_OK
    }

    unsafe extern "C" fn accept_utf8(
        userdata: *mut c_void,
        _candidate: *const u8,
        _length: u64,
        _out_error: *mut u64,
    ) -> u32 {
        // SAFETY: the test keeps its probe alive until cancellation.
        unsafe { &*userdata.cast::<ValidatorProbe>() }
            .calls
            .fetch_add(1, Ordering::SeqCst);
        UI_OK
    }

    #[derive(Default)]
    struct PostProbe {
        callbacks: AtomicUsize,
        destroys: AtomicUsize,
    }

    #[derive(Default)]
    struct BindingErrorProbe {
        callbacks: AtomicUsize,
        destroys: AtomicUsize,
        order: Mutex<Vec<&'static str>>,
    }

    unsafe extern "C" fn binding_error_callback(userdata: *mut c_void) {
        // SAFETY: tests keep the probe alive until the Subscription destroy callback finishes.
        let probe = unsafe { &*(userdata.cast::<BindingErrorProbe>()) };
        probe.callbacks.fetch_add(1, Ordering::SeqCst);
        probe.order.lock().expect("order lock").push("error");
    }

    unsafe extern "C" fn binding_error_destroy(userdata: *mut c_void) {
        // SAFETY: tests keep the probe alive until the Subscription destroy callback finishes.
        let probe = unsafe { &*(userdata.cast::<BindingErrorProbe>()) };
        probe.destroys.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn binding_source_callback(userdata: *mut c_void) {
        // SAFETY: tests keep the probe alive until the Property Subscription is destroyed.
        let probe = unsafe { &*(userdata.cast::<BindingErrorProbe>()) };
        probe.order.lock().expect("order lock").push("property");
    }

    #[derive(Default)]
    struct SelfCancelProbe {
        callbacks: AtomicUsize,
        destroys: AtomicUsize,
        subscription: AtomicU64,
        cancel_status: AtomicUsize,
        destroy_threads: Mutex<Vec<std::thread::ThreadId>>,
    }

    unsafe extern "C" fn record_post_callback(userdata: *mut c_void) {
        // SAFETY: the test keeps a live PostProbe at the transferred userdata address.
        let probe = unsafe { &*(userdata.cast::<PostProbe>()) };
        probe.callbacks.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn record_post_destroy(userdata: *mut c_void) {
        // SAFETY: the test keeps a live PostProbe at the transferred userdata address.
        let probe = unsafe { &*(userdata.cast::<PostProbe>()) };
        probe.destroys.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn cancel_self_callback(userdata: *mut c_void) {
        // SAFETY: the test keeps SelfCancelProbe alive through the destroy callback.
        let probe = unsafe { &*(userdata.cast::<SelfCancelProbe>()) };
        probe.callbacks.fetch_add(1, Ordering::SeqCst);
        let mut error = u64::MAX;
        // SAFETY: callback cancellation is part of the approved reentrant subscription contract.
        let status = unsafe {
            ui_abi_1_subscription_cancel(probe.subscription.load(Ordering::SeqCst), &mut error)
        };
        probe.cancel_status.store(status as usize, Ordering::SeqCst);
    }

    unsafe extern "C" fn cancel_self_destroy(userdata: *mut c_void) {
        // SAFETY: the test keeps SelfCancelProbe alive through the Runtime shutdown.
        let probe = unsafe { &*(userdata.cast::<SelfCancelProbe>()) };
        probe.destroys.fetch_add(1, Ordering::SeqCst);
        probe
            .destroy_threads
            .lock()
            .expect("destroy thread log lock")
            .push(std::thread::current().id());
    }

    unsafe extern "C" fn record_wake(userdata: *mut c_void, priority: u32) {
        // SAFETY: tests pass a live BridgeProbe pointer and keep it alive through destroy.
        let probe = unsafe { &*(userdata.cast::<BridgeProbe>()) };
        probe
            .priorities
            .lock()
            .expect("probe lock is healthy")
            .push(priority);
    }

    unsafe extern "C" fn record_destroy(userdata: *mut c_void) {
        // SAFETY: tests pass a live BridgeProbe pointer and keep it alive through destroy.
        let probe = unsafe { &*(userdata.cast::<BridgeProbe>()) };
        probe.destroyed.fetch_add(1, Ordering::SeqCst);
    }

    fn complete_runtime_shutdown_at_host_boundary(runtime: u64) {
        let mut scope = 0;
        // SAFETY: scope is writable and Runtime is driven on its creator thread.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_runtime_host_begin_event(runtime, &mut scope) },
            UI_OK
        );
        let mut request = 0;
        // SAFETY: request output is writable; the accepted async request is completed below.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_runtime_request_shutdown(runtime, 0, &mut request, std::ptr::null_mut())
            },
            UI_PENDING
        );
        // SAFETY: scope was returned by the matching begin call.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_runtime_host_end_event(runtime, scope) },
            UI_OK
        );
        assert_eq!(ui_abi_1_handle_release(request), UI_OK);
    }

    unsafe extern "C" fn send_wake(userdata: *mut c_void, _priority: u32) {
        // SAFETY: tests pass a live Sender pointer and keep it alive through the worker request.
        let sender = unsafe { &*(userdata.cast::<mpsc::Sender<()>>()) };
        let _ = sender.send(());
    }

    #[test]
    fn numeric_expand_f32_returns_exact_f64_and_clears_optional_error() {
        let mut output = f64::NAN;
        let mut error = u64::MAX;
        // SAFETY: both output pointers refer to writable storage.
        let status =
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_numeric_expand_f32(-f32::MIN_POSITIVE, &mut output, &mut error) };

        assert_eq!(status, UI_OK);
        assert_eq!(output, -f32::MIN_POSITIVE as f64);
        assert_eq!(error, 0);
    }

    #[test]
    fn runtime_create_returns_distinct_owned_handles_and_clears_optional_error() {
        let mut first = u64::MAX;
        let mut first_error = u64::MAX;
        // SAFETY: both outputs refer to writable storage.
        let first_status = unsafe { ui_abi_1_runtime_create(&mut first, &mut first_error) };
        let mut second = u64::MAX;
        // SAFETY: the required output refers to writable storage and null optional error is allowed.
        let second_status = unsafe { ui_abi_1_runtime_create(&mut second, std::ptr::null_mut()) };

        assert_eq!(first_status, UI_OK);
        assert_ne!(first, 0);
        assert_eq!(first_error, 0);
        assert_eq!(second_status, UI_OK);
        assert_ne!(second, 0);
        assert_ne!(first, second);
    }

    #[test]
    fn command_executes_cancels_and_exposes_read_only_state_properties() {
        let mut runtime = 0;
        assert_eq!(
            // SAFETY: runtime output points to writable storage.
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let bridge = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        assert_eq!(
            // SAFETY: bridge callback state remains alive until after Runtime shutdown.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&bridge as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );

        let mut eligibility = 0;
        // SAFETY: output points to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe {
                ui_abi_1_property_create_bool(runtime, 1, &mut eligibility, std::ptr::null_mut())
            },
            UI_OK
        );
        let probe = CommandProbe::default();
        let mut command = 0;
        // SAFETY: callback probe remains live through shutdown and outputs are writable.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe {
                ui_abi_1_command_create_async(
                    runtime,
                    eligibility,
                    Some(command_start),
                    (&probe as *const CommandProbe).cast_mut().cast(),
                    Some(command_destroy),
                    &mut command,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );

        let mut state_property = 0;
        let mut can_execute_property = 0;
        // SAFETY: both output handles point to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_command_state_property(command, &mut state_property) },
            UI_OK
        );
        // SAFETY: both output handles point to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_command_can_execute_property(command, &mut can_execute_property) },
            UI_OK
        );
        let mut state = -1;
        let mut can_execute = 0;
        // SAFETY: getters write to valid scalar outputs.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_property_get_i64(state_property, &mut state) },
            UI_OK
        );
        // SAFETY: getters write to valid scalar outputs.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_property_get_bool(can_execute_property, &mut can_execute) },
            UI_OK
        );
        assert_eq!((state, can_execute), (0, 1));
        let mut writable_property = 0;
        assert_eq!(
            // SAFETY: output handle points to writable storage.
            unsafe {
                ui_abi_1_property_create_i64(
                    runtime,
                    0,
                    &mut writable_property,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut rejected_binding = u64::MAX;
        assert_eq!(
            // SAFETY: output handles are writable; state Property cannot be a Binding target.
            unsafe {
                ui_abi_1_binding_create(
                    writable_property,
                    state_property,
                    2,
                    &mut rejected_binding,
                    std::ptr::null_mut(),
                )
            },
            UI_UNSUPPORTED
        );
        assert_eq!(rejected_binding, 0);
        assert_eq!(ui_abi_1_handle_release(writable_property), UI_OK);
        // SAFETY: the valid BOOL Property handle and optional error output are used synchronously.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_property_set_bool(eligibility, 0, std::ptr::null_mut()) },
            UI_OK
        );
        // SAFETY: output points to writable BOOL storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_property_get_bool(can_execute_property, &mut can_execute) },
            UI_OK
        );
        assert_eq!(can_execute, 0);
        let mut disabled_operation = u64::MAX;
        // SAFETY: output points to writable operation ID storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe {
                ui_abi_1_command_execute(command, &mut disabled_operation, std::ptr::null_mut())
            },
            UI_UNSUPPORTED
        );
        assert_eq!(disabled_operation, 0);
        // SAFETY: the valid BOOL Property handle and optional error output are used synchronously.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_property_set_bool(eligibility, 1, std::ptr::null_mut()) },
            UI_OK
        );
        // SAFETY: output points to writable BOOL storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_property_get_bool(can_execute_property, &mut can_execute) },
            UI_OK
        );
        assert_eq!(can_execute, 1);
        // Public setters cannot modify the Core-owned operation state.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_property_set_i64(state_property, 5, std::ptr::null_mut()) },
            UI_UNSUPPORTED
        );

        let mut operation_id = 0;
        // SAFETY: operation ID output is writable.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_command_execute(command, &mut operation_id, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(operation_id, 1);
        assert_eq!(probe.starts.load(Ordering::SeqCst), 1);
        assert_eq!(probe.operation_id.load(Ordering::SeqCst), operation_id);
        let retained_command = probe.retained_command.load(Ordering::SeqCst);
        assert_ne!(retained_command, 0);
        // Releasing the public handle leaves the Runtime-owned active Command intact.
        assert_eq!(ui_abi_1_handle_release(command), UI_OK);
        // SAFETY: getters write to valid scalar outputs.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_property_get_i64(state_property, &mut state) },
            UI_OK
        );
        // SAFETY: getters write to valid scalar outputs.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_property_get_bool(can_execute_property, &mut can_execute) },
            UI_OK
        );
        assert_eq!((state, can_execute), (1, 0));

        let mut rejected_operation = u64::MAX;
        // SAFETY: output is writable; the running command rejects reentry.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe {
                ui_abi_1_command_execute(
                    retained_command,
                    &mut rejected_operation,
                    std::ptr::null_mut(),
                )
            },
            UI_UNSUPPORTED
        );
        assert_eq!(rejected_operation, 0);

        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe {
                ui_abi_1_command_cancel(retained_command, operation_id, std::ptr::null_mut())
            },
            UI_OK
        );
        let token = probe.retained_cancel.load(Ordering::SeqCst);
        assert_ne!(token, 0);
        let query_token = token;
        assert!(
            std::thread::spawn(move || {
                let mut requested = 0;
                // SAFETY: output is writable and retained cancel-token handle is thread-safe.
                assert_eq!(
                    // SAFETY: this test supplies live handles and valid writable output pointers.
                    unsafe { ui_abi_1_command_cancel_token_requested(query_token, &mut requested) },
                    UI_OK
                );
                requested == 1
            })
            .join()
            .expect("worker cancellation query completes")
        );

        assert_eq!(
            ui_abi_1_command_complete_cancelled(
                probe.retained_command.load(Ordering::SeqCst),
                operation_id
            ),
            UI_OK
        );
        // SAFETY: getters write to valid scalar outputs.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_property_get_i64(state_property, &mut state) },
            UI_OK
        );
        // SAFETY: getters write to valid scalar outputs.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_property_get_bool(can_execute_property, &mut can_execute) },
            UI_OK
        );
        assert_eq!((state, can_execute), (5, 1));
        let mut failure = u64::MAX;
        // SAFETY: output handle points to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_command_failure_snapshot(retained_command, &mut failure) },
            UI_OK
        );
        assert_eq!(failure, 0);
        assert_eq!(
            // SAFETY: this test supplies live handles and valid writable output pointers.
            unsafe { ui_abi_1_command_close(retained_command, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);

        let second_probe = CommandProbe::default();
        let mut second_command = 0;
        assert_eq!(
            // SAFETY: second callback state remains live until Runtime shutdown.
            unsafe {
                ui_abi_1_command_create_async(
                    runtime,
                    0,
                    Some(command_start),
                    (&second_probe as *const CommandProbe).cast_mut().cast(),
                    Some(command_destroy),
                    &mut second_command,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut second_operation = 0;
        assert_eq!(
            // SAFETY: second operation output points to writable storage.
            unsafe {
                ui_abi_1_command_execute(
                    second_command,
                    &mut second_operation,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        assert_eq!(second_operation, 2);
        assert_eq!(
            ui_abi_1_command_complete_cancelled(
                second_probe.retained_command.load(Ordering::SeqCst),
                second_operation,
            ),
            UI_OK
        );
        assert_eq!(
            // SAFETY: command is live and null is permitted for optional error output.
            unsafe { ui_abi_1_command_close(second_command, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(second_probe.destroys.load(Ordering::SeqCst), 1);

        complete_runtime_shutdown_at_host_boundary(runtime);
        assert_eq!(ui_abi_1_handle_release(state_property), UI_OK);
        assert_eq!(ui_abi_1_handle_release(can_execute_property), UI_OK);
        assert_eq!(ui_abi_1_handle_release(eligibility), UI_OK);
        assert_eq!(
            ui_abi_1_handle_release(probe.retained_command.load(Ordering::SeqCst)),
            UI_OK
        );
        assert_eq!(
            ui_abi_1_handle_release(probe.retained_cancel.load(Ordering::SeqCst)),
            UI_OK
        );
        assert_eq!(ui_abi_1_handle_release(second_command), UI_OK);
        assert_eq!(
            ui_abi_1_handle_release(second_probe.retained_command.load(Ordering::SeqCst)),
            UI_OK
        );
        assert_eq!(
            ui_abi_1_handle_release(second_probe.retained_cancel.load(Ordering::SeqCst)),
            UI_OK
        );
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }
    #[test]
    fn runtime_post_transfers_ownership_and_dispatches_one_callback_per_turn() {
        let mut runtime = 0;
        assert_eq!(
            // SAFETY: runtime output points to writable storage.
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let bridge = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        assert_eq!(
            // SAFETY: bridge userdata remains valid and callback functions obey the ABI contract.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&bridge as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let first = PostProbe::default();
        let second = PostProbe::default();
        assert_eq!(
            // SAFETY: post userdata remains valid until the owned destroy callback runs.
            unsafe {
                ui_abi_1_runtime_post(
                    runtime,
                    Some(record_post_callback),
                    (&first as *const PostProbe).cast_mut().cast(),
                    Some(record_post_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let runtime_token = runtime;
        let second_userdata = (&second as *const PostProbe) as usize;
        let worker_status = std::thread::spawn(move || {
            // SAFETY: second remains live until the joined worker and later UI dispatch finish.
            unsafe {
                ui_abi_1_runtime_post(
                    runtime_token,
                    Some(record_post_callback),
                    second_userdata as *mut c_void,
                    Some(record_post_destroy),
                    std::ptr::null_mut(),
                )
            }
        })
        .join()
        .expect("worker post completes");
        assert_eq!(worker_status, UI_OK);
        assert_eq!(first.callbacks.load(Ordering::SeqCst), 0);
        assert_eq!(first.destroys.load(Ordering::SeqCst), 0);
        assert_eq!(
            bridge.priorities.lock().expect("bridge lock").as_slice(),
            [1]
        );

        assert_eq!(
            // SAFETY: dispatch runs on the Runtime creator thread.
            unsafe { ui_abi_1_runtime_dispatch_turn(runtime) },
            UI_OK
        );
        assert_eq!(first.callbacks.load(Ordering::SeqCst), 1);
        assert_eq!(first.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(second.callbacks.load(Ordering::SeqCst), 0);
        assert_eq!(second.destroys.load(Ordering::SeqCst), 0);
        assert_eq!(
            // SAFETY: dispatch runs on the Runtime creator thread.
            unsafe { ui_abi_1_runtime_dispatch_turn(runtime) },
            UI_OK
        );
        assert_eq!(second.callbacks.load(Ordering::SeqCst), 1);
        assert_eq!(second.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(bridge.destroyed.load(Ordering::SeqCst), 0);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn command_start_failure_is_an_accepted_operation_and_snapshot_is_immutable() {
        let mut runtime = 0;
        // SAFETY: output points to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let bridge = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        // SAFETY: bridge state remains live through Runtime shutdown.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&bridge as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let probe = CommandProbe::default();
        let mut command = 0;
        // SAFETY: userdata remains live until shutdown and output is writable.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe {
                ui_abi_1_command_create_async(
                    runtime,
                    0,
                    Some(command_start_failure),
                    (&probe as *const CommandProbe).cast_mut().cast(),
                    Some(command_destroy),
                    &mut command,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut state_property = 0;
        let mut can_execute_property = 0;
        // SAFETY: output handles point to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_command_state_property(command, &mut state_property) },
            UI_OK
        );
        // SAFETY: output handles point to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_command_can_execute_property(command, &mut can_execute_property) },
            UI_OK
        );

        let mut operation = 0;
        // SAFETY: output operation ID points to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_command_execute(command, &mut operation, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(operation, 1);
        let mut state = 0;
        // SAFETY: output points to writable scalar storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_property_get_i64(state_property, &mut state) },
            UI_OK
        );
        assert_eq!(state, 4);
        let mut failure = 0;
        // SAFETY: output handle points to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_command_failure_snapshot(command, &mut failure) },
            UI_OK
        );
        assert_ne!(failure, 0);
        let mut status = 0;
        // SAFETY: output status points to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_error_status(failure, &mut status) },
            UI_OK
        );
        assert_eq!(status, 12);
        let mut domain = 0;
        let mut code = 0;
        // SAFETY: both output scalars point to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_error_code(failure, &mut domain, &mut code) },
            UI_OK
        );
        assert_eq!((domain, code), (0, 0));

        let mut second_operation = 0;
        // SAFETY: output operation ID points to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe {
                ui_abi_1_command_execute(command, &mut second_operation, std::ptr::null_mut())
            },
            UI_OK
        );
        assert_eq!(second_operation, 2);
        let mut current_failure = u64::MAX;
        // SAFETY: output handle points to writable storage.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_command_failure_snapshot(command, &mut current_failure) },
            UI_OK
        );
        assert_eq!(current_failure, 0);
        let mut preserved_status = 0;
        // SAFETY: previously returned owned Error remains valid and output is writable.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_error_status(failure, &mut preserved_status) },
            UI_OK
        );
        assert_eq!(preserved_status, 12);
        assert_eq!(
            ui_abi_1_command_complete_cancelled(command, second_operation),
            UI_OK
        );
        assert_eq!(ui_abi_1_handle_release(failure), UI_OK);
        // SAFETY: command handle is live and null is permitted for optional error output.
        assert_eq!(
            // SAFETY: this test supplies live handles and writable outputs.
            unsafe { ui_abi_1_command_close(command, std::ptr::null_mut()) },
            UI_OK
        );

        complete_runtime_shutdown_at_host_boundary(runtime);
        assert_eq!(ui_abi_1_handle_release(command), UI_OK);
        assert_eq!(ui_abi_1_handle_release(state_property), UI_OK);
        assert_eq!(ui_abi_1_handle_release(can_execute_property), UI_OK);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn runtime_post_rejection_and_shutdown_cancellation_preserve_ownership_rules() {
        let mut runtime = 0;
        assert_eq!(
            // SAFETY: runtime output points to writable storage.
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let no_bridge = PostProbe::default();
        let mut error = u64::MAX;
        assert_eq!(
            // SAFETY: the live userdata pointer is valid; the call rejects due to missing bridge.
            unsafe {
                ui_abi_1_runtime_post(
                    runtime,
                    Some(record_post_callback),
                    (&no_bridge as *const PostProbe).cast_mut().cast(),
                    Some(record_post_destroy),
                    &mut error,
                )
            },
            UI_UNSUPPORTED
        );
        assert_eq!(error, 0);
        assert_eq!(no_bridge.callbacks.load(Ordering::SeqCst), 0);
        assert_eq!(no_bridge.destroys.load(Ordering::SeqCst), 0);
        assert_eq!(
            // SAFETY: null callback is validated before userdata is accessed.
            unsafe {
                ui_abi_1_runtime_post(
                    runtime,
                    None,
                    std::ptr::null_mut(),
                    Some(record_post_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_ARGUMENT
        );

        let bridge = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        assert_eq!(
            // SAFETY: bridge userdata remains valid and callback functions obey the ABI contract.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&bridge as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let cancelled = PostProbe::default();
        assert_eq!(
            // SAFETY: post userdata remains valid through shutdown cancellation and destroy.
            unsafe {
                ui_abi_1_runtime_post(
                    runtime,
                    Some(record_post_callback),
                    (&cancelled as *const PostProbe).cast_mut().cast(),
                    Some(record_post_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut scope = 0;
        assert_eq!(
            // SAFETY: scope points to writable storage.
            unsafe { ui_abi_1_runtime_host_begin_event(runtime, &mut scope) },
            UI_OK
        );
        let mut request = 0;
        assert_eq!(
            // SAFETY: request points to writable storage; active host scope yields pending.
            unsafe {
                ui_abi_1_runtime_request_shutdown(runtime, 0, &mut request, std::ptr::null_mut())
            },
            UI_PENDING
        );
        assert_eq!(
            // SAFETY: dispatch runs on the Runtime creator thread.
            unsafe { ui_abi_1_runtime_dispatch_turn(runtime) },
            UI_OK
        );
        assert_eq!(cancelled.callbacks.load(Ordering::SeqCst), 0);
        assert_eq!(cancelled.destroys.load(Ordering::SeqCst), 0);
        assert_eq!(
            // SAFETY: scope ID was returned by this Runtime and has not yet been closed.
            unsafe { ui_abi_1_runtime_host_end_event(runtime, scope) },
            UI_OK
        );
        assert_eq!(cancelled.callbacks.load(Ordering::SeqCst), 0);
        assert_eq!(cancelled.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(bridge.destroyed.load(Ordering::SeqCst), 1);
        assert_eq!(ui_abi_1_handle_release(request), UI_OK);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn runtime_create_rejects_null_required_output_and_clears_optional_error() {
        let mut error = u64::MAX;
        // SAFETY: null required output is handled and the optional error output is writable.
        let status = unsafe { ui_abi_1_runtime_create(std::ptr::null_mut(), &mut error) };

        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(error, 0);
    }

    #[test]
    fn runtime_create_catches_core_panics_as_internal_error() {
        let result = catch_runtime_creation(|| panic!("injected Runtime creation panic"));

        assert_eq!(result, Err(UI_INTERNAL_ERROR));
    }

    #[test]
    fn runtime_is_ui_thread_reports_creator_and_worker_threads() {
        let mut runtime = 0;
        // SAFETY: `runtime` is writable output storage and the optional error output is null.
        let create_status = unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) };
        assert_eq!(create_status, UI_OK);

        let mut is_ui_thread = u8::MAX;
        // SAFETY: `is_ui_thread` is writable output storage.
        let status = unsafe { ui_abi_1_runtime_is_ui_thread(runtime, &mut is_ui_thread) };
        assert_eq!(status, UI_OK);
        assert_eq!(is_ui_thread, 1);

        let worker_result = std::thread::spawn(move || {
            let mut is_ui_thread = u8::MAX;
            // SAFETY: the output is writable, and the Runtime handle was created and remains registered.
            let status = unsafe { ui_abi_1_runtime_is_ui_thread(runtime, &mut is_ui_thread) };
            (status, is_ui_thread)
        })
        .join()
        .expect("worker query completes");
        assert_eq!(worker_result, (UI_OK, 0));
    }

    #[test]
    fn runtime_is_ui_thread_rejects_invalid_handles_and_null_output() {
        for runtime in [0, u64::MAX] {
            let mut is_ui_thread = u8::MAX;
            // SAFETY: `is_ui_thread` is writable output storage.
            let status = unsafe { ui_abi_1_runtime_is_ui_thread(runtime, &mut is_ui_thread) };
            assert_eq!(status, UI_INVALID_HANDLE);
            assert_eq!(is_ui_thread, 0);
        }

        // SAFETY: a null required output is handled before dereferencing.
        let status = unsafe { ui_abi_1_runtime_is_ui_thread(0, std::ptr::null_mut()) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
    }

    #[test]
    fn runtime_is_ui_thread_catches_core_panics_as_internal_error() {
        let result = catch_runtime_query(|| panic!("injected Runtime query panic"));

        assert_eq!(result, Err(UI_INTERNAL_ERROR));
    }

    #[test]
    fn runtime_shutdown_request_is_completed_at_outer_host_boundary() {
        let mut runtime = 0;
        // SAFETY: runtime output is writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let probe = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        // SAFETY: probe remains live until after host-end invokes destroy.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&probe as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        // Registration transfers callback ownership only once and only from the UI thread.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&probe as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_ARGUMENT
        );
        let worker_runtime = runtime;
        let worker_probe = &probe as *const BridgeProbe as usize;
        assert_eq!(
            std::thread::spawn(move || {
                // SAFETY: probe remains live and worker registration is rejected before transfer.
                unsafe {
                    ui_abi_1_runtime_set_ui_loop_bridge(
                        worker_runtime,
                        Some(record_wake),
                        worker_probe as *mut c_void,
                        Some(record_destroy),
                        std::ptr::null_mut(),
                    )
                }
            })
            .join()
            .expect("worker registration returns"),
            UI_WRONG_THREAD
        );
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    u64::MAX,
                    Some(record_wake),
                    std::ptr::null_mut::<c_void>(),
                    None,
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_HANDLE
        );
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    None,
                    std::ptr::null_mut(),
                    None,
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_ARGUMENT
        );

        let mut scope = 0;
        // SAFETY: scope output is writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_host_begin_event(runtime, &mut scope) },
            UI_OK
        );
        let mut request = u64::MAX;
        // SAFETY: request output is writable and optional error is null.
        let status = unsafe {
            ui_abi_1_runtime_request_shutdown(runtime, 0, &mut request, std::ptr::null_mut())
        };
        assert_eq!(status, UI_PENDING);
        assert_ne!(request, 0);
        assert_eq!(
            *probe.priorities.lock().expect("probe lock is healthy"),
            [2]
        );

        // SAFETY: the scope ID was returned by the matching begin call.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_host_end_event(runtime, scope) },
            UI_OK
        );
        assert_eq!(probe.destroyed.load(Ordering::SeqCst), 1);

        let mut state = u32::MAX;
        let mut reason = u32::MAX;
        // SAFETY: both outputs are writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_shutdown_request_query(
                    request,
                    &mut state,
                    &mut reason,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        assert_eq!((state, reason), (1, 0));
        assert_eq!(ui_abi_1_handle_release(request), UI_OK);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_shutdown_request_query(
                    request,
                    &mut state,
                    &mut reason,
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_HANDLE
        );
    }

    #[test]
    fn runtime_shutdown_receipt_survives_host_stop_and_explicit_retry() {
        let mut runtime = 0;
        // SAFETY: runtime output is writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let probe = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        // SAFETY: probe remains live through the retried shutdown boundary.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&probe as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut first_request = 0;
        // SAFETY: output is writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_request_shutdown(
                    runtime,
                    0,
                    &mut first_request,
                    std::ptr::null_mut(),
                )
            },
            UI_PENDING
        );
        // SAFETY: Runtime handle is borrowed and the optional error output is null.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_host_stopped(runtime, 73, std::ptr::null_mut()) },
            UI_OK
        );
        let mut state = 0;
        let mut reason = 0;
        // SAFETY: both outputs are writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_shutdown_request_query(
                    first_request,
                    &mut state,
                    &mut reason,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        assert_eq!((state, reason), (2, 73));

        let mut scope = 0;
        // Begin signals that the same UI host has resumed.
        // SAFETY: scope output is writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_host_begin_event(runtime, &mut scope) },
            UI_OK
        );
        let mut retry_request = 0;
        // SAFETY: request output is writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_request_shutdown(
                    runtime,
                    0,
                    &mut retry_request,
                    std::ptr::null_mut(),
                )
            },
            UI_PENDING
        );
        assert_ne!(retry_request, first_request);
        // SAFETY: the scope ID was returned by the matching begin call.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_host_end_event(runtime, scope) },
            UI_OK
        );
        assert_eq!(probe.destroyed.load(Ordering::SeqCst), 1);
        for request in [first_request, retry_request] {
            // SAFETY: both state outputs are writable.
            assert_eq!(
                // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
                unsafe {
                    ui_abi_1_shutdown_request_query(
                        request,
                        &mut state,
                        &mut reason,
                        std::ptr::null_mut(),
                    )
                },
                UI_OK
            );
            assert_eq!((state, reason), (1, 0));
            assert_eq!(ui_abi_1_handle_release(request), UI_OK);
        }
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn runtime_shutdown_worker_waits_for_host_cleanup() {
        let mut runtime = 0;
        // SAFETY: runtime output is writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let (sender, receiver) = mpsc::channel();
        // SAFETY: sender remains alive until worker request and host cleanup complete.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(send_wake),
                    (&sender as *const mpsc::Sender<()>).cast_mut().cast(),
                    None,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let worker_runtime = runtime;
        let worker = std::thread::spawn(move || {
            // SAFETY: shutdown only borrows the Runtime handle; optional error is null.
            unsafe { ui_abi_1_runtime_shutdown(worker_runtime, std::ptr::null_mut()) }
        });
        receiver
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("worker shutdown schedules the host loop");
        let mut scope = 0;
        // SAFETY: scope output is writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_host_begin_event(runtime, &mut scope) },
            UI_OK
        );
        // SAFETY: scope ID was returned by the matching begin call.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_host_end_event(runtime, scope) },
            UI_OK
        );
        assert_eq!(worker.join().expect("worker shutdown returns"), UI_OK);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn runtime_shutdown_rejects_invalid_mode_and_initializes_outputs() {
        let mut runtime = 0;
        // SAFETY: runtime output is writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let mut request = u64::MAX;
        // SAFETY: request output is writable.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_request_shutdown(runtime, 99, &mut request, std::ptr::null_mut())
            },
            UI_INVALID_ARGUMENT
        );
        assert_eq!(request, 0);
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_request_shutdown(
                    runtime,
                    0,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_ARGUMENT
        );
        let mut request = u64::MAX;
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_request_shutdown(u64::MAX, 0, &mut request, std::ptr::null_mut())
            },
            UI_INVALID_HANDLE
        );
        assert_eq!(request, 0);
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe {
                ui_abi_1_runtime_request_shutdown(runtime, 0, &mut request, std::ptr::null_mut())
            },
            super::UI_UNSUPPORTED
        );
        assert_eq!(request, 0);
        let mut scope = u64::MAX;
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_host_begin_event(runtime, &mut scope) },
            UI_OK
        );
        // SAFETY: scope ID was returned by the matching begin call.
        assert_eq!(
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_runtime_host_end_event(runtime, scope) },
            UI_OK
        );
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn handle_retain_returns_distinct_token_and_keeps_source_valid() {
        let mut runtime = 0;
        // SAFETY: `runtime` is writable output storage and the optional error output is null.
        let create_status = unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) };
        assert_eq!(create_status, UI_OK);

        let mut retained = u64::MAX;
        // SAFETY: `retained` is writable output storage.
        let retain_status = unsafe { ui_abi_1_handle_retain(runtime, &mut retained) };
        assert_eq!(retain_status, UI_OK);
        assert_ne!(retained, 0);
        assert_ne!(retained, runtime);

        for handle in [runtime, retained] {
            let mut is_ui_thread = 0;
            // SAFETY: `is_ui_thread` is writable storage and both tokens remain registered.
            let status = unsafe { ui_abi_1_runtime_is_ui_thread(handle, &mut is_ui_thread) };
            assert_eq!(status, UI_OK);
            assert_eq!(is_ui_thread, 1);
        }
    }

    #[test]
    fn handle_retain_rejects_invalid_token_and_null_output() {
        for borrowed in [0, u64::MAX] {
            let mut owned = u64::MAX;
            // SAFETY: `owned` is writable output storage.
            let status = unsafe { ui_abi_1_handle_retain(borrowed, &mut owned) };
            assert_eq!(status, UI_INVALID_HANDLE);
            assert_eq!(owned, 0);
        }

        // SAFETY: a null required output is handled before dereferencing.
        let status = unsafe { ui_abi_1_handle_retain(0, std::ptr::null_mut()) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
    }

    #[test]
    fn handle_retain_catches_core_panics_as_internal_error() {
        let result = catch_runtime_retain(|| panic!("injected Runtime retain panic"));

        assert_eq!(result, Err(UI_INTERNAL_ERROR));
    }

    #[test]
    fn handle_retain_maps_core_resource_failure_to_out_of_memory() {
        let result = catch_runtime_retain(|| Err(RuntimeRetainError::OutOfMemory));

        assert_eq!(result, Err(UI_OUT_OF_MEMORY));
    }

    #[test]
    fn numeric_narrow_applies_mode_and_rejects_invalid_or_unrepresentable_inputs() {
        let mut output = f32::NAN;
        let mut error = u64::MAX;
        let midpoint = 1.0_f64 + 2_f64.powi(-24);
        // SAFETY: both output pointers refer to writable storage.
        let status = unsafe { ui_abi_1_numeric_narrow_f64(midpoint, 1, &mut output, &mut error) };
        assert_eq!(status, UI_OK);
        assert_eq!(output.to_bits(), 0x3f80_0000);
        assert_eq!(error, 0);

        output = f32::NAN;
        error = u64::MAX;
        // SAFETY: both output pointers refer to writable storage.
        let status = unsafe { ui_abi_1_numeric_narrow_f64(0.1, 0, &mut output, &mut error) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(output.to_bits(), 0);
        assert_eq!(error, 0);

        output = f32::NAN;
        // SAFETY: the output pointer refers to writable storage and null optional error is allowed.
        let status =
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_numeric_narrow_f64(1.0, 2, &mut output, std::ptr::null_mut()) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(output.to_bits(), 0);

        output = f32::NAN;
        // SAFETY: the output pointer refers to writable storage; null optional error is allowed.
        let status =
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_numeric_narrow_f64(f64::MAX, 1, &mut output, std::ptr::null_mut()) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(output.to_bits(), 0);
    }

    #[test]
    fn numeric_conversion_rejects_null_required_outputs() {
        let mut error = u64::MAX;
        // SAFETY: null required output is handled and error points to writable storage.
        let status = unsafe { ui_abi_1_numeric_expand_f32(1.0, std::ptr::null_mut(), &mut error) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(error, 0);

        error = u64::MAX;
        // SAFETY: null required output is handled and error points to writable storage.
        let status =
            // SAFETY: any non-null output points to live test-owned storage; null optional outputs are permitted.
            unsafe { ui_abi_1_numeric_narrow_f64(1.0, 1, std::ptr::null_mut(), &mut error) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(error, 0);
    }

    #[test]
    fn get_version_reports_initial_stable_runtime_version() {
        let mut major = u32::MAX;
        let mut minor = u32::MAX;
        let mut patch = u32::MAX;
        // SAFETY: all output pointers refer to writable `u32` storage.
        let status = unsafe { ui_abi_1_get_version(&mut major, &mut minor, &mut patch) };

        assert_eq!(status, UI_OK);
        assert_eq!((major, minor, patch), (1, 0, 0));
    }

    #[test]
    fn property_create_kind_maps_initial_types_and_runtime_anchor_survives_runtime_release() {
        let mut runtime = 0;
        // SAFETY: runtime points to writable storage.
        let status = unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) };
        assert_eq!(status, UI_OK);

        let mut properties = [0_u64; 6];
        let mut error = u64::MAX;
        // SAFETY: each output points to writable storage; the runtime handle is live.
        let status =
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_create_bool(runtime, 1, &mut properties[0], &mut error) };
        assert_eq!(status, UI_OK);
        assert_eq!(error, 0);
        // SAFETY: each output points to writable storage; the runtime handle is live.
        let status = unsafe {
            ui_abi_1_property_create_i64(runtime, i64::MIN, &mut properties[1], &mut error)
        };
        assert_eq!(status, UI_OK);
        let mut initial_text = *b"a\0b";
        // SAFETY: the input byte array and output values remain valid for the call.
        let status = unsafe {
            ui_abi_1_property_create_utf8(
                runtime,
                initial_text.as_ptr(),
                initial_text.len() as u64,
                &mut properties[2],
                &mut error,
            )
        };
        assert_eq!(status, UI_OK);
        initial_text.fill(b'x');
        // SAFETY: null with length zero is accepted as an empty byte sequence; outputs are writable.
        let status = unsafe {
            ui_abi_1_property_create_utf8(
                runtime,
                std::ptr::null(),
                0,
                &mut properties[5],
                &mut error,
            )
        };
        assert_eq!(status, UI_OK);
        // SAFETY: each output points to writable storage; the runtime handle is live.
        let status = unsafe {
            ui_abi_1_property_create_f64(runtime, f64::NAN, &mut properties[3], &mut error)
        };
        assert_eq!(status, UI_OK);
        // SAFETY: each output points to writable storage; the runtime handle is live.
        let status = unsafe {
            ui_abi_1_property_create_decimal(runtime, u64::MAX, &mut properties[4], &mut error)
        };
        assert_eq!(status, UI_OK);
        assert_eq!(error, 0);
        assert!(properties.iter().all(|property| *property != 0));

        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
        for (property, expected_kind) in properties.into_iter().zip([1, 2, 3, 5, 6, 3]) {
            let mut kind = u32::MAX;
            // SAFETY: kind points to writable storage and the owned Property remains live.
            let status = unsafe { ui_abi_1_property_kind(property, &mut kind) };
            assert_eq!(status, UI_OK);
            assert_eq!(kind, expected_kind);
            assert_eq!(ui_abi_1_handle_release(property), UI_OK);
            assert_eq!(ui_abi_1_handle_release(property), UI_INVALID_HANDLE);
        }
    }

    #[test]
    fn property_create_and_kind_failures_zero_outputs_and_reject_wrong_thread() {
        let mut runtime = 0;
        // SAFETY: runtime points to writable storage.
        let status = unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) };
        assert_eq!(status, UI_OK);
        let mut property = u64::MAX;
        let mut error = u64::MAX;

        // SAFETY: outputs point to writable storage and bool=2 is rejected before use.
        let status =
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_create_bool(runtime, 2, &mut property, &mut error) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!((property, error), (0, 0));

        property = u64::MAX;
        error = u64::MAX;
        // SAFETY: null positive-length input is rejected; outputs are valid writable storage.
        let status = unsafe {
            ui_abi_1_property_create_utf8(runtime, std::ptr::null(), 1, &mut property, &mut error)
        };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!((property, error), (0, 0));

        property = u64::MAX;
        let invalid_utf8 = [0xff];
        // SAFETY: invalid UTF-8 input is readable for one byte and outputs are writable.
        let status = unsafe {
            ui_abi_1_property_create_utf8(
                runtime,
                invalid_utf8.as_ptr(),
                1,
                &mut property,
                &mut error,
            )
        };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!((property, error), (0, 0));

        property = u64::MAX;
        // SAFETY: zero runtime handle is invalid; outputs point to writable storage.
        let status = unsafe { ui_abi_1_property_create_i64(0, 9, &mut property, &mut error) };
        assert_eq!(status, UI_INVALID_HANDLE);
        assert_eq!((property, error), (0, 0));

        let mut valid_property = 0;
        // SAFETY: outputs point to writable storage and the runtime handle is live.
        let status =
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_create_i64(runtime, 9, &mut valid_property, &mut error) };
        assert_eq!(status, UI_OK);

        let worker = std::thread::spawn(move || {
            let mut worker_property = u64::MAX;
            let mut worker_error = u64::MAX;
            // SAFETY: output storage is valid; Runtime create is rejected on a worker thread.
            let create_status = unsafe {
                ui_abi_1_property_create_bool(runtime, 0, &mut worker_property, &mut worker_error)
            };
            let mut kind = u32::MAX;
            // SAFETY: output storage is valid; kind query is rejected on a worker thread.
            let kind_status = unsafe { ui_abi_1_property_kind(valid_property, &mut kind) };
            (
                create_status,
                worker_property,
                worker_error,
                kind_status,
                kind,
            )
        });
        assert_eq!(
            worker.join().expect("worker should finish"),
            (UI_WRONG_THREAD, 0, 0, UI_WRONG_THREAD, 0)
        );

        let mut kind = u32::MAX;
        // SAFETY: zero token is invalid and output storage is writable.
        let status = unsafe { ui_abi_1_property_kind(0, &mut kind) };
        assert_eq!(status, UI_INVALID_HANDLE);
        assert_eq!(kind, 0);
        assert_eq!(ui_abi_1_handle_release(valid_property), UI_OK);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn property_entrypoints_reject_null_required_outputs() {
        let mut runtime = 0;
        let mut error = u64::MAX;
        // SAFETY: runtime points to writable storage.
        let status = unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) };
        assert_eq!(status, UI_OK);

        // SAFETY: null required output is handled and optional error output is writable.
        let status =
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_create_i64(runtime, 1, std::ptr::null_mut(), &mut error) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(error, 0);
        let mut kind = u32::MAX;
        // SAFETY: null required output is handled without dereference.
        let status = unsafe { ui_abi_1_property_kind(0, std::ptr::null_mut()) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        // SAFETY: zero token is invalid and kind output is writable.
        let status = unsafe { ui_abi_1_property_kind(0, &mut kind) };
        assert_eq!(status, UI_INVALID_HANDLE);
        assert_eq!(kind, 0);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn property_getters_return_the_initial_value_for_each_supported_kind() {
        let mut runtime = 0;
        // SAFETY: runtime points to writable storage.
        let status = unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) };
        assert_eq!(status, UI_OK);

        let mut properties = [0_u64; 5];
        let mut error = 0;
        // SAFETY: all output pointers refer to writable storage and Runtime is live.
        let status =
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_create_bool(runtime, 1, &mut properties[0], &mut error) };
        assert_eq!(status, UI_OK);
        // SAFETY: all output pointers refer to writable storage and Runtime is live.
        let status = unsafe {
            ui_abi_1_property_create_i64(runtime, i64::MIN, &mut properties[1], &mut error)
        };
        assert_eq!(status, UI_OK);
        let initial_text = *b"A\0\xe6\x97\xa5";
        // SAFETY: initial_text is readable for its declared length and outputs are writable.
        let status = unsafe {
            ui_abi_1_property_create_utf8(
                runtime,
                initial_text.as_ptr(),
                initial_text.len() as u64,
                &mut properties[2],
                &mut error,
            )
        };
        assert_eq!(status, UI_OK);
        // SAFETY: all output pointers refer to writable storage and Runtime is live.
        let status = unsafe {
            ui_abi_1_property_create_f64(runtime, -13.25, &mut properties[3], &mut error)
        };
        assert_eq!(status, UI_OK);
        let decimal = 0x8123_4567_89ab_cdef;
        // SAFETY: all output pointers refer to writable storage and Runtime is live.
        let status = unsafe {
            ui_abi_1_property_create_decimal(runtime, decimal, &mut properties[4], &mut error)
        };
        assert_eq!(status, UI_OK);
        assert_eq!(error, 0);

        let mut bool_value = u8::MAX;
        // SAFETY: scalar output points to writable storage and Property kind is BOOL.
        let status = unsafe { ui_abi_1_property_get_bool(properties[0], &mut bool_value) };
        assert_eq!(status, UI_OK);
        assert_eq!(bool_value, 1);

        let mut i64_value = i64::MAX;
        // SAFETY: scalar output points to writable storage and Property kind is I64.
        let status = unsafe { ui_abi_1_property_get_i64(properties[1], &mut i64_value) };
        assert_eq!(status, UI_OK);
        assert_eq!(i64_value, i64::MIN);

        let mut required = u64::MAX;
        // SAFETY: null/zero is the documented UTF8 length-query form; out_length is writable.
        let status = unsafe {
            ui_abi_1_property_get_utf8(properties[2], std::ptr::null_mut(), 0, &mut required)
        };
        assert_eq!(status, UI_OK);
        assert_eq!(required, initial_text.len() as u64);
        let mut text = [0_u8; 5];
        required = u64::MAX;
        // SAFETY: text is writable for its full declared capacity and out_length is writable.
        let status = unsafe {
            ui_abi_1_property_get_utf8(
                properties[2],
                text.as_mut_ptr(),
                text.len() as u64,
                &mut required,
            )
        };
        assert_eq!(status, UI_OK);
        assert_eq!(required, initial_text.len() as u64);
        assert_eq!(&text, &initial_text);

        let mut f64_value = f64::NAN;
        // SAFETY: scalar output points to writable storage and Property kind is F64.
        let status = unsafe { ui_abi_1_property_get_f64(properties[3], &mut f64_value) };
        assert_eq!(status, UI_OK);
        assert_eq!(f64_value.to_bits(), (-13.25_f64).to_bits());

        let mut decimal_value = 0;
        // SAFETY: scalar output points to writable storage and Property kind is DECIMAL.
        let status = unsafe { ui_abi_1_property_get_decimal(properties[4], &mut decimal_value) };
        assert_eq!(status, UI_OK);
        assert_eq!(decimal_value, decimal);

        for property in properties {
            assert_eq!(ui_abi_1_handle_release(property), UI_OK);
        }
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn property_setters_commit_each_kind_and_preserve_equivalent_representations() {
        let mut runtime = 0;
        // SAFETY: runtime points to writable storage.
        // SAFETY: this call uses live test handles and valid local buffers/outputs.
        let status = unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) };
        assert_eq!(status, UI_OK);

        let mut properties = [0_u64; 8];
        let mut error = u64::MAX;
        // SAFETY: each handle output is writable and Runtime is live.
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_create_bool(runtime, 1, &mut properties[0], &mut error) },
            UI_OK
        );
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_create_i64(runtime, 7, &mut properties[1], &mut error) },
            UI_OK
        );
        let old_text = *b"old";
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe {
                ui_abi_1_property_create_utf8(
                    runtime,
                    old_text.as_ptr(),
                    3,
                    &mut properties[2],
                    &mut error,
                )
            },
            UI_OK
        );
        let initial_nan = f64::from_bits(0x7ff8_0000_0000_0011);
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe {
                ui_abi_1_property_create_f64(runtime, initial_nan, &mut properties[3], &mut error)
            },
            UI_OK
        );
        let decimal_one_tenth = (1_u64 << 58) | 10;
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe {
                ui_abi_1_property_create_decimal(
                    runtime,
                    decimal_one_tenth,
                    &mut properties[4],
                    &mut error,
                )
            },
            UI_OK
        );
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_create_f64(runtime, -0.0, &mut properties[5], &mut error) },
            UI_OK
        );
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_create_f64(runtime, 1.25, &mut properties[6], &mut error) },
            UI_OK
        );
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_create_decimal(runtime, 0, &mut properties[7], &mut error) },
            UI_OK
        );
        assert_eq!(error, 0);

        // SAFETY: optional error output is writable and each handle is a live matching Property.
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_set_bool(properties[0], 0, &mut error) },
            UI_OK
        );
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_set_i64(properties[1], i64::MIN, &mut error) },
            UI_OK
        );
        let new_text = *b"A\0\xe6\x97\xa5";
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe {
                ui_abi_1_property_set_utf8(
                    properties[2],
                    new_text.as_ptr(),
                    new_text.len() as u64,
                    &mut error,
                )
            },
            UI_OK
        );
        let different_nan = f64::from_bits(0x7ff8_0000_0000_0042);
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_set_f64(properties[3], different_nan, &mut error) },
            UI_OK
        );
        let decimal_one_hundredth = (2_u64 << 58) | 100;
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe {
                ui_abi_1_property_set_decimal(properties[4], decimal_one_hundredth, &mut error)
            },
            UI_OK
        );
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_set_f64(properties[5], 0.0, &mut error) },
            UI_OK
        );
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_set_f64(properties[6], 2.5, &mut error) },
            UI_OK
        );
        let decimal_one_and_quarter = (2_u64 << 58) | 125;
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe {
                ui_abi_1_property_set_decimal(properties[7], decimal_one_and_quarter, &mut error)
            },
            UI_OK
        );

        let mut bool_value = u8::MAX;
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_get_bool(properties[0], &mut bool_value) },
            UI_OK
        );
        assert_eq!(bool_value, 0);
        let mut i64_value = 0;
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_get_i64(properties[1], &mut i64_value) },
            UI_OK
        );
        assert_eq!(i64_value, i64::MIN);
        let mut text = [0_u8; 5];
        let mut length = 0;
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe {
                ui_abi_1_property_get_utf8(
                    properties[2],
                    text.as_mut_ptr(),
                    text.len() as u64,
                    &mut length,
                )
            },
            UI_OK
        );
        assert_eq!(length, new_text.len() as u64);
        assert_eq!(text, new_text);
        let mut f64_value = 0.0;
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_get_f64(properties[3], &mut f64_value) },
            UI_OK
        );
        assert_eq!(f64_value.to_bits(), initial_nan.to_bits());
        let mut decimal_value = 0;
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_get_decimal(properties[4], &mut decimal_value) },
            UI_OK
        );
        assert_eq!(decimal_value, decimal_one_tenth);
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_get_f64(properties[5], &mut f64_value) },
            UI_OK
        );
        assert_eq!(f64_value.to_bits(), (-0.0_f64).to_bits());
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_get_f64(properties[6], &mut f64_value) },
            UI_OK
        );
        assert_eq!(f64_value, 2.5);
        assert_eq!(
            // SAFETY: this call uses live test handles and valid local buffers/outputs.
            unsafe { ui_abi_1_property_get_decimal(properties[7], &mut decimal_value) },
            UI_OK
        );
        assert_eq!(decimal_value, decimal_one_and_quarter);

        for property in properties {
            assert_eq!(ui_abi_1_handle_release(property), UI_OK);
        }
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn property_setters_reject_bad_inputs_without_changing_values() {
        let mut runtime = 0;
        // SAFETY: runtime points to writable storage.
        let status = unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) };
        assert_eq!(status, UI_OK);
        let mut properties = [0_u64; 2];
        let mut error = u64::MAX;
        // SAFETY: outputs are writable and Runtime is live.
        let status =
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_create_i64(runtime, 42, &mut properties[0], &mut error) };
        assert_eq!(status, UI_OK);
        let original_text = *b"safe";
        // SAFETY: input bytes are readable and output storage is writable.
        let status = unsafe {
            ui_abi_1_property_create_utf8(
                runtime,
                original_text.as_ptr(),
                original_text.len() as u64,
                &mut properties[1],
                &mut error,
            )
        };
        assert_eq!(status, UI_OK);
        assert_eq!(error, 0);

        // SAFETY: optional error output is writable; BOOL=2 is rejected before state access.
        let status = unsafe { ui_abi_1_property_set_bool(properties[0], 2, &mut error) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        // SAFETY: null positive-length input is rejected before dereference.
        let status =
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_set_utf8(properties[1], std::ptr::null(), 1, &mut error) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        let invalid_utf8 = [0xff];
        // SAFETY: invalid_utf8 is readable for its declared byte length.
        let status = unsafe {
            ui_abi_1_property_set_utf8(
                properties[1],
                invalid_utf8.as_ptr(),
                invalid_utf8.len() as u64,
                &mut error,
            )
        };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        // SAFETY: optional error output is writable; Property is live but has kind I64.
        let status = unsafe { ui_abi_1_property_set_bool(properties[0], 1, &mut error) };
        assert_eq!(status, UI_WRONG_TYPE);
        // SAFETY: optional error output is writable; token zero is rejected.
        let status = unsafe { ui_abi_1_property_set_i64(0, 99, &mut error) };
        assert_eq!(status, UI_INVALID_HANDLE);
        assert_eq!(error, 0);

        let worker = std::thread::spawn(move || {
            let mut worker_error = u64::MAX;
            // SAFETY: optional error output is writable and the owned Property handle is live.
            let status = unsafe { ui_abi_1_property_set_i64(properties[0], 99, &mut worker_error) };
            (status, worker_error)
        });
        assert_eq!(
            worker.join().expect("worker should finish"),
            (UI_WRONG_THREAD, 0)
        );

        let mut actual = 0;
        // SAFETY: output is writable and the Property remains live.
        let status = unsafe { ui_abi_1_property_get_i64(properties[0], &mut actual) };
        assert_eq!(status, UI_OK);
        assert_eq!(actual, 42);
        let mut actual_text = [0_u8; 4];
        let mut actual_length = 0;
        // SAFETY: output buffer and length are writable and the Property remains live.
        let status = unsafe {
            ui_abi_1_property_get_utf8(
                properties[1],
                actual_text.as_mut_ptr(),
                actual_text.len() as u64,
                &mut actual_length,
            )
        };
        assert_eq!(status, UI_OK);
        assert_eq!(actual_length, original_text.len() as u64);
        assert_eq!(actual_text, original_text);
        for property in properties {
            assert_eq!(ui_abi_1_handle_release(property), UI_OK);
        }
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn property_getters_reject_wrong_kind_invalid_handle_and_worker_thread() {
        let mut runtime = 0;
        // SAFETY: runtime points to writable storage.
        let status = unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) };
        assert_eq!(status, UI_OK);
        let mut property = 0;
        // SAFETY: property output points to writable storage.
        let status = unsafe {
            ui_abi_1_property_create_i64(runtime, 42, &mut property, std::ptr::null_mut())
        };
        assert_eq!(status, UI_OK);

        let mut wrong_type_output = u8::MAX;
        // SAFETY: scalar output is writable; the live Property has a different kind.
        let status = unsafe { ui_abi_1_property_get_bool(property, &mut wrong_type_output) };
        assert_eq!(status, UI_WRONG_TYPE);
        assert_eq!(wrong_type_output, 0);

        let mut invalid_output = i64::MAX;
        // SAFETY: scalar output is writable; token zero is invalid.
        let status = unsafe { ui_abi_1_property_get_i64(0, &mut invalid_output) };
        assert_eq!(status, UI_INVALID_HANDLE);
        assert_eq!(invalid_output, 0);

        // SAFETY: null required output is checked before dereference.
        let status = unsafe { ui_abi_1_property_get_i64(property, std::ptr::null_mut()) };
        assert_eq!(status, UI_INVALID_ARGUMENT);

        let worker = std::thread::spawn(move || {
            let mut output = i64::MAX;
            // SAFETY: output is writable; this Runtime's Property belongs to the creator thread.
            let status = unsafe { ui_abi_1_property_get_i64(property, &mut output) };
            (status, output)
        });
        assert_eq!(
            worker.join().expect("worker should finish"),
            (UI_WRONG_THREAD, 0)
        );
        assert_eq!(ui_abi_1_handle_release(property), UI_OK);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn property_get_utf8_short_or_invalid_buffer_never_partially_writes() {
        let mut runtime = 0;
        // SAFETY: runtime points to writable storage.
        let status = unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) };
        assert_eq!(status, UI_OK);
        let mut property = 0;
        let mut error = 0;
        let bytes = *b"long";
        // SAFETY: bytes and output storage are valid for the call.
        let status = unsafe {
            ui_abi_1_property_create_utf8(
                runtime,
                bytes.as_ptr(),
                bytes.len() as u64,
                &mut property,
                &mut error,
            )
        };
        assert_eq!(status, UI_OK);

        let mut short = [0xa5; 3];
        let mut length = u64::MAX;
        // SAFETY: short is writable for its full capacity and length points to writable storage.
        let status = unsafe {
            ui_abi_1_property_get_utf8(
                property,
                short.as_mut_ptr(),
                short.len() as u64,
                &mut length,
            )
        };
        assert_eq!(status, UI_BUFFER_TOO_SMALL);
        assert_eq!(length, 4);
        assert_eq!(short, [0xa5; 3]);

        length = u64::MAX;
        // SAFETY: null positive-capacity buffer is rejected; length points to writable storage.
        let status =
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_get_utf8(property, std::ptr::null_mut(), 4, &mut length) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(length, 0);

        length = u64::MAX;
        // SAFETY: the required length output is null and is checked before dereference.
        let status = unsafe {
            ui_abi_1_property_get_utf8(property, std::ptr::null_mut(), 0, std::ptr::null_mut())
        };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(length, u64::MAX);

        assert_eq!(ui_abi_1_handle_release(property), UI_OK);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn property_subscription_release_is_not_cancel_and_explicit_cancel_is_idempotent() {
        let mut runtime = 0;
        // SAFETY: the output is writable.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let bridge = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        // SAFETY: bridge userdata remains live through Runtime shutdown.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&bridge as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut property = 0;
        // SAFETY: the output is writable and Runtime is live on its UI thread.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_property_create_i64(runtime, 0, &mut property, std::ptr::null_mut())
            },
            UI_OK
        );
        let probe = PostProbe::default();
        let mut subscription = u64::MAX;
        let mut error = u64::MAX;
        // SAFETY: probe outlives cancellation; callbacks use the declared C ABI.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_property_subscribe(
                    property,
                    Some(record_post_callback),
                    (&probe as *const PostProbe).cast_mut().cast(),
                    Some(record_post_destroy),
                    &mut subscription,
                    &mut error,
                )
            },
            UI_OK
        );
        assert_ne!(subscription, 0);
        assert_eq!(error, 0);
        assert_eq!(
            probe.callbacks.load(Ordering::SeqCst),
            0,
            "no initial notification"
        );

        let mut retained = 0;
        // SAFETY: retained is writable and subscription is a borrowed valid token.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_handle_retain(subscription, &mut retained) },
            UI_OK
        );
        assert_ne!(retained, subscription);
        assert_eq!(ui_abi_1_handle_release(subscription), UI_OK);
        // Releasing one owned Subscription token must leave the active registration intact.
        // SAFETY: The Property is live on the Runtime UI thread and error output is writable.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_set_i64(property, 0, &mut error) },
            UI_OK
        );
        assert_eq!(
            probe.callbacks.load(Ordering::SeqCst),
            0,
            "equivalent Property values do not notify"
        );
        // SAFETY: property remains live and error output is writable.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_set_i64(property, 1, &mut error) },
            UI_OK
        );
        assert_eq!(probe.callbacks.load(Ordering::SeqCst), 1);
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 0);
        // SAFETY: The Property is live on the Runtime UI thread; its value is unchanged.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_set_i64(property, 1, &mut error) },
            UI_OK
        );
        assert_eq!(probe.callbacks.load(Ordering::SeqCst), 1);

        // SAFETY: cancellation is on the Runtime UI thread and error output is writable.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_subscription_cancel(retained, &mut error) },
            UI_OK
        );
        assert_eq!(error, 0);
        // SAFETY: repeated cancellation of a valid inactive token succeeds.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_subscription_cancel(retained, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
        // SAFETY: Property setter stays on the Runtime UI thread.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_set_i64(property, 2, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(probe.callbacks.load(Ordering::SeqCst), 1);
        assert_eq!(ui_abi_1_handle_release(retained), UI_OK);
        complete_runtime_shutdown_at_host_boundary(runtime);
        assert_eq!(ui_abi_1_handle_release(property), UI_OK);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn property_subscription_self_cancel_and_registration_failures_preserve_ownership() {
        let mut runtime = 0;
        // SAFETY: the output is writable.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let bridge = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        // SAFETY: bridge userdata remains live through Runtime shutdown.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&bridge as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut property = 0;
        // SAFETY: the output is writable and Runtime is live on its UI thread.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_property_create_i64(runtime, 0, &mut property, std::ptr::null_mut())
            },
            UI_OK
        );
        let probe = SelfCancelProbe::default();
        let mut subscription = u64::MAX;
        let mut error = u64::MAX;
        // SAFETY: probe outlives its callback and destroy operations.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_property_subscribe(
                    property,
                    Some(cancel_self_callback),
                    (&probe as *const SelfCancelProbe).cast_mut().cast(),
                    Some(cancel_self_destroy),
                    &mut subscription,
                    &mut error,
                )
            },
            UI_OK
        );
        probe.subscription.store(subscription, Ordering::SeqCst);
        // SAFETY: the setter dispatches the callback at the UI-thread host boundary.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_property_set_i64(property, 1, &mut error) },
            UI_OK
        );
        assert_eq!(probe.callbacks.load(Ordering::SeqCst), 1);
        assert_eq!(probe.cancel_status.load(Ordering::SeqCst), UI_OK as usize);
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(
            *probe
                .destroy_threads
                .lock()
                .expect("destroy thread log lock"),
            vec![std::thread::current().id()]
        );
        // SAFETY: a canceled but still-owned handle remains valid; the worker is rejected by the
        // UI-thread precondition before any subscription state is touched.
        let worker_cancel = std::thread::spawn(move || {
            let mut worker_error = u64::MAX;
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            let status = unsafe { ui_abi_1_subscription_cancel(subscription, &mut worker_error) };
            (status, worker_error)
        })
        .join()
        .expect("worker cancel attempt completes");
        assert_eq!(worker_cancel, (UI_WRONG_THREAD, 0));
        // SAFETY: zero is an invalid Subscription token and is rejected before lookup.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe { ui_abi_1_subscription_cancel(0, std::ptr::null_mut()) },
            UI_INVALID_HANDLE
        );

        let rejected = PostProbe::default();
        let mut rejected_token = u64::MAX;
        // SAFETY: null callback is rejected before userdata ownership can transfer.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_property_subscribe(
                    property,
                    None,
                    (&rejected as *const PostProbe).cast_mut().cast(),
                    Some(record_post_destroy),
                    &mut rejected_token,
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_ARGUMENT
        );
        assert_eq!(rejected_token, 0);
        assert_eq!(rejected.destroys.load(Ordering::SeqCst), 0);
        // SAFETY: a null required output is rejected before any ownership transfer.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_property_subscribe(
                    property,
                    Some(record_post_callback),
                    (&rejected as *const PostProbe).cast_mut().cast(),
                    Some(record_post_destroy),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_ARGUMENT
        );
        assert_eq!(rejected.destroys.load(Ordering::SeqCst), 0);
        rejected_token = u64::MAX;
        // SAFETY: null destroy is rejected and does not take callback/userdata ownership.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_property_subscribe(
                    property,
                    Some(record_post_callback),
                    (&rejected as *const PostProbe).cast_mut().cast(),
                    None,
                    &mut rejected_token,
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_ARGUMENT
        );
        assert_eq!(rejected_token, 0);
        // SAFETY: null property is an invalid borrowed token; no closure ownership transfers.
        assert_eq!(
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            unsafe {
                ui_abi_1_property_subscribe(
                    0,
                    Some(record_post_callback),
                    (&rejected as *const PostProbe).cast_mut().cast(),
                    Some(record_post_destroy),
                    &mut rejected_token,
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_HANDLE
        );
        assert_eq!(rejected_token, 0);
        let property_token = property;
        let userdata = (&rejected as *const PostProbe) as usize;
        let worker_status = std::thread::spawn(move || {
            let mut output = u64::MAX;
            // SAFETY: all pointed-to userdata remains live through the joined worker; ABI rejects
            // the operation on the non-UI thread before taking ownership.
            // SAFETY: The test supplies live handles and writable outputs for this ABI call.
            let status = unsafe {
                ui_abi_1_property_subscribe(
                    property_token,
                    Some(record_post_callback),
                    userdata as *mut c_void,
                    Some(record_post_destroy),
                    &mut output,
                    std::ptr::null_mut(),
                )
            };
            (status, output)
        })
        .join()
        .expect("worker subscription attempt completes");
        assert_eq!(worker_status, (UI_WRONG_THREAD, 0));
        assert_eq!(rejected.destroys.load(Ordering::SeqCst), 0);

        complete_runtime_shutdown_at_host_boundary(runtime);
        assert_eq!(ui_abi_1_handle_release(subscription), UI_OK);
        assert_eq!(ui_abi_1_handle_release(property), UI_OK);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }
    // This integration-style test passes only live handles and local writable outputs; worker
    // calls use null optional outputs and are deliberately rejected before accessing Runtime state.
    #[allow(clippy::undocumented_unsafe_blocks)]
    #[test]
    fn binding_lifecycle_supports_one_time_release_cancel_and_validation() {
        let mut runtime = 0;
        // SAFETY: runtime output is writable.
        assert_eq!(
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let bridge = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        // SAFETY: probe stays live through shutdown, and callbacks are thread-safe test functions.
        assert_eq!(
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&bridge as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let create_i64 = |initial, out: &mut u64| {
            // SAFETY: output is writable and Runtime is used from its UI thread.
            unsafe { ui_abi_1_property_create_i64(runtime, initial, out, std::ptr::null_mut()) }
        };
        let (mut once_source, mut once_target) = (0, 0);
        assert_eq!(create_i64(10, &mut once_source), UI_OK);
        assert_eq!(create_i64(0, &mut once_target), UI_OK);
        let (mut binding, mut error) = (u64::MAX, u64::MAX);
        // SAFETY: both Property handles are live, same-Runtime I64 values, and outputs are writable.
        assert_eq!(
            unsafe {
                ui_abi_1_binding_create(once_source, once_target, 1, &mut binding, &mut error)
            },
            UI_OK
        );
        assert_eq!((binding, error), (0, 0));
        let mut value = 0;
        // SAFETY: target value output is writable.
        assert_eq!(
            unsafe { ui_abi_1_property_get_i64(once_target, &mut value) },
            UI_OK
        );
        assert_eq!(value, 10);
        // SAFETY: setter is called on the Runtime UI thread.
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(once_source, 11, &mut error) },
            UI_OK
        );
        // SAFETY: target value output is writable.
        assert_eq!(
            unsafe { ui_abi_1_property_get_i64(once_target, &mut value) },
            UI_OK
        );
        assert_eq!(value, 10, "OneTime does not retain a graph edge");

        let (mut one_way_source, mut one_way_target) = (0, 0);
        assert_eq!(create_i64(20, &mut one_way_source), UI_OK);
        assert_eq!(create_i64(0, &mut one_way_target), UI_OK);
        assert_eq!(
            unsafe {
                ui_abi_1_binding_create(one_way_source, one_way_target, 2, &mut binding, &mut error)
            },
            UI_OK
        );
        assert_ne!(binding, 0);
        assert_eq!(error, 0);
        assert_eq!(ui_abi_1_handle_release(binding), UI_OK);
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(one_way_source, 21, &mut error) },
            UI_OK
        );
        assert_eq!(
            unsafe { ui_abi_1_property_get_i64(one_way_target, &mut value) },
            UI_OK
        );
        assert_eq!(
            value, 21,
            "releasing the owner token leaves Runtime Binding active"
        );

        let (mut two_way_source, mut two_way_target) = (0, 0);
        assert_eq!(create_i64(30, &mut two_way_source), UI_OK);
        assert_eq!(create_i64(0, &mut two_way_target), UI_OK);
        assert_eq!(
            unsafe {
                ui_abi_1_binding_create(two_way_source, two_way_target, 3, &mut binding, &mut error)
            },
            UI_OK
        );
        assert_ne!(binding, 0);
        let mut retained_binding = 0;
        assert_eq!(
            unsafe { ui_abi_1_handle_retain(binding, &mut retained_binding) },
            UI_OK
        );
        assert_ne!(retained_binding, binding);
        assert_eq!(ui_abi_1_handle_release(binding), UI_OK);
        let worker_binding = retained_binding;
        let worker_status = std::thread::spawn(move || unsafe {
            ui_abi_1_binding_cancel(worker_binding, std::ptr::null_mut())
        })
        .join()
        .expect("worker cancel attempt completes");
        assert_eq!(worker_status, UI_WRONG_THREAD);
        assert_eq!(
            unsafe { ui_abi_1_binding_cancel(retained_binding, &mut error) },
            UI_OK
        );
        assert_eq!(error, 0);
        assert_eq!(
            unsafe { ui_abi_1_binding_cancel(retained_binding, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(two_way_source, 31, &mut error) },
            UI_OK
        );
        assert_eq!(
            unsafe { ui_abi_1_property_get_i64(two_way_target, &mut value) },
            UI_OK
        );
        assert_eq!(
            value, 30,
            "cancel preserves the last value and blocks future propagation"
        );
        let canceled_binding = retained_binding;
        assert_eq!(ui_abi_1_handle_release(canceled_binding), UI_OK);

        let mut bool_property = 0;
        assert_eq!(
            unsafe {
                ui_abi_1_property_create_bool(runtime, 0, &mut bool_property, std::ptr::null_mut())
            },
            UI_OK
        );
        binding = u64::MAX;
        assert_eq!(
            unsafe {
                ui_abi_1_binding_create(two_way_source, bool_property, 2, &mut binding, &mut error)
            },
            UI_WRONG_TYPE
        );
        assert_eq!((binding, error), (0, 0));

        let mut foreign_runtime = 0;
        assert_eq!(
            unsafe { ui_abi_1_runtime_create(&mut foreign_runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let foreign_bridge = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        assert_eq!(
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    foreign_runtime,
                    Some(record_wake),
                    (&foreign_bridge as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut foreign_property = 0;
        assert_eq!(
            unsafe {
                ui_abi_1_property_create_i64(
                    foreign_runtime,
                    0,
                    &mut foreign_property,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        assert_eq!(
            unsafe {
                ui_abi_1_binding_create(
                    two_way_source,
                    foreign_property,
                    2,
                    &mut binding,
                    &mut error,
                )
            },
            UI_WRONG_RUNTIME
        );
        assert_eq!((binding, error), (0, 0));
        complete_runtime_shutdown_at_host_boundary(foreign_runtime);
        assert_eq!(foreign_bridge.destroyed.load(Ordering::SeqCst), 1);
        assert_eq!(ui_abi_1_handle_release(foreign_property), UI_OK);
        assert_eq!(ui_abi_1_handle_release(foreign_runtime), UI_OK);

        binding = u64::MAX;
        assert_eq!(
            unsafe {
                ui_abi_1_binding_create(
                    two_way_source,
                    two_way_target,
                    99,
                    &mut binding,
                    &mut error,
                )
            },
            UI_INVALID_ARGUMENT
        );
        assert_eq!((binding, error), (0, 0));
        error = u64::MAX;
        assert_eq!(
            unsafe {
                ui_abi_1_binding_create(
                    two_way_source,
                    two_way_source,
                    2,
                    std::ptr::null_mut(),
                    &mut error,
                )
            },
            UI_INVALID_ARGUMENT
        );
        assert_eq!(error, 0);
        assert_eq!(
            unsafe { ui_abi_1_binding_cancel(0, std::ptr::null_mut()) },
            UI_INVALID_HANDLE
        );

        complete_runtime_shutdown_at_host_boundary(runtime);
        assert_eq!(bridge.destroyed.load(Ordering::SeqCst), 1);
        for property in [
            once_source,
            once_target,
            one_way_source,
            one_way_target,
            two_way_source,
            two_way_target,
            bool_property,
        ] {
            assert_eq!(ui_abi_1_handle_release(property), UI_OK);
        }
        assert_eq!(ui_abi_1_handle_release(canceled_binding), UI_INVALID_HANDLE);
        assert_eq!(ui_abi_1_handle_release(runtime), UI_OK);
    }

    #[test]
    fn get_version_rejects_each_missing_output_and_zeros_remaining_outputs() {
        for null_index in 0..3 {
            let mut outputs = [u32::MAX; 3];
            let mut pointers = [
                &mut outputs[0] as *mut u32,
                &mut outputs[1] as *mut u32,
                &mut outputs[2] as *mut u32,
            ];
            pointers[null_index] = std::ptr::null_mut();

            // SAFETY: each non-null pointer refers to a distinct writable `u32`; the null pointer is handled by the function.
            let status = unsafe { ui_abi_1_get_version(pointers[0], pointers[1], pointers[2]) };

            assert_eq!(status, UI_INVALID_ARGUMENT);
            for (index, value) in outputs.iter().enumerate() {
                if index != null_index {
                    assert_eq!(
                        *value, 0,
                        "output {index} must be zero when another output is null"
                    );
                }
            }
        }
    }

    #[test]
    fn has_feature_reports_only_complete_public_contracts() {
        for feature_id in [1, 2, 4, 5, 6] {
            let mut supported = 1;
            // SAFETY: `supported` is valid writable storage for the output value.
            let status = unsafe { ui_abi_1_has_feature(feature_id, &mut supported) };
            assert_eq!(status, UI_OK);
            assert_eq!(supported, 0, "feature {feature_id} is not implemented");
        }

        let mut numeric_supported = 0;
        // SAFETY: `numeric_supported` is valid writable storage for the output value.
        let status = unsafe { ui_abi_1_has_feature(3, &mut numeric_supported) };
        assert_eq!(status, UI_OK);
        assert_eq!(numeric_supported, 1);

        let mut post_dispatch_supported = 0;
        // SAFETY: the output points to writable storage.
        let status = unsafe { ui_abi_1_has_feature(7, &mut post_dispatch_supported) };
        assert_eq!(status, UI_OK);
        assert_eq!(post_dispatch_supported, 1);

        let mut unknown_supported = 1;
        // SAFETY: `unknown_supported` is valid writable storage for the output value.
        let status = unsafe { ui_abi_1_has_feature(0xffff_ffff, &mut unknown_supported) };
        assert_eq!(status, UI_OK);
        assert_eq!(unknown_supported, 0);

        let mut invalid_supported = 1;
        // SAFETY: `invalid_supported` is valid writable storage for the output value.
        let status = unsafe { ui_abi_1_has_feature(0, &mut invalid_supported) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(invalid_supported, 0);

        // SAFETY: a null output pointer is explicitly handled before it is dereferenced.
        let null_status = unsafe { ui_abi_1_has_feature(1, std::ptr::null_mut()) };
        assert_eq!(null_status, UI_INVALID_ARGUMENT);
    }

    #[test]
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn i64_validator_error_and_initial_binding_rejection_are_atomic() {
        let mut runtime = 0;
        // SAFETY: outputs are valid writable locals.
        assert_eq!(
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let mut property = 0;
        // SAFETY: output is writable and Runtime is valid on this UI thread.
        assert_eq!(
            unsafe {
                ui_abi_1_property_create_i64(runtime, 4, &mut property, std::ptr::null_mut())
            },
            UI_OK
        );
        let probe = ValidatorProbe::default();
        let mut registration = 0;
        // SAFETY: probe outlives registration and cancellation.
        assert_eq!(
            unsafe {
                ui_abi_1_property_validator_register_i64(
                    property,
                    Some(reject_negative_i64),
                    (&probe as *const ValidatorProbe).cast_mut().cast(),
                    Some(validator_destroy),
                    &mut registration,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        assert_eq!(probe.calls.load(Ordering::SeqCst), 0);
        let mut duplicate = 1;
        // SAFETY: failed registration leaves callback ownership with this test.
        assert_eq!(
            unsafe {
                ui_abi_1_property_validator_register_i64(
                    property,
                    Some(reject_negative_i64),
                    (&probe as *const ValidatorProbe).cast_mut().cast(),
                    Some(validator_destroy),
                    &mut duplicate,
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_ARGUMENT
        );
        assert_eq!(duplicate, 0);
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 0);

        let mut error = 0;
        // SAFETY: nullable error output is valid.
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(property, 4, &mut error) },
            UI_OK
        );
        // SAFETY: rejected set produces an owned immutable Error.
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(property, -2, &mut error) },
            UI_VALIDATION_FAILED
        );
        assert_ne!(error, 0);
        assert_eq!(probe.calls.load(Ordering::SeqCst), 2);
        let mut value = 0;
        // SAFETY: output is writable.
        assert_eq!(
            unsafe { ui_abi_1_property_get_i64(property, &mut value) },
            UI_OK
        );
        assert_eq!(value, 4);
        let mut status = 0;
        // SAFETY: output is writable.
        assert_eq!(unsafe { ui_abi_1_error_status(error, &mut status) }, UI_OK);
        assert_eq!(status, UI_VALIDATION_FAILED);
        let (mut domain, mut code) = (0, 0);
        // SAFETY: both outputs are writable.
        assert_eq!(
            unsafe { ui_abi_1_error_code(error, &mut domain, &mut code) },
            UI_OK
        );
        assert_eq!((domain, code), (41, 9));
        let message = b"negative values are rejected";
        let mut length = 0;
        // SAFETY: null buffer with zero capacity queries required byte length.
        assert_eq!(
            unsafe { ui_abi_1_error_message_utf8(error, std::ptr::null_mut(), 0, &mut length) },
            UI_OK
        );
        assert_eq!(length as usize, message.len());
        let mut short = [0x5a; 3];
        // SAFETY: writable buffer is valid for its declared capacity.
        assert_eq!(
            unsafe {
                ui_abi_1_error_message_utf8(
                    error,
                    short.as_mut_ptr(),
                    short.len() as u64,
                    &mut length,
                )
            },
            UI_BUFFER_TOO_SMALL
        );
        assert_eq!(short, [0x5a; 3]);
        let mut output = vec![0; message.len()];
        // SAFETY: output buffer is writable for the declared capacity.
        assert_eq!(
            unsafe {
                ui_abi_1_error_message_utf8(
                    error,
                    output.as_mut_ptr(),
                    output.len() as u64,
                    &mut length,
                )
            },
            UI_OK
        );
        assert_eq!(output, message);
        let mut retained_error = 0;
        // SAFETY: output is writable; retained copy owns the same immutable Error.
        assert_eq!(
            unsafe { ui_abi_1_handle_retain(error, &mut retained_error) },
            UI_OK
        );
        assert_eq!(ui_abi_1_handle_release(error), UI_OK);
        // SAFETY: retained handle stays valid after original release.
        assert_eq!(
            unsafe { ui_abi_1_error_status(retained_error, &mut status) },
            UI_OK
        );
        assert_eq!(ui_abi_1_handle_release(retained_error), UI_OK);

        let mut registration_copy = 0;
        // SAFETY: output is writable and registration is live.
        assert_eq!(
            unsafe { ui_abi_1_handle_retain(registration, &mut registration_copy) },
            UI_OK
        );
        assert_eq!(ui_abi_1_handle_release(registration), UI_OK);
        // SAFETY: registration's retained owner keeps validation active.
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(property, -3, &mut error) },
            UI_VALIDATION_FAILED
        );
        assert_eq!(ui_abi_1_handle_release(error), UI_OK);
        let worker_registration = registration_copy;
        let worker_cancel = std::thread::spawn(move || {
            // SAFETY: cancel accepts a valid handle and nullable output; this checks UI affinity.
            unsafe { ui_abi_1_property_validator_cancel(worker_registration, std::ptr::null_mut()) }
        })
        .join()
        .expect("worker cancellation result");
        assert_eq!(worker_cancel, UI_WRONG_THREAD);
        // SAFETY: output is writable; cancellation removes the validator and runs destroy once.
        assert_eq!(
            unsafe { ui_abi_1_property_validator_cancel(registration_copy, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
        // SAFETY: candidate is accepted after cancellation.
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(property, -3, std::ptr::null_mut()) },
            UI_OK
        );

        let bool_probe = ValidatorProbe::default();
        let mut bool_property = 0;
        // SAFETY: writable output and live Runtime.
        assert_eq!(
            unsafe {
                ui_abi_1_property_create_bool(runtime, 0, &mut bool_property, std::ptr::null_mut())
            },
            UI_OK
        );
        let mut bool_registration = 0;
        // SAFETY: probe outlives cancellation.
        assert_eq!(
            unsafe {
                ui_abi_1_property_validator_register_bool(
                    bool_property,
                    Some(accept_bool),
                    (&bool_probe as *const ValidatorProbe).cast_mut().cast(),
                    Some(validator_destroy),
                    &mut bool_registration,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        // SAFETY: nullable setter error output.
        assert_eq!(
            unsafe { ui_abi_1_property_set_bool(bool_property, 1, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(bool_probe.calls.load(Ordering::SeqCst), 1);
        // SAFETY: nullable cancellation error output.
        assert_eq!(
            unsafe { ui_abi_1_property_validator_cancel(bool_registration, std::ptr::null_mut()) },
            UI_OK
        );

        let utf8_probe = ValidatorProbe::default();
        let initial_utf8 = b"initial";
        let mut utf8_property = 0;
        // SAFETY: initial bytes are readable and output writable.
        assert_eq!(
            unsafe {
                ui_abi_1_property_create_utf8(
                    runtime,
                    initial_utf8.as_ptr(),
                    initial_utf8.len() as u64,
                    &mut utf8_property,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut utf8_registration = 0;
        // SAFETY: probe outlives cancellation.
        assert_eq!(
            unsafe {
                ui_abi_1_property_validator_register_utf8(
                    utf8_property,
                    Some(accept_utf8),
                    (&utf8_probe as *const ValidatorProbe).cast_mut().cast(),
                    Some(validator_destroy),
                    &mut utf8_registration,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let updated_utf8 = b"updated";
        // SAFETY: input bytes are readable for the declared length.
        assert_eq!(
            unsafe {
                ui_abi_1_property_set_utf8(
                    utf8_property,
                    updated_utf8.as_ptr(),
                    updated_utf8.len() as u64,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        assert_eq!(utf8_probe.calls.load(Ordering::SeqCst), 1);
        // SAFETY: nullable cancellation error output.
        assert_eq!(
            unsafe { ui_abi_1_property_validator_cancel(utf8_registration, std::ptr::null_mut()) },
            UI_OK
        );

        let target_probe = ValidatorProbe::default();
        let mut source = 0;
        let mut target = 0;
        // SAFETY: outputs are writable.
        assert_eq!(
            unsafe { ui_abi_1_property_create_i64(runtime, -7, &mut source, std::ptr::null_mut()) },
            UI_OK
        );
        // SAFETY: outputs are writable.
        assert_eq!(
            unsafe { ui_abi_1_property_create_i64(runtime, 0, &mut target, std::ptr::null_mut()) },
            UI_OK
        );
        let mut target_registration = 0;
        // SAFETY: probe outlives registration cancellation.
        assert_eq!(
            unsafe {
                ui_abi_1_property_validator_register_i64(
                    target,
                    Some(reject_negative_i64),
                    (&target_probe as *const ValidatorProbe).cast_mut().cast(),
                    Some(validator_destroy),
                    &mut target_registration,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut binding = 1;
        let mut binding_error = 0;
        for mode in [1, 2, 3] {
            // SAFETY: outputs are writable; rejected initial sync must leave binding zero.
            assert_eq!(
                unsafe {
                    ui_abi_1_binding_create(source, target, mode, &mut binding, &mut binding_error)
                },
                UI_VALIDATION_FAILED
            );
            assert_eq!(binding, 0);
            assert_ne!(binding_error, 0);
            assert_eq!(ui_abi_1_handle_release(binding_error), UI_OK);
            binding_error = 0;
        }
        assert_eq!(target_probe.calls.load(Ordering::SeqCst), 3);
        let mut target_value = 9;
        // SAFETY: output is writable.
        assert_eq!(
            unsafe { ui_abi_1_property_get_i64(target, &mut target_value) },
            UI_OK
        );
        assert_eq!(target_value, 0);
        // SAFETY: source setter has no requested Error output.
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(source, 7, std::ptr::null_mut()) },
            UI_OK
        );
        // SAFETY: valid output pointers; success proves failed create published no graph edge.
        assert_eq!(
            unsafe { ui_abi_1_binding_create(source, target, 2, &mut binding, &mut binding_error) },
            UI_OK
        );
        assert_ne!(binding, 0);
        assert_eq!(binding_error, 0);

        // SAFETY: cancel output is nullable; all handles are owned by this test.
        assert_eq!(
            unsafe {
                ui_abi_1_property_validator_cancel(target_registration, std::ptr::null_mut())
            },
            UI_OK
        );
        for handle in [
            binding,
            target_registration,
            registration_copy,
            bool_registration,
            bool_property,
            utf8_registration,
            utf8_property,
            property,
            source,
            target,
            runtime,
        ] {
            assert_eq!(ui_abi_1_handle_release(handle), UI_OK);
        }
    }

    #[test]
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn binding_error_snapshot_and_subscription_follow_binding_delivery_order() {
        let mut runtime = 0;
        assert_eq!(
            unsafe { ui_abi_1_runtime_create(&mut runtime, std::ptr::null_mut()) },
            UI_OK
        );
        let bridge = BridgeProbe {
            priorities: Mutex::new(Vec::new()),
            destroyed: AtomicUsize::new(0),
        };
        assert_eq!(
            unsafe {
                ui_abi_1_runtime_set_ui_loop_bridge(
                    runtime,
                    Some(record_wake),
                    (&bridge as *const BridgeProbe).cast_mut().cast(),
                    Some(record_destroy),
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut source = 0;
        let mut target = 0;
        assert_eq!(
            unsafe { ui_abi_1_property_create_i64(runtime, 7, &mut source, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(
            unsafe { ui_abi_1_property_create_i64(runtime, 7, &mut target, std::ptr::null_mut()) },
            UI_OK
        );
        let validator = ValidatorProbe::default();
        let mut registration = 0;
        assert_eq!(
            unsafe {
                ui_abi_1_property_validator_register_i64(
                    target,
                    Some(reject_negative_i64),
                    (&validator as *const ValidatorProbe).cast_mut().cast(),
                    Some(validator_destroy),
                    &mut registration,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut binding = 0;
        assert_eq!(
            unsafe {
                ui_abi_1_binding_create(source, target, 2, &mut binding, std::ptr::null_mut())
            },
            UI_OK
        );
        let (mut empty_error, mut empty_property, mut empty_direction) =
            (u64::MAX, u64::MAX, u32::MAX);
        assert_eq!(
            unsafe {
                ui_abi_1_binding_error_snapshot(
                    binding,
                    &mut empty_error,
                    &mut empty_property,
                    &mut empty_direction,
                )
            },
            UI_OK
        );
        assert_eq!((empty_error, empty_property, empty_direction), (0, 0, 0));
        let mut partially_null = u64::MAX;
        assert_eq!(
            unsafe {
                ui_abi_1_binding_error_snapshot(
                    binding,
                    std::ptr::null_mut(),
                    &mut partially_null,
                    std::ptr::null_mut(),
                )
            },
            UI_INVALID_ARGUMENT
        );
        assert_eq!(partially_null, 0);
        let probe = BindingErrorProbe::default();
        let userdata = (&probe as *const BindingErrorProbe).cast_mut().cast();
        let mut error_subscription = 0;
        assert_eq!(
            unsafe {
                ui_abi_1_binding_subscribe_error(
                    binding,
                    Some(binding_error_callback),
                    userdata,
                    Some(binding_error_destroy),
                    &mut error_subscription,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let mut property_subscription = 0;
        assert_eq!(
            unsafe {
                ui_abi_1_property_subscribe(
                    source,
                    Some(binding_source_callback),
                    userdata,
                    Some(binding_error_destroy),
                    &mut property_subscription,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        assert_eq!(
            probe.callbacks.load(Ordering::SeqCst),
            0,
            "subscription has no initial notification"
        );

        let mut change_scope = 0;
        assert_eq!(
            unsafe { ui_abi_1_runtime_host_begin_event(runtime, &mut change_scope) },
            UI_OK
        );
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(source, -3, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(source, -4, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(
            unsafe { ui_abi_1_runtime_host_end_event(runtime, change_scope) },
            UI_OK
        );
        assert_eq!(unsafe { ui_abi_1_runtime_dispatch_turn(runtime) }, UI_OK);
        assert_eq!(
            *probe.order.lock().expect("order lock"),
            vec!["error", "property"]
        );
        assert_eq!(probe.callbacks.load(Ordering::SeqCst), 1);

        let (mut error, mut rejected, mut direction) = (0, 0, 0);
        assert_eq!(
            unsafe {
                ui_abi_1_binding_error_snapshot(binding, &mut error, &mut rejected, &mut direction)
            },
            UI_OK
        );
        assert_ne!(error, 0);
        assert_ne!(rejected, 0);
        assert_eq!(direction, 1);
        let mut code = (0, 0);
        assert_eq!(
            unsafe { ui_abi_1_error_code(error, &mut code.0, &mut code.1) },
            UI_OK
        );
        assert_eq!(code, (41, 9));
        assert_eq!(
            unsafe { ui_abi_1_property_set_i64(source, 12, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(unsafe { ui_abi_1_runtime_dispatch_turn(runtime) }, UI_OK);
        assert_eq!(
            probe.callbacks.load(Ordering::SeqCst),
            2,
            "clearing the current error is also observable"
        );
        let (mut cleared_error, mut cleared_property, mut cleared_direction) =
            (u64::MAX, u64::MAX, u32::MAX);
        assert_eq!(
            unsafe {
                ui_abi_1_binding_error_snapshot(
                    binding,
                    &mut cleared_error,
                    &mut cleared_property,
                    &mut cleared_direction,
                )
            },
            UI_OK
        );
        assert_eq!(
            (cleared_error, cleared_property, cleared_direction),
            (0, 0, 0)
        );
        let mut retained_code = (0, 0);
        assert_eq!(
            unsafe { ui_abi_1_error_code(error, &mut retained_code.0, &mut retained_code.1) },
            UI_OK,
            "issued Error snapshots remain immutable after recovery"
        );
        assert_eq!(retained_code, (41, 9));
        assert_eq!(ui_abi_1_handle_release(error), UI_OK);
        assert_eq!(ui_abi_1_handle_release(rejected), UI_OK);

        assert_eq!(
            unsafe { ui_abi_1_binding_cancel(binding, std::ptr::null_mut()) },
            UI_OK
        );
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
        let mut shutdown_binding = 0;
        assert_eq!(
            unsafe {
                ui_abi_1_binding_create(
                    source,
                    target,
                    2,
                    &mut shutdown_binding,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        let shutdown_probe = BindingErrorProbe::default();
        let mut shutdown_subscription = 0;
        assert_eq!(
            unsafe {
                ui_abi_1_binding_subscribe_error(
                    shutdown_binding,
                    Some(binding_error_callback),
                    (&shutdown_probe as *const BindingErrorProbe)
                        .cast_mut()
                        .cast(),
                    Some(binding_error_destroy),
                    &mut shutdown_subscription,
                    std::ptr::null_mut(),
                )
            },
            UI_OK
        );
        complete_runtime_shutdown_at_host_boundary(runtime);
        assert_eq!(shutdown_probe.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 2);
        for handle in [
            shutdown_subscription,
            shutdown_binding,
            property_subscription,
            error_subscription,
            registration,
            binding,
            source,
            target,
            runtime,
        ] {
            assert_eq!(ui_abi_1_handle_release(handle), UI_OK);
        }
    }
}
