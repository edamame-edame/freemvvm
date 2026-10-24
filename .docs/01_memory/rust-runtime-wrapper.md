<!-- scope: Rust safe Runtime wrapper, sys FFI, UI affinity, and cleanup tickets -->
# Rust Runtime wrapper memory

- `freemvvm-sys` is a raw FFI crate; it must not depend on the internal Core crate. `freemvvm` is the safe facade and uses the shared `freemvvm_abi_1` dynamic artifact.
- On Windows the Cargo-built C ABI import library is named `freemvvm_abi_1.dll.lib`; Rust's default `#[link(name = "freemvvm_abi_1")]` looked for the wrong name. Use a Windows `#[link(name = "freemvvm_abi_1.dll")]` and keep the runtime DLL on the executable search path.
- The development build script accepts `FREEMVVM_NATIVE_LIB_DIR`; the repository test commands build `freemvvm-c-abi` first and set that directory plus PATH/LD_LIBRARY_PATH. This is a developer integration path, not artifact fetch/profile distribution behavior.
- Reserve cleanup registry slots before any C ABI call that can return an owned handle. A reserved slot can be changed to Pending without allocating during Drop. Preserve raw status/domain/code and copy valid UTF-8 error messages before releasing an error handle.
- UI Runtime and borrowed UI context are `!Send`/`!Sync`; shutdown request receipts are thread-independent and keep a retained Runtime anchor so the primary Runtime handle can be released first.
- `tools/dev.py rust-wrapper-runtime-test` exercises four fake sys cases and compile-fail doctests; `rust-wrapper-runtime-consumer-test` links and runs a separate example against the real C ABI.
- Local `tools/dev.py check`, `minimum`, focused Runtime tests, and the real C ABI consumer passed on implementation commit `8752272`; Windows/Linux CI run 40077169840 also passed.
