# C ABI Runtime shutdown and host-loop bridge

- Updated: 2026-10-18
- Status: complete
- Scope: `UNIT-ABI-007` connects the approved Runtime shutdown contract, owned completion receipt, and UI-loop host bridge to the versioned C ABI.

## Approved basis

- Requirements: [RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003), [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005), [RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006), [RQ-06-007](../11_requirements/RQ-06_threading-dispatch.md#rq-06-007), and [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Host境界・終了要求・完了観測”, “公開・内部interface案”, and “主要な呼出しと失敗境界”.
- ABI design: [Runtime ABI](../13_detail_design/runtime-objects/abi.md) v1.1, “UI loop接続のABI宣言” and shutdown status/ownership tables.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.7, UT-RUNTIME-004/005 and C ABI shutdown/bridge test proposals.

## Adopted behavior

- Preserve `runtime_shutdown` as the synchronous compatibility entrypoint.
- Add `runtime_request_shutdown(mode, out_request)`, request query, host event begin/end, host stop notification, and the one-time UI-loop bridge registration as described by DD v1.1.
- A request accepted during a callback/event returns without waiting; shutdown is serviced at the outer host event boundary before ordinary queued work. Worker SYNC waits for completion or definitive host stop.
- Completion is observable as Pending, Completed, or HostUnavailable(reason). Only Completed means cleanup finished; same-host resume requires explicit retry through the adopted DD contract.
- Keep the DD precondition that callers complete shutdown before releasing the final public Runtime handle. Generic handle release does not implicitly trigger shutdown or gain a Runtime-specific exception.

## Steps

- [x] Register `UNIT-ABI-007` as planned in `tools/units.yaml` and add its C ABI test oracle to `core-runtime.md` before source changes.
- [x] Carry wake priority through Dispatcher and the C ABI bridge callback (`UI_WAKE_NORMAL` / `UI_WAKE_CONTROL`).
- [x] Add Core ABI-support operations for bridge registration, shutdown admission, owned request receipt lookup/release, completion query, host event boundaries, host stop/resume, and explicit same-request retry.
- [x] Preserve the existing DD API surface: a host event begin after the same host resumes reopens admission, and a subsequent explicit `runtime_request_shutdown` retries the existing HostUnavailable completion record and returns a fresh owned receipt for that record. Do not add a new retry symbol.
- [x] Add panic-contained C ABI entrypoints and status/output mapping for `runtime_shutdown`, `runtime_request_shutdown`, `shutdown_request_query`, bridge registration, and host boundary notifications; preserve output-zeroing and ownership-on-failure rules.
- [x] Test priority mapping, UI-thread-only/one-time registration, callback/event deferral, worker SYNC completion, owned receipt query/release, host-stop reporting/retry, invalid handles, and pointer validation.
- [x] Run the focused tests, workspace development checks, documentation/link validation, formatting, and `git diff --check`.
- [x] Mark the unit ready only after implementation and corresponding tests pass.


## Validation

Validation completed 2026-10-18:

- `cargo test --workspace` — passed (56 Rust unit tests, plus doc-test targets).
- `.venv\Scripts\python.exe tools\dev.py check` — passed after applying `cargo fmt`; Rust/Python formatting, Python lint, documentation-link validation all passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed after adding the documented safety justification for the C callback adapter's `Send`/`Sync` implementations.
- `git diff --check` — passed.

Static documentation validation is not a substitute for the Rust tests or full development checks.

Implementation note: the existing Error handle subsystem is not implemented yet. The optional `out_error` outputs for the shutdown entrypoints are initialized to zero; status and the request query's host-stop reason remain observable. This follows the ABI's optional-detail/resource-failure behavior and does not change the accepted shutdown state transitions.
