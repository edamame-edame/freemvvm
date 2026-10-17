# Rust Core Runtime: host-serialized shutdown request admission

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-014`.

- `HostRegistration::request_shutdown` requires a Pending completion tied to the same UI thread as the Dispatcher. Under the host state lock it verifies Available, reserves weak-record capacity, arms/coalesces Dispatcher shutdown, and then registers the completion.
- Dispatcher wake scheduling occurs after releasing the HostRegistration state lock, avoiding bridge calls while holding the host mutex. The bridge wake only schedules a future UI-loop turn.
- A stopped host or closed Dispatcher rejects admission without registering the completion or creating a new priority flag. Repeated accepted requests coalesce at the Dispatcher while their records remain associated with host stop/completion observation.
- UT-RUNTIME-005 covers worker admission, stopped-host and closed-dispatcher rejection, duplicate coalescing, host stop/retry, and eventual boundary cleanup. Retry wake rearming and public request handles remain later work.
