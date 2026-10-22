# Runtime UI thread context propagation

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-022`.

- `UNIT-CORE-022` added explicit `new_for_ui_thread` constructors for Dispatcher, RuntimeCleanupQueue, HostRegistration, and ShutdownCompletion. Existing constructors remain wrappers that capture the current thread.
- Focused test proves a Dispatcher constructed on a worker with a caller-owned context accepts dispatch on that owner and rejects it on the worker.
- All 35 workspace unit tests, Clippy, format, and `tools/dev.py check` pass. Document links report 76 active documents and 5,866 local links.
- Runtime composition should capture one context and pass it to all components; the public facade and `runtime_create` are still unimplemented.
