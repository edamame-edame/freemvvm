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

## Property delivery snapshot preparation (`UNIT-CORE-046`)

- `PropertyDeliveryItem` contains the changed Property identity and strong subscriber pins in registration order. Runtime prepares the whole pending generation on the UI thread while following the store-then-change-queue lock order; the method does not invoke callbacks.
- Item-vector capacity is reserved before taking the generation. If resolving any Property or creating any subscriber snapshot fails, the complete identity generation is restored in order and no partial item vector is returned.
- A successful prepare consumes only the current generation. Later setter changes form a separate next generation. Returned items retain subscriber `Arc`s until the consumer drops them.
- Focused oracle `UT-CORE-PROPERTY-NOTIFY-002`: 2 passed, including stale-identity restoration. Workspace: 108 tests passed; `tools/dev.py check` passed.
- Callback invocation, nested setter handling, outer UI safe-point scheduling, the 128-item slice budget, Binding propagation, and shutdown cancellation remain later work.

## Property delivery item invocation (`UNIT-CORE-047`)

- `PropertyDeliveryItem::deliver` walks its prepared strong subscriber pins in order and calls `Subscription::invoke`; each invocation rechecks activity at start. User callbacks run without Runtime store/queue locks held by this method.
- A callback can update the Property and register another subscriber. The update becomes a later queue generation, and the new subscriber is absent from the already-prepared item. If an earlier callback cancels a later pinned subscriber, the later `invoke` observes cancellation and skips its callback.
- Focused oracle `UT-CORE-PROPERTY-NOTIFY-003`: 1 passed. Workspace: 109 tests passed; `tools/dev.py check` passed.
- This unit processes only one prepared item. Outer safe-point scheduling, notification priority and the 128-item slice budget, Binding propagation, callback exception/diagnostic integration, and shutdown interruption remain later work.

## Property delivery bounded slice (`UNIT-CORE-048`)

- `RuntimeCore::process_property_delivery_slice` is UI-thread-only and processes up to 128 complete Property delivery items. Each complete item invokes its whole prepared subscriber snapshot before counting toward the limit.
- The Runtime keeps the remainder of the current generation in an owned iterator and drains it before preparing a later generation. A subsequent slice resumes in order; callback-driven next-generation changes do not replace or reorder the current remainder.
- A reentrant slice call from a subscriber is declined while the outer slice is active, preventing notification callbacks from interrupting one another. The method returns `(completed_count, has_more)`; host wake scheduling is a later integration unit.
- Focused oracle `UT-CORE-PROPERTY-NOTIFY-004`: passed with 129 ordered items across 128+1 slices. Workspace: 110 tests passed; `tools/dev.py check` passed.
- UI-thread validation happens before slice state is touched; a worker attempt returns `WrongThread` without starting callbacks. The fixture also calls the processor recursively from a subscriber and confirms that the nested slice is declined.
- Host safe-point placement and yielding, Binding budget composition, shutdown interruption, and callback exception diagnostics remain later work.

## Property notification host boundary (`UNIT-CORE-049`)

- `RuntimeCore::end_host_event` only starts a Property-delivery slice after the outermost host scope closes. A nested scope close returns without delivering notifications.
- Pending shutdown is committed before notification work. If a notification callback requests shutdown, the active Property item finishes all of its snapshotted subscribers; the slice then returns before taking another item, and the control shutdown is committed at that boundary.
- `dispatch_turn` declines reentrant dispatch while a notification slice is active, so a notification callback cannot start a queued ordinary callback inline.
- When a slice reports remaining work, `Dispatcher::schedule_pending_work_wake` schedules one Normal wake only if no normal/control wake is already pending. A later dispatch turn runs at most one ordinary callback and then the next notification slice.
- `UNIT-CORE-049` focused tests passed (8 related tests); workspace passed (114 tests); `.venv\\Scripts\\python.exe -X utf8 tools/dev.py check` passed. Property public subscribe ABI, Binding propagation, exception diagnostics, and Property-first teardown remain separate work.
- `UNIT-ABI-012` connects public Core C ABI Property setters to the host boundary. `UNIT-CORE-050` clears pending generations/items and cancels Runtime-owned active subscriptions before UI Property state is cleared at shutdown.
