# Core Property state

- Updated: 2026-10-19
- Status: complete
- Candidate unit: `UNIT-CORE-038`

## Scope

Connect the approved initial Property value storage to Runtime-owned UI object registration. A Property is registered with an initial typed value, resolved through a validated Runtime handle on the UI thread, read as an owned value, and updated using the existing `ValueStorage` kind/equivalence/commit rules. Do not add property validators, dirty tracking, Binding propagation, change notifications, public C ABI entrypoints, or wrapper APIs in this unit.

## Approved basis

- Requirements: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002), [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017), [RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018), [RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Basic design: [Property values and update consistency](../12_design/runtime-objects/model-values.md) v1.0, Property commit and UI-thread boundary.
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, `ValueStorage` and Runtime-owned `UiObjectStore`.
- Unit-test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-CORE-PROPERTY-001.

## Steps

1. Add the focused unit test design and register `UNIT-CORE-038` before changing Rust source.
2. Add an internal `PropertyState` around `ValueStorage` and a typed internal property handle kind.
3. Add RuntimeCore operations to register, read, and apply an already-validated candidate. Validate token kind and Runtime identity, hold the validation pin for the state access, and enforce UI-thread access.
4. Test initial value round trip, changed and equivalent updates, kind mismatch without mutation, invalid/wrong-kind/wrong-Runtime tokens, worker access, and closed Runtime behavior. Continue using existing Runtime UI-store cleanup.
5. Focused Property test passed; `cargo test --workspace --locked` passed (87 tests); `.venv\Scripts\python.exe tools/dev.py check` passed; documentation/link checks and `git diff --check` passed.
6. Record the implementation learning. The implementation and verification scope is complete; repository publication follows the previously authorized per-unit workflow.

## Boundaries

- Candidate values passed to this internal unit are assumed already validated. Validator invocation and validation outcomes are a later unit.
- This unit does not publish API signatures or define notification scheduling/dirty state; those require separate approved implementation slices.
- `UNIT-*` and `UT-*` are implementation/test tracking identifiers only; no design-decision IDs are introduced.
