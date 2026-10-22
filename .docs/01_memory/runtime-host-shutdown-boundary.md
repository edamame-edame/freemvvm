# Rust Core Runtime: outer host event shutdown boundary

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-012`.

- A HostRegistration scope can trigger Dispatcher shutdown only after the matching outermost event scope closes successfully. Nested scope closure leaves shutdown pending.
- The boundary calls `run_pending_shutdown`, which does nothing and leaves queued ordinary callbacks untouched when no shutdown is pending. It never starts ordinary work itself.
- When shutdown is pending, Dispatcher transitions to Closing, cancels accepted queued work on the UI thread, then invokes the supplied shutdown handler and becomes Closed. Requests made within the active callback therefore wait until its caller reaches the outer event boundary.
- UT-RUNTIME-004 verifies nested-scope no-op, callback completion, UI-thread destruction of queued work, and shutdown-handler ordering. Platform adapters, Runtime coordinator ownership, and ABI integration remain separate units.
