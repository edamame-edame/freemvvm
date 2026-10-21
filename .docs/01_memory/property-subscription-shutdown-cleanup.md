# Property Subscription shutdown cleanup

Scope: Rust Core Property Subscription teardown, shutdown cleanup, and public C ABI ownership (`UNIT-CORE-050` / `UNIT-CORE-051` / `UNIT-ABI-013`).

- Runtime shutdown cleanup first takes and drops prepared delivery work, clears the pending Property change generation, and takes the active Subscription owner registry.
- Subscription cancellation runs outside the owner registry lock and before Property UI state is cleared, so each callback closure is destroyed on the UI thread exactly once.
- The host boundary commits pending shutdown before starting queued notification callbacks. Shutdown completion is observable only after these cleanup actions finish.
- Focused oracle: `UT-CORE-PROPERTY-NOTIFY-006`; focused and workspace tests plus `tools/dev.py check` passed on 2026-10-20. Public subscribe/cancel ABI and Property-first teardown remain later units.

Property-first teardown is implemented by `UNIT-CORE-051`: final-release tickets wake the registered host loop, drain at an outer UI safe point, remove that Property's pending and prepared notification work, cancel its Runtime-owned subscriptions, then destroy its UI state. While delivering an item, the Runtime drains accepted finalizers between callback starts so releasing the owner from one callback suppresses the remaining subscribers. See [owner teardown plan](../20_todo/property-subscription-owner-teardown.md) and `UT-CORE-PROPERTY-NOTIFY-007`.

The public Property subscribe/cancel ABI is implemented by `UNIT-ABI-013`. The Core keeps independently owned Subscription tokens in a public-handle table while Runtime remains the strong owner of active registrations. Retaining or releasing one token does not cancel the registration; explicit cancel is UI-thread-only and idempotent, and an in-flight self-cancel defers closure destruction until callback return. Property teardown and Runtime shutdown still cancel active registrations through the Runtime owner registry. See [public ABI plan](../20_todo/property-subscription-public-abi.md) and `UT-ABI-013`.
