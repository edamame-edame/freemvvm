# Runtime UI thread context

- Status: complete; commit and push recorded in Git history.
- Scope: `UNIT-CORE-021` centralizes the thread identity captured when a Runtime-affine Core component is created. It does not add a public Runtime or C ABI entry point.
- Requirement: [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001) binds UI state operations to the Runtime creation thread.
- Design: [Runtime実行context・所有・終了](../13_detail_design/runtime-objects/runtime.md), especially Runtime creation/UI-thread identity and the `RuntimeState` responsibilities.
- Test case: [TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01).

## Plan

1. Register the implementation unit and its focused unit-test before source edits.
2. Add an internal `UiThreadContext` that captures `std::thread::current().id()` and answers whether the caller is on that thread.
3. Replace duplicated identity comparisons in Dispatcher, Runtime cleanup, host registration, and shutdown completion with this context while preserving existing error paths.
4. Test same-thread and worker-thread queries, then run the focused Core tests, workspace checks, documentation checks, and `git diff --check`.
5. Record the result and implementation lesson in `.docs/01_memory/`, then commit and push this unit.

## Acceptance

- Creation captures exactly one thread identity; same-thread query is true and a worker-thread query is false.
- Existing dispatcher, cleanup, host-scope, and shutdown tests keep their behavior and status results.
- The unit-test and its design/RQ/case mapping are registered in `tools/units.yaml`.
- This unit makes no claim that a public Runtime creation API or end-to-end `RQ-06-001` behavior is complete.

## Result

- Added `UiThreadContext` and routed Dispatcher, RuntimeCleanupQueue, HostRegistration, and ShutdownCompletion thread checks through it.
- Focused test passed: creator thread is recognized; a worker thread is not; the original context remains valid afterward.
- `cargo test --workspace --locked`: passed (34 unit tests across Core and C ABI; doc-test targets contain no tests).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check`, `.venv\\Scripts\\python.exe tools\\dev.py check`, and `git diff --check`: passed.
- Public Runtime creation/query and end-to-end UI-operation coverage remain future units.
