# Runtime owner and UI-loop bridge registration

- Status: complete; commit and push recorded in Git history.
- Scope: `UNIT-CORE-023` composes the existing Runtime thread context, cleanup queue, host registration, and optional Dispatcher in an internal `RuntimeCore`. It does not implement public Runtime handles or C ABI entry points.
- Requirements: [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001), [RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002).
- Design: [Runtime実行context・所有・終了](../13_detail_design/runtime-objects/runtime.md), RuntimeState and Dispatcher responsibilities; [共通C ABI](../13_detail_design/runtime-objects/abi.md), UI-loop bridge registration.
- Test case: [TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01).

## Plan

1. Register `UNIT-CORE-023` and its unit-test design before source edits.
2. Add an internal `RuntimeCore` that captures one `UiThreadContext`, initializes cleanup and host components with it, and starts without a Dispatcher/bridge.
3. Add UI-thread-only, once-only bridge registration while the cleanup lifecycle remains Running. Create the Dispatcher with the same context.
4. Test bridge-free creation, successful registration, duplicate registration, worker-thread rejection, and rejection after shutdown starts.
5. Run focused Runtime tests, the workspace tests, Clippy, format, `tools/dev.py check`, and `git diff --check`.
6. Record results in `.docs/01_memory/`, then commit and push this unit.

## Acceptance

- RuntimeCore and all owned Runtime-affine pieces use one captured thread context.
- Bridge absence does not prevent RuntimeCore construction or UI-thread identity queries.
- Bridge ownership enters the Core only after a valid UI-thread registration on a Running Runtime; every rejected attempt leaves the installed bridge unchanged.
- Rejected registration preserves the caller's ownership of the proposed bridge.
- Existing standalone component constructors and tests retain their current behavior.
- Public `runtime_create`, opaque Runtime handle issuance, and full shutdown orchestration remain separate work.

## Result

- Added an internal RuntimeCore that captures one context and owns the cleanup queue, host registration, and optional Dispatcher.
- RuntimeCore starts without a bridge. UI-thread registration installs one Dispatcher using the same context. Worker, duplicate, and Closing-state attempts fail without replacing the installed bridge.
- Focused test also exercised host event scope and Runtime-owned cleanup queue access.
- `cargo test --workspace --locked`: passed (30 Core and 6 C ABI unit tests; doc-test targets contain no tests).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check`, `tools/dev.py check`, and `git diff --check`: passed.
- Static document-link check reported 76 active documents and 5,869 local links.
- Public Runtime handle issuance, `runtime_create`, and complete shutdown orchestration remain future units.
