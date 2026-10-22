# Runtime Binding active ownership and cancellation

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-063` Core lifecycle foundation implemented and verified

## Goal

Runtime-owned active Binding lifecycleのCore基盤を実装する。OneWay/TwoWayのactive connectionをRuntimeが強参照し、Property所有tokenのrelease後もendpointを保持する。内部cancelまたはshutdown後はその接続の保留・将来伝播を止め、Property確定値を保持する。

## Approved basis

- Requirements: [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-07-011](../11_requirements/RQ-07_binding.md#rq-07-011)。RQ-07 v0.7、RQ-05承認履歴を確認済み。
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1「Binding registration lifecycle」「通知slice内の伝播」。
- Runtime ownership: [Runtime: execution context, ownership, and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.2。
- Unit-test oracles: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-BINDING-GRAPH-CANCEL-001` and `UT-CORE-RUNTIME-BINDING-LIFECYCLE-001`.

## Steps

- [x] Register `UNIT-CORE-063` and the graph/lifecycle oracles before source changes.
- [x] Give each active OneWay/TwoWay connection a stable lifecycle identity so cancellation removes its topology and invalidates queued work.
- [x] Keep active connections under Runtime ownership independently from their external owner tokens; make explicit cancellation idempotent.
- [x] Cancel all active connections during Runtime shutdown and preserve committed Property values.
- [x] Verify endpoint token release retention, pending/future propagation suppression after cancel, TwoWay unlinking, and shutdown cleanup.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

## Scope boundary

This unit implements Core Runtime active registration, endpoint retention through Property token release, internal cancellation, topology removal, and shutdown invalidation for OneWay/TwoWay. Public Binding owner-token creation/cancel C ABI entrypoints, wrapper methods, Property Validator callbacks, Binding error snapshots/notifications, and OneTime's public handle result remain later units. It does not alter approved RQ/DD behavior.

## Verification

- Focused tests: graph OneWay/TwoWay cancellation, Runtime endpoint retention, canceled pending/future OneWay propagation, TwoWay group unlink, and shutdown cleanup passed.
- `cargo test --workspace --locked`: 148 tests passed (Core 116, C ABI 32).
- `.venv\\Scripts\\python.exe -X utf8 tools/dev.py check`: passed, including Rust clippy/build/test harness, C/C++ environment tests, C# build, and Python package checks.
- `cargo fmt --all -- --check`, `.venv\\Scripts\\python.exe -X utf8 tools/check_docs.py`, units JSON parse, and `git diff --check`: passed.
