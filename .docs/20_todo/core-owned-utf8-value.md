# Core owned UTF-8 value

- Updated: 2026-10-19
- Status: complete; `UNIT-CORE-027` ready after registered checks
- Candidate unit: `UNIT-CORE-027`

## Scope

Implement the narrow Core primitive needed to validate UTF-8 byte input and own an exact copy. Keep pointer/length validation, ABI status mapping, buffer output, normalization, and Property semantics outside this unit.

## Approved basis

- Requirements: [RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003), [RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020), and [RQ-05-004](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-004).
- Detailed design: [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, UTF-8 ownership rules.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-VALUE-001.

## Steps

1. Register `UNIT-CORE-027` as planned, linked to the approved UTF-8 design and UT-VALUE-001.
2. Add a private owned UTF-8 value type with fallible reservation, strict validation, exact byte preservation, and copy-before-return semantics.
3. Add focused tests for empty input, multibyte text, embedded NUL, invalid UTF-8, and source-buffer mutation after construction.
4. Run the focused/workspace tests, standard development checks, documentation checks, formatting, and whitespace validation.
5. Mark the unit ready and commit/push after the registered behavior check passes.

## Completion record

- Added the private `OwnedUtf8` Core value and strict byte validation with owned copying.
- Focused UT-VALUE-001 command passed (3 tests); `cargo test --workspace --locked` passed (61 tests).
- `.venv\Scripts\python.exe tools\dev.py check`, `tools/check_docs.py`, formatting, and `git diff --check` passed.
- The type remains internal and is not yet wired to a public Property or C ABI operation.

## Boundaries

- Do not expose a C ABI symbol or public Rust API.
- Do not decide a product-level maximum string size or add Unicode normalization/replacement.
- Rust allocation abort behavior is not converted into an artificial recoverable OOM contract; only `try_reserve` failure is returned when reported by the allocator.
