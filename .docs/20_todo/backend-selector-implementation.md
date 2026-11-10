# PresentationBackend selector and artifact implementation plan

- Plan version: 0.1
- State: plan approved by user (2026-11-10); implementation in progress
- Target release: `0.0.1`
- Parent branch: `change/platform-support-scope`
- Requirements: [RQ-11-025](../11_requirements/RQ-11_platform.md#rq-11-025)
- Approved designs: [Platform BD 1.3](../12_design/platform/abstraction.md), [Platform DD 2.8](../13_detail_design/platform/internal-layers.md), [Window Control DD 1.7](../13_detail_design/ui/controls.md), [C ABI DD 1.7](../13_detail_design/runtime-objects/abi.md), [package DD 1.2](../13_detail_design/build/toolchain-packages.md)
- Test designs: [case matrix 1.0](../30_release_test/requirement-test-matrix.md), [assertions 1.4](../30_release_test/requirements-test-assertions.md), [UI unit tests 1.9](../32_unit_test/ui.md)
- Design discussion: [selector API and test review](presentation-backend-selector-api-test-review.md)

## 1. Goal

Implement explicit `PresentationBackend` selection when creating a Window while preserving the existing default Window creation path. Keep the choice immutable for the Window object's lifetime, reject unknown or incompatible selections without silent fallback, and make the compiled backend set part of the native artifact build identity.

Python's current pure-Python wrapper and separate native runtime loading remain intact. If Python must load more than one backend library into one shared Core process, design and approve the plugin ABI and loader lifecycle as a separate gate before implementing that route.

## 2. Current implementation facts

- `crates/c-abi/src/lib.rs` selects a platform factory in `ui_abi_1_runtime_create` using target `cfg`. `crates/c-abi/Cargo.toml` has target-specific dependencies on `freemvvm-platform-linux` and `freemvvm-platform-windows`.
- Core creates a Window in `crates/core-runtime/src/ui_controls.rs`; `RuntimeCore::window_open` in `crates/core-runtime/src/runtime.rs` asks the Runtime's single platform factory to create its peer. The Window currently has no backend selector.
- The active Linux and Windows packages each have one registered platform adapter. This plan adds selection mechanics, but does not claim that Vulkan, WinAPI/GDI, Wayland/X11, framebuffer, or another alternative is already implemented or selectable.
- All language wrappers call the shared C ABI Window-create symbol. The Python wheel is currently a pure-Python `ctypes` facade; its loader lives in `wrappers/python/src/freemvvm/_native.py`.
- The unit catalog is sharded under `tools/units/`; implementation units must be registered there with concrete paths, tests, requirements, approved design references, and commands before being marked `ready`.

## 3. Scope and boundaries

Included:

- Core-owned Window selector value, registry resolution, failure atomicity, and immutability.
- Additive C ABI creation symbol and public header declaration; keep the existing symbol and its default behavior.
- Explicit backend build features/profile metadata on the native C ABI artifact, with target-specific dependencies and no unintended backend linked into an artifact.
- Rust, C++, C#, and Python Window creation surfaces using their language conventions; no setter or runtime hot swap.
- Unit, C ABI consumer, wrapper parity, native link/package, and CI checks appropriate to the changed units.
- Python packaging decision gate for the optional backend-plugin distribution path.

Not included:

- Implementing new renderer/surface adapters (for example Vulkan, WinAPI GDI, Wayland/X11, fbdev, or software framebuffer).
- Runtime-time Window backend swapping, fallback, preference heuristics, or catalog-enumeration APIs.
- Claiming a profile can select an identity unless its native artifact actually registers and documents that descriptor.
- Broadly changing existing native-library discovery or adding Python backend plugin loading before its ABI/lifecycle design is approved.

## 4. Branch and change flow

Create one Core APIs branch from the current parent after this plan is approved. Complete Core and C ABI changes there, then run the shared native artifact and all existing wrapper consumer checks. For each wrapper language, create a sibling branch from the updated Core APIs branch; do not base a wrapper branch on another wrapper branch.

```text
change/platform-support-scope
└── core-apis/presentation-backend-selector
    ├── wrapper-rust/presentation-backend-selector
    ├── wrapper-cpp/presentation-backend-selector
    ├── wrapper-csharp/presentation-backend-selector
    └── wrapper-python/presentation-backend-selector
```

If implementation exposes a Core/C ABI change while on a wrapper branch, stop that wrapper, return to the current parent/change branch, make a new Core APIs child, run Core/C ABI and all-wrapper compatibility checks, then recreate affected wrapper siblings from the updated Core branch. Push each implementation unit after its matching tests pass. Submit one grouped merge PR after all plan units and CI gates complete; remove merged implementation branches according to `.docs/02_git/rule.md`.

## 5. Implementation units and gates

The IDs below are proposed unit IDs; reserve/confirm them against `tools/units/` when registering the planned entries. Assign the next available numbers if a collision exists.

### Gate A — build feature contract (resolved 2026-11-10)

The initial registered identities follow the existing adapter/profile pairs: `windows.win32-d2d` for Windows 11 x64 and `linux.drm-gbm-egl-gles` for Ubuntu 24.04 x64 / the verified Debian 12 VM profile. Each currently has one registered descriptor; the selector adds no claim that another backend exists. The matching shared-native-artifact Cargo features are `presentation-backend-windows-win32-d2d` and `presentation-backend-linux-drm-gbm-egl-gles`. They will be optional target-specific platform dependencies and both will be enabled in the default feature set; target-specific dependency resolution includes only the adapter for the artifact target. The target triple plus enabled feature set will be recorded as artifact identity. Other target/profile feature sets must not advertise an unsupported default.

Build features belong to the shared `freemvvm-c-abi` artifact, not language wrappers. An artifact's feature set controls which backend descriptors and adapter dependencies are compiled; changing that set produces a distinct artifact identity/digest. A profile may include several compatible descriptors only when each is actually linked and registered. Reject a configuration that has no usable default for a UI profile.

Do not implement Python multi-library plugins in this gate. The plugin ABI, version negotiation, load location, dependency resolution, signing/trust, error translation, and handle lifetime require a dedicated design document and approval.

### Unit 1 — Core Window selector (`UNIT-CORE-094`, complete)

- Paths: `crates/core-runtime/src/ui_controls.rs`, `runtime.rs`, `ui_window.rs`, `crates/platform-linux/src/lib.rs`, `crates/platform-windows/src/lib.rs`, and Core tests.
- Work: add optional selector input to Core Window creation; preserve `create_window(runtime)` as the profile-default behavior for internal callers/tests. Store a Core-owned immutable identity/default selector on the Window. Resolve only against the Runtime profile's registered descriptor(s); reject an unregistered identity at create. At open, validate descriptor/surface/scene compatibility and initialize only that descriptor. Preserve Core Window/content when open fails. Reopen the same Window with the same identity; only destroying/recreating a Window allows a different choice.
- Tests: `UT-CORE-BACKEND-SELECTION-001` for default, explicit registered identity, unknown identity, incompatible surface/capability, init failure, no fallback, failure atomicity, close/reopen, and independent new Window. Assert factory-call records so a second backend cannot be attempted silently.
- Gate passed on `bbacaca4fd916d9ed3f3c554a8c3f3a45e88c752`: focused selector test, Core suite (204 tests), workspace Clippy, formatting, Rust 1.85 minimum check, and document/link validation.

### Unit 2 — C ABI selector and native header (`UNIT-ABI-023`, complete)

- Paths: `crates/c-abi/src/lib.rs`, `crates/c-abi/include/freemvvm/abi.h`, C/C++ ABI probes and consumer tests, ABI unit catalog.
- Work: add `ui_abi_1_window_create_with_presentation_backend(runtime, backend_id_utf8, backend_id_byte_length, out_window, out_error)`. Keep `ui_abi_1_window_create` unchanged and default-only. Validate pointer/length and UTF-8; synchronously copy the selector into Core; return zero output and no partial Window on failure; use existing status/error conventions.
- Tests: `UT-ABI-WINDOW-BACKEND-001`; header C/C++ compilation; runtime C consumer for default and explicit create; all existing ABI symbols/features remain compatible.
- Gate passed locally: C ABI suite (41 tests), workspace Clippy, format check, C and C++ header probes, and C/C++ ABI runtime consumer smoke. Core tests were already green on the parent Core unit revision.

### Unit 3 — target artifact backend features and identity (`UNIT-BUILD-BACKEND-001`, complete)

- Paths: `crates/c-abi/Cargo.toml`, `crates/c-abi/build.rs`, `crates/c-abi/src/lib.rs`, target profile metadata, build tests and unit catalog.
- Work: expose build-time inclusion of backend candidates at the shared native artifact boundary; keep target OS dependency constraints; define the default feature/profile for Windows 11 x64, Ubuntu 24.04 x64, and Debian 12 x64 VM. Generate build metadata containing profile, target triple, requested/active backend feature sets, registered backend IDs and artifact identity. Reject a target/profile with no usable default descriptor. Keep release target support declarations unchanged.
- Tests: build each existing official profile with its default feature set; build/check a deliberately reduced feature set; inspect Cargo dependency tree and linked symbols/artifact contents to verify excluded backend code is absent where the design permits; verify artifact identities differ when feature sets differ. Ensure `cargo build --workspace --locked` and package consumers still use one matching shared artifact.
- Gate: Rust workspace gates plus C/C++/C#/Rust/Python build/import/consumer checks against the same artifact. Do not infer a frame-rate gain from static linking; record binary-size deltas, and benchmark only if a real alternative implementation exists.
- Gate passed on `89cf3c5d6fe02bbae7796561d68adb853c3e79be`. C ABI default and Windows-only feature tests passed (42 each); no-backend Windows build rejected with a clear profile error; target-specific Cargo dependency trees and differing artifact identities were checked. Rust workspace tests, Clippy, format, Rust 1.85 workspace check, C++ 15 CTests and relocated package consumer, C# runtime/consumer, Python Window consumer, unit catalog and document/link validation passed locally. Windows and Ubuntu bootstrap CI passed in [run 42814890666](https://github.com/edamame-edame/freemvvm/actions/runs/42814890666). Windows x64 default vs Windows-only feature artifact size was 2,772,992 bytes in both builds (0-byte delta); no performance claim is made.
- Debian 12 VM profile is intentionally not reported as verified by the Ubuntu job. This environment had no reachable Docker daemon or confirmed Debian VM; its explicit profile build/runtime evidence remains under the release profile gate.

### Unit 4 — Rust wrapper (`UNIT-WRAP-RS-WINDOW-BACKEND-001` proposed)

- Paths: `wrappers/rust/freemvvm-sys/src/lib.rs`, `wrappers/rust/freemvvm/src/native_api.rs`, `ui.rs`, tests and consumer example if needed.
- Work: expose optional backend identity on Window creation with Rust `Option<&str>`-style API (final method shape must match existing wrapper conventions); retain the existing default convenience call. No mutable selector method.
- Tests: wrapper unit and consumer tests for default/explicit/error lifetime, plus `cargo test`/Clippy/fmt/minimum toolchain.
- Gate passed on `wrapper-rust/presentation-backend-selector` commit `618cb378ceeb7d4c1a0ad14b5256aa936ead5339`: the Rust wrapper exposes `Window::create_with_presentation_backend(ui, Option<&str>)`, keeps `Window::create(ui)` as the profile-default convenience, and forwards UTF-8 identity bytes only for the synchronous additive C ABI call. Fake-API tests cover default/explicit forwarding and returned errors; the linked consumer verifies the active Windows backend succeeds and an unknown identity fails. Full Rust workspace tests, Clippy, formatting, Rust 1.85 check, document/link checks, and unit catalog validation passed. Windows and Ubuntu bootstrap CI passed in [run 42817185615](https://github.com/edamame-edame/freemvvm/actions/runs/42817185615).

### Unit 5 — C++ wrapper (`UNIT-WRAP-CPP-WINDOW-BACKEND-001` proposed)

- Paths: `wrappers/cpp/include/freemvvm/ui_controls.hpp`, `src/ui_controls.cpp`, runtime dispatch table, tests and package consumer.
- Work: add an optional creation argument compatible with the existing C++ standard and exceptions-disabled result/error pattern; preserve source compatibility for default create. No post-create mutation.
- Tests: wrapper runtime tests, package relocation/consumer test, CMake build and lint/format gates.

### Unit 6 — C# wrapper (`UNIT-WRAP-CS-WINDOW-BACKEND-001` proposed)

- Paths: `wrappers/csharp/FreeMvvm/NativeApi.cs`, Window facade, tests and consumer.
- Work: add a generated/import declaration for the additive C ABI symbol and a creation overload/options input following existing `TryCreate`/`UiResult` patterns; preserve default create and error mapping.
- Tests: .NET build, runtime tests, package/consumer tests and wrapper parity against the shared native artifact.

### Unit 7 — Python wrapper (`UNIT-WRAP-PY-WINDOW-BACKEND-001` proposed)

- Paths: `wrappers/python/src/freemvvm/_native.py`, `__init__.py`, tests, pyproject/package tests.
- Work: add keyword-only `presentation_backend: str | None = None`, encode UTF-8 and pass the bytes for the duration of the C ABI call. Keep the current native runtime loader and pure-Python wrapper wheel. Do not add mutable backend properties.
- Tests: Python unit tests for bytes and error mapping, runtime consumer parity, wheel build/install/import. If the plugin design gate is later approved for the same release, add a separate Python package unit covering a backend-specific platform wheel and a PyInstaller `_MEIPASS` load smoke with actual runtime dependencies.

## 6. Shared verification and completion criteria

- Add each unit to the relevant `tools/units/*.yaml` shard as `planned`, then set `ready` only after source/test paths and commands exist. Keep release case `TC-BACKEND-SELECT-01` linked to RQ-11-025 and all implementation units.
- For every implementation unit, run its focused tests and gates in the same source revision. Commit and push that unit in Japanese after those checks pass, recording the commit SHA and CI result in the unit catalog.
- After Core/C ABI changes, verify all existing wrapper builds and real consumer probes before creating wrapper sibling branches.
- Compatibility gate passed on Core APIs revision `fc32f41c31664e39e30780fc197ab7587d306cdd` (2026-11-10): Rust workspace tests, C++ build/15 CTest cases, relocated C++ package consumer, C# runtime/consumer, and Python Window C ABI consumer all passed locally on Windows. The local repository runner could not start Python pytest because its `.venv` is CPython 3.15 while the runner requires standard CPython 3.13; the direct Python consumer passed, and the same revision's Windows and Ubuntu 24.04 bootstrap CI jobs both passed. This is host/CI compatibility evidence, not a test of the new wrapper selector surfaces, which remain in Units 4–7.
- At the end, run the repository's documented full checks for Windows and Linux CI, Python package tests, C/C++ package consumers, .NET consumers, Rust workspace and minimum compiler, static document/link/catalog validation, and target artifact inspection. Hardware profile verification remains separate from fake tests.
- Completion requires: default API compatibility; explicit selector lifecycle and no-fallback tests pass; official artifacts include only their declared feature set and have matching identities; all four wrappers preserve parity; CI passes; plan unit catalog and evidence links are complete; one grouped PR is ready for review.

## 7. Explicitly deferred decision

The currently approved package design shares the same canonical native C ABI artifact among all four wrappers. That design does not support Python having a private compiled feature set without changing package boundaries. If Python needs multiple dynamically loadable backend libraries, prepare a separate review plan/design for plugin C ABI, registration, version negotiation, dependency loading and trust, failure mapping, shutdown/unload, wheel/sdist build workflow, and PyInstaller packaging. No plugin ABI, `setup.py`-direct install flow, or loader-path behavior is implemented by this plan without that approval.

## Approval and change history

| Date | Plan version | Result |
| --- | --- | --- |
| 2026-11-10 | 0.1 | Created from the approved backend selector/API/test documents. Awaiting user review before implementation and before committing this plan. |
| 2026-11-10 | 0.1 | User's "ok" approved the implementation plan. Start with Core Window selector; confirm unit IDs against catalog before registration. |
| 2026-11-10 | 0.1 | Gate A resolved from the currently registered profile adapters. Began UNIT-CORE-094 on `core-apis/presentation-backend-selector`. |
| 2026-11-10 | 0.1 | UNIT-CORE-094 passed local gates and was committed/pushed as `bbacaca4fd916d9ed3f3c554a8c3f3a45e88c752` on `core-apis/presentation-backend-selector`. |
| 2026-11-10 | 0.1 | UNIT-ABI-023 passed C ABI, C/C++ header, and runtime consumer gates on the current Core APIs branch. |
| 2026-11-10 | 0.1 | UNIT-ABI-023 was committed/pushed as `d035b5caf0dcaadd6b33e1e824f58f2652035597`. Local Windows gates passed; Linux CI remains part of the branch integration gate. |
| 2026-11-10 | 0.1 | UNIT-BUILD-BACKEND-001 passed local feature/profile/artifact checks and Windows/Ubuntu CI; implementation commit `89cf3c5d6fe02bbae7796561d68adb853c3e79be`. |
| 2026-11-10 | 0.1 | UNIT-WRAP-RS-WINDOW-BACKEND-001 passed local Rust wrapper and consumer gates plus Windows/Ubuntu CI; implementation commit `618cb378ceeb7d4c1a0ad14b5256aa936ead5339` on `wrapper-rust/presentation-backend-selector`. |
