# C ABI Property creation and kind query

- Updated: 2026-10-19
- Status: complete; implementation and verification passed
- Candidate unit: `UNIT-ABI-009`

## Scope

Expose the approved initial Property creation and `property_kind` operations through the versioned C ABI for BOOL, I64, UTF8, F64, and DECIMAL. A successful create returns one owned opaque Property handle; generic `handle_release` consumes it. Property tokens remain anchored to their owning Runtime until released. Creation and kind query are UI-thread-only. OBJECT stays reserved.

This unit excludes value getters/setters, validators, subscriptions, notifications, Bindings, and OBJECT Property support. It does not publish a C header or change the Core API into a wrapper-facing public interface.

## Approved basis

- Requirements: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017), [RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003), [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006), [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018), [RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020), [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002), and [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001).
- Detailed design: [Runtime ABI](../13_detail_design/runtime-objects/abi.md) v1.2, Property creation, kind, output and UI-thread rules; [Property notification and Binding propagation](../13_detail_design/events/notifications-binding.md) v1.1, initial kinds and create signatures; [Object handles, values, and errors](../13_detail_design/runtime-objects/objects-values.md) v1.2, owned token, Runtime anchor, and release cleanup.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md), UT-ABI-009.

## Steps

1. Record the focused behavioral oracle in `core-runtime.md` and register `UNIT-ABI-009` as planned in `tools/units.yaml` before Rust source edits.
2. Add internal Core ABI support to create each approved typed Property, resolve its owner for kind query, and release the Property token through the existing generic release operation while retaining the Runtime anchor for the token lifetime.
3. Add five create entrypoints and the `property_kind` C ABI entrypoint. Initialize every valid output before validation; reject invalid bool, invalid UTF-8/null-length combinations, invalid Runtime/property tokens, and wrong-thread calls without leaving partial state.
4. Test all five kind mappings, UTF-8 validation, zero-on-failure outputs, owner lifetime, UI-thread checks, null required outputs, and exactly-once release behavior. UTF-8 copy mechanics are covered by the existing Core value tests; this ABI slice accepts embedded NUL input.
5. Focused C ABI tests passed; `cargo test --workspace --locked` passed (91 tests); `.venv\\Scripts\\python.exe tools/dev.py check` passed; documentation links, Rust format/clippy, workspace build, bootstrap probes, and `git diff --check` passed.
6. Record implementation learning. Commit/push follows the previously authorized per-unit workflow after final review.

## Boundaries and implementation notes

- Stable ABI kind IDs are BOOL=1, I64=2, UTF8=3, F64=5, DECIMAL=6. OBJECT=4 is not creatable.
- UTF8 accepts null only with zero length, validates UTF-8, copies bytes before return, and preserves embedded NUL.
- `out_property` is required and initialized to zero; optional `out_error` is initialized to zero. Error snapshot creation is not added in this slice; the primary status is preserved.
- Do not derive the Runtime from token bits or keep a strong Runtime anchor after Property token release. Avoid holding a process registry lock while invoking Runtime/UI cleanup.
- The detailed-design rule that API calls are UI-thread-only applies to both creation and kind query. Generic release remains callable through the existing handle-release contract.

## Completion record

- Added C ABI create operations for BOOL/I64/UTF8/F64/DECIMAL and `property_kind`; PROPERTY kind IDs are 1/2/3/5/6, with OBJECT=4 reserved.
- Runtime owner anchors are retained alongside each public Property token and removed after successful generic release. A live Property remains queryable after its Runtime handle is released.
- The implementation and tests do not add getters/setters, Validator, subscription, notification, Binding, OBJECT Property, or Error snapshot creation.
