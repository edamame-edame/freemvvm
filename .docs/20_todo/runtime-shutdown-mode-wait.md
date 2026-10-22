# Shutdown request mode and synchronous wait boundary

- Updated: 2026-10-18
- Status: implemented and verified
- Scope: `UNIT-CORE-016` makes synchronous shutdown requests wait only when issued from a worker thread; UI-thread requests return after admission to avoid waiting on their own event loop.

## Approved basis

- Requirements: [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005), [RQ-06-007](../11_requirements/RQ-06_threading-dispatch.md#rq-06-007), and [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Host境界・終了要求・完了観測” and “主要な呼出しと失敗境界”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-005.

## Slice boundary

- Add internal ASYNC/SYNC request modes and report Pending versus terminal completion.
- ASYNC returns after request admission without waiting.
- SYNC from a worker waits until Completed or HostUnavailable; SYNC on the UI thread returns after admission and never waits inside a callback/event.
- Preserve request admission, priority scheduling, host-stop/retry behavior, and public ABI as existing or later units.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-005.
- [x] Register `UNIT-CORE-016` as planned before editing source.
- [x] Add mode/result types and route worker SYNC to completion wait.
- [x] Extend UT-RUNTIME-005 for ASYNC Pending, UI-thread SYNC return, and worker SYNC terminal result.
- [x] Run focused/core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready after source and registered tests passed.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md).

- Focused UT-RUNTIME-005: passed.
- `cargo test -p freemvvm-core`: 19 passed.
- `.venv\Scripts\python.exe tools\dev.py check`: passed, including workspace Clippy/build, C/C++ probes, C# build, and Python checks.
- `.venv\Scripts\python.exe tools\check_docs.py`: 76 active documents and 5,853 local links passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
