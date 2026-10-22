# Core Binding OneWay graph invariants

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-052`

## Goal

Add a Core-only topology model for OneWay Binding edges between singleton synchronization groups. Reject self-connections, duplicate edges, a second input to one target, and cycles without changing the accepted graph. This unit does not create or synchronize Properties, expose a C ABI, or add TwoWay group merging.

## Approved basis

- Requirements: [RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007), [RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015).
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1, Binding graph and approved OneWay topology rules; [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.2, Binding graph failure categories.
- Requirement cases: [TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01).
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-BINDING-GRAPH-001`.
- Workflow: [unit-test strategy](../32_unit_test/unit-test-strategy.md), [Git rules](../02_git/rule.md), [Rust coding rules](../00_rule/coding_rule_rust.md).

## Steps

- [x] Register `UNIT-CORE-052` and the focused graph oracle before source changes.
- [x] Add a private graph model and prepare/commit path for OneWay edges between singleton groups.
- [x] Verify legal chains and reject self, duplicate, second-input, and cycle connections with unchanged prior topology.
- [x] Run focused and workspace tests, the development gate, format, and diff checks; record the result and update the implementation memory.

## Verification

- Focused: `cargo test -p freemvvm-core binding_graph`
- Workspace: `cargo test --workspace --locked`
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check`
- Hygiene: `git diff --check`

All checks passed on 2026-10-21: focused graph tests (2), workspace tests (121), `tools/dev.py check`, `cargo fmt --all -- --check`, documentation links, and `git diff --check`.

## Scope boundary

This slice only validates graph topology. Binding handles and Runtime ownership, TwoWay union, initial value copy, propagation ordering, cancellation, validation, error snapshots, and C ABI remain later units. The approved topology rules are not extended here.
