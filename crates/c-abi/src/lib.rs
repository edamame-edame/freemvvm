//! Versioned C ABI boundary over the internal Core.
//!
//! The public API is under construction and is not distributable yet.

use freemvvm_core::abi_support::{
    RuntimeCreateError, RuntimeQueryError, runtime_create_handle,
    runtime_is_ui_thread as core_runtime_is_ui_thread,
};
use freemvvm_core::numeric::{NarrowMode, expand_f32_to_f64, narrow_f64_to_f32};
use std::panic::{AssertUnwindSafe, catch_unwind};

const UI_OK: u32 = 0;
const UI_INVALID_ARGUMENT: u32 = 1;
const UI_INVALID_HANDLE: u32 = 2;
const UI_OUT_OF_MEMORY: u32 = 8;
const UI_INTERNAL_ERROR: u32 = 9;
const NUMERIC_NARROW_EXACT: u32 = 0;
const NUMERIC_NARROW_NEAREST_EVEN: u32 = 1;
const RUNTIME_VERSION_MAJOR: u32 = 1;
const RUNTIME_VERSION_MINOR: u32 = 0;
const RUNTIME_VERSION_PATCH: u32 = 0;

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

    if feature_id == 3 {
        // F32/F64 conversion symbols and their complete contracts are implemented.
        // SAFETY: the caller guarantees that the non-null output points to writable `u8` storage.
        unsafe { out_supported.write(1) };
    }

    UI_OK
}

#[cfg(test)]
mod tests {
    use super::{
        UI_INTERNAL_ERROR, UI_INVALID_ARGUMENT, UI_INVALID_HANDLE, UI_OK, catch_runtime_creation,
        catch_runtime_query, ui_abi_1_get_version, ui_abi_1_has_feature,
        ui_abi_1_numeric_expand_f32, ui_abi_1_numeric_narrow_f64, ui_abi_1_runtime_create,
        ui_abi_1_runtime_is_ui_thread,
    };

    #[test]
    fn numeric_expand_f32_returns_exact_f64_and_clears_optional_error() {
        let mut output = f64::NAN;
        let mut error = u64::MAX;
        // SAFETY: both output pointers refer to writable storage.
        let status =
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
            unsafe { ui_abi_1_numeric_narrow_f64(1.0, 2, &mut output, std::ptr::null_mut()) };
        assert_eq!(status, UI_INVALID_ARGUMENT);
        assert_eq!(output.to_bits(), 0);

        output = f32::NAN;
        // SAFETY: the output pointer refers to writable storage; null optional error is allowed.
        let status =
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
}
