# C ABI Property setters

- Updated: 2026-10-19
- Status: complete; implementation and verification passed
- Candidate unit: `UNIT-ABI-011`

## Scope

Expose typed Property setters for BOOL, I64, UTF8, F64, and DECIMAL. Calls are UI-thread-only, reject a kind mismatch, validate `ui_bool` and UTF8 input, and commit only after required input validation/copy succeeds. Equivalent candidates preserve the stored representation. OBJECT stays reserved.

This unit does not implement Validator registration/evaluation, notification delivery/subscriptions, Binding propagation, OBJECT setters, or validation error snapshots. These are separately tracked behaviors; this unit verifies direct setter/getter value results and failure atomicity.

## Approved basis

- Requirements: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017), [RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018), [RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019), [RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003), [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006), [RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019), [RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020), [RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Runtime ABI](../13_detail_design/runtime-objects/abi.md) v1.2, Property setter signatures and failure rules; [Property notification and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1, fixed kinds, equivalence, validation-before-commit, and notification boundary; [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, input copy and no partial state update.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-ABI-011.

## Steps

1. Register the setter-focused UT oracle and `UNIT-ABI-011` before changing source.
2. Add Core typed setter adapters that resolve the Property owner, validate UI-thread and handle state, then apply a same-kind candidate.
3. Add the five versioned C ABI entrypoints, validating bool and UTF8 inputs and preserving the original property on any rejection.
4. Test all kinds, same-value/equivalent representation retention, wrong kind, invalid token, wrong thread, bad bool, and UTF8 null/length and invalid sequence.
5. Focused setter tests passed (2 tests); `cargo test --workspace --locked` passed (96 tests); `.venv\Scripts\python.exe tools/dev.py check` passed, including docs links, Rust format/clippy/build, C/C++ probes, C# build, and Python package checks. `git diff --check` passed.
6. Record implementation learning, then commit and push under the previously authorized per-unit workflow.

## Boundaries

- BOOL accepts only 0 and 1. F64 accepts the approved NaN and infinity domain. DECIMAL uses the canonical 64-bit encoding, for which every bit pattern is valid.
- UTF8 `(NULL, 0)` denotes empty input; null with positive length is invalid. Input bytes are copied before commit, embedded NUL is preserved, and malformed UTF8 is rejected.
- Successful changed setters update the committed value synchronously. This unit does not yet expose subscriptions, so callback notification behavior is outside its executable slice and remains a separate follow-up.
- Equivalent F64 candidates (signed zero or any NaN pair) and numeric-equivalent DECIMAL candidates preserve the existing stored representation.
