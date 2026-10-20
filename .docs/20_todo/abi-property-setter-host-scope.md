# C ABI Property setter host scope

- Updated: 2026-10-20
- Status: complete; `UNIT-ABI-012` passed and pushed

## Goal

Run each valid public Core C ABI Property setter inside a Runtime host-event scope. The outermost setter boundary must process the pending Property notification slice according to the approved notification design. Nested setter calls must defer delivery to their enclosing scope, and a setter failure must still close the scope.

## Approved basis

- Requirements: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002), [RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009), [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001), and [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005).
- Detailed design: [Property notifications and Binding propagation](../13_detail_design/events/notifications-binding.md), outermost API safe point and bounded delivery; [Runtime ABI](../13_detail_design/runtime-objects/abi.md), host event scope.
- Unit-test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md), `UT-ABI-012`.
- Workflow: [unit-test strategy](../32_unit_test/unit-test-strategy.md), [Git rules](../02_git/rule.md), [Rust coding rules](../00_rule/coding_rule_rust.md).

## Steps

1. Register `UNIT-ABI-012` and `UT-ABI-012` before source changes.
2. Add a RuntimeCore operation that opens a host scope, applies one typed Property candidate, and always closes the scope, preserving the operation error when both the setter and close fail.
3. Route Core-side C ABI Property setters through that operation and preserve existing API error mapping.
4. Verify outermost successful delivery, nested deferral, and scope closure after a rejected candidate.
5. Run the focused tests, workspace tests, repository development gate, documentation/link checks, formatting, and `git diff --check`.
6. Record implementation and test evidence in this plan, unit-test documentation, and `.docs/01_memory`; commit and push this verified unit.

## Scope boundary

This unit connects the existing Core Property setter path to the approved Runtime host boundary. It does not add a public subscription ABI, Binding propagation, callback exception diagnostics, or a new notification policy.

## Verification

- `cargo test -p freemvvm-core property_setter_host_scope` — passed (1 test).
- `cargo test --workspace --locked` — passed (115 tests: 30 C ABI, 85 Core Runtime).
- `.venv\Scripts\python.exe -X utf8 tools/dev.py check` — passed, including 6,106 local links, Rust formatting/Clippy/build/tests, C/C++ probes, C# build, and Python package checks.
- `git diff --check` — passed.
- The focused oracle confirms delivery at a standalone setter's outer boundary, nested setter deferral, and closure of a nested scope after a wrong-kind candidate while preserving `WrongType` mapping. The existing C ABI typed setter tests also passed in the workspace run.
