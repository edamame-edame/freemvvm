# Core Runtime object registration and cleanup memory

Scope: `crates/core-runtime/src/runtime.rs`, `UNIT-CORE-037`.

- Construct object state through a factory only after checking the Runtime UI thread and `Running` state. This avoids creating UI-affine state on rejected worker or Closed calls.
- Runtime object registration inserts state first, derives `ObjectIdentity` from the store, creates the Runtime-bound `CoreObjectControl`, then registers the token. If HandleRegistry registration fails, destroy the new state on the UI thread and return the original registry error.
- `RuntimeObjectCleanupQueue` delegates reservation to `RuntimeCleanupQueue`; the accepted ticket destroys the identity from `UiObjectStore` before running `CoreObjectControl` completion. Thus worker release acceptance is separate from UI-thread cleanup completion.
- Shutdown clears the entire store before marking each tracked control cleaned. A finalizer ticket racing after that clear may see `InvalidIdentity`; treat that as already-cleaned only after the shared shutdown-clear flag is published.
- Runtime queue currently drains finalizer tickets at shutdown; this unit does not add normal-turn queue draining or a process-global object-kind registry.
