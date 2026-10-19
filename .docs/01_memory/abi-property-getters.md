# Rust C ABI: Property typed getters

Scope: `UNIT-ABI-010`, Property read operations at the versioned C ABI boundary.

- Use `RuntimeCore::with_property_value` to hold the registry validation pin and UI object-store lock only for the internal adapter read. Do not clone UTF8 values just to copy into the caller's buffer.
- Map kind mismatch to `UI_WRONG_TYPE`; map invalid/released token, worker thread, and closed runtime through the established Property/ABI error path. Required scalar and length outputs start at zero.
- UTF8 `(NULL, 0, out_length)` is a length query. A short buffer returns required length without writing any buffer byte. Successful copy is exact and not NUL-terminated; separate size/copy calls do not share a snapshot.
- In C ABI tests, `cargo test -p freemvvm-c-abi property_get` runs the getter value, kind/thread, and UTF8-buffer oracles.
