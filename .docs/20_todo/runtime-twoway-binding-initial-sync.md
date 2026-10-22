# Runtime TwoWay Binding初期同期とgroup commit

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-061`

## Goal

Runtime内のTwoWay初期Binding処理を追加する。source値の準備、targetへの初期値適用、TwoWay同期groupの登録、変更通知の予約を一つのcommit境界に置き、失敗時は値・graph・通知queueを変更しない。

## Approved basis

- Requirements: [RQ-07-003](../11_requirements/RQ-07_binding.md#rq-07-003)、[RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)、[RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015)。
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1「Binding作成」「TwoWay同期グループ」。
- Object/value design: [Object management: values and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2「Property commit / typed value equivalence」。
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-RUNTIME-TWOWAY-INIT-001`.

## Steps

- [x] Register `UNIT-CORE-061` and its TwoWay initial-sync atomicity oracle before source changes.
- [x] Prepare source value and TwoWay group proposal before mutation.
- [x] Commit changed target value, its notification, and graph merge together; equivalent initial value commits only the graph merge.
- [x] Verify rejected kind/topology preparation preserves values, graph connectivity, and pending notifications; repeated target dirtiness remains coalesced.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

## Verification

- Focused: `cargo test -p freemvvm-core runtime_two_way_binding_initial` — 2 passed.
- Workspace: `cargo test --workspace --locked` — 141 passed.
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check` — passed, including document links, clippy, builds, C/C++ probes, .NET, and Python package checks.
- Hygiene: `cargo fmt --all -- --check`; `git diff --check`.

All checks passed on 2026-10-21.

## Scope boundary

This adds the internal Runtime TwoWay initial-sync and group-registration primitive. It does not implement TwoWay group propagation at delivery, direct-setter ordering, validators/Binding error snapshots, public Binding ownership/handle/cancel, or C ABI.
