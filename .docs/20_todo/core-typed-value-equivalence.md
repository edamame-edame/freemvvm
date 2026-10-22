# Core typed value equivalence

- Updated: 2026-10-19
- Status: complete
- Candidate unit: `UNIT-CORE-030`

## Scope

Add an internal tagged value enum for BOOL, I64, UTF8, and F64 and compare values only within the same kind. Reuse the existing UTF-8 ownership and F64 equivalence primitives. Decimal, ValueStorage commit, validation, and notifications remain separate work.

## Approved basis

- Requirements: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017), [RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018), and [RQ-02-020](../11_requirements/RQ-02_core-features.md#rq-02-020).
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, typed `ValueStorage` and comparison/storage separation.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-VALUE-003. This slice covers BOOL/I64/UTF8/F64 only; DECIMAL and Property side effects are not claimed complete.

## Steps

1. Registered `UNIT-CORE-030` with dependencies on the UTF-8 and F64 primitives.
2. Added the tagged internal value type and a pure `equivalent` operation.
3. Added tests for same-kind equality, cross-kind rejection, UTF-8 bytes, and F64 special-value rules.
4. Focused tests passed (3 tests); workspace tests passed (68 tests); `tools/dev.py check`, documentation validation, formatting, and `git diff --check` passed.
5. Unit is ready for commit/push.
