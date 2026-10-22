# Shutdown retry wake rearming after host resume

- Updated: 2026-10-18
- Status: implemented and verified
- Scope: `UNIT-CORE-015` reconnects explicit same-host retry to a fresh Dispatcher wake after the UI loop resumes.

## Approved basis

- Requirements: [RQ-06-007](../11_requirements/RQ-06_threading-dispatch.md#rq-06-007) and [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Host/contextとcompletionの所有補足” and “HostUnavailable → Pending”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-005.

## Slice boundary

- Require an Available HostRegistration, a HostUnavailable completion, and a still-pending shutdown on the same UI Dispatcher.
- Re-arm the completion record and issue a fresh host wake after releasing the host state lock, even if the prior wake was already marked pending before host stop.
- Reject retry if the Dispatcher has no pending shutdown or is closed, without changing the completion state.
- Leave host adapter lifecycle hooks and public request handles for later units.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-005.
- [x] Register `UNIT-CORE-015` as planned before editing source.
- [x] Add explicit Dispatcher shutdown wake rearming.
- [x] Connect HostRegistration retry to the same Dispatcher and guard against invalid retry state.
- [x] Extend UT-RUNTIME-005 for fresh wake scheduling and invalid retry rejection.
- [x] Run focused/core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready after source and registered tests passed.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md).

- Focused UT-RUNTIME-005: passed.
- `cargo test -p freemvvm-core`: 19 passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed through `tools/dev.py check`.
- `cargo fmt --all -- --check`: passed.
- `.venv\Scripts\python.exe tools\check_docs.py`: 76 active documents and 5,853 local links passed.
- `.venv\Scripts\python.exe tools\dev.py check`: passed, including workspace build, C/C++ probes, C# build, and Python checks.
- `git diff --check`: passed; Git only reported its existing LF-to-CRLF working-copy notices.
