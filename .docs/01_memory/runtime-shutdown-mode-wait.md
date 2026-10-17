# Rust Core Runtime: shutdown request modes and wait boundaries

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-016`.

- `ShutdownRequestMode::Async` admits the request and immediately reports its current observation, normally Pending.
- `ShutdownRequestMode::Sync` waits on the completion condition only when called from a worker thread. UI-thread calls return after admission so an active callback/event cannot wait on the UI loop that must perform cleanup.
- The worker wait maps only terminal record states to Completed or HostUnavailable(reason); Pending is not reported as completion.
- UT-RUNTIME-005 uses a wake channel to synchronize the worker request with UI cleanup without timing-based sleeps.
