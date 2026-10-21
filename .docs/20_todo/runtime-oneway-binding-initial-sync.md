# Runtime OneWay Binding初期同期

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-056`

## Goal

Runtime UI thread上でOneWay接続を初期化する。Property handle/Runtime所属を検証し、source snapshot・kind・graph topologyを確認する。target値が変わる場合はnotification queue容量を確保した後でtarget値とgraph edgeをcommitし、必要なtarget notificationを一件だけ作る。

## Approved basis

- Requirements: [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)、[RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015)。
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1「Binding作成」; [Object management: values and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2「Property typed value」。
- Requirement cases: [TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-BIND-INIT-REJECT-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-init-reject-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)。
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-RUNTIME-BINDING-INIT-001`.
- Prior units: [initial copy preparation](binding-initial-value-copy.md), `UNIT-CORE-054`; [graph/value proposal preparation](binding-initial-graph-prepare.md), `UNIT-CORE-055`.

## Steps

- [x] Register `UNIT-CORE-056` and the focused Runtime create/atomicity oracle before source changes.
- [x] Store the Binding graph on RuntimeCore and add an internal OneWay initial-sync operation.
- [x] Verify changed/equivalent value success, notification behavior, and kind/topology/Runtime/thread failure atomicity.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

- [x] Complete `UNIT-CORE-056`.

## Verification

- Focused: `cargo test -p freemvvm-core runtime_one_way_binding_initial`
- Workspace: `cargo test --workspace --locked`
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check`
- Hygiene: `git diff --check`

All checks passed on 2026-10-21: focused tests (3), workspace tests (134), `tools/dev.py check`, `cargo fmt --all -- --check`, and `git diff --check`.

## Scope boundary

This is the internal OneWay initial-sync foundation. It does not implement application-supplied validators (none are connected to Core PropertyState yet), Runtime Binding strong ownership/public handle/cancel, post-create propagation, TwoWay or OneTime modes, or public C ABI. An internal successful initial edge is not completion of the public Binding contract.
