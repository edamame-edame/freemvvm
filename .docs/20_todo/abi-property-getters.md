# C ABI Property getters

- Updated: 2026-10-19
- Status: complete; implementation and verification passed
- Candidate unit: `UNIT-ABI-010`

## Scope

Expose the approved typed Property getters for BOOL, I64, UTF8, F64, and DECIMAL through the versioned C ABI. Scalar getters return the current committed value. UTF8 uses caller-owned `(buffer, capacity, out_length)` storage and preserves exact bytes, including embedded NUL. All getters are UI-thread-only and reject a getter whose type does not match the Property's fixed kind. OBJECT remains reserved.

This unit excludes setters, validators, notifications, subscriptions, Bindings, object-valued Properties, Error snapshots, and same-snapshot guarantees across separate UTF8 size-query and copy calls.

## Approved basis

- Requirements: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017), [RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003), [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006), [RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019), [RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020), [RQ-04-021](../11_requirements/RQ-04_abi-boundary.md#rq-04-021), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Runtime ABI](../13_detail_design/runtime-objects/abi.md) v1.2, Property getter signatures, wrong-type/thread behavior, and output initialization; [Property notification and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1, initial Property type representations; [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, caller-buffer no-partial-write rule.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-ABI-010.

## Steps

1. Record the focused getter oracle and register `UNIT-ABI-010` as planned in `tools/units.yaml` before changing Rust source.
2. Add an internal borrowed-state read boundary in Core so scalar extraction and UTF8 caller-buffer copying happen while the validated Property pin, UI-thread check, and Runtime state borrow are active, without allocating a duplicate UTF8 value.
3. Add typed getter support and C ABI entrypoints. Initialize scalar/length outputs before validation; map invalid handle, wrong kind, wrong thread, closed Runtime, null output, and short buffer to their defined status results.
4. Test all five kind-specific getters, wrong-kind rejection, invalid handles, null required output, UI-thread rejection, UTF8 size query, exact copy with embedded NUL, and short-buffer no-partial-write behavior.
5. Focused getter tests passed; `cargo test --workspace --locked` passed (94 tests); `.venv\Scripts\python.exe tools/dev.py check` passed, including documentation links, Rust format/clippy, workspace build, C/C++ probes, C# build, and Python package checks; `git diff --check` passed.
6. Record implementation learning. Commit/push follows the previously authorized per-unit workflow after final review.

## Completion record

- Added typed getters for BOOL, I64, UTF8, F64, and DECIMAL with the approved `UI_WRONG_TYPE`, thread, invalid-handle, and output-initialization behavior.
- UTF8 uses a borrowed Property state and caller buffer. Size queries, exact byte copies, and short-buffer preservation are verified; no NUL terminator or cross-call snapshot guarantee is added.
- The implementation does not add setters, validation, notification, subscriptions, Bindings, OBJECT, or Error snapshots.

## Boundaries

- Stable kind IDs are BOOL=1, I64=2, UTF8=3, F64=5, DECIMAL=6. Getter function signatures use their declared scalar C types; DECIMAL returns the same canonical `uint64_t` bit representation.
- UTF8 `(NULL, 0, out_length)` is a successful required-byte-count query. A null buffer with positive capacity is invalid. If capacity is too small, return `UI_BUFFER_TOO_SMALL`, set only `out_length` to the required byte count, and leave the entire caller buffer unchanged. Successful copy is not NUL-terminated.
- The size-query and copy calls may observe different values if a subsequent setter changes the Property; no snapshot pin crosses ABI calls.
- Preserve the existing public Runtime owner anchor and generic Property release behavior from UNIT-ABI-009.
