# Property delivery snapshot preparation

- Updated: 2026-10-20
- Status: complete; implementation and verification passed
- Candidate unit: `UNIT-CORE-046`

## Scope

Prepare one ordered pending Property generation as `PropertyDeliveryItem`s. Each item carries its Property identity and the active subscriber snapshot in registration order. Preparation only pins callback targets; it does not invoke callbacks or schedule a UI-loop slice.

Generation preparation must be failure-atomic: if item/snapshot allocation or Property resolution fails, restore the full identity generation in its original order so a committed change is not silently lost. Hold locks in the existing Runtime order (Property store, then change queue) and release the Property store lock before the returned items can be processed.

## Approved basis

- Requirements: [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md), Notification queue and subscription; DeliveryItem snapshot and ordered callback targets.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-NOTIFY-002.

## Steps

1. Register UNIT-CORE-046 and the focused generation/snapshot oracle before Rust source edits.
2. Add a delivery item carrying Property identity and strong pins from its ordered weak subscriber snapshot.
3. Prepare the whole pending generation before consuming it; restore all identities on lookup or snapshot failure.
4. Verify changed-Property order, subscriber order, snapshot pin lifetime, no callback invocation during preparation, next-generation separation, and generation restoration after a stale identity error.
5. Run focused/workspace tests, repository development gate, documentation checks, and `git diff --check`; update memory and commit/push under the authorized per-unit workflow.

## Verification result

- Focused: `cargo test -p freemvvm-core property_delivery_generation_snapshot --locked` — 2 passed.
- Workspace: `cargo test --workspace --locked` — 108 passed (30 C ABI, 78 Core Runtime).
- Development gate: `.venv\Scripts\python.exe tools/dev.py check` — passed, including docs links, formatting/lint, Rust clippy/build/test harness, C/C++ probes, C# build, and Python wheel/import checks.
- Unit inventory validation, documentation links, and `git diff --check` — passed.

## Boundary

This unit creates delivery-ready snapshots only. It does not invoke subscribers, process nested setter changes, schedule at outer UI boundaries, enforce the 128-item slice budget, integrate Binding propagation, or implement shutdown cancellation.
