# Rust Core Runtime: same-host shutdown retry and Dispatcher wake

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-015`.

- A same-host retry is accepted only on the registered UI thread, while the host is Available, the completion is HostUnavailable, and its associated Dispatcher is Running with shutdown pending.
- Validate/re-arm the Dispatcher while holding the HostRegistration state lock, then transition and re-register the completion. This prevents stop/retry races from leaving a Pending completion detached from a stopped host.
- Schedule the fresh UI-loop wake only after releasing the host state lock. Explicit retry must schedule even if the dispatcher's old `wake_pending` flag is still true because the earlier host loop stopped before consuming that wake.
- Reject an invalid/closed dispatcher before changing the completion state. UT-RUNTIME-005 checks both rejection preservation and the fresh wake count.
