# C ABI public header

- Updated: 2026-10-22
- Status: implemented and verified; ready for commit
- Scope: publish the declarations for the C ABI functions that are currently exported by `freemvvm-c-abi`, and compile them as C11 and C++17.

## Approved basis

- Requirements: [RQ-04 ABI boundary](../11_requirements/RQ-04_abi-boundary.md), [RQ-05 ownership and callbacks](../11_requirements/RQ-05_ownership-callbacks.md), [RQ-06 threading and dispatch](../11_requirements/RQ-06_threading-dispatch.md), and [RQ-07 Binding](../11_requirements/RQ-07_binding.md).
- Architecture: [ADR-0001](../04_approval/adr-0001-rust-core.md), Accepted.
- Detailed design: [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.2, including fixed-width types, callbacks, ownership, status values, and currently implemented functions.
- Test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md#ut-abi-public-header-001).

## Slice boundary

- Add a guarded C header with fixed-width ABI types, callback typedefs, documented status constants, and declarations matching the currently exported `ui_abi_1_*` functions.
- Verify inclusion and representative typed function pointers in C11 and C++17 compile-only object targets.
- Do not declare design-only functions that are not currently exported. Do not add library linking, install/export packaging, wrapper APIs, or claim that the runtime is distributable.

## Steps

- [x] Confirm approved ABI roots and inspect current Rust exports.
- [x] Register the test oracle and implementation unit before source changes.
- [x] Add the public header and C/C++ compile-only probes.
- [x] Update development documentation to describe the header's actual status.
- [x] Run focused build, workspace tests, documentation checks, and development gates.
- [x] Record implementation learnings and verification evidence.

## Validation

The focused test command is recorded in `tools/units.yaml`. Compile-only object targets ensure declarations parse in both languages without requiring an implemented or linked native distribution artifact.

- `cmake --build --preset dev --target freemvvm_abi_header_probes`: passed; C11 and C++17 objects compiled.
- Header/export inventory comparison: passed; all 47 exported names declared once, with no extra declarations.
- `cargo test --workspace --all-targets --locked`: passed (156 tests).
- `.venv\Scripts\python.exe tools\dev.py check`: passed, including document links, native probes, Rust checks, .NET build, and Python package checks.
- `.venv\Scripts\python.exe tools\check_docs.py`: passed (76 active documents, 6262 local links).
- `git diff --check`: passed.
