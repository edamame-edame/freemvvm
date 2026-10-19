# Core Runtime: Property change queue

Scope: `UNIT-CORE-040`, Runtime-owned accumulation of changed Property identities.

- `PropertyChangeQueue` keeps the first-change order and coalesces repeated `ObjectIdentity` values until `take_generation`; changes afterward form the next pending generation.
- `RuntimeCore::apply_property_candidate` checks equivalence and reserves queue capacity while the Property state is locked, before committing a changed value. Queue reservation failure therefore leaves the Property unchanged.
- Only `ValueCommit::Changed` records an identity. Kind mismatch, unchanged values, invalid token, and wrong-thread access do not add entries.
- This is internal queue state only. It does not yet schedule callback delivery, implement subscriber lifecycle, or define transaction/safe-point processing.

## Subscription lifecycle state (`UNIT-CORE-041`)

- `SubscriptionLifecycle` is only the active/in-flight state primitive. `begin_delivery` rejects starts after cancellation; cancellation defers destroy readiness while a started delivery is in flight; the final `finish_delivery` after cancellation produces destroy readiness once. The focused oracle covers these transitions and repeated cancel/finish.
- It does not own or invoke callback/userdata/destroy, register an ABI token, or perform UI-thread scheduling. Those remain explicit follow-on integration work.

## Subscription callback closure (`UNIT-CORE-042`)

- `Subscription` owns the callback/userdata/destroy tuple and composes it with the lifecycle primitive. It invokes user code outside its internal locks; self-cancel leaves userdata alive until callback return, when destroy runs once. Focused tests cover ordinary cancel, post-cancel rejection, and self-cancel.
- Delivery/cancel thread affinity is a caller precondition until Runtime integration enforces it. Runtime registry ownership, ABI token behavior, notification scheduling, and shutdown remain later work.
