# Rust Core Runtime: cleanup completion commit

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-010`.

- Use `RuntimeCleanupQueue::drain_shutdown_and_complete` as the join point for UI cleanup and completion publication. UNIT-CORE-013 invokes it from the HostRegistration outer-event shutdown handler; it drains finalizers and tracked UI cleanup through Closed, then commits the request record to Completed.
- Do not publish Completed before cleanup; host-stop notification remains a separate HostUnavailable result and does not close the Runtime.
- HostRegistration outer event and Dispatcher shutdown are connected for this internal path. Platform adapters, public request handles, and retry orchestration remain later units.
