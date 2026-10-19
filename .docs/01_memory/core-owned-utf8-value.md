# Core value memory: owned UTF-8 inputs

Scope: freemvvm Core UTF-8 boundary values.

- `OwnedUtf8` validates incoming bytes with `str::from_utf8` and copies into a Core-owned `String` before returning.
- Preserve the exact byte sequence: embedded NUL is valid, and this layer must not normalize, replace, or truncate.
- Use `String::try_reserve_exact` before copying so a reported reservation failure can be surfaced without a partially constructed value.
- Pointer/null/length validation and ABI status mapping belong at the C ABI entrypoint, not in this byte-slice value type.
- UT-VALUE-001 already covers empty, multibyte, embedded-NUL, invalid-byte, and source-buffer mutation behavior.
- `OwnedUtf8::copy_to_buffer(None)` is the internal capacity query; `Some(slice)` checks full byte capacity before copying and returns required byte length without partially changing a short destination. No trailing NUL is written.
- F64 Property-value equivalence is `(both NaN) || numeric equality`: NaN payload/sign do not matter, signed zero compares equal, and opposite infinities remain unequal. Keep this separate from stored representation normalization.
- `TypedValue` models BOOL, I64, UTF8, F64, and DECIMAL and compares only matching variants. UTF8 comparison is exact owned-string equality; F64 delegates to `f64_equivalent`. `DecimalValue` retains its original sign/scale/coefficient while comparing numerically after local trailing-zero reduction; zero sign and scale do not affect equivalence. This remains a pure value primitive, not Property storage/commit/notification behavior.
