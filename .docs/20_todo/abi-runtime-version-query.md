# C ABI runtime version query

- Updated: 2026-10-18
- Status: implemented and verified; ready for commit
- Scope: `UNIT-ABI-002` implements and verifies `ui_abi_1_get_version`.

## Approved basis

- Requirements: [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006), [RQ-04-009](../11_requirements/RQ-04_abi-boundary.md#rq-04-009), [RQ-04-028](../11_requirements/RQ-04_abi-boundary.md#rq-04-028), and [RQ-04-030](../11_requirements/RQ-04_abi-boundary.md#rq-04-030).
- Detailed design: [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.1, `get_version`, required output initialization, and stable version compatibility.
- Distribution design: [Toolchain and packages](../13_detail_design/build/toolchain-packages.md) v1.1, initial stable Runtime 1.0.0 / ABI 1.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-ABI-002; release cases TC-ABI-COMP-02 and TC-ABI-REP-01.
- Architecture constraint: [ADR-0001](../04_approval/adr-0001-rust-core.md), Accepted.

## Slice boundary

- Export the approved three-output `ui_abi_1_get_version` ABI signature.
- Return stable Runtime version `1.0.0`; its ABI major agrees with the `ui_abi_1_` symbol prefix.
- Reject any null required output pointer with `UI_INVALID_ARGUMENT`; initialize every non-null output to zero before returning an error.
- Keep pre-release text and package metadata outside this numeric stable-version query.

## Steps

- [x] Confirm current approved RQ-04-009/RQ-04-028/RQ-04-030 and ABI v1.1 output contract.
- [x] Register `UNIT-ABI-002` as planned before editing implementation.
- [x] Add focused success and null-output behavior tests.
- [x] Implement the exported function to satisfy those tests.
- [x] Run focused/workspace tests, development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after its registered command passes.

## Validation

The exact unit test command is recorded in `tools/units.yaml`. No public header or distribution artifact is added by this implementation slice.

- `cargo test -p freemvvm-c-abi get_version`: passed (2 tests).
- `cargo test --workspace --all-targets --locked`: passed (23 tests).
- `.venv\Scripts\python.exe tools\dev.py check`: passed.
- `.venv\Scripts\python.exe tools\check_docs.py`: passed (76 active documents, 5858 local links).
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `dumpbin /exports target\debug\freemvvm_abi_1.dll`: confirmed `ui_abi_1_get_version` is exported.
