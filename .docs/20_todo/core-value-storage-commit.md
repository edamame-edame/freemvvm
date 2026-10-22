# Core typed value storage commit

- Updated: 2026-10-19
- Status: complete
- Candidate unit: `UNIT-CORE-032`

## Scope

Add private `ValueStorage` around a typed initial value. It accepts only already-validated candidates, rejects a different value kind without mutation, retains the current representation when values are equivalent, and commits a changed same-kind candidate. Validator invocation, dirty tracking, Binding propagation, and notification delivery remain separate units.

## Approved basis

- Requirements: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002), [RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018), [RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019), and [RQ-02-020](../11_requirements/RQ-02_core-features.md#rq-02-020), RQ-02 v0.5 approved.
- Basic design: [Typed value areas and comparison/conversion](../12_design/runtime-objects/model-values.md) v1.0 approved, validate then compare and commit only on a real difference.
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2 approved, `ValueStorage` typed value and representation-preserving comparison.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-VALUE-006. This unit does not claim Property validation or notification behavior complete.

## Steps

1. Registered `UNIT-CORE-032` and `UT-VALUE-006` before editing implementation.
2. Added a kind-aware `ValueStorage` update operation for prevalidated typed candidates.
3. Verified same-value representation retention, changed-value commit across all kinds, and cross-kind no-mutation behavior.
4. Focused tests passed (3 tests); workspace tests passed (75 tests); `tools/dev.py check`, documentation validation, formatting, and `git diff --check` passed.
5. Unit is ready for commit/push.
