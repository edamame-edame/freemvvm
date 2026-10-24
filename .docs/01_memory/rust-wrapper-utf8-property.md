[Rust wrapper / UTF-8 Property]
# Rust UTF-8 Property implementation notes

- Reuse the existing sealed `PropertyValue` mapping and `Property<T>` owner; remove a `Copy` bound if adding owned `String` values.
- UTF-8 C ABI uses `(pointer, byte_length)`. Rust `String::as_bytes()` preserves embedded NUL and is already valid UTF-8; empty slices are accepted with length zero.
- UTF-8 getter requires a length query, allocation, and copy. If the copy returns `BUFFER_TOO_SMALL`, query the new length and retry. Reject invalid UTF-8 from native output, guard impossible returned lengths, and use `try_reserve_exact` so allocation failure returns `OUT_OF_MEMORY`.
- Keep retry logic behind an injectable closure helper so tests can force growth between query/copy and verify short-buffer behavior without modifying the read-only `native_api.rs` or `handle.rs`.
- A standalone native ABI consumer should cover multibyte text, supplementary-plane characters, embedded NUL, empty String, and explicit close.
