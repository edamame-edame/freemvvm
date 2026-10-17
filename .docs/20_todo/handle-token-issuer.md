# Handle token issuer — first implementation slice

- Updated: 2026-10-17
- Status: complete; test-design document v0.2 approved by the user
- Scope: Rust Core process-wide token allocation only; no handle table, Runtime/kind validation, retain/release, or ABI surface.

## Basis

- Approved requirement: RQ-04-018.
- Approved detail design: [objects-values.md](../13_detail_design/runtime-objects/objects-values.md), v1.1, “handle登録と検証”.
- Test contract: [core-runtime.md](../32_unit_test/core-runtime.md) UT-HANDLE-003; implementation-unit contract: [implementation-test-inventory.md](../32_unit_test/implementation-test-inventory.md) v1.1.
- Git workflow: [rule.md](../02_git/rule.md).

## Steps

- [x] Confirm the approved token invariants, existing test oracle, workspace status, and current branch before editing.
- [x] Add a private Core token issuer that returns unique nonzero 64-bit tokens, never wraps or reuses an issued token, and fails closed on exhaustion or synchronization failure.
- [x] Add focused Rust unit tests for ordinary issuance, issuing the maximum token exactly once, exhaustion, and concurrent uniqueness.
- [x] Register the implementation unit, source path, test command, requirement/design links, and logical test case in `tools/units.yaml`; mark ready after test-design v0.2 approval.
- [x] Record the implementation learning in `.docs/01_memory/` and update this plan with exact validation results.
- [x] Run the unit test and relevant repository checks; inspect diff/status and preserve unrelated changes.

## Completion boundary

This slice is complete when the allocator and its tests pass on the pinned Rust toolchain and the manifest points only at real files and commands. Handle registration, lookup, validation, retain/release, ABI exposure, and wrapper integration remain subsequent slices.

## Validation result

- `cargo test -p freemvvm-core handle_registry::token::tests`: 4 passed.
- `rustup run 1.85.0 cargo test -p freemvvm-core handle_registry::token::tests`: 4 passed.
- `cargo fmt --all --check`: passed.
- `.venv/Scripts/python.exe tools/dev.py check`: all 16 bootstrap gates passed, including workspace Clippy/build/test-harness build, C/C++ probes, C# build, Python wheel/import, and 76 active documents / 5,813 local links.
- The user approved the complete test-design document v0.2 on 2026-10-17; `UNIT-CORE-001` is ready in `tools/units.yaml`.
