# Runtime post ownership and shutdown cancellation

- Updated: 2026-10-17
- Status: complete; design and test design approved
- Scope: `UNIT-CORE-007` adds callback/destroy ownership to accepted Dispatcher work, preserves caller ownership on OOM/Closed rejection, and destroys queued work once on the UI thread during shutdown.

## Approved basis

- Requirements: [RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008) and [RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “UI dispatcherとhost loop” / “失敗と所有権”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-003.

## Slice boundary

- Represent queued work as callback plus destroy action. On normal invocation, invoke then destroy once on the UI thread.
- If queue capacity cannot be reserved or the dispatcher is Closed, return the complete work item to the caller without running callback or destroy.
- At a UI-thread shutdown safe point, transition to Closed, cancel all queued callbacks, and run each accepted item's destroy action once on that UI thread.
- Keep shutdown-priority scheduling, cancel of an active callback, host-stop completion records, and public ABI wiring for later slices. This close operation rejects invocation during an active dispatch turn; the Runtime shutdown coordinator must call it at a safe point.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-003.
- [x] Register `UNIT-CORE-007` as planned before editing source.
- [x] Implement explicit work ownership, OOM/Closed preservation, and UI-thread cancellation.
- [x] Run UT-RUNTIME-003, all core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after source and registered tests pass.

## Validation

The test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md). Validation passed: UT-RUNTIME-003, all 17 core tests, workspace Clippy/build harness, C/C++ probes, .NET build, Python wheel build/import, doc links, and `git diff --check`.
