# Runtime OneWay伝播順序の準備

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-057`

## Goal

承認済みDAGから、直接変更されたProperty群に届くOneWay edgeだけを抽出し、上流から下流へ評価できる順序でowned snapshotとして返す。これは値更新を行わない準備sliceとし、Runtime通知処理への結合・値commit・Binding error状態は後続単位に残す。

## Approved basis

- Requirements: [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)。
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1「通知slice内の伝播」「配信中のgraph変更可視性」。
- Requirement cases: [TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)。
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-BINDING-PROPAGATION-ORDER-001`.

## Steps

- [x] Register `UNIT-CORE-057` and the focused order/reachability oracle before source changes.
- [x] Add a fallible graph snapshot operation that returns only edges reachable from the input dirty properties, in deterministic upstream-to-downstream order.
- [x] Verify branching, chain, unrelated edge exclusion, TwoWay group identity resolution, and empty input.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

## Verification

- Focused: `cargo test -p freemvvm-core binding_propagation_order` — 2 passed.
- Workspace: `cargo test --workspace --locked` — 136 passed.
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check` — passed, including document links, clippy, builds, C/C++ probes, .NET, and Python package checks.
- Hygiene: `cargo fmt --all -- --check`; `git diff --check`.

All checks passed on 2026-10-21.

## Scope boundary

This unit computes topology/order only. It does not read/commit Property values, integrate with Runtime delivery, create/cancel Binding owners, run validators, or implement TwoWay/OneTime propagation. Snapshot OOM returns an error without changing graph topology.
