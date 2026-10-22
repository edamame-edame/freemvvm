# Runtime通知sliceへのOneWay伝播統合

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-060`

## Goal

各Property generationのsubscriber callbackを始める前に、generation内dirty PropertyからOneWay伝播をcommitする。伝播targetの通知は現在generationのsnapshotに混ぜず次generationへ進め、Runtime sliceは従来どおり各Property itemを完了して処理する。

## Approved basis

- Requirements: [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-008](../11_requirements/RQ-07_binding.md#rq-07-008)。
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1「通知slice内の伝播」; [Object management: values and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2「外側処理区間 / 次generation」。
- Requirement cases: [TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)。
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-RUNTIME-BINDING-DELIVERY-001`.

## Steps

- [x] Register `UNIT-CORE-060` and the propagation-before-callback / next-generation oracle before source changes.
- [x] Run ordered propagation while preparing each dirty generation, before any subscriber callback begins.
- [x] Preserve the dirty generation if snapshot/candidate/notification reservation fails before propagation commit.
- [x] Verify callback reads see propagated values and target notifications are delivered in the following generation without recursion.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

## Verification

- Focused: `cargo test -p freemvvm-core runtime_one_way_delivery_propagates_before_subscribers_and_uses_next_generation` — 1 passed; Property delivery generation tests — 2 passed.
- Workspace: `cargo test --workspace --locked` — 139 passed.
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check` — passed, including document links, clippy, builds, C/C++ probes, .NET, and Python package checks.
- Hygiene: `cargo fmt --all -- --check`; `git diff --check`.

All checks passed on 2026-10-21.

## Scope boundary

This connects Runtime OneWay propagation to Property delivery for current internal registrations. It does not add application validators, Binding error snapshots, Runtime TwoWay/OneTime value behavior, Binding public owner/handle/cancel, or diagnostics/watchdog.
