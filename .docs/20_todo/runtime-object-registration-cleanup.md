# Runtime object registration and cleanup

- Updated: 2026-10-19
- Status: complete
- Candidate unit: `UNIT-CORE-037`

## Scope

Add a UI-thread-only `RuntimeCore` operation that constructs UI state on its owner thread, inserts it into the Runtime's `UiObjectStore`, creates a matching `CoreObjectControl`, and registers that control in a supplied `HandleRegistry`. Return the token only after all registration steps succeed. If registration fails, destroy the just-inserted state on the UI thread and preserve the original registry error. Bind the control's cleanup queue to the Runtime store so an accepted final-release ticket destroys that identity on the UI thread before marking the control's UI state cleaned. Keep handle kind generic; this unit does not define public object kinds or expose a C ABI constructor.

## Approved basis

- Requirements: [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001), [RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010), [RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2 approved, `CoreObjectControl` identity, `HandleRegistry` registration and cleanup tickets; [Runtime execution context, ownership, and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.2 approved, UI-thread creation/destruction and cleanup queue.
- Unit test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-RUNTIME-OBJECT-REG-001.

## Steps

1. Register `UNIT-CORE-037`, source/test mapping, and UT-RUNTIME-OBJECT-REG-001 before source edits.
2. Add a Runtime cleanup adapter that delegates tracking/reservation and wraps each object finalizer to destroy its store identity before invoking the control completion callback.
3. Add RuntimeCore registration that checks UI thread and Running state before constructing state, inserts it, creates the control, and registers its handle. Roll back state on any registry failure.
4. Test successful registration, typed resolution through the control identity, rejection of worker/Closed registration before invoking the factory, final release from a worker followed by UI-thread drain, and deterministic registration failure with rollback.
5. Focused registration and existing control-routing tests passed; workspace tests passed (86 tests); `tools/dev.py check` passed, including Rust Clippy/build/format, documentation links, C/C++ bootstrap tests, C# build, and Python package checks. `git diff --check` passed.
6. Record results and implementation learning, then commit and push this unit.

## Boundaries

- This unit does not define a concrete public handle-kind enum, process-global object registry, C ABI constructor, or object-specific API.
- Runtime cleanup tickets are currently drained at shutdown; this unit preserves that established scheduling behavior.
- Registration allocation failures remain reported by the existing `RegistryError` taxonomy; no public error code is added.
