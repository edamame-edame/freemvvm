# Runtime post host-entry design gap

Scope: freemvvm C ABI runtime dispatch.

- The DD already promises `runtime_post`, FIFO execution on the Runtime UI thread, and at most one callback per non-reentrant turn.
- A host bridge wake callback only asks the host to schedule a future turn; the C ABI must also expose or define how that host turn enters Core dispatch.
- The user selected a UI-thread `runtime_dispatch_turn(runtime)` entrypoint. It opens/closes a nested host scope and executes at most one callback; `runtime_host_end_event` remains solely responsible for completing shutdown at the outer event boundary.
- Preserve ownership transfer only after queue insertion succeeds; accepted callback/userdata/destroy are consumed by exactly one execute-or-cancel path.
- The C ABI implementation advertises feature 7 (`runtime.post_dispatch.v1`) only after post/dispatch entrypoints are present. Shutdown request must emit a Control wake even when a Normal wake is pending, to promote host scheduling priority.
- UNIT-ABI-008 is registered ready only after focused C ABI execute/cancel/ownership tests, dispatcher priority coverage, workspace tests, and standard repository checks pass.
