# Runtime shutdown completion commit

- Updated: 2026-10-17
- Status: complete; design and test design approved
- Scope: `UNIT-CORE-010` connects UI-thread `RuntimeCleanupQueue::drain_shutdown` to the shutdown completion record, publishing `Completed` only after cleanup reaches `Closed`.

## Approved basis

- Requirements: [RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006) and [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Host境界・終了要求・完了観測” / “completion commit”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-005.

## Slice boundary

- Drain existing cleanup tickets and UI-state cleanup through the approved Runtime shutdown path.
- Commit `Completed` only after the cleanup queue reports `Closed`.
- Leave HostRegistration, Dispatcher wiring, public ABI request handles, and cleanup retry orchestration out of this internal join point.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-005.
- [x] Register `UNIT-CORE-010` as planned before editing source.
- [x] Implement the UI-thread cleanup-to-completion commit operation.
- [x] Run UT-RUNTIME-005, all core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after source and registered tests pass.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md). UT-RUNTIME-005, all 19 core tests, workspace formatting/lint/build checks, C/C++ probes, .NET build, Python wheel build/import, document links, and `git diff --check` passed.
