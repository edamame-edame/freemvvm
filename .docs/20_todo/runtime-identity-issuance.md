# Runtime identity issuance

- Status: complete; commit and push recorded in Git history.
- Scope: `UNIT-CORE-024` assigns each internal `RuntimeCore` a process-unique, nonzero identity from the same monotonic issuer used for public handle tokens. It does not expose the identity through the C ABI or add the handle registry to RuntimeCore.
- Requirements: [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018), [RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019).
- Design: [オブジェクト管理：handle・値・error](../13_detail_design/runtime-objects/objects-values.md), Runtime identity and token issuance; [Runtime実行context・所有・終了](../13_detail_design/runtime-objects/runtime.md), Runtime identity ownership.
- Test case: [TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01), [TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01).

## Plan

1. Register `UNIT-CORE-024` and its focused test design before source edits.
2. Add an internal `RuntimeIdentity` value and allocate it from the process-wide token issuer shared with HandleRegistry.
3. Make RuntimeCore creation report identity allocation failure without constructing a partially initialized Runtime.
4. Test nonzero, distinct identities across concurrent RuntimeCore creation and keep existing owner/bridge tests passing.
5. Run focused Runtime tests, workspace tests, Clippy, format, `tools/dev.py check`, and `git diff --check`.
6. Record results in `.docs/01_memory/`, then commit and push this unit.

## Acceptance

- Every successfully constructed RuntimeCore has a nonzero identity unique within the process token namespace.
- Identity issuance is serialized with public handle token issuance, so the two kinds of value cannot collide or be reused.
- Exhaustion or issuer synchronization failure prevents RuntimeCore construction.
- The identity remains internal; no public ABI or Runtime shutdown behavior is added in this unit.

## Result

- Added `RuntimeIdentity` to RuntimeCore and issued it from `PROCESS_TOKEN_ISSUER`, the same synchronized monotonic namespace used for public handle tokens.
- RuntimeCore construction now fails with `IdentityUnavailable` if issuance fails; no partially initialized Runtime is returned.
- A concurrent creation test verified 256 RuntimeCore identities are nonzero and unique.
- `cargo test --workspace --locked`: passed (31 Core and 6 C ABI unit tests; doc-test targets contain no tests).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check`, `tools/dev.py check`, and `git diff --check`: passed.
- Document-link check reported 76 active documents and 5,873 local links. C/C++ environment probes, C# build, Python checks, and wheel build/install/import passed.
- Public Runtime handle issuance, `runtime_create`, and shutdown integration remain separate units.
