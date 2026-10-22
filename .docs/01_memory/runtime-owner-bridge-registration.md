# Runtime owner and UI-loop bridge registration

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-023`.

- `UNIT-CORE-023` added an internal RuntimeCore owning one UiThreadContext, RuntimeCleanupQueue, HostRegistration, and an optional Dispatcher.
- Approved ABI DD allows Runtime construction without a host loop bridge; registration is UI-thread-only, once-only, and rejected after shutdown begins.
- Dispatcher is created only after successful registration and uses the same thread context. Registration takes a borrowed `Arc` and clones it only at commit, so worker/duplicate/closing failures preserve caller ownership and leave the current bridge unchanged.
- All 36 workspace unit tests, Clippy, format, and `tools/dev.py check` pass. Document links report 76 active documents and 5,869 local links.
- This internal Core owner does not by itself implement C ABI Runtime handle storage or public `runtime_create`.
