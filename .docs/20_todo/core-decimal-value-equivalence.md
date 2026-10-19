# Core decimal value representation and equivalence

- Updated: 2026-10-19
- Status: complete
- Candidate unit: `UNIT-CORE-031`

## Scope

Add an internal Decimal value that retains sign, scale, and coefficient, validates the approved range (scale 0–31 and coefficient 0–2^58−1), and compares numeric equivalence without rewriting its stored representation. Add DECIMAL as a `TypedValue` kind. Property commit, validation callbacks, and notifications remain separate work.

## Approved basis

- Requirements: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017) and [RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019), RQ-02 v0.5 approved.
- Basic design: [Typed value areas and comparison/conversion](../12_design/runtime-objects/model-values.md) v1.0 approved, Decimal sign/scale/coefficient and representation-preserving same-value behavior.
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2 approved, Decimal range and normalization/equivalence boundary.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-VALUE-005. Wire encoding, parsing/formatting, and Property side effects are out of scope.

## Steps

1. Registered `UNIT-CORE-031` and `UT-VALUE-005` before editing implementation.
2. Added a validated internal Decimal representation and numeric equivalence that leaves stored fields unchanged.
3. Added Decimal to `TypedValue` equivalence, including cross-kind rejection.
4. Focused tests passed (4 tests); workspace tests passed (72 tests); `tools/dev.py check`, documentation validation, formatting, and `git diff --check` passed.
5. Unit is ready for commit/push.
