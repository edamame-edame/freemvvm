[Rust wrapper / Input events] 2026-11-03 Rust Input event facade unit

- Rust Input facade is implemented in `wrappers/rust/freemvvm/src/input.rs`; ABI bindings belong in `freemvvm-sys`, while the safe wrapper's `NativeApi` methods are mirrored in the linked implementation and fake API.
- `InputEvent<'event>` carries a private hidden handle in each public variant so callback-scoped borrowed data cannot escape; compile-fail doctest checks the lifetime boundary.
- Root a `FnMut` callback only for successful Core registration. On failed registration, reclaim wrapper-owned state; on cancellation, let the ABI destroy callback reclaim it once.
- Catch user panic across the FFI boundary and mark that event handled per the approved Rust wrapper DD. Do not mark handled for payload decode/read failures; the design specifies suppression only for panic.
- Input subscription is available on concrete Widget wrappers and generic `Widget`; use the existing `Subscription`, Runtime identity, UI thread checks, and cleanup anchors.
- Validate with `cargo test --locked -p freemvvm`, `cargo clippy --locked -p freemvvm --all-targets -- -D warnings`, `cargo +1.85.0 check --workspace --all-targets --locked`, `cargo fmt --all -- --check`, `python tools/check_docs.py`, and `git diff --check`.
