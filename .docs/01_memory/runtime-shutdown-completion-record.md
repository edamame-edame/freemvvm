# Rust Core Runtime: shutdown completion record

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-009`.

- Keep completion state independent of Runtime UI objects and share it through an `Arc`; wait/query observers do not own or cancel the underlying shutdown request.
- `Pending` can be reported as `HostUnavailable(reason)` to wake synchronous waiters without claiming cleanup finished. A later explicit retry on the Runtime creation thread re-arms the record to `Pending`.
- Commit `Completed` only after UI cleanup; it is terminal and a later host-stop notification must not overwrite it. Completion and host-stop transitions use the same mutex/condition variable.
- This unit supplies only the record primitive. HostRegistration, dispatcher integration, public request handles, and cleanup retry orchestration remain separate work.
