# C ABI Binding create/cancel lifecycle

- Updated: 2026-10-21
- Status: complete; `UNIT-ABI-014` implemented and verified

## Goal

Expose the approved Binding creation and explicit cancellation contract through the versioned C ABI. OneTime performs only its initial copy and returns handle 0; OneWay and TwoWay return owned Binding handles. Releasing a Binding handle alone leaves the Runtime-owned connection active.

## Approved basis

- Requirements: [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002), [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002), [RQ-07-003](../11_requirements/RQ-07_binding.md#rq-07-003), [RQ-07-011](../11_requirements/RQ-07_binding.md#rq-07-011).
- Detailed design: [C ABI Binding contract](../13_detail_design/runtime-objects/abi.md), [Binding lifecycle](../13_detail_design/events/notifications-binding.md), [Runtime ownership and shutdown](../13_detail_design/runtime-objects/runtime.md).
- Existing Core lifecycle foundation: `UNIT-CORE-063`.
- Test oracle: [Core Runtime and C ABI tests](../32_unit_test/core-runtime.md#ut-abi-binding-lifecycle-001).

## Steps

- [x] Register `UNIT-ABI-014` and focused ABI test oracle before implementation.
- [x] Connect OneTime, OneWay, and TwoWay creation to the approved ABI mode values and output rules.
- [x] Add owned Binding handle registration, explicit idempotent cancel, and release-without-cancel behavior.
- [x] Verify failure outputs, invalid/wrong-thread handles, propagation after release, and propagation stop after cancel.
- [x] Run focused/workspace tests, development gate, format, docs links, and diff checks; record results and implementation memory.

## Scope boundary

This unit includes only synchronous Binding creation and cancellation for core-managed Properties. Property Validator registration, Binding error snapshots and subscriptions, type conversion, and language wrappers remain separate units. Existing approved RQ/DD behavior remains the oracle.

## Verification

- Focused: `cargo test -p freemvvm-c-abi binding_lifecycle` passed.
- Workspace: `cargo test --workspace --locked` passed (149 tests: C ABI 33, Core 116).
- Development gate: `.venv\Scripts\python.exe -X utf8 tools/dev.py check` passed, including Rust clippy/build, C/C++ probes, C# build, and Python package checks.
- Documentation links (76 documents, 6,226 local links), `cargo fmt --all -- --check`, units manifest JSON parse, and `git diff --check` passed.
- OneTime is implemented as an initial Runtime connection followed by immediate internal cancel; persistent modes retain the connection identity under owned Binding tokens. Validator/error snapshot/subscription APIs remain separate.
