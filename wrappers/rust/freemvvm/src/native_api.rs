use std::sync::Arc;

pub(crate) const OK: u32 = freemvvm_sys::OK;
pub(crate) const WRONG_RUNTIME: u32 = freemvvm_sys::WRONG_RUNTIME;
pub(crate) const WRONG_THREAD: u32 = freemvvm_sys::WRONG_THREAD;
pub(crate) const CLOSED: u32 = freemvvm_sys::CLOSED;
#[cfg(test)]
pub(crate) const BUFFER_TOO_SMALL: u32 = freemvvm_sys::BUFFER_TOO_SMALL;
pub(crate) const OUT_OF_MEMORY: u32 = freemvvm_sys::OUT_OF_MEMORY;
pub(crate) const INTERNAL_ERROR: u32 = freemvvm_sys::INTERNAL_ERROR;
pub(crate) const UNSUPPORTED: u32 = freemvvm_sys::UNSUPPORTED;
pub(crate) const PENDING: u32 = freemvvm_sys::PENDING;

pub(crate) trait NativeApi: Send + Sync {
    fn get_version(&self) -> (u32, u32, u32, u32);
    fn runtime_create(&self) -> (u32, u64, u64);
    fn runtime_is_ui_thread(&self, runtime: u64) -> (u32, u8);
    fn runtime_shutdown(&self, runtime: u64) -> (u32, u64);
    fn runtime_request_shutdown(&self, runtime: u64, mode: u32) -> (u32, u64, u64);
    fn shutdown_request_query(&self, request: u64) -> (u32, u32, u32, u64);
    fn handle_retain(&self, borrowed: u64) -> (u32, u64);
    fn handle_release(&self, owned: u64) -> u32;
    fn error_status(&self, error: u64) -> (u32, u32);
    fn error_code(&self, error: u64) -> (u32, u32, u32);
    fn error_message_utf8(&self, error: u64, buffer: Option<&mut [u8]>) -> (u32, u64);
}

pub(crate) type SharedApi = Arc<dyn NativeApi>;

pub(crate) struct LinkedApi;

impl NativeApi for LinkedApi {
    fn get_version(&self) -> (u32, u32, u32, u32) {
        let mut major = 0;
        let mut minor = 0;
        let mut patch = 0;
        // SAFETY: all out-pointers refer to initialized local storage for this call.
        let status =
            unsafe { freemvvm_sys::ui_abi_1_get_version(&mut major, &mut minor, &mut patch) };
        (status, major, minor, patch)
    }

    fn runtime_create(&self) -> (u32, u64, u64) {
        let mut runtime = 0;
        let mut error = 0;
        // SAFETY: the ABI writes only to the two valid out-pointers.
        let status = unsafe { freemvvm_sys::ui_abi_1_runtime_create(&mut runtime, &mut error) };
        (status, runtime, error)
    }

    fn runtime_is_ui_thread(&self, runtime: u64) -> (u32, u8) {
        let mut yes = 0;
        // SAFETY: `yes` is valid out-storage and `runtime` is borrowed by this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_runtime_is_ui_thread(runtime, &mut yes) };
        (status, yes)
    }

    fn runtime_shutdown(&self, runtime: u64) -> (u32, u64) {
        let mut error = 0;
        // SAFETY: `error` is valid out-storage and `runtime` is borrowed by this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_runtime_shutdown(runtime, &mut error) };
        (status, error)
    }

    fn runtime_request_shutdown(&self, runtime: u64, mode: u32) -> (u32, u64, u64) {
        let mut request = 0;
        let mut error = 0;
        // SAFETY: both output pointers are valid for the duration of the ABI call.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_runtime_request_shutdown(runtime, mode, &mut request, &mut error)
        };
        (status, request, error)
    }

    fn shutdown_request_query(&self, request: u64) -> (u32, u32, u32, u64) {
        let mut state = 0;
        let mut reason = 0;
        let mut error = 0;
        // SAFETY: all output pointers are valid for the duration of the ABI call.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_shutdown_request_query(
                request,
                &mut state,
                &mut reason,
                &mut error,
            )
        };
        (status, state, reason, error)
    }

    fn handle_retain(&self, borrowed: u64) -> (u32, u64) {
        let mut owned = 0;
        // SAFETY: `owned` is valid out-storage and the input is borrowed for this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_handle_retain(borrowed, &mut owned) };
        (status, owned)
    }

    fn handle_release(&self, owned: u64) -> u32 {
        // SAFETY: callers transfer one live owned token to this function.
        unsafe { freemvvm_sys::ui_abi_1_handle_release(owned) }
    }

    fn error_status(&self, error: u64) -> (u32, u32) {
        let mut status_value = 0;
        // SAFETY: `status_value` is valid out-storage and the error is borrowed for this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_error_status(error, &mut status_value) };
        (status, status_value)
    }

    fn error_code(&self, error: u64) -> (u32, u32, u32) {
        let mut domain = 0;
        let mut code = 0;
        // SAFETY: both outputs are valid and the error is borrowed for this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_error_code(error, &mut domain, &mut code) };
        (status, domain, code)
    }

    fn error_message_utf8(&self, error: u64, buffer: Option<&mut [u8]>) -> (u32, u64) {
        let mut length = 0;
        let (pointer, capacity) = match buffer {
            Some(bytes) => (bytes.as_mut_ptr(), bytes.len() as u64),
            None => (std::ptr::null_mut(), 0),
        };
        // SAFETY: a provided slice supplies writable storage of `capacity` bytes; None uses
        // the ABI's documented null/zero-capacity query form.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_error_message_utf8(error, pointer, capacity, &mut length)
        };
        (status, length)
    }
}
