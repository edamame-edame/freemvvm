# Runtime handle registration and borrowed resolution

- Status: complete; commit and push recorded in Git history.
- Scope: `UNIT-CORE-026` adds the internal process-wide-token-backed table entries needed to issue and resolve opaque Runtime handles. Public C ABI entry points and token retain/release are separate units because last Runtime release must be coordinated with shutdown completion.
- Requirements: [RQ-04-004](../11_requirements/RQ-04_abi-boundary.md#rq-04-004), [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018), [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Design: [共通C ABI：宣言・検査・互換性](../13_detail_design/runtime-objects/abi.md), opaque handle and `runtime_create`; [Runtime実行context・所有・終了](../13_detail_design/runtime-objects/runtime.md), Runtime identity and thread-query behavior.
- Test case: [TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01), [TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01).

## Plan

1. Register `UNIT-CORE-026` and its focused test design before source edits.
2. Add a synchronized Runtime-handle table that issues nonzero tokens from the shared process token issuer and stores an `Arc<RuntimeCore>` per token.
3. Resolve a borrowed token to a temporary strong owner without exposing internal pointers; invalid tokens fail without changing table contents.
4. Test successful and invalid lookup, distinct Runtime tokens, worker-thread resolution, and fail-closed behavior when table synchronization is poisoned.
5. Run the focused test, workspace tests, Clippy, format, `tools/dev.py check`, and `git diff --check`.
6. Record results in `.docs/01_memory/`, then commit and push this unit.

## Acceptance

- Each inserted RuntimeCore receives a unique, nonzero opaque handle from the same process token namespace used by other handles and Runtime identities.
- Borrowed resolution returns the same Runtime owner, including when lookup occurs on a worker thread; it does not grant UI-thread permission.
- Invalid tokens and synchronization failure do not mutate the table.
- The implementation does not decide public ABI error mapping or last-token release/shutdown ordering.

## Result

- Added a synchronized Runtime handle table with a process-wide singleton and test-local constructor. Each insertion issues from the shared token namespace and clones one Arc to the RuntimeCore only after token issuance succeeds, preserving caller ownership on failure.
- Borrowed resolution clones a temporary strong owner and works from worker threads without granting UI-thread access. Zero and unregistered nonzero tokens are rejected; failed lookup leaves entries usable.
- `cargo test --workspace --locked`: passed (34 Core and 6 C ABI unit tests; doc-test targets contain no tests).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check`, `tools/dev.py check`, and `git diff --check`: passed.
- Document-link check reported 76 active documents and 5,884 local links. C/C++ environment probes, C# build, Python checks, and wheel build/install/import passed.
- Public C ABI `runtime_create`, token retain/release, and last-token shutdown ordering remain separate units.
