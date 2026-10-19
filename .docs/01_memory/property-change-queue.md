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

## Runtime Subscription owner registry (`UNIT-CORE-043`)

- `RuntimeCore` holds active Subscription `Arc`s in registration-ordered vectors keyed by Property identity. Registration reserves capacity in the owner registry and Property weak index before publishing either entry, rejects reuse of one Subscription pointer, and preserves caller ownership on failure.
- `PropertyState` receives the matching weak entry on successful Runtime registration. Snapshot upgrades active entries in registration order into temporary `Arc` pins. Cancel removes only the selected owner and matching weak index entry while holding state locks in store-then-owner order, then calls `Subscription::cancel` after releasing both locks. These operations require the Runtime UI thread.
- This registry is not yet wired to public token release, Property teardown, notification scheduling, or Runtime shutdown draining.
- Focused tests confirm Runtime ownership survives dropping the registering caller's `Arc`; the callback is pinned and invoked outside the map lock, and explicit cancel removes the owner before destroying the closure.
- `UNIT-CORE-045` / `UT-CORE-PROPERTY-SUBSCRIPTION-005` verifies multiple ordered subscriptions per Property, caller-owner drop, snapshot pinning, individual cancel, duplicate pointer rejection, and wrong-thread rejection. Focused and related tests passed (3 total); workspace passed (106 tests); development gate passed.

## Property Subscription weak index (`UNIT-CORE-044`)

- `PropertyState` keeps only `Weak<Subscription>` entries, so it is an index rather than an owner. Adding the same Subscription pointer is idempotent; capacity is reserved before appending.
- `snapshot_subscriptions` reserves temporary snapshot capacity first, upgrades live entries, retains only active Subscriptions, and returns strong pins for later delivery after the caller releases the Property store lock. Expired and canceled entries are pruned.
- Focused oracle `UT-CORE-PROPERTY-SUBSCRIPTION-004`: 1 passed. Workspace: 105 tests passed; `tools/dev.py check` passed.
- This is not yet connected to Runtime registration or notification delivery. Scheduling, callback dispatch, public subscribe/cancel ABI, and shutdown drain remain follow-on work.
