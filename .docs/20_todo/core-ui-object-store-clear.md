# Core Runtime UI object store clear

- Updated: 2026-10-19
- Status: complete
- Candidate unit: `UNIT-CORE-035`

## Scope

Add a UI-thread-only, idempotent operation to remove every live value from `UiObjectStore` without allocating. After a successful clear, every previously issued `ObjectIdentity` must fail resolution, and dropping the store's values must happen on its captured UI thread. This is a store primitive for the approved Runtime shutdown cleanup contract; connecting the store to `RuntimeCore` and its cleanup queue is a separate unit. Do not change public handle ownership or Property behavior.

## Approved basis

- Requirements: [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001) and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2 approved, `UiObjectStore` UI-thread confinement and UI-thread destruction; [Runtime execution context, ownership, and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.2 approved, shutdown drains tracked UI cleanup before Closed.
- Unit test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-OBJECT-STORE-002.

## Steps

1. Register `UNIT-CORE-035`, its source/test mapping, and UT-OBJECT-STORE-002 before source edits.
2. Add `UiObjectStore::clear` with UI-thread validation before mutation, no allocation, and idempotent behavior.
3. Verify successful clear drops all live values exactly once and invalidates their identities; verify a worker-thread clear fails without dropping or mutating values.
4. Run focused tests, workspace tests, format, lint, documentation checks, and `git diff --check`.
5. Focused object-store tests passed (7 tests); workspace tests passed (84 tests); `tools/dev.py check` passed, including Rust Clippy/build/format, documentation links, C/C++ bootstrap tests, C# build, and Python package checks. `git diff --check` passed.
6. Record the result and implementation learning, then commit and push this unit.

## Boundaries

- This unit does not register a shutdown callback or change `RuntimeCore` ownership.
- This unit does not decide destructor-panic policy or change `RuntimeCleanupQueue` semantics.
- Passing static documentation validation does not count as a product test.
