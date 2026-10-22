# Binding初期値コピー候補の準備

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-054`

## Goal

初期Binding同期の準備段階として、sourceのtyped valueをtarget用に型検証・所有コピーし、commit前の候補として返す。候補準備だけではtargetの確定値、Binding graph、notification queueを変更しない。

## Approved basis

- Requirements: [RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)。
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1「Binding作成」; [Object management: values and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2「typed value equivalence / failure atomicity」。
- Requirement case: [TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)。
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-BINDING-INIT-COPY-001`.
- Prior unit: [TwoWay group merge](binding-twoway-group-merge.md), `UNIT-CORE-053`.

## Steps

- [x] Register `UNIT-CORE-054` and the focused initial-copy oracle before source changes.
- [x] Implement typed, fallible candidate copying with target-kind validation.
- [x] Verify candidate preparation is non-mutating, equivalent values need no candidate, and UTF-8 ownership is independent.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

- [x] Complete `UNIT-CORE-054`.

## Verification

- Focused: `cargo test -p freemvvm-core binding_initial_copy`
- Workspace: `cargo test --workspace --locked`
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check`
- Hygiene: `git diff --check`

All checks passed on 2026-10-21: focused tests (4), workspace tests (128), `tools/dev.py check` including documentation links, clippy/build/C++ probes/.NET build/Python wheel checks, `cargo fmt --all -- --check`, and `git diff --check`.

## Scope boundary

This unit prepares an owned typed candidate only. It does not run target validators, commit the target value, publish a Binding graph edge/handle, enqueue notification, implement OneTime semantics, or expose C ABI. Those steps must join the candidate commit in a later atomic Runtime operation.
