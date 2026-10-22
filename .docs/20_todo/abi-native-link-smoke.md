# Native C ABI link and runtime smoke

- Updated: 2026-10-22
- Status: implemented and verified; ready for commit
- Scope: build the Rust C ABI `cdylib`, link a native C consumer against it, load it on the host, and exercise version query plus Runtime create/shutdown/release.

## Approved basis

- Requirements: [RQ-04 ABI boundary](../11_requirements/RQ-04_abi-boundary.md), especially status and ABI version queries; [RQ-12 build/distribution](../11_requirements/RQ-12_build-distribution.md), especially target-specific artifact and initial CI gate.
- Architecture: [ADR-0001](../04_approval/adr-0001-rust-core.md), Accepted Rust Core with versioned C ABI.
- Detailed design: [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.2 and [Build/package](../13_detail_design/build/toolchain-packages.md) v1.1 Windows/Linux native compile/link gates.
- Test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md#ut-abi-native-link-smoke-001).

## Slice boundary

- Integrate Cargo's host-native C ABI library output with CMake for Windows x64 and Linux x64.
- Link and run one C consumer that checks Runtime version `1.0.0`, creates a Runtime, shuts it down, and releases the handle through the public header.
- This is a host-native smoke test only. It does not create a relocatable CMake package, install/export target, signed artifact, release support claim, or language wrapper.

## Steps

- [x] Confirm accepted ABI/version and initial native target gates.
- [x] Register the smoke oracle and `UNIT-ABI-018` before source changes.
- [x] Add Cargo native build integration, imported CMake target, C consumer, CTest entry, and a compiler-environment-aware focused command.
- [x] Run the focused build/test, workspace checks, and documentation gates.
- [x] Record evidence and implementation learnings.

## Validation

The exact focused command is in `tools/units.yaml`. Passing means the host compiler linked and loaded the current native library and the selected ABI calls returned expected values. It does not imply other targets or packaging work.

- `.venv\Scripts\python.exe tools\dev.py abi-smoke`: passed; C consumer linked to the Rust DLL and CTest completed version/Runtime lifecycle checks.
- `.venv\Scripts\python.exe tools\dev.py check`: passed.
- `cargo test --workspace --all-targets --locked`: passed (156 tests).
- `.venv\Scripts\python.exe tools\check_docs.py` and `git diff --check`: passed.
