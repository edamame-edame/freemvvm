# Graceful host stop drains pending shutdown

- Updated: 2026-10-18
- Status: implemented and verified
- Scope: `UNIT-CORE-017` makes `before_stop` close shutdown admission and service pending priority cleanup before the host loop is reported stopped.

## Approved basis

- Requirements: [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006), and [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Host/contextとcompletionの所有補足” / “HostScopeの対応”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-005.

## Slice boundary

- At a UI-thread host-stop boundary with no active HostScope, first stop admission of new requests.
- If priority shutdown is pending, cancel queued ordinary work, drain Runtime cleanup, and publish Completed before returning from `before_stop`.
- If no priority shutdown is pending, preserve the Runtime cleanup state and begin stopping the host.
- Keep abrupt `stopped(reason)` notification distinct: it may report HostUnavailable only when cleanup did not complete.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-005.
- [x] Register `UNIT-CORE-017` as planned before editing source.
- [x] Integrate `before_stop` with Dispatcher priority shutdown and Runtime cleanup.
- [x] Extend UT-RUNTIME-005 for graceful completion-before-stop and abrupt host-unavailable behavior.
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
