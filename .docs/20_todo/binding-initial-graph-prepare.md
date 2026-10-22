# Binding初期値とgraph接続の準備

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-055`

## Goal

OneWay Binding作成準備として、graph topologyの検査とtarget用初期値候補の準備を一つのprepared proposalにまとめる。self/cycle/duplicate/second-input、kind mismatch、copy failureではgraphの論理状態とtarget値を変更しない。

## Approved basis

- Requirements: [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)、[RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015)。
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1「Binding作成」; [Object management: values and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2「typed value equivalence / failure atomicity」。
- Requirement case: [TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-BIND-INIT-REJECT-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-init-reject-01)。
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-BINDING-INIT-PREPARE-001`.
- Prior units: [initial copy candidate](binding-initial-value-copy.md), `UNIT-CORE-054`; [TwoWay group merge](binding-twoway-group-merge.md), `UNIT-CORE-053`.

## Steps

- [x] Register `UNIT-CORE-055` and the focused prepare/failure-atomicity oracle before source changes.
- [x] Combine OneWay graph validation and typed initial-copy preparation into one owned proposal.
- [x] Verify successful preparation changes neither graph topology/generation nor target value; rejection preserves existing topology and target.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

- [x] Complete `UNIT-CORE-055`.

## Verification

- Focused: `cargo test -p freemvvm-core binding_initial_prepare`
- Workspace: `cargo test --workspace --locked`
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check`
- Hygiene: `git diff --check`

All checks passed on 2026-10-21: focused tests (3), workspace tests (131), `tools/dev.py check`, `cargo fmt --all -- --check`, and `git diff --check`.

## Scope boundary

This unit prepares graph and typed-value candidates only. It does not run registered application validators, commit target value and graph together, reserve/commit notification, create Runtime ownership or public Binding handles, implement TwoWay/OneTime behavior, or expose C ABI. Candidate commit integration must keep existing approved validation and atomicity boundaries.
