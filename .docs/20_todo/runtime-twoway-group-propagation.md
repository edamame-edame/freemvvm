# Runtime TwoWay同期group伝播

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-062` implemented and verified

## Goal

Property notification generationの準備時にTwoWay groupを同期し、最後に成功した直接setterを勝者にする。OneWay edgeがTwoWay groupを入力元または出力先とする場合も、同じ上流順序でgroup全体へ反映する。

## Approved basis

- Requirements: [RQ-07-003](../11_requirements/RQ-07_binding.md#rq-07-003)、[RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-008](../11_requirements/RQ-07_binding.md#rq-07-008)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)。
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1「外側UI処理区間の直接setter順序」「TwoWay同期グループ」「通知slice内の伝播」。
- Object/value design: [Object management: values and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2「Property generation / next generation」。
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-RUNTIME-TWOWAY-PROPAGATION-001`.

## Steps

- [x] Register `UNIT-CORE-062` and the last-successful-direct-setter/group delivery oracle before source changes.
- [x] Retain per-generation direct-change recency separately from first-change notification order, including restoration after preparation failure.
- [x] Prepare OneWay and TwoWay group candidates in graph order, with incoming OneWay source values overriding same-generation direct target values.
- [x] Commit changed group members before subscriber callbacks; coalesce already-current members and enqueue other changed members for the next generation.
- [x] Verify latest successful direct setter wins, a failed attempt does not win, OneWay source/target group interaction, and callbacks see committed group values without recursion.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

## Scope boundary

This implements Runtime Property value propagation for existing internal OneWay edges and TwoWay groups. It does not implement application validators/Binding error snapshots, public Binding ownership/handle/cancel, C ABI, OneTime, or non-convergence diagnostics.

## Verification

- Focused tests: `runtime_two_way_binding_initial_updates_the_entire_existing_target_group`, `runtime_two_way_group_propagation`, `runtime_one_way_edges_propagate_into_and_out_of_two_way_groups`, and `runtime_two_way_binding_initial` passed.
- `cargo test --workspace --locked`: 144 tests passed.
- `.venv\\Scripts\\python.exe -X utf8 tools/dev.py check`: passed, including Rust clippy/build/test harness, C/C++ environment tests, C# build, and Python package checks.
- `cargo fmt --all -- --check`, `.venv\\Scripts\\python.exe -X utf8 tools/check_docs.py`, units JSON parse, and `git diff --check`: passed.
