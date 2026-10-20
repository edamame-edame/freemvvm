# Property Subscription shutdown cleanup

Scope: Rust Core `UNIT-CORE-050`, Runtime shutdown cleanup for pending Property notifications and active Subscription owners.

- Runtime shutdown cleanup first takes and drops prepared delivery work, clears the pending Property change generation, and takes the active Subscription owner registry.
- Subscription cancellation runs outside the owner registry lock and before Property UI state is cleared, so each callback closure is destroyed on the UI thread exactly once.
- The host boundary commits pending shutdown before starting queued notification callbacks. Shutdown completion is observable only after these cleanup actions finish.
- Focused oracle: `UT-CORE-PROPERTY-NOTIFY-006`; focused and workspace tests plus `tools/dev.py check` passed on 2026-10-20. Public subscribe/cancel ABI and Property-first teardown remain later units.
