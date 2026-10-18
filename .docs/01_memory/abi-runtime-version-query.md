# Rust C ABI: stable runtime version query

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-ABI-002`.

- RQ-04-030 v0.5 is approved and sets the initial stable Runtime to `1.0.0` / ABI 1; toolchain-packages.md v1.1 records the same. Do not use the workspace Cargo package version `0.0.0` as the public runtime version.
- `ui_abi_1_get_version` returns stable numeric MAJOR/MINOR/PATCH. Pre-release text and package version metadata are outside this query; the API does not return an `rc.N` string.
- Required outputs are independently checked for null. Every non-null output is initialized to zero before an invalid-argument return; valid calls return `(1, 0, 0)`.
- UT-ABI-002 covers the success tuple, each null position, and zero initialization of remaining outputs. `dumpbin /exports target\debug\freemvvm_abi_1.dll` verifies the Windows cdylib symbol.
