//! Versioned C ABI boundary over the internal Core.
//!
//! The public API is under construction and is not distributable yet.

const UI_OK: u32 = 0;
const UI_INVALID_ARGUMENT: u32 = 1;
const RUNTIME_VERSION_MAJOR: u32 = 1;
const RUNTIME_VERSION_MINOR: u32 = 0;
const RUNTIME_VERSION_PATCH: u32 = 0;

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

    UI_OK
}

#[cfg(test)]
mod tests {
    use super::{UI_INVALID_ARGUMENT, UI_OK, ui_abi_1_get_version, ui_abi_1_has_feature};

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
        for feature_id in 1..=6 {
            let mut supported = 1;
            // SAFETY: `supported` is valid writable storage for the output value.
            let status = unsafe { ui_abi_1_has_feature(feature_id, &mut supported) };
            assert_eq!(status, UI_OK);
            assert_eq!(supported, 0, "feature {feature_id} is not implemented");
        }

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
