# Core F64 value equivalence

- Updated: 2026-10-19
- Status: complete; `UNIT-CORE-029` ready after registered checks
- Candidate unit: `UNIT-CORE-029`

## Scope

Implement the pure F64 equivalence rule used by Property value comparison. Treat any two NaNs as equivalent and rely on numeric equality for signed zero, equal infinities, and ordinary finite values. This unit does not store a Property or emit notifications.

## Approved basis

- Requirements: [RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018) and [RQ-02-020](../11_requirements/RQ-02_core-features.md#rq-02-020).
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, F64 equivalence and comparison/storage separation; [Model values](../12_design/runtime-objects/model-values.md) v1.0, kind-specific numeric comparison.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-VALUE-003.

## Steps

1. Register `UNIT-CORE-029` as planned, linked to the approved F64 rules and UT-VALUE-003.
2. Add an internal pure equivalence function to the Core value module.
3. Test distinct NaN encodings, positive/negative zero, same/opposite infinities, equal finite values, and unequal finite values.
4. Run focused/workspace tests, standard development checks, documentation validation, formatting, and whitespace validation.
5. Mark ready and commit/push after registered checks pass.

## Completion record

- Added pure `f64_equivalent` comparison without changing stored values.
- Focused UT-VALUE-003 filter passed; `cargo test --workspace --locked` passed (65 tests).
- `.venv\Scripts\python.exe tools\dev.py check`, documentation validation, formatting, and `git diff --check` passed.
- Property commit and notification suppression remain outside this unit.

## Boundaries

- Preserve the original F64 representation; this comparator does not normalize stored values.
- Leave same-value Property commit/notification behavior for a later ValueStorage unit.
- Do not add F32 conversion behavior here; it remains in the existing numeric codec.
