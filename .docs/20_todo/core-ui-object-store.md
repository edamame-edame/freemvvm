# Core Runtime UI object store

- Updated: 2026-10-19
- Status: complete
- Candidate unit: `UNIT-CORE-034`

## Scope

Add a Runtime UI-thread-confined arena that stores state by the existing `ObjectIdentity`. Resolution and destruction require the current generation; reusing a slot advances its generation so a stale identity cannot resolve to a later object. Reject off-thread access. Do not integrate Property APIs or change public handle ownership in this unit.

## Approved basis

- Requirements: [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001), [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2 approved, `UiObjectStore` UI-thread confinement, generation-checked resolution, and generation advancement on slot reuse.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-OBJECT-STORE-001.

## Steps

1. Registered `UNIT-CORE-034` and `UT-OBJECT-STORE-001` before editing implementation.
2. Added the UI-thread-confined generic arena using `ObjectIdentity`.
3. Tested insert/resolve/destroy, stale identity after reuse, off-thread rejection, and generation exhaustion without wraparound.
4. Focused tests passed (5 tests); workspace tests passed (82 tests); `tools/dev.py check`, documentation validation, formatting, and `git diff --check` passed.
5. Unit is ready for commit/push.
