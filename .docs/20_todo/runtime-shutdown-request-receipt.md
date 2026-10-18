# Owned shutdown request receipt

- Updated: 2026-10-18
- Status: implemented and verified; ready for commit
- Scope: `UNIT-CORE-019` adds an owned, thread-independent observation handle for an accepted shutdown request; releasing the receipt does not cancel shutdown.

## Approved basis

- Requirement: [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “ShutdownRequest” and “CompletionRecord”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-005.

## Slice boundary

- Return an owned ShutdownRequest receipt only after shutdown admission succeeds.
- Allow the receipt to query Pending, Completed, or HostUnavailable(reason) from any thread and to wait for terminal state where synchronous worker behavior requires it.
- Receipt drop releases only that observer; HostRegistration retains accepted completion records until cleanup or stop.
- Make shutdown-boundary cleanup publish completion for HostRegistration-owned records without requiring an external observer to stay alive.
- Leave public ABI handle creation and Runtime Closed-shell ownership for later units.

## Steps

- [x] Confirm approved RQ-06-012, DD v1.1, and UT-RUNTIME-005.
- [x] Register `UNIT-CORE-019` as planned before editing source.
- [x] Add owned ShutdownRequest query/wait receipt and return it after admission.
- [x] Decouple cleanup commit from a caller-held completion observer.
- [x] Extend UT-RUNTIME-005 for cross-thread query and receipt release before cleanup.
- [x] Run focused/core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after source and registered tests pass.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md).

- `cargo test -p freemvvm-core runtime::tests::completion_record_distinguishes_host_stop_and_allows_ui_retry`: passed.
- `cargo test -p freemvvm-core`: 19 passed.
- `cargo clippy -p freemvvm-core --all-targets -- -D warnings`: passed.
- `.venv\Scripts\python.exe tools\dev.py check`: passed (including workspace checks and C/C++ bootstrap probes).
- `.venv\Scripts\python.exe tools\check_docs.py`: passed (76 active documents, 5853 local links).
- `cargo fmt --all -- --check` and `git diff --check`: passed.
