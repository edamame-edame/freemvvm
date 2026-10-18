# Rust C ABI: Runtime UI-thread query

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-ABI-005`.

- Query the process Runtime handle registry to pin the `RuntimeCore` owner for the duration of `runtime_is_ui_thread`; do not infer thread identity from the query caller's current execution path.
- A valid handle returns `1` only on the thread captured when the Runtime was created and `0` on other threads. The query is allowed from any thread and remains a read-only identity check.
- Zero or unknown Runtime handles map to `UI_INVALID_HANDLE`; a null required output maps to `UI_INVALID_ARGUMENT`; the output is initialized to zero before handle lookup. Registry synchronization failure or a caught panic maps to `UI_INTERNAL_ERROR`.
- `UT-ABI-005` exercises the C ABI from creator/worker threads plus invalid handles, null output, and panic conversion. Runtime owner release remains separate until shutdown can safely coordinate the final owner drop.
