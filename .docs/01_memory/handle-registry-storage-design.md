# freemvvm handle-registry storage and release-thread design

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, Rust core-runtime HandleRegistry.

- Approved `objects-values.md` v1.2 specifies a synchronized token map, `Arc<CoreObjectControl>`, `ValidationPin`, UI-thread-only `UiObjectStore`, and Runtime cleanup tickets for Runtime-owned objects.
- RQ-05-010 requires retain/release from any thread and distinguish accepted release from UI-resource cleanup completion when deferred. RQ-06-001 confines UI-state operations to the Runtime UI thread; RQ-06-003 requires synchronization of shared mutable state.
- Before the user's selection, the ownership boundary, pin lifetime, final-release dispatch, queue-failure semantics, and treatment of non-Runtime immutable objects were open; implementation must follow the approved DD revision rather than choose them by convenience.
- The selected architecture separates thread-safe identity/lifetime control from thread-confined UI state, with registry detachment under lock and final UI cleanup handed to Runtime after unlocking.
- User selected Option A (“A推奨で”) and approved the complete DD v1.2 on 2026-10-17. Runtime-free immutable ErrorSnapshot ownership remains unchanged and outside this revision.
- User approved `.docs/32_unit_test/core-runtime.md` v0.4 on 2026-10-17. At that point `UNIT-CORE-004` stayed `planned` until its source and commands existed and passed, as required by `.docs/32_unit_test/implementation-test-inventory.md`.
- `UNIT-CORE-004` now implements the synchronized registry map, call-duration validation pins, token retention, and final-release cleanup reservation in `crates/core-runtime/src/handle_registry/registry.rs`. The focused module suite passed all five approved cases; `tools/units.yaml` marks this unit `ready`. The Runtime shutdown queue/drain integration remains separately planned as `UNIT-CORE-005`.
- `UNIT-CORE-005` now implements the Runtime cleanup queue and shutdown drain in `crates/core-runtime/src/runtime.rs`. Shutdown accepts/drains queued finalizers and tracked UI-state cleanup before publishing `Closed`; the approved test also verifies that a remaining public shell token releases after shutdown without scheduling UI cleanup. All 14 core-runtime tests and the full development check passed, so both CORE-004 and CORE-005 are `ready`.
