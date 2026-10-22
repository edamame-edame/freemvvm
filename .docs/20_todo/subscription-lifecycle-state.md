# Subscription lifecycle state

- Updated: 2026-10-19
- Status: complete; implementation and verification passed
- Candidate unit: `UNIT-CORE-041`

## Scope

Implement the Core state transitions required by Property Subscription cancellation: active registrations may start deliveries; cancellation prevents future delivery starts; an already-started delivery remains in flight; destruction becomes eligible exactly when cancellation has occurred and the final in-flight delivery has ended. This is only the lifecycle state primitive. It does not register ABI handles, invoke callbacks, run destroy callbacks, or schedule notification delivery.

## Approved basis

- Requirements: [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007), [RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008), and [RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009).
- Basic design: [Runtime execution context, ownership, and shutdown](../12_design/runtime-objects/runtime.md), registration ownership and callback lifetime.
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md), Subscription active state, cancel, and in-flight callback ownership.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-SUBSCRIPTION-001.

## Steps

1. Registered UT-CORE-PROPERTY-SUBSCRIPTION-001 and `UNIT-CORE-041` before source edits.
2. Added a thread-safe lifecycle primitive with active/in-flight transitions and an explicit one-time destroy-ready result.
3. Focused tests passed (3 tests); workspace tests passed (100 tests total).
4. `.venv\Scripts\python.exe tools/dev.py check` passed, including docs links (76 active documents, 6045 links), Rust format/clippy/build/test-harness, C/C++ probes, C# build, and Python package checks. `git diff --check` passed.
5. Recorded the implementation boundary and learning. Commit/push follows the authorized per-unit workflow.

## Boundary

This unit does not provide a usable subscription API or notification behavior. The next slices must connect the primitive to Runtime-owned subscription storage, public token release/cancel, callback invocation, and UI-thread destruction without weakening the approved ownership rules.
