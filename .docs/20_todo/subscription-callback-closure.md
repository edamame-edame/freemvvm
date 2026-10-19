# Subscription callback closure

- Updated: 2026-10-19
- Status: complete; implementation and verification passed
- Candidate unit: `UNIT-CORE-042`

## Scope

Connect the Subscription lifecycle state to one owned `(callback, userdata, destroy)` closure. A delivery invokes the callback only while active. Explicit cancel prevents later starts. If cancel occurs during callback execution, userdata remains owned until that callback returns, then destroy runs once. This is an internal Core object, not yet registered in Runtime storage or exposed through C ABI.

## Approved basis

- Requirements: [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007), [RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008), and [RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009).
- Basic design: [Runtime execution context, ownership, and shutdown](../12_design/runtime-objects/runtime.md), registration ownership and callback lifetime.
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md), Subscription cancellation and in-flight callback lifetime.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-SUBSCRIPTION-002.

## Steps

1. Registered UT-CORE-PROPERTY-SUBSCRIPTION-002 and `UNIT-CORE-042` before source edits.
2. Added an internal Subscription that owns callback, userdata, and destroy callback alongside its lifecycle state.
3. Callbacks run outside internal locks; destroy is called only once after cancellation and zero in-flight callbacks coincide.
4. Focused tests passed (2 tests); workspace tests passed (102 tests total).
5. `.venv\Scripts\python.exe tools/dev.py check` passed, including docs links (76 active documents, 6051 links), Rust format/clippy/build/test-harness, C/C++ probes, C# build, and Python package checks. `git diff --check` passed.
6. The Runtime UI-thread precondition and not-yet-integrated boundaries are recorded above and in the Core Runtime memory note.

## Boundary

Callers must invoke delivery/cancel on the Runtime UI thread; Runtime integration that enforces this precondition, token ownership/release, Property indexing, notification scheduling, and shutdown draining are later units. This object does not yet make Property notifications usable through the public ABI.
