# Rust C ABI: feature availability query

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-ABI-001`.

- `ui_abi_1_has_feature` is a `no_mangle` C export with `u32` feature/status values and a `u8` output. Null output returns `UI_INVALID_ARGUMENT`; a writable output is initialized to false before validating feature ID.
- Feature ID 0 is invalid. Unknown IDs return `UI_OK` and false. Known IDs also return false until every symbol and external contract in that feature group is implemented.
- The current initial stable ABI version is not selected in the approved design; do not infer it from the workspace package version `0.0.0` when implementing `get_version`.
- UT-ABI-001 verifies known IDs 1–6 false, unknown ID false, ID 0 invalid with initialized output, and null output rejection. Windows `dumpbin /exports` can confirm the symbol is present in the cdylib.
