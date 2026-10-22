# Core Property state memory

Scope: `UNIT-CORE-038`, RuntimeCore Property state registration/read/update.

- Keep `PropertyState` as a thin wrapper around `ValueStorage`; preserve `TypedValue::equivalent` as the only same-value oracle rather than deriving `PartialEq` for F64-bearing values.
- Validate token kind and Runtime identity through `HandleRegistry` and hold its `ValidationPin` while resolving the identity in the Runtime-owned `UiObjectStore`.
- Keep Property state reads and candidate commits UI-thread-only. Reject wrong-thread access before touching the store; worker update tests should also verify the committed value remains unchanged.
- The current internal candidate method is for already-validated values and returns `ValueCommit`; property validators, dirty aggregation, Binding propagation, notifications, and public ABI methods are separate units.
