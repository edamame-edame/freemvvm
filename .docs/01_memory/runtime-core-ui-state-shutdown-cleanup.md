# Core Runtime UI state store shutdown cleanup memory

Scope: `crates/core-runtime/src/runtime.rs`, `UNIT-CORE-036`.

- `RuntimeCore` captures one `UiThreadContext` and gives it to both `RuntimeCleanupQueue` and its `UiObjectStore`.
- Store cleanup is registered during RuntimeCore construction. If registration fails, construction fails with an internal RuntimeCore error rather than returning a partially initialized owner.
- The queue's tracked cleanup closure retains the shared store until drain, recovers a poisoned store mutex, and clears it on the captured UI thread. The queue processes accepted finalizer tickets before tracked UI-state callbacks, then publishes `Closed` only when all work has drained.
- Runtime shutdown integration is internal infrastructure; this unit adds no object-specific state API or public handle behavior.
