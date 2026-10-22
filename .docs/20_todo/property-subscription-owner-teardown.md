# Property Subscription owner teardown

- Updated: 2026-10-21
- Status: complete; `UNIT-CORE-051` verified on 2026-10-21

## Goal

When the last public owner of a Property is released, run its UI finalizer at a Runtime safe point. The finalizer removes pending notification work for that Property, cancels every Runtime-owned Subscription indexed by it, destroys callback closures on the UI thread after any in-flight callback returns, and then destroys the Property state. The Runtime remains available for other objects.

## Approved basis

- Requirements: [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007), [RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008), [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001), [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.2, registration ownership and UI-thread cleanup; [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1, Property teardown and pending delivery.
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-CORE-PROPERTY-NOTIFY-007`.
- Implementation memory: [Property Subscription cleanup](../01_memory/property-subscription-shutdown-cleanup.md).
- Workflow: [unit-test strategy](../32_unit_test/unit-test-strategy.md), [Git rules](../02_git/rule.md), [Rust coding rules](../00_rule/coding_rule_rust.md).

## Steps

- [x] Register `UNIT-CORE-051` and `UT-CORE-PROPERTY-NOTIFY-007` before changing Rust sources.
- [x] Drain accepted object cleanup tickets at the outer Runtime host safe point while Runtime remains Running; wake the host loop for accepted cleanup from any thread.
- [x] During a Property finalizer, remove its pending change and prepared delivery items, take its Subscription owner bucket, cancel outside locks, then destroy the Property state.
- [x] Ensure queued delivery checks cancellation between callback starts so Property teardown stops later callbacks in the same item.
- [x] Verify Property-first teardown destroys each closure once on the UI thread, suppresses pending callbacks, removes all Property references, and leaves the Runtime usable; run affected checks and record results.

## Verification

- Focused: `cargo test -p freemvvm-core property_owner_teardown`
- Workspace: `cargo test --workspace --locked`
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check`
- Hygiene: `git diff --check`

Results on 2026-10-21: focused 2 tests passed; workspace 117 tests passed; development gate passed, including 6,122 local links, Rust fmt/Clippy/build/test harness, C/C++ probes, C# build, and Python package checks. The focused tests cover worker-thread release waking a normal host turn, pending delivery suppression, exactly-once UI-thread closure destruction, release from inside a callback stopping later subscribers, and continued Runtime use.

## Scope boundary

This unit covers Core Runtime Property-first finalization and safe-point draining. It does not add public C ABI subscribe/cancel entrypoints or Binding teardown.
