[Rust wrapper / F64 Property]
# Rust F64 Property implementation notes

- Implement `f64` in the existing sealed `PropertyValue` trait to preserve the approved generic `Property<f64>` API. Do not create a second wrapper type.
- The C ABI maps Rust `f64` directly to C `double`; use create/get/set symbols without numeric narrowing or normalization.
- The F64 contract treats all NaNs as equal and `+0.0`/`-0.0` as equal; equal assignments preserve the stored representation. A consumer test can create a NaN payload or negative zero, set an equivalent value with different bits, and inspect `to_bits()` afterward.
- Infinity is permitted. Use the real shared C ABI consumer for mapping and special-value behavior, then run Rust minimum and cross-platform bootstrap CI.
- If an attached `property.rs` is read-only, never edit that snapshot. The workspace counterpart is separately governed by the user's explicit scope decision.
