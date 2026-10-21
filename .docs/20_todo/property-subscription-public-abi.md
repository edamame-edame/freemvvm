# Property subscribe/cancel public ABI

- Updated: 2026-10-21
- Status: complete; `UNIT-ABI-013`

## Goal

Expose the approved Property notification subscription contract through the versioned C ABI. Successful subscription transfers callback/userdata/destroy ownership to Core and returns an owned Subscription token. Explicit UI-thread cancellation is idempotent and prevents new callback starts; releasing or retaining Subscription tokens does not alter the active registration.

## Approved basis

- Requirements: [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002), [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006), [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007), [RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008), [RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009), [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001), [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005).
- Detailed design: [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.2, Property subscribe/cancel signatures and failure/ownership rules; [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1; [Runtime ownership and cleanup](../13_detail_design/runtime-objects/runtime.md) v1.2.
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-ABI-013`.
- Related implementation memory: [Property Subscription cleanup](../01_memory/property-subscription-shutdown-cleanup.md).
- Workflow: [unit-test strategy](../32_unit_test/unit-test-strategy.md), [Git rules](../02_git/rule.md), [Rust coding rules](../00_rule/coding_rule_rust.md).

## Steps

- [x] Register `UNIT-ABI-013` and `UT-ABI-013` before changing Rust sources.
- [x] Add Core ABI support for Property subscribe/cancel and Subscription token lookup, retain, and release while leaving active Runtime ownership independent.
- [x] Export `ui_abi_1_property_subscribe` and `ui_abi_1_subscription_cancel` with required callback/destroy validation and failure-atomic outputs.
- [x] Verify success, release-versus-cancel semantics, repeated cancel, self-cancel, callback destruction, invalid inputs/handles, wrong thread, and token retain/release; run affected checks and record results.

## Verification

- Focused: `cargo test -p freemvvm-c-abi property_subscription`
- Workspace: `cargo test --workspace --locked`
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check`
- Hygiene: `git diff --check`

All checks passed on 2026-10-21: focused subscription tests (2), workspace tests (119), `tools/dev.py check`, `cargo fmt --all -- --check`, and `git diff --check`.

## Scope boundary

This unit adds the C ABI and Core token support for Property subscriptions. It does not add language wrappers, Binding subscriptions, Validator registrations, generated headers, or feature IDs that are not assigned by the approved ABI design.
