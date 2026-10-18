# C ABI Runtime shutdown: adopted DD behavior

Scope: freemvvm Runtime shutdown ABI implementation.

- On 2026-10-18 the user selected the approved DD v1.1 Runtime shutdown behavior as the implementation basis.
- Keep `runtime_shutdown` as the synchronous compatibility entrypoint and expose the DD's request/receipt, completion query, host event-boundary, stopped-host, and bridge-registration operations.
- UI callback/event requests return after acceptance; the outer host event boundary performs shutdown before ordinary queued work. Worker SYNC waits for completion or definitive host stop. Only Completed means cleanup finished.
- HostUnavailable remains observable on the accepted completion. Same-host resume followed by an explicit shutdown request retries the existing completion record; release of a request receipt does not cancel shutdown.
- Preserve the DD precondition that shutdown completes before final public Runtime-handle release. Generic release does not trigger implicit shutdown and has no Runtime-only exception.
- Before exposing host-stop/retry behavior, verify the chosen public call sequence against Runtime DD v1.1 and UT-RUNTIME-005; do not claim RQ-06-012 complete if a public caller cannot observe or retry the same accepted record.
- Implementation slice UNIT-ABI-007 keeps receipt handles in a separate process registry that pins the Runtime shell; reserve the token slot before accepting shutdown so a successful request cannot become unobservable because receipt allocation failed afterward.
- C bridge callbacks may execute on any thread. The C contract must require thread-safe userdata access and must not unwind across the C ABI; the adapter synchronizes in-flight wake callbacks before its one-time UI-thread destroy callback.
- The generic Error handle subsystem is still absent. Optional shutdown `out_error` outputs therefore remain zero while `ui_status` and request-query host-stop reason carry the implemented observable result.
