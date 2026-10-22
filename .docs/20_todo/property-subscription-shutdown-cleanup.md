# Property Subscription shutdown cleanup

- Updated: 2026-10-20
- Status: complete; `UNIT-CORE-050` passed and pushed

## Goal

At Runtime shutdown, discard pending Property notification work, cancel every active Runtime-owned Subscription, and destroy its callback closure exactly once on the UI thread before clearing Property state. No queued notification callback may start after shutdown takes priority.

## Approved basis

- Requirements: [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007), [RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008), [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.2, shutdown cleanup and registration lifecycle; [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1, shutdown precedence and Subscription ownership.
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-PROPERTY-NOTIFY-006`.
- Workflow: [unit-test strategy](../32_unit_test/unit-test-strategy.md), [Git rules](../02_git/rule.md), [Rust coding rules](../00_rule/coding_rule_rust.md).

## Steps

- [x] Register `UNIT-CORE-050` and `UT-CORE-PROPERTY-NOTIFY-006` before changing Rust sources.
- [x] Make the pending change queue, prepared-delivery iterator, and Runtime Subscription owner registry available to the Runtime's shutdown cleanup action.
- [x] At the first UI-state cleanup boundary, discard pending notification state, take active Subscription owners out of the registry, and cancel them outside the registry lock.
- [x] Clear Property UI state only after Subscription closures have been destroyed.
- [x] Verify shutdown wins over a queued notification, no callback starts, and destroy runs exactly once on the UI thread; run the affected checks and record results.

## Verification

- `cargo test -p freemvvm-core property_notification_shutdown_cancels_pending` — passed (1 test).
- `cargo test --workspace --locked` — passed (115 tests: 30 C ABI, 85 Core Runtime).
- `.venv\Scripts\python.exe -X utf8 tools/dev.py check` — passed, including 6,113 local links, Rust formatting/Clippy/build/tests, C/C++ probes, C# build, and Python package checks.
- `git diff --check` — passed.
- The focused test confirms a queued change is discarded when shutdown wins at the outer boundary, the active owner registry is drained, no notification callback starts, the closure is destroyed exactly once on the UI thread, and shutdown completion is reported only after cleanup.

## Scope boundary

This unit handles Runtime shutdown only. It does not add public subscribe/cancel entrypoints, Property-first teardown, Binding cleanup, or callback exception diagnostics.
