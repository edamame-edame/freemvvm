# C ABI Runtime UI-thread query implementation plan

- Scope: implement only `ui_abi_1_runtime_is_ui_thread` using the existing Runtime handle registry and captured Runtime thread identity.
- Requirements: RQ-04-004 (opaque Runtime handle), RQ-04-006 (language-independent status), RQ-04-015 (no Rust panic across ABI), RQ-04-019 (invalid-reference failure result), and RQ-06-001 (Runtime-bound UI thread).
- Approved design: `.docs/13_detail_design/runtime-objects/abi.md` 1.1, API declaration, common status, and output initialization; `.docs/13_detail_design/runtime-objects/runtime.md` 1.1, Runtime identity query from any thread; `.docs/12_design/runtime-objects/runtime.md` 1.0, Runtime thread identity responsibility.
- Test design: verify `true` on the Runtime creation thread, `false` from a worker, `UI_INVALID_HANDLE` with zero output for an unknown handle, `UI_INVALID_ARGUMENT` for a null required output, and panic-to-`UI_INTERNAL_ERROR` conversion.

## Steps

1. Record `UT-ABI-005` in `.docs/32_unit_test/core-runtime.md` and `UNIT-ABI-005` in `tools/units.yaml`.
2. Add a narrow hidden Core support query that validates a nonzero Runtime token, pins the resolved Runtime owner during the call, and reads its captured UI-thread identity.
3. Implement the C ABI entrypoint with scalar output initialization, status mapping, and panic containment.
4. Run the focused C ABI and Core Runtime-handle tests, repository checks, MSRV check, documentation link check, and `git diff --check`.
5. Commit and push the verified unit.

## Boundaries

- The query is valid from any thread and returns a boolean; it does not enforce UI-thread restrictions on other APIs.
- Runtime shutdown and handle release remain separate units. This query does not mutate Runtime state.
- The implementation follows the already approved ABI and Runtime design; it does not revise either document.

## Progress

- Plan and test traceability: complete; registered as `UNIT-ABI-005` / `UT-ABI-005`.
- Implementation and validation: complete. Focused Runtime UI-thread query tests and Runtime handle registry tests passed; `tools/dev.py check`, `tools/dev.py minimum`, `tools/check_docs.py`, and `git diff --check` passed. Commit/push follows after final diff review.
