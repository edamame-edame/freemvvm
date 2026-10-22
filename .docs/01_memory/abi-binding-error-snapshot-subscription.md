# Scope: C ABI Binding error snapshots and subscriptions

- A Binding error snapshot returns an immutable Error handle and a newly owned Property handle for the rejected identity. Resolve the Property identity from an active Binding pin; it may not be the direct endpoint of the Binding handle being queried when group propagation is involved.
- Keep current error state inside the Runtime Binding owner. Issued Error handles are independent immutable snapshots and remain readable after the current state clears or changes.
- Binding error subscriptions share the Subscription lifecycle primitive, but their owner is a Binding connection rather than a Property identity. Binding cancel and Runtime shutdown cancel these registrations and call destroy once; releasing the subscription handle alone does not cancel.
- Queue error notifications by Binding connection during propagation and coalesce repeated changes within the pending delivery generation. Deliver after all propagation and before Property subscriber callbacks. Callbacks have no initial notification.
- For ABI implementation and validation, see [Binding validator/error plan](../20_todo/validator-binding-errors.md), [C ABI test oracle](../32_unit_test/core-runtime.md#ut-abi-binding-error-001), and `UNIT-ABI-016` in `tools/units.yaml`.
