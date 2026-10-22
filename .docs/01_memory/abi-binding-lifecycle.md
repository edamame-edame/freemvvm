# Scope: C ABI Binding creation, ownership, and cancellation

- For new C ABI Binding entrypoints, consult the approved ABI Binding section and RQ-05/RQ-07. OneTime returns handle 0 after initial copy; OneWay/TwoWay return an owned Binding handle.
- Binding handle release drops only that external owner token. Runtime retains the active connection until explicit cancel or shutdown; `handle_retain` creates an independent Binding owner token.
- Explicit cancel is UI-thread-only, idempotent while the handle remains valid, removes propagation, and keeps committed Property values. The canceled handle is still released separately.
- The current Core ABI support stores the internal connection identity behind the public Binding token. Keep that internal graph identity out of the C ABI.
- The lifecycle unit does not implement Property Validators, Binding error snapshots/subscriptions, type conversion, or language wrappers. Do not advertise the full Binding feature until those required public surfaces are implemented.
- For implementation and test evidence, see [C ABI Binding lifecycle plan](../20_todo/abi-binding-lifecycle.md), [C ABI test oracle](../32_unit_test/core-runtime.md#ut-abi-binding-lifecycle-001), and `UNIT-ABI-014` in `tools/units.yaml`.
