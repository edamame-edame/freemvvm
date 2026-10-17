# Runtime shutdown priority dispatch

- Updated: 2026-10-17
- Status: complete; design and test design approved
- Scope: `UNIT-CORE-008` gives the Dispatcher an idempotent, allocation-free shutdown request and processes it before queued callbacks at the next UI turn, cancelling queued callbacks on the UI thread before running the supplied shutdown handler.

## Approved basis

- Requirements: [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005) and [RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “UI dispatcherとhost loop” / “shutdown control task”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-004.

## Slice boundary

- A shutdown request made during a callback does not interrupt it. It records a priority control item and guarantees a wake for the next turn.
- On the next UI turn, shutdown transitions the dispatcher out of Running before other queued callbacks can start, cancels queued work without invoking callbacks, and runs the shutdown handler on the UI thread.
- Repeated requests coalesce; post requests after shutdown begins are rejected with ownership returned to the caller.
- The test represents pending notification work as queued dispatcher work; the actual notification engine and Runtime/ABI host integration remain separate units.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-004.
- [x] Register `UNIT-CORE-008` as planned before editing source.
- [x] Implement the priority request state and UI-turn shutdown path.
- [x] Run UT-RUNTIME-004, all core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after source and registered tests pass.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md). UT-RUNTIME-004 and all 18 core tests passed. Workspace formatting, lint/build checks, C/C++ probes, .NET build, Python wheel build/import, document links, and `git diff --check` also passed.
