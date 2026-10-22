# Runtime-owned object control factory

- Status: complete; commit and push recorded in Git history.
- Scope: `UNIT-CORE-025` lets RuntimeCore create a `CoreObjectControl` bound to its own identity and cleanup queue. It does not add an object store, public handle, or process-level registry instance.
- Requirements: [RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019), [RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010), [RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003).
- Design: [オブジェクト管理：handle・値・error](../13_detail_design/runtime-objects/objects-values.md), Runtime identity, CoreObjectControl, and HandleRegistry; [Runtime実行context・所有・終了](../13_detail_design/runtime-objects/runtime.md), Runtime ownership and cleanup queue.
- Test case: [TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01), [TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01), [TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01).

## Plan

1. Register `UNIT-CORE-025` and its focused test design before source edits.
2. Add a RuntimeCore factory that accepts an object identity and binds the resulting CoreObjectControl to this Runtime's identity and cleanup queue.
3. Test registration and validation through a temporary HandleRegistry: matching Runtime identity succeeds, another Runtime is rejected, and accepted final cleanup remains on the owning Runtime queue.
4. Run the focused test, workspace tests, Clippy, format, `tools/dev.py check`, and `git diff --check`.
5. Record results in `.docs/01_memory/`, then commit and push this unit.

## Acceptance

- Callers cannot accidentally pair a RuntimeCore-created control block with a different Runtime identity or cleanup queue.
- The shared HandleRegistry continues to own its synchronized token map independently of RuntimeCore instances.
- Cross-Runtime validation fails without changing the control block or token state.
- No UI object storage or public ABI behavior is introduced.

## Result

- RuntimeCore now creates `CoreObjectControl` using its own `RuntimeIdentity` and cleanup queue.
- A shared HandleRegistry test verified owning-Runtime validation, cross-Runtime rejection, and worker-thread final release queued for cleanup on the owning Runtime's UI thread.
- `cargo test --workspace --locked`: passed (32 Core and 6 C ABI unit tests; doc-test targets contain no tests).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check`, `tools/dev.py check`, and `git diff --check`: passed.
- Document-link check reported 76 active documents and 5,879 local links. C/C++ environment probes, C# build, Python checks, and wheel build/install/import passed.
- Public Runtime handle issuance, object storage, and C ABI integration remain separate units.
