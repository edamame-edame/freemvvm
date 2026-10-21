# Runtime OneWay伝播の値・通知commit

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-059`

## Goal

dirty Property generationからOneWay伝播候補を準備し、全target notification queue slotを先に予約した後、変更target値と次generation通知を一括commitする。伝播target通知は現在のdirty generationに混ぜず、次generationへ追加する。

## Approved basis

- Requirements: [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)。
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1「通知slice内の伝播」; [Object management: values and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2「Property commit / 次配信回」。
- Requirement cases: [TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)。
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-RUNTIME-BINDING-COMMIT-001`.

## Steps

- [x] Register `UNIT-CORE-059` and the propagation commit/next-generation oracle before source changes.
- [x] Prepare the ordered OneWay candidates while holding the Runtime UI state and graph stable.
- [x] Reserve every changed target's next-generation notification slot before committing any target value.
- [x] Verify chain/branch values, queue coalescing/order, and unchanged-value behavior.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

## Verification

- Focused: `cargo test -p freemvvm-core runtime_one_way_propagation_commits_candidates` — 1 passed.
- Workspace: `cargo test --workspace --locked` — 138 passed.
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check` — passed, including document links, clippy, builds, C/C++ probes, .NET, and Python package checks.
- Hygiene: `cargo fmt --all -- --check`; `git diff --check`.

All checks passed on 2026-10-21.

## Scope boundary

This unit commits only fixed-kind OneWay values already prepared by Core. Application validators and Binding error snapshots are not yet connected; do not treat this internal path as full public Binding support. Subscriber delivery ordering, Runtime delivery-loop integration, TwoWay/OneTime, Binding owner/handle/cancel remain later units.
