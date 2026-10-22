# Rust C ABI: feature availability query

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-ABI-001`.

- `ui_abi_1_has_feature` is a `no_mangle` C export with `u32` feature/status values and a `u8` output. Null output returns `UI_INVALID_ARGUMENT`; a writable output is initialized to false before validating feature ID.
- At UNIT-ABI-001, feature ID 0 was invalid and all known feature IDs were false because no complete feature group was implemented. UNIT-ABI-003 later enables ID 3 after implementing both numeric conversion symbols.
- `get_version` is implemented separately as UNIT-ABI-002; see `abi-runtime-version-query.md` for the current approved version query contract.
- UT-ABI-001 originally verified known IDs 1–6 false, unknown ID false, ID 0 invalid with initialized output, and null output rejection. It now verifies ID 3 true after UNIT-ABI-003 completes that feature; other known IDs and unknown IDs remain false. Windows `dumpbin /exports` confirms the symbol is present in the cdylib.
