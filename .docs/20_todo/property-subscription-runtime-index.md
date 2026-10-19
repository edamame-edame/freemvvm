# Runtime Property Subscription index integration

- Updated: 2026-10-19
- Status: complete; implementation and verification passed
- Candidate unit: `UNIT-CORE-045`

## Scope

Connect Runtime subscription registration to the owning `PropertyState` weak index. A Property can have multiple active subscriptions in registration order; Runtime retains each as a strong owner, while Property stores only weak references. Cancel removes and cancels exactly the selected registration. Snapshotting through the Property index must preserve order and pin entries only for the snapshot lifetime.

The unit remains internal: it does not add public handles/C ABI, notification scheduling, callback delivery from a Property change, or shutdown draining.

## Approved basis

- Requirements: [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002), [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md), Property weak subscriber index and delivery snapshot; ordered registration and cancellation; [C ABI](../13_detail_design/runtime-objects/abi.md), Property subscription and cancellation contract.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-SUBSCRIPTION-005.

## Steps

1. Register UNIT-CORE-045 and its focused test oracle before changing Rust source.
2. Change Runtime ownership from one Subscription per Property to an ordered collection; wire successful registration into the Property weak index with failure-atomic reservation.
3. Cancel one selected Subscription without removing its siblings; keep callback/destroy outside owner and Property locks.
4. Verify registration order, Runtime retention after caller owners drop, exact cancellation/pruning, duplicate protection, and wrong-thread rejection.
5. Run focused/workspace tests, repository development gate, documentation checks, and `git diff --check`; update memory and commit/push under the authorized per-unit workflow.

## Verification result

- Focused: `cargo test -p freemvvm-core runtime_property_subscription_index --locked` — 1 passed; related owner tests — 2 passed.
- Workspace: `cargo test --workspace --locked` — 106 passed (30 C ABI, 76 Core Runtime).
- Development gate: `.venv\Scripts\python.exe tools/dev.py check` — passed, including docs links, formatting/lint, Rust clippy/build/test harness, C/C++ probes, C# build, and Python wheel/import checks.
- Unit inventory validation, documentation links, and `git diff --check` — passed.

## Boundary

This joins two existing internal structures and does not invoke callbacks during snapshot. Property teardown, pending delivery scheduling, callback delivery from `PropertyChangeQueue`, public token/ABI integration, and Runtime shutdown draining remain later units.
