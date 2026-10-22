# Core Property kind query

- Updated: 2026-10-19
- Status: complete
- Candidate unit: `UNIT-CORE-039`

## Scope

Expose an internal RuntimeCore query that returns the fixed `ValueKind` of a registered Property. The initial typed value determines the kind. Validate token kind and Runtime identity, hold the `ValidationPin` while resolving state, and keep UI-store access on the Runtime UI thread. Do not map kind to a public C ABI integer or add Property create/get/set entrypoints here.

## Approved basis

- Requirements: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017) and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Property notification and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1, initial Property kinds and `property_kind`; [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, token validation pin and UI-thread store resolution.
- Unit-test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-KIND-001.

## Steps

1. Add the focused test design and register `UNIT-CORE-039` before source edits.
2. Add a typed kind query through `TypedValue`, `PropertyState`, and `RuntimeCore` without changing value equality or public ABI.
3. Test BOOL, I64, UTF8, F64, and DECIMAL; reject invalid, wrong-kind, wrong-Runtime, worker-thread, and closed-object access without modifying state.
4. Focused kind-query test passed; `cargo test --workspace --locked` passed (88 tests); `.venv\Scripts\python.exe tools/dev.py check` passed; documentation/link checks and `git diff --check` passed.
5. Record the implementation learning. The implementation and verification scope is complete; repository publication follows the previously authorized per-unit workflow.

## Boundaries

- OBJECT remains reserved and is not a `TypedValue` kind.
- Public kind numeric IDs and C ABI output initialization remain for the C ABI slice.
- This query does not expose Property values, invoke validators, or generate notifications.
