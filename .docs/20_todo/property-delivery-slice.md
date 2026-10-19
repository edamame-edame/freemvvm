# Property delivery slice processing

- Updated: 2026-10-20
- Status: complete; focused implementation unit
- Candidate unit: `UNIT-CORE-048`

## Scope

Process prepared Property delivery items in order with the approved limit of at most 128 completed items per slice. Keep any unprocessed items in the same order for the next slice; after draining that prepared generation, later Property changes may form subsequent generations. Return whether work remains so a later host-boundary unit can schedule another turn.

This unit does not connect slices to host safe points or wake scheduling, integrate Binding propagation, classify callback exceptions, or drain notifications during shutdown.

## Approved basis

- Requirements: [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007), [RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md), “Notification queueと購読” and “有限作業枠と保留”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-NOTIFY-004.

## Steps

1. Register UNIT-CORE-048 and the focused 129-item slice oracle before source edits.
2. Add Runtime-owned pending delivery state and a UI-thread slice processor that invokes at most 128 complete items without holding queue/store locks across callbacks.
3. Verify worker-thread rejection, non-reentrant callback processing, and that a 129-item generation runs as 128 then 1, preserving order and delivering every item once.
4. Run the focused/workspace tests, repository development gate, documentation checks, and `git diff --check`; update memory and commit/push under the authorized per-unit workflow.

## Boundary

The result reports pending work but does not itself schedule a wake. The next host-integration unit will place slices at the approved outer UI boundary and yield to ordinary UI work after the limit. Binding work budget composition and shutdown interruption remain later units.

## Verification

- `cargo test -p freemvvm-core property_delivery_ --locked`: passed (4 related tests, including UT-CORE-PROPERTY-NOTIFY-004).
- `cargo test --workspace --locked`: passed (110 tests: 30 C ABI and 80 Core Runtime).
- `.venv\\Scripts\\python.exe -X utf8 tools/dev.py check`: passed, including document links, Clippy, workspace build, C/C++ probes, C# build, and Python package checks.
- UT-CORE-PROPERTY-NOTIFY-004 delivered 129 callbacks in order across two slices (128 then 1), observed no callback reentry, and confirmed no remaining work after the second slice.
- A worker-thread attempt returned `WrongThread` before starting any callback.
