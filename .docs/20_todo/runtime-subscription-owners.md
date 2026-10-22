# Runtime-owned Subscription registry

- Updated: 2026-10-19
- Status: complete; implementation and verification passed
- Candidate unit: `UNIT-CORE-043`

## Scope

Add a Runtime-owned active Subscription collection keyed by the subscription object's `ObjectIdentity`. Registration stores an independent strong `Arc`, lookup pins the object for delivery outside the registry lock, and explicit cancel removes the Runtime owner before canceling the closure. Operations are UI-thread-only. This does not create public handles, Property weak indexes, schedule notifications, or drain registrations during Runtime shutdown.

## Approved basis

- Requirements: [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002), [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Basic design: [Runtime execution context, ownership, and shutdown](../12_design/runtime-objects/runtime.md), Runtime registration ownership and UI-thread operations.
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md), Runtime strong owner and Subscription cancel.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-SUBSCRIPTION-003.

## Steps

1. Registered UT-CORE-PROPERTY-SUBSCRIPTION-003 and `UNIT-CORE-043` before source edits.
2. Added a Runtime-local identity-to-Subscription strong-owner map with fallible capacity reservation and duplicate protection.
3. Added UI-thread-checked register, pin-for-delivery, and cancel operations; callback/destroy runs outside the Runtime registry lock.
4. Focused tests passed (2 tests); workspace tests passed (104 tests total). Tests cover caller Arc drop, callback pin, exactly-once destroy, duplicate preservation, and worker-thread rejection.
5. `.venv\Scripts\python.exe tools/dev.py check` passed, including docs links (76 active documents, 6057 links), Rust format/clippy/build/test-harness, C/C++ probes, C# build, and Python package checks. `git diff --check` passed.
6. Recorded the Runtime-owner boundary and follow-up integration needs in Core Runtime memory.

## Boundary

This internal owner map establishes independent Runtime ownership, but is not yet wired to public token release, Property teardown, notification queue delivery, or shutdown drain. Those require follow-up integration units.
