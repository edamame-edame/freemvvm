# Property delivery item callback invocation

- Updated: 2026-10-20
- Status: complete; focused implementation unit
- Candidate unit: `UNIT-CORE-047`

## Scope

Add sequential callback invocation to a prepared `PropertyDeliveryItem`. The item uses its existing ordered snapshot, starts only Subscriptions that remain active at their turn, and calls user code without holding Property-store or queue locks. A callback's Property update is recorded for a later generation; a subscription added during one item is absent from that item's snapshot.

This unit does not choose when the Runtime invokes delivery items, schedule a safe-point/slice, or add Binding propagation.

## Approved basis

- Requirements: [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007), [RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md), subscriber order, cancellation during delivery, and changes from callbacks.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-NOTIFY-003.

## Steps

1. Register UNIT-CORE-047 and the focused callback oracle before Rust source edits.
2. Make one `PropertyDeliveryItem` invoke its pinned subscribers sequentially through the Subscription lifecycle primitive.
3. Verify active-state recheck, callback reentry into Runtime APIs without held locks, next-generation Property updates, exclusion of new registrations from the current snapshot, and cancellation of a later snapshotted subscriber.
4. Run focused/workspace tests, repository development gate, documentation checks, and `git diff --check`; update memory and commit/push under the authorized per-unit workflow.

## Boundary

The method executes only the supplied item and returns after its snapshot is processed. Outer UI boundary detection, notification priority, the 128-item budget, callback exception/diagnostic integration, Binding propagation, and shutdown interruption remain later units.

## Verification

- `cargo test -p freemvvm-core property_delivery_item_invocation --locked`: passed (1 focused test).
- `cargo test --workspace --locked`: passed (109 tests: 30 C ABI and 79 Core Runtime).
- `.venv\\Scripts\\python.exe -X utf8 tools/dev.py check`: passed, including document links, Clippy, workspace build, C/C++ probes, C# build, and Python package checks.
- The test confirms callback reentry can update the Property and add a subscriber, the update is queued for the next generation, the new subscriber is excluded from the current snapshot, and canceling the later pinned subscriber prevents its callback and destroys its closure once.
