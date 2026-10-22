# Core Property kind query memory

Scope: `UNIT-CORE-039`, internal RuntimeCore kind inspection for registered Properties.

- Derive the kind from the initial `TypedValue`; `PropertyState` should not store a second mutable kind field that can drift from its `ValueStorage`.
- Resolve the token through the expected Property handle kind and Runtime identity, retain the validation pin, then enforce UI-thread access before resolving the Runtime-owned state.
- Keep stable public C ABI kind numbers out of the Core `ValueKind` enum; map them in the ABI adapter when that API slice is implemented.
- OBJECT kind 4 is reserved by the ABI design and is not a constructible Core `TypedValue`.
