# Scope: Public C ABI header

- A public header must reflect only C ABI functions actually exported by `crates/c-abi/src/lib.rs`; declarations in the approved detailed design can still be future-only.
- Keep the header C-compatible with fixed-width types and `extern "C"` guards. Compile-only C11/C++17 probes validate declaration compatibility without claiming library linkage or package distribution.
- When this header is added, update `DEVELOPMENT.md` to distinguish header availability from still-unimplemented native link/install/export packaging.
- At UNIT-ABI-017, a mechanical symbol-name comparison confirmed the header declared all 47 current exports and no unimplemented design-only symbols; add this comparison when the export surface changes.
