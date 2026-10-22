# Opaque handle token value — implementation slice 2

- Updated: 2026-10-17
- Status: implementation and local validation complete
- Scope: checked conversion between the ABI's `uint64_t` handle value and a Core-internal nonzero token type. No exported function, header, pointer conversion, or registry table.

## Basis

- Approved requirement: [RQ-04-004](../11_requirements/RQ-04_abi-boundary.md#rq-04-004).
- Approved detailed designs: [共通C ABI](../13_detail_design/runtime-objects/abi.md) v1.1 (`ui_handle` is `uint64_t`, zero is invalid) and [handle・値・error](../13_detail_design/runtime-objects/objects-values.md) v1.1 (opaque, nonzero token).
- Approved test design: [core-runtime.md](../32_unit_test/core-runtime.md) v0.2, UT-HANDLE-002 invalid-token case.
- Rust rule: [coding_rule_rust.md](../00_rule/coding_rule_rust.md).

## Steps

- [x] Record user approval of test-design document v0.2 and mark the completed issuer unit ready.
- [x] Add a transparent internal `HandleToken` newtype over `NonZeroU64`; checked conversion rejects zero and conversion back returns the same `uint64_t` value.
- [x] Test zero rejection, valid boundary-value round trips, and representation size/alignment.
- [x] Add this implementation slice to `tools/units.yaml` with its source, command, requirement, design documents, and UT-HANDLE-002 mapping.
- [x] Save the scoped learning under `.docs/01_memory/` and record exact validation results here.
- [x] Run the focused tests on the pinned and minimum Rust toolchains; run relevant repository gates and inspect the diff.

## Completion boundary

The result is a typed, zero-rejecting conversion for future registry and ABI work. It does not create the public C header, exported ABI functions, live-handle lookup, or ownership behavior.

## Validation result

- `cargo test -p freemvvm-core`: 6 passed, including both token-value tests.
- `rustup run 1.85.0 cargo test -p freemvvm-core`: 6 passed.
- `cargo fmt --all --check`: passed.
- `.venv/Scripts/python.exe tools/dev.py check`: all 16 bootstrap gates passed; 76 active documents and 5,813 local links validated.
- Windows x64 only; no exported ABI or cross-target layout claim was tested in this slice.
