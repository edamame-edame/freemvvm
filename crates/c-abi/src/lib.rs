//! Versioned C ABI boundary over the internal Core.
//!
//! The public API is under construction and is not distributable yet.

const UI_OK: u32 = 0;
const UI_INVALID_ARGUMENT: u32 = 1;

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
    use super::{UI_INVALID_ARGUMENT, UI_OK, ui_abi_1_has_feature};

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
