# Host outer-event shutdown dispatch boundary

- Updated: 2026-10-17
- Status: complete; design and test design approved
- Scope: `UNIT-CORE-012` connects HostRegistration's outermost `end_event` boundary to Dispatcher priority shutdown service, without starting another ordinary callback.

## Approved basis

- Requirements: [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005) and [RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Host/contextとcompletionの所有補足” / “end_eventの最外境界”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-004.

## Slice boundary

- Only a successfully closed outermost HostScope may service a pending shutdown request.
- Nested event-scope closure does not service shutdown.
- At the outer boundary, drain/cancel queued ordinary work on the UI thread and invoke the supplied shutdown handler before returning to the host.
- If no shutdown is pending, this boundary does not consume an ordinary queued callback.
- Leave platform-specific event adapters, full Runtime coordinator ownership, public ABI, and cleanup retry policy for later units.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-004.
- [x] Register `UNIT-CORE-012` as planned before editing source.
- [x] Add UI-thread Dispatcher priority service and connect it to outermost HostRegistration end_event.
- [x] Run UT-RUNTIME-004, all core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after source and registered tests pass.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md).
Validation passed: UT-RUNTIME-004, all 19 core tests, Clippy, Rust formatting, full development checks (document links, workspace builds, test harness, C/C++ probes, .NET build, Python wheel/import), and `git diff --check`.
