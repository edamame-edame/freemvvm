# C ABI Runtime handle retain implementation plan

- Scope: implement `ui_abi_1_handle_retain` for the Runtime handles currently issued by the Core, creating an independent token while preserving the borrowed input token.
- Requirements: RQ-04-005 (explicit retain/release ownership references), RQ-04-006 (status), RQ-04-015 (no Rust panic across ABI), RQ-04-018 (safe owner-reference issuance), RQ-04-019 (invalid reference result), and RQ-05-002 (independent ownership references).
- Approved design: `.docs/13_detail_design/runtime-objects/abi.md` 1.1, ownership API and common statuses; `.docs/13_detail_design/runtime-objects/objects-values.md` 1.2, retain preserves the input token and creates a new token for the same object.
- Test design: verify successful distinct nonzero tokens to the same Runtime, source token remains usable, invalid/zero source leaves output zero, null output rejection, allocation/status mapping, and panic containment. Add a registry-level test to ensure retain pins the same Runtime owner and preserves the source entry.

## Steps

1. Record `UT-RUNTIME-HANDLE-RET-001` and `UT-ABI-006` in `.docs/32_unit_test/core-runtime.md`; register `UNIT-ABI-006` in `tools/units.yaml`.
2. Add an atomic Runtime registry retain operation that validates, reserves, issues, and inserts a new token under the registry lock.
3. Expose a narrow Core ABI-support function and add the C ABI entrypoint with output initialization, status mapping, and panic containment.
4. Run focused registry/C ABI tests, workspace standard checks, MSRV check, docs validation, and `git diff --check`.
5. Commit and push the verified unit.

## Boundaries

- `handle_retain` covers Runtime handles currently issued by the Core. Future object/error handle kinds must use the same public contract when those types are implemented.
- Do not implement `handle_release` in this unit. A Runtime's final public owner must be coordinated with shutdown completion before its UI-affine state can be dropped.
- Do not implement or change shutdown/host bridge behavior here.

## Progress

- Plan and test traceability: complete; registered as `UNIT-ABI-006`, `UT-RUNTIME-HANDLE-RET-001`, and `UT-ABI-006`.
- Implementation and validation: complete. Focused registry and C ABI tests passed; `tools/dev.py check`, `tools/dev.py minimum`, inventory traceability validation, and `git diff --check` passed. Commit/push follows after final diff review.
