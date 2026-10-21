# Runtime OneWay伝播候補の準備

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-058`

## Goal

dirty Property identityから、既存OneWay graphの上流順edgeに沿ってRuntime内のowned value candidateを準備する。chainの中間target candidateは後続edgeのsourceとして使い、全target stateと通知queueは準備中に変更しない。

## Approved basis

- Requirements: [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)。
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1「通知slice内の伝播」; [Object management: values and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2「Property typed value」。
- Requirement cases: [TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)。
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-RUNTIME-BINDING-PROP-001`.

## Steps

- [x] Register `UNIT-CORE-058` and the chain candidate/failure-atomicity oracle before source changes.
- [x] Add a Runtime helper that snapshots source values and prepares target candidates along ordered OneWay edges, using prior candidates for downstream edges.
- [x] Verify chain/branch candidate results and that preparation leaves Property values, graph, and notification queue unchanged.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

## Verification

- Focused: `cargo test -p freemvvm-core runtime_one_way_propagation_prepares_chain_candidates` — 1 passed.
- Workspace: `cargo test --workspace --locked` — 137 passed.
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check` — passed, including document links, clippy, builds, C/C++ probes, .NET, and Python package checks.
- Hygiene: `cargo fmt --all -- --check`; `git diff --check`.

All checks passed on 2026-10-21.

## Scope boundary

This unit prepares candidates only. It does not commit target values, add target notifications, apply application validators, execute subscriber callbacks, or support Runtime TwoWay/OneTime propagation. Target validation, Binding error snapshots, and value/queue commit are later units.
