# HandleEntry record — implementation slice 3

- Updated: 2026-10-17
- Status: implementation, local validation, and test-design v0.3 approval complete
- Scope: generic live-reference record with kind and optional Runtime identity. No token map, concrete object type, Runtime identity representation, or thread-safety bound is selected here.

## Basis

- Approved requirements: [RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019), [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002).
- Approved detailed design: [objects-values.md](../13_detail_design/runtime-objects/objects-values.md) v1.1, “実装規則 / クラス設計の補足”. It names `HandleEntry` fields and matching/consume operations; object storage mechanics remain outside its scope.
- Approved test design: [core-runtime.md](../32_unit_test/core-runtime.md) v0.3, including two focused cases for entry metadata matching and one-shot owner consumption (user approval, 2026-10-17).
- Rust rule: [coding_rule_rust.md](../00_rule/coding_rule_rust.md).

## Steps

- [x] Confirm that Runtime and object owner concrete types are not yet implemented and are intentionally left generic in this slice.
- [x] Add `HandleEntry<K, R, O>` carrying kind, optional Runtime identity, and one optional owner reference; support kind/runtime matching, live-state query, and one-shot consume that returns the owner to its caller.
- [x] Add tests for match/mismatch without consuming the owner and one-shot consume that preserves a separately owned peer reference.
- [x] Add the two detailed tests to the core-runtime test document as v0.3, keeping v0.2 approval history.
- [x] Register the unit, source, command, requirement/design links, and test IDs in `tools/units.yaml`.
- [x] Run focused Rust tests and repository checks; save the scoped learning and exact results.

## Completion boundary

This slice implements the `HandleEntry` value semantics only. The registry's synchronized map, concrete object-owner type, Runtime identity type, pinning and final-release dispatch remain subsequent work; they must use approved Runtime/threading contracts rather than assumptions introduced here.

## Validation result

- `cargo test -p freemvvm-core`: 8 passed on Rust 1.90.0 and minimum 1.85.0.
- `cargo fmt --all --check`: passed.
- `.venv/Scripts/python.exe tools/dev.py check`: all 16 bootstrap gates passed; 76 active documents and 5,818 local links validated.
- The test-design document v0.3 is approved. `UNIT-CORE-001` through `UNIT-CORE-003` are ready; implementation and local validation are recorded separately from test-design approval.
