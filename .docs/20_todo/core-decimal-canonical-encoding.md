# Core Decimal canonical value encoding

- Updated: 2026-10-19
- Status: complete
- Candidate unit: `UNIT-CORE-033`

## Scope

Add conversions between the internal `DecimalValue` fields and the approved canonical numeric `uint64_t` representation: sign at bit 63, scale at bits 58–62, and coefficient at bits 0–57. All 64-bit patterns are valid under the approved range. Host byte order and byte-array serialization remain out of scope.

## Approved basis

- Requirements: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017) and [RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019), RQ-02 v0.5 approved.
- Detailed design: [Notifications and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1 approved, Decimal canonical `uint64_t` encoding.
- Basic design: [Typed value areas and comparison/conversion](../12_design/runtime-objects/model-values.md) v1.0 approved.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-VALUE-007. This does not add a C ABI Property function or byte serialization.

## Steps

1. Registered `UNIT-CORE-033` and `UT-VALUE-007` before editing implementation.
2. Added canonical bit conversion methods on `DecimalValue`.
3. Verified field boundaries, sign, scale, coefficient, and round trips across all scale values.
4. Focused tests passed (2 tests); workspace tests passed (77 tests); `tools/dev.py check`, documentation validation, formatting, and `git diff --check` passed.
5. Unit is ready for commit/push.
