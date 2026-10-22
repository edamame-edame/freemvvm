# Runtime HostRegistration scopes and stop notification

- Updated: 2026-10-17
- Status: complete; design and test design approved
- Scope: `UNIT-CORE-011` implements UI-thread event scope pairing/depth, outermost-boundary detection, host stopping/stopped/available states, and fan-out of stop reasons to registered completion records.

## Approved basis

- Requirements: [RQ-06-007](../11_requirements/RQ-06_threading-dispatch.md#rq-06-007) and [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Host/contextとcompletionの所有補足” / “HostScopeの対応”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-005.

## Slice boundary

- Scope IDs are monotonically issued, non-reused, and popped only in LIFO order on the Runtime creation thread.
- Invalid end_event calls leave scope depth unchanged; the outermost successful end reports a safe boundary.
- before_stop requires an empty scope stack. stopped may be reported from any thread and marks registered pending completion records HostUnavailable with the reason.
- A host can resume on the UI thread; each stopped request must be explicitly re-armed on that thread while the host is available.
- Leave platform adapters, Runtime/Dispatcher wiring, public ABI calls, and priority shutdown execution at end_event to later units.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-005.
- [x] Register `UNIT-CORE-011` as planned before editing source.
- [x] Implement HostRegistration state, event scope pairing, stop notification, resume, and explicit retry.
- [x] Run UT-RUNTIME-005, all core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after source and registered tests pass.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md). UT-RUNTIME-005, all 19 core tests, workspace formatting/lint/build checks, C/C++ probes, .NET build, Python wheel build/import, document links, and `git diff --check` passed.
