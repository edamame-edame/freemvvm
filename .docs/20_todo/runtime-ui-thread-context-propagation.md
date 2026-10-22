# Runtime UI thread context propagation

- Status: complete; commit and push recorded in Git history.
- Scope: `UNIT-CORE-022` lets Runtime-affine components receive one shared `UiThreadContext`; it does not add the Runtime facade or public ABI.
- Requirement: [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Design: [Runtime実行context・所有・終了](../13_detail_design/runtime-objects/runtime.md), UI thread identity and RuntimeState responsibilities.
- Test case: [TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01).

## Plan

1. Register `UNIT-CORE-022` and its focused test before editing Rust source.
2. Add explicit shared-context constructors for Dispatcher, RuntimeCleanupQueue, HostRegistration, and ShutdownCompletion. Keep current convenience constructors capturing the current thread so isolated tests remain source-compatible.
3. Verify a Dispatcher created on a worker with the UI context captured on the caller can dispatch on the caller and rejects direct dispatch on the worker.
4. Run the focused test, all workspace tests, Clippy, format, development checks, document links, and `git diff --check`.
5. Record the completed result in this plan and `.docs/01_memory/`, then commit and push.

## Acceptance

- Every Runtime-affine component can be initialized against the same captured thread identity.
- Existing constructors retain their current semantics.
- Direct dispatch returns `WrongThread` on a non-owner thread and continues to succeed on the supplied owner thread.
- The unit does not imply Runtime composition, handle issuance, or public `runtime_create` is implemented.

## Result

- Added explicit shared-context constructors to Dispatcher, RuntimeCleanupQueue, HostRegistration, and ShutdownCompletion. Existing constructors still capture the current thread.
- Focused test passed: a Dispatcher built on a worker with the caller's context dispatches successfully on the caller and returns `WrongThread` on the worker.
- `cargo test --workspace --locked`: passed (29 Core and 6 C ABI unit tests; doc-test targets contain no tests).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check`, `.venv\\Scripts\\python.exe tools\\dev.py check`, and `git diff --check`: passed.
- Static document-link check reported 76 active documents and 5,866 local links.
- Runtime facade, handle issuance, and public `runtime_create` remain future units.
