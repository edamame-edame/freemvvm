# Core and C ABI: F32/F64 conversion

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-ABI-003`.

- `crates/core-runtime/src/numeric.rs` implements F32 expansion and F64 narrowing using explicit IEEE bit classification and integer nearest-even rounding. Compile-time assertions restrict the codec to binary32/binary64 widths and format parameters.
- It canonicalizes NaNs to quiet NaNs while retaining sign, preserves signed zero and infinity, and rejects finite magnitudes above F32 max. Exact mode requires F64 round-trip bit equality for finite nonzero values.
- The C ABI numeric functions initialize non-null `out_error` and valid required output storage to 0 before mode/conversion validation. Null output, invalid mode, exactness failure, and finite overflow return `UI_INVALID_ARGUMENT`; `UI_VALIDATION_FAILED` remains for Property setter rejection.
- Feature ID 3 (`numeric.f32_bridge.v1`) is true now that both conversion symbols and their full behavior are present; other feature IDs remain false until their complete contracts are implemented.
- Tests cover representable values, signed zero, infinities, NaN quieting, exact rejection, finite overflow, ties-to-even at normal/subnormal boundaries, null outputs, invalid mode, and 50,000 deterministic IEEE samples compared with Rust's float cast.
