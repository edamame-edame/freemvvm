# Runtime shutdown completion record

- Updated: 2026-10-17
- Status: complete; design and test design approved
- Scope: `UNIT-CORE-009` implements the internal `Pending` / `Completed` / `HostUnavailable(reason)` completion record, thread-independent query/wait, and explicit retry on the Runtime creation thread.

## Approved basis

- Requirements: [RQ-06-007](../11_requirements/RQ-06_threading-dispatch.md#rq-06-007) and [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Host境界・終了要求・完了観測” / “CompletionRecord”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-005.

## Slice boundary

- Store a shared completion state independent of Runtime UI objects, observable and waitable from any thread.
- Host stop changes Pending to HostUnavailable with a reason and wakes waiters without claiming cleanup completion.
- Cleanup completion is terminal and wins a race with host-stop notification once committed.
- Only the Runtime creation/UI thread may explicitly move HostUnavailable back to Pending for retry.
- Leave host lifecycle registration, dispatcher wiring, public ABI handles, and actual cleanup retry orchestration for later units.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-005.
- [x] Register `UNIT-CORE-009` as planned before editing source.
- [x] Implement the completion record state, query, wait, host-stop, completion, and UI retry operations.
- [x] Run UT-RUNTIME-005, all core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after source and registered tests pass.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md). UT-RUNTIME-005, all 19 core tests, workspace formatting/lint/build checks, C/C++ probes, .NET build, Python wheel build/import, document links, and `git diff --check` passed.
