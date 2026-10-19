# Property Subscription weak index

- Updated: 2026-10-19
- Status: complete; implementation and verification passed
- Candidate unit: `UNIT-CORE-044`

## Scope

Give each `PropertyState` a weak index of its Subscriptions. Registration stores only a `Weak<Subscription>` so Property does not become an owner. Snapshotting upgrades live active entries into temporary strong references for delivery outside the Property store lock, and prunes canceled or expired entries. This unit does not schedule or invoke notifications.

## Approved basis

- Requirements: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017), [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002), [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md), Property weak subscriber index and delivery snapshot.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-SUBSCRIPTION-004.

## Steps

1. Register the focused weak-index oracle and `UNIT-CORE-044` before source edits.
2. Add failure-atomic weak registration and active snapshot/pruning to `PropertyState`.
3. Verify the Property index does not keep a canceled/dropped Subscription alive, snapshots pin active entries, and canceled entries are omitted.
4. Run focused/workspace tests, repository development gate, and `git diff --check`; update memory and commit/push under the authorized per-unit workflow.

## Verification result

- Focused: `cargo test -p freemvvm-core property_subscription_weak_index --locked` — 1 passed.
- Workspace: `cargo test --workspace --locked` — 105 passed (30 C ABI, 75 Core Runtime).
- Development gate: `.venv\Scripts\python.exe tools/dev.py check` — passed, including docs links, formatting/lint, Rust clippy/build/test harness, C/C++ probes, C# build, and Python wheel/import checks.
- `git diff --check` — passed.

## Boundary

The snapshot is an internal temporary owner only. Notification generation order, callback delivery, UI safe-point scheduling, public subscribe/cancel ABI, and shutdown drain remain separate units.
