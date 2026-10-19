# Rust C ABI: Property typed setters

Scope: `UNIT-ABI-011`, setting initial supported Property values through the versioned C ABI.

- Reuse `RuntimeCore::apply_property_candidate`; it validates the live token, Runtime, UI thread, closed state, and fixed kind before committing through `ValueStorage`.
- Map kind mismatch to `UI_WRONG_TYPE`, malformed UTF8 / invalid BOOL to `UI_INVALID_ARGUMENT`, UI-thread violations to `UI_WRONG_THREAD`, and UTF8 copy allocation failure to `UI_OUT_OF_MEMORY`.
- UTF8 setter accepts `(NULL, 0)` for empty bytes, preserves embedded NUL exactly, and copies before applying. It rejects null with positive length and invalid UTF8 without changing the prior value.
- F64 equivalent NaN and signed-zero candidates, and numerically equivalent DECIMAL encodings, retain the existing representation. All canonical DECIMAL bit patterns are valid.
- Unit excludes validators and subscription notification delivery; those APIs are separate implementation slices.
