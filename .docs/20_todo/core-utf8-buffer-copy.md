# Core UTF-8 caller-buffer copy

- Updated: 2026-10-19
- Status: complete; `UNIT-CORE-028` ready after registered checks
- Candidate unit: `UNIT-CORE-028`

## Scope

Add an internal copy helper for an existing `OwnedUtf8` value. It must report byte length, support a no-buffer capacity query, and compare capacity before writing so a short buffer remains unchanged.

## Approved basis

- Requirement: [RQ-04-021](../11_requirements/RQ-04_abi-boundary.md#rq-04-021).
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, UTF-8/buffer rule; [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.2, UTF-8 caller-buffer contract.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-VALUE-002.

## Steps

1. Register `UNIT-CORE-028` as planned with a dependency on `UNIT-CORE-027`.
2. Add a no-buffer length-query outcome and all-or-nothing copy behavior to `OwnedUtf8`.
3. Test length query, one-byte-short sentinel preservation, and exact-capacity byte copying with multibyte/NUL-containing input.
4. Run focused/workspace tests, standard development checks, documentation validation, formatting, and whitespace validation.
5. Mark ready and commit/push after the registered behavior checks pass.

## Completion record

- Added `OwnedUtf8::copy_to_buffer` with an explicit length-query result and preflight capacity check.
- Focused UT-VALUE-002 command passed (3 tests); `cargo test --workspace --locked` passed (64 tests).
- `.venv\Scripts\python.exe tools\dev.py check`, `tools/check_docs.py`, formatting, and `git diff --check` passed.
- The helper remains internal; C ABI pointer/capacity validation and `UI_BUFFER_TOO_SMALL` mapping remain future integration work.

## Boundaries

- Do not add C ABI pointer validation, status mapping, or public API symbols.
- Report lengths in bytes; do not append a NUL terminator.
- The capacity query and a later copy are separate reads; this helper does not promise that a mutable source will remain unchanged between API calls.
