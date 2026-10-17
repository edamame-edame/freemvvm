# Rust Core Runtime: HostRegistration scopes and stop notification

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-011`.

- Keep active host event scopes in a thread-local LIFO stack keyed by a stable per-registration Arc identity plus a monotonically increasing per-registration scope ID. This detects cross-Runtime nesting as well as mismatched end_event calls.
- `end_event` mutates only when the exact host/scope pair is at the top; its boolean result reports whether the entire UI-thread host scope stack is empty. `before_stop` requires that safe outer boundary.
- Store completion records as weak references. `notify_stopped(reason)` atomically changes pump state and drains the weak list, then notifies every still-live pending record without running host/user callbacks under the registration lock.
- After the same UI host resumes, explicit retry holds the host state lock while it re-arms and re-registers the completion record, preventing a stop/retry race from leaving a request Pending on a stopped host.
- UNIT-CORE-012 connects the outer scope boundary to Dispatcher priority shutdown. Platform-specific event hooks, full Runtime coordinator ownership, and public ABI wiring remain separate units.
