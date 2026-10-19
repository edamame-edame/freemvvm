# RuntimeCore UI state store shutdown cleanup

- Updated: 2026-10-19
- Status: complete
- Candidate unit: `UNIT-CORE-036`

## Scope

Make `RuntimeCore` own a UI-thread-confined `UiObjectStore` and register its clear operation with that Runtime's `RuntimeCleanupQueue`. The Runtime shutdown drain must clear all stored state on the captured UI thread before publishing `Closed`. The store and cleanup queue share one `UiThreadContext`. Keep the stored state type-erased and internal so later object-specific units can add concrete state types without changing shutdown ordering. Do not add public handle or Property APIs in this unit.

## Approved basis

- Requirements: [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001), [RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2 approved, Runtime-owned UI-thread `UiObjectStore`; [Runtime execution context, ownership, and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.2 approved, all UI state cleanup completes before `Closed`.
- Unit test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-RUNTIME-UI-STORE-001.

## Steps

1. Register `UNIT-CORE-036`, its source/test mapping, and UT-RUNTIME-UI-STORE-001 before source edits.
2. Construct one shared-context store with `RuntimeCore`; register a cleanup closure with the Runtime's queue that clears the store on shutdown.
3. Test that inserted state is dropped on the Runtime UI thread, identities stop resolving, and the queue reaches `Closed` only after cleanup. Also verify wrong-thread store access remains rejected.
4. Focused RuntimeCore shutdown test passed; workspace tests passed (85 tests); `tools/dev.py check` passed, including Rust Clippy/build/format, documentation links, C/C++ bootstrap tests, C# build, and Python package checks. `git diff --check` passed.
5. Record results and implementation learning, then commit and push this unit.

## Boundaries

- This unit does not add object-specific state APIs, public handles, or Property semantics.
- Cleanup registration failure aborts RuntimeCore construction; it must not return a partially initialized Runtime.
- Map a new internal RuntimeCore construction error to the existing internal C ABI error result.
- Destructor-panic policy remains outside this unit.
