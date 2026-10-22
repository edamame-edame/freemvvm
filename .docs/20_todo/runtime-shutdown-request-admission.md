# Shutdown request admission and priority dispatch

- Updated: 2026-10-17
- Status: complete; design and test design approved
- Scope: `UNIT-CORE-014` admits a prepared ShutdownCompletion and arms Dispatcher priority shutdown as one host-serialized request operation.

## Approved basis

- Requirements: [RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003), [RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006), and [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Prepared → Pending”, host-stop race, and “Host/contextとcompletionの所有補足”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-005.

## Slice boundary

- While HostRegistration is Available, reserve completion registry capacity, arm/coalesce Dispatcher shutdown, then register the completion under the host state lock.
- Schedule the host wake only after releasing the host state lock.
- Reject stopped/stopping hosts and closed dispatchers without registering a Pending completion or leaving a new shutdown flag behind.
- Keep completion allocation, retry-after-stop wake rearming, public request handles, and ABI entry points outside this slice.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-005.
- [x] Register `UNIT-CORE-014` as planned before editing source.
- [x] Separate Dispatcher shutdown flag admission from bridge wake scheduling.
- [x] Add HostRegistration request admission with stopped/closed rejection and coalescing.
- [x] Extend UT-RUNTIME-005 for worker admission, stopped-host rejection, and priority completion.
- [x] Run focused and core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after source and registered tests pass.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md).
Validation passed: UT-RUNTIME-005, all 19 core tests, Clippy, Rust formatting, full development checks (document links, workspace builds, test harness, C/C++ probes, .NET build, Python wheel/import), and `git diff --check`.
