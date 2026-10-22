# Native C++ ABI link and runtime smoke

- Updated: 2026-10-22
- Status: committed and pushed (`f3c3f78`)
- Scope: compile and link a native C++17 consumer against the Cargo-generated C ABI library, then exercise its C-linkage declarations at runtime.

## Approved basis

- Requirements: [RQ-04 ABI boundary](../11_requirements/RQ-04_abi-boundary.md), [RQ-03 language support](../11_requirements/RQ-03_language-support.md), and [RQ-12 build/distribution](../11_requirements/RQ-12_build-distribution.md).
- Architecture: [ADR-0001](../04_approval/adr-0001-rust-core.md), Accepted Rust Core with versioned C ABI.
- Detailed design: [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.2, [C++ wrapper](../13_detail_design/wrappers/cpp.md) v1.1, and [Build/package](../13_detail_design/build/toolchain-packages.md) v1.1.
- Test oracle: [Core Runtime tests](../32_unit_test/core-runtime.md#ut-abi-cpp-native-link-smoke-001).

## Slice boundary

- Build the existing C ABI native artifact, compile a C++17 translation unit that includes `freemvvm/abi.h`, link it to that artifact, load it, and exercise version query plus Runtime host-boundary shutdown lifecycle.
- This verifies C-linkage usability from C++ only. It does not establish the not-yet-finalized public C++ wrapper API, exceptions/RAII behavior, relocatable package, or additional platform support.

## Steps

- [x] Confirm the accepted C ABI, C++ language target, and host-native smoke boundary.
- [x] Register the oracle and `UNIT-ABI-019` before source changes.
- [x] Add a C++17 consumer and CMake/CTest focused target.
- [x] Run focused smoke, workspace tests, and development/document checks.
- [x] Record results and implementation learnings.

## Validation

The exact focused command is recorded in `tools/units.yaml`. Passing proves only the tested host's C++ compiler/linker/loader path to the native C ABI.

- `.venv\Scripts\python.exe tools\dev.py abi-smoke-cpp`: passed; C++17 consumer linked to and loaded the Rust DLL, then completed the version and Runtime lifecycle checks.
- `.venv\Scripts\python.exe tools\dev.py check`: passed.
- `cargo test --workspace --all-targets --locked`: passed (156 tests).
- `.venv\Scripts\python.exe tools\check_docs.py` and `git diff --check`: passed.
