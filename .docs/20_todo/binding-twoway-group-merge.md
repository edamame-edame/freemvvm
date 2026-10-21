# Core Binding TwoWay synchronization-group merge

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-053`

## Goal

Extend the Core Binding graph so a TwoWay connection can merge two singleton or existing synchronization groups. Prepare the merge against current topology and commit only if the merged graph remains valid. Rejected or stale proposals must leave the accepted groups and OneWay edges unchanged.

## Approved basis

- Requirements: [RQ-07-003](../11_requirements/RQ-07_binding.md#rq-07-003), [RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007), [RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015).
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1, TwoWay synchronization groups and topology validation after merge; [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.2, Binding graph failure categories and duplicate-edge precedence.
- Requirement case: [TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01).
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-BINDING-GRAPH-002`.
- Prior unit: [OneWay graph invariants](binding-oneway-graph-invariants.md), `UNIT-CORE-052`.

## Steps

- [x] Register `UNIT-CORE-053` and the focused TwoWay merge oracle before source changes.
- [x] Add prepared/committed group merge while preserving OneWay edge topology.
- [x] Reject merges that create a self-edge, duplicate edge, target with multiple inputs, or directed cycle; ensure failed/stale proposals do not change graph state.
- [x] Run focused and workspace tests, the development gate, format, documentation links, and diff checks; record results and implementation memory.

## Verification

- Focused: `cargo test -p freemvvm-core binding_graph`
- Workspace: `cargo test --workspace --locked`
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check`
- Hygiene: `git diff --check`

All checks passed on 2026-10-21: focused graph tests (5), workspace tests (124), `tools/dev.py check`, `cargo fmt --all -- --check`, documentation links, and `git diff --check`.

## Scope boundary

This unit models TwoWay group membership and verifies graph topology only. It does not create a Binding handle, copy Property values, propagate changes, cancel a connection, add validation/error snapshots, or expose C ABI functions.
