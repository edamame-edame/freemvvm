# C ABI F32/F64 conversion

- Updated: 2026-10-18
- Status: implemented and verified; ready for commit
- Scope: `UNIT-ABI-003` implements the pure Core codec and its two C ABI conversion entry points for RQ-04-029.

## Approved basis

- Requirements: [RQ-04-002](../11_requirements/RQ-04_abi-boundary.md#rq-04-002), [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006), [RQ-04-010](../11_requirements/RQ-04_abi-boundary.md#rq-04-010), and [RQ-04-029](../11_requirements/RQ-04_abi-boundary.md#rq-04-029).
- Basic design: [Model values](../12_design/runtime-objects/model-values.md) v1.0, F32 route and numeric conversion.
- Detailed design: [Objects, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, FloatCodec and Numeric conversion; [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.1, conversion API declarations.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-NUMERIC-001 and UT-ABI-003; release cases TC-NUM-01 and TC-ABI-REP-01.
- Architecture constraint: [ADR-0001](../04_approval/adr-0001-rust-core.md), Accepted.

## Slice boundary

- Implement exact F32-to-F64 expansion and F64-to-F32 conversion with `EXACT=0` and `NEAREST_EVEN=1`.
- Use deterministic bit-level classification and nearest-even rounding; do not depend on ambient floating-point rounding mode.
- Add compile-time format checks for binary32 and binary64 widths and exponent/significand parameters before advertising the feature.
- Reject finite values outside F32 range and values that cannot round-trip under `EXACT`.
- Normalize NaN to quiet NaN without preserving payload; preserve signed zero and infinity sign.
- Export the approved C ABI functions, initialize outputs before validation, and return status without adding parser, formatter, Property kinds, or wrapper APIs.
- Advertise feature ID 3 (`numeric.f32_bridge.v1`) only after both conversion symbols and their full behavior are present; leave other incomplete feature IDs unsupported.

## Steps

- [x] Confirm approved RQ-04-029, BD v1.0, DD v1.2/v1.1, and UT-NUMERIC-001.
- [x] Register `UNIT-ABI-003` as planned before editing implementation.
- [x] Implement Core FloatCodec and focused numeric boundary tests.
- [x] Add C ABI entry points and tests for mode, output, and failure behavior.
- [x] Run focused/workspace tests, development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready only after both registered commands pass.

## Validation

The Core and C ABI unit test commands are recorded in `tools/units.yaml`. The conversion API remains an implementation under construction and does not create a distributed header.

- `cargo test -p freemvvm-core numeric::tests`: passed (7 tests, including deterministic IEEE cast comparison across finite samples).
- `cargo test -p freemvvm-c-abi numeric`: passed (3 tests).
- `cargo test -p freemvvm-c-abi has_feature`: passed; feature ID 3 is reported supported.
- `cargo test --workspace --all-targets --locked`: passed (33 tests).
- `.venv\Scripts\python.exe tools\dev.py check`: passed.
- `.venv\Scripts\python.exe tools\check_docs.py`: passed (76 active documents, 5862 local links).
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `dumpbin /exports target\debug\freemvvm_abi_1.dll`: confirmed both numeric conversion symbols are exported.
