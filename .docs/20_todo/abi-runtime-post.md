# C ABI Runtime post and host dispatch turn

- Updated: 2026-10-19
- Status: complete for UNIT-ABI-008; behavioral and workspace verification passed
- Candidate unit: `UNIT-ABI-008`

## Scope

Connect the already specified `runtime_post` behavior to the C ABI and host-loop bridge. A successful post transfers callback/userdata/destroy ownership to Core; callbacks execute serially on the Runtime UI thread, and destroy runs exactly once after execution or cancellation. A rejected post leaves ownership with the caller.

## Approved basis

- Requirements: [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006), [RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003), [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001), [RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002), and [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, especially FIFO post queue, one callback per non-reentrant dispatch turn, and shutdown priority.
- ABI design: [Runtime ABI](../13_detail_design/runtime-objects/abi.md) v1.1, `runtime_post` declaration and ownership/status rules; bridge supplement from UNIT-ABI-007.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), dispatcher FIFO/wake and post ownership test oracles.

## Adopted dispatch entrypoint

The approved design says the bridge asks the host to schedule a future turn, and that a turn invokes at most one callback. The C ABI currently has no public operation for the host to enter that turn. `runtime_host_end_event` currently closes the scope and commits pending shutdown, but does not drain ordinary post work. Implementing `runtime_post` alone would therefore accept callbacks that no public call can execute.

The user selected the recommended UI-thread-only `runtime_dispatch_turn(runtime)` C ABI operation. The host calls it from the future turn scheduled by `UI_WAKE_NORMAL`; it executes at most one callback, does not re-enter an active turn, and returns with later queued work eligible for a new wake. Pending shutdown retains priority at the outer event boundary and is never run inside the bridge callback. Each dispatch call creates and closes a nested host event scope, allowing an existing outer host scope to remain authoritative for shutdown commit.

The alternative of making `runtime_host_end_event` dispatch a callback was not selected; its shutdown-boundary behavior stays unchanged.

Before implementation:

1. Record the selected API in DD v1.2 and update approval/revision metadata.
2. Add a focused test-design supplement for acceptance ownership, UI-thread dispatch, FIFO/non-reentrancy, one callback per turn, wake coalescing, shutdown cancellation, and failure ownership.
3. Register `UNIT-ABI-008` as planned in `tools/units.yaml`.
4. Implement the Core ABI support and C ABI entrypoint; preserve C callback non-unwind and thread-safe userdata requirements.
5. Run the focused behavior tests, workspace tests/checks, documentation validation, formatting, and `git diff --check`.
6. Mark ready and commit/push only after the registered behavior checks pass.

## Completion record

- Implemented the C ABI `runtime_post` and `runtime_dispatch_turn` entrypoints; normal dispatch runs one callback on the UI thread within a nested host event scope.
- Shutdown-pending dispatch skips ordinary callbacks and lets the outer host boundary cancel queued work and invoke each destroy callback once. A new control request emits a Control wake even when a Normal wake is already pending.
- Registered feature 7 (`runtime.post_dispatch.v1`) only after both entrypoints and their ownership/cancellation behavior were implemented.
- Focused tests, `cargo test --workspace`, `tools/dev.py check`, documentation validation, formatting, and diff checks passed on the pending revision. Commit/push follows the user's established per-unit authorization.

## Decision

Adopted on 2026-10-19: add `runtime_dispatch_turn(runtime)`; preserve `runtime_host_end_event` as the host event boundary that commits shutdown, without draining regular post work itself.
