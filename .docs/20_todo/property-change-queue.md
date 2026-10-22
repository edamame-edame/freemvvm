# Core Property change queue

- Updated: 2026-10-19
- Status: complete; implementation and verification passed
- Candidate unit: `UNIT-CORE-040`

## Scope

Record the identity of a Property when a typed setter commits a changed value. Keep the first-change order and coalesce repeated changes to the same object identity until the pending generation is taken. Once taken, later changes are collected for the next generation. This is the Core queue foundation only; it does not dispatch subscribers or implement callback timing/lifecycle.

## Approved basis

- Requirements: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017), [RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025), [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Property notification and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1, changed Property identity, generation order, and coalescing; [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, Runtime-owned object identity and failure-atomic state update.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-NOTIFY-001.

## Steps

1. Register the focused queue oracle and `UNIT-CORE-040` before source changes.
2. Add a Runtime-owned pending Property identity queue with duplicate coalescing and `try_reserve` before committing a changed candidate.
3. Integrate changed commits with the queue; unchanged and failed updates must not add an identity.
4. Test first-change order, repeated-change coalescing, next-generation separation after taking the current generation, and no queue change for unchanged/failed updates.
5. Focused queue test passed (1 test); `cargo test --workspace --locked` passed (97 tests); `.venv\Scripts\python.exe tools/dev.py check` passed, including docs links, Rust format/clippy/build, C/C++ probes, C# build, and Python package checks. `git diff --check` passed.
6. Record implementation learning and commit/push under the previously authorized per-unit workflow.

## Boundary

Taking a generation is an internal Core operation. Subscriber registration, callback delivery, safe-point scheduling, transaction scopes, the 128-item work budget, and shutdown cancellation remain separate slices. This unit does not claim end-to-end notification behavior.
