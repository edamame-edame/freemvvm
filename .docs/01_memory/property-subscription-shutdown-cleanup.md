# Property Subscription shutdown cleanup

Scope: Rust Core Property Subscription teardown and shutdown cleanup (`UNIT-CORE-050` / `UNIT-CORE-051`).

- Runtime shutdown cleanup first takes and drops prepared delivery work, clears the pending Property change generation, and takes the active Subscription owner registry.
- Subscription cancellation runs outside the owner registry lock and before Property UI state is cleared, so each callback closure is destroyed on the UI thread exactly once.
- The host boundary commits pending shutdown before starting queued notification callbacks. Shutdown completion is observable only after these cleanup actions finish.
- Focused oracle: `UT-CORE-PROPERTY-NOTIFY-006`; focused and workspace tests plus `tools/dev.py check` passed on 2026-10-20. Public subscribe/cancel ABI and Property-first teardown remain later units.

Property-first teardown is implemented by `UNIT-CORE-051`: final-release tickets wake the registered host loop, drain at an outer UI safe point, remove that Property's pending and prepared notification work, cancel its Runtime-owned subscriptions, then destroy its UI state. While delivering an item, the Runtime drains accepted finalizers between callback starts so releasing the owner from one callback suppresses the remaining subscribers. See [owner teardown plan](../20_todo/property-subscription-owner-teardown.md) and `UT-CORE-PROPERTY-NOTIFY-007`.
