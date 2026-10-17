# Rust Core Runtime: graceful host stop and abrupt stop notification

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-017`.

- `HostRegistration::before_stop` runs on the UI thread with no open HostScope and verifies it is paired with a Dispatcher owned by the same UI thread.
- It changes host admission to Stopping before invoking Dispatcher shutdown, so concurrent new requests cannot race in after the stop boundary.
- If a priority shutdown is pending, Dispatcher cancels unstarted work and invokes Runtime cleanup before returning; only then does the completion become Completed. With no pending shutdown, `before_stop` leaves the Runtime cleanup queue Running.
- An `AlreadyDispatching` failure restores host availability because no shutdown handler ran. Abrupt `notify_stopped(reason)` remains the separate path that marks accepted pending requests HostUnavailable without claiming cleanup completion.
