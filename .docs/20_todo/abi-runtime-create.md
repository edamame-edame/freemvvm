# C ABI Runtime creation implementation plan

- Scope: implement only `ui_abi_1_runtime_create` as the first C ABI operation that creates a Runtime owner handle.
- Requirements: RQ-04-004 (opaque handle), RQ-04-006 (status result), RQ-04-015 (no Rust panic across ABI), and RQ-04-018 (safe owner-reference issuance).
- Approved design: `.docs/13_detail_design/runtime-objects/abi.md` 1.1, API declaration, output/error rules, ownership table, and boundary panic rule; `.docs/13_detail_design/runtime-objects/runtime.md` 1.1, Runtime creation without a host-loop bridge.
- Test design: add a focused case verifying successful nonzero Runtime handle creation, optional error output initialization, null required output rejection, and zeroed output on failure. Injected allocation/token failures remain covered by Core-level registry tests where deterministic; status mapping is checked at the C ABI boundary where practical.

## Steps

1. Record the focused test in `.docs/32_unit_test/core-runtime.md` and register `UNIT-ABI-004` in `tools/units.yaml`.
2. Expose a narrow hidden Core support function that creates a `RuntimeCore`, registers its owning `Arc` in the process Runtime table, and maps registry allocation failure separately from other internal failures.
3. Implement the C ABI entrypoint with output initialization, null validation, panic-to-`UI_INTERNAL_ERROR` conversion, and status mapping (`UI_OUT_OF_MEMORY` for fallible registry reservation; `UI_INTERNAL_ERROR` for other creation/issuance failures).
4. Run the focused Core and C ABI tests, then workspace-required checks for the touched unit. Review `git diff --check` and traceability.
5. Commit and push this verified implementation unit.

## Boundaries

- Do not implement Runtime retain/release, shutdown, or `runtime_is_ui_thread` in this unit.
- Do not add error-object creation. `out_error` remains zero when no error handle can be produced, while the original failure status is returned.
- The C ABI entrypoint is implementation work under the already approved ABI design; this plan does not approve or revise the design itself.

## Progress

- Plan and test traceability: complete; registered as `UNIT-ABI-004` / `UT-ABI-004`.
- Implementation and validation: complete. Focused Runtime creation tests and Runtime handle registry tests passed; `tools/dev.py check` and `tools/dev.py minimum` passed. Commit/push follows after final diff review.
