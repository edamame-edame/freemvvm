# Release a closed Runtime shell from a worker

- Updated: 2026-10-18
- Status: implemented and verified; ready for commit
- Scope: `UNIT-CORE-020` verifies that a remaining Runtime-owned handle can be released from a worker after UI cleanup has committed Closed, without scheduling UI cleanup again.

## Approved basis

- Requirements: [RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003), [RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010).
- Detailed design: [Object handles and values](../13_detail_design/runtime-objects/objects-values.md) v1.2, “Runtime object thread-safe owner and UI state separation”; [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, shutdown states.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-HANDLE-SHUTDOWN-001.
- Architecture constraint: [ADR-0001](../04_approval/adr-0001-rust-core.md), Accepted.

## Slice boundary

- Complete UI cleanup and publish `Closed` while one public object token remains.
- Release that final token on a worker thread and verify the shell is consumed without another UI cleanup ticket.
- Keep post-shutdown UI use and immutable Error reads outside this slice; those APIs are not implemented yet.

## Steps

- [x] Confirm approved RQ, DD, and UT-HANDLE-SHUTDOWN-001.
- [x] Register `UNIT-CORE-020` as planned before editing source.
- [x] Add a focused worker-release test to the existing shutdown test design case.
- [x] Run focused/core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after the registered command passes.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md).

- `cargo test -p freemvvm-core runtime::tests::closed_runtime_shell_releases_on_worker_without_ui_cleanup`: passed.
- `cargo test -p freemvvm-core`: 20 passed.
- `cargo clippy -p freemvvm-core --all-targets -- -D warnings`: passed.
- `.venv\Scripts\python.exe tools\dev.py check`: passed.
- `.venv\Scripts\python.exe tools\check_docs.py`: passed (76 active documents, 5853 local links).
- `cargo fmt --all -- --check` and `git diff --check`: passed.
