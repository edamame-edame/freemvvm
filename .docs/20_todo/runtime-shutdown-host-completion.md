# Host shutdown cleanup and completion wiring

- Updated: 2026-10-17
- Status: complete; design and test design approved
- Scope: `UNIT-CORE-013` connects the outer HostRegistration shutdown boundary to RuntimeCleanupQueue draining and ShutdownCompletion publication.

## Approved basis

- Requirements: [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005), [RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006), and [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Shutdown時の状態遷移”, “Host境界・終了要求・完了観測”, and “主要な呼出しと失敗境界”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-005.

## Slice boundary

- On a pending shutdown at a valid outer host event boundary, run `RuntimeCleanupQueue::drain_shutdown_and_complete` from the Dispatcher shutdown handler.
- Publish `Completed` only after cleanup reaches Closed; preserve the existing pending/no-op behavior when no shutdown is pending.
- Propagate cleanup/commit errors to the caller. Retry orchestration and external host adapters remain separate units.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-005.
- [x] Register `UNIT-CORE-013` as planned before editing source.
- [x] Add the host-boundary cleanup/commit entry point and error mapping.
- [x] Extend UT-RUNTIME-005 to exercise boundary-driven cleanup, completion ordering, and no-pending behavior.
- [x] Run focused and core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after source and registered tests pass.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md).
Validation passed: UT-RUNTIME-005, all 19 core tests, Clippy, Rust formatting, full development checks (document links, workspace builds, test harness, C/C++ probes, .NET build, Python wheel/import), and `git diff --check`.
