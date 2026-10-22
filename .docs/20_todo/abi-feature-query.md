# C ABI feature query

- Updated: 2026-10-18
- Status: implemented and verified; ready for commit
- Scope: `UNIT-ABI-001` implements `ui_abi_1_has_feature`, including output initialization, invalid ID handling, and unsupported/unknown feature results.

## Approved basis

- Requirements: [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006), [RQ-04-010](../11_requirements/RQ-04_abi-boundary.md#rq-04-010).
- Detailed design: [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.1, feature ID assignments and `has_feature` rules.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-ABI-001; release cases TC-ABI-COMP-02 and TC-ABI-REP-01.
- Architecture constraint: [ADR-0001](../04_approval/adr-0001-rust-core.md), Accepted.

## Slice boundary

- Require a non-null output pointer and initialize it to false before returning.
- Treat feature ID 0 as invalid. Return `UI_OK` with false for known but not fully implemented features and unknown IDs.
- Do not advertise any feature until all of its public symbols and contracts are implemented.
- Defer `get_version`; the initial stable ABI version number is not fixed in the approved versioning detail.

## Steps

- [x] Confirm approved RQ-04-006/RQ-04-010, ABI v1.1, and UT-ABI-001.
- [x] Register `UNIT-ABI-001` as planned before editing source.
- [x] Add the exported C ABI feature query and focused behavior tests.
- [x] Run focused/workspace tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after the registered command passes.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md).

- `cargo test -p freemvvm-c-abi has_feature`: passed.
- `cargo test --workspace --all-targets --locked`: passed (21 tests).
- `.venv\Scripts\python.exe tools\dev.py check`: passed.
- `.venv\Scripts\python.exe tools\check_docs.py`: passed (76 active documents, 5853 local links).
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `dumpbin /exports target\debug\freemvvm_abi_1.dll`: confirmed `ui_abi_1_has_feature` is exported.
