# Runtime UI thread context

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-021`.

- `UNIT-CORE-021` added `UiThreadContext` in `runtime.rs`; Dispatcher, RuntimeCleanupQueue, HostRegistration, and ShutdownCompletion now query through it while retaining their existing error mapping.
- Focused unit-test and all workspace tests pass. `tools/dev.py check` also passes; its static document check reported 76 active documents and 5,864 local links.
- Verification for this unit covers same-thread and worker-thread identity only. It does not prove the public UI API's full RQ-06-001 behavior.
- Before the next C ABI Runtime unit, compose the existing cleanup/dispatcher/host components and handle registry under one Runtime owner; do not expose a placeholder Runtime handle.
