# HandleRegistry storage and release-thread design decision

- Updated: 2026-10-17
- Status: UNIT-CORE-004 and UNIT-CORE-005 implemented and ready
- Scope: implement the approved Runtime-owned HandleRegistry storage, validation pins, final-release handoff, and failure behavior in small testable slices.

## Basis and unresolved gap

- Approved requirements: [RQ-04 ABI boundary](../11_requirements/RQ-04_abi-boundary.md#rq-04-018), [RQ-05 ownership](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010), and [RQ-06 threading](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001), [RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003).
- Approved detailed design: [handle and value design](../13_detail_design/runtime-objects/objects-values.md) v1.2, [runtime and shutdown design](../13_detail_design/runtime-objects/runtime.md) v1.1, and [ABI design](../13_detail_design/runtime-objects/abi.md) v1.1.
- The approved class design names `HandleRegistry`, `HandleEntry`, `ValidationPin`, and a common `CoreObject` ownership target. It requires a synchronized token map, independent public references, lock-free object destruction/callback work, and a call-duration pin. It explicitly leaves concrete object ownership, Runtime identity representation, final UI cleanup, and dispatch mechanics open.
- RQ-05-010 allows retain/release from any thread and distinguishes acceptance from completion when UI resource destruction is deferred. RQ-06-001 reserves UI state operations for the Runtime UI thread; RQ-06-003 requires synchronization for shared mutable state. The registry must satisfy these together without destroying UI-affine state on an arbitrary releasing thread.

## Approved direction

Select an ownership boundary for registry records and pins, then define what happens when the last public reference is released off the UI thread, the Runtime is closing, or cleanup cannot be enqueued. The decision must state which work occurs under the registry lock, how the object remains alive across validation, and how failed release preserves caller ownership as required by RQ-05-012.

## Considered options and selected direction

| Option | Representation | Last release off UI thread | Trade-off |
| --- | --- | --- | --- |
| A. Thread-safe shared owner | Registry entries and validation pins clone an `Arc`-like owner to a `Send + Sync` object core; UI-affine state is behind Runtime/UI-thread dispatch. | Atomically detach the public entry, then queue the final UI cleanup after releasing the registry lock. | Straightforward cross-thread pinning; risks making UI-bound object internals appear freely shareable unless the core/UI-state split is explicit. |
| B. UI-thread-only object store plus thread-safe release ticket | Registry stores a synchronized ticket/control record; object state stays in a UI-thread-confined arena. A ticket tracks public refs and in-flight pins, and cleanup is transferred to the Runtime. | Release consumes its token and queues a cleanup ticket; the UI thread resolves the object identity and destroys the state after pins drain. | Strong thread-affinity boundary; requires generation-safe identity, queue lifetime rules, and a precise accepted-versus-completed result for RQ-05-010. |
| C. Per-entry deferred closure | Each entry owns a type-erased finalizer/owner; registry removes the entry under lock and invokes or transfers the finalizer outside the lock. | Finalizer dispatches to the Runtime UI thread where needed. | Minimal global machinery, but heterogeneous finalizers and queue-failure ownership are harder to reason about and test consistently. |

The user selected **Option A** with “A推奨で” and approved the complete DD v1.2 with “ok” (2026-10-17). The approved v1.2 applies the design to Runtime-bound objects and leaves Runtime-free immutable ErrorSnapshot ownership unchanged and outside this revision.

## Approved realization

Use **Option A with an explicit split between thread-safe identity/lifetime control and UI-thread-confined state**. Each Runtime-bound registry entry and validation pin shares an `Arc<CoreObjectControl>`; UI state remains in a Runtime-owned UI-thread-only `UiObjectStore` keyed by a generation-protected identity. A pin keeps control metadata alive but does not authorize off-thread UI-state access.

On the final public release, reserve a Runtime cleanup ticket before consuming the token. If the queue cannot reserve it, preserve the live token and fail the release. Once reserved, remove the token under registry synchronization, then activate the ticket after dropping locks. The Runtime retains the ticket until in-flight pins drain and the UI thread destroys the state. Runtime shutdown drains accepted tickets before Closed; host suspension retains accepted tickets for the existing resume/drain path. Successful release reports token consumption and cleanup acceptance, not UI destruction completion.

## Approval boundary

DD v1.2 and test-design v0.4 are approved. UNIT-CORE-004 and UNIT-CORE-005 are `ready` after their source paths and approved tests passed, per the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md).

## Steps

- [x] Record approval of HandleEntry test-design v0.3 and set `UNIT-CORE-001` through `UNIT-CORE-003` ready.
- [x] Identify the exact storage/lifetime gap in approved DD and its linked requirements.
- [x] Compare viable owner and release-handoff models and write a reviewable recommendation.
- [x] Record user selection of Option A and prepare the complete `objects-values.md` v1.2 review draft.
- [x] Obtain and record user approval of DD v1.2.
- [x] Add registry/pin/final-release cases to `core-runtime.md` v0.4 and register `UNIT-CORE-004` as planned.
- [x] Obtain and record user approval of the complete test-design v0.4.
- [x] Split the implementation into UNIT-CORE-004 (registry map/pins/final release) and UNIT-CORE-005 (Runtime shutdown drain) so the first change does not invent an unimplemented Runtime host.
- [x] Implement UNIT-CORE-004 registry table, pins, and release handoff against the approved design; add focused unit cases and validate (5/5 pass).
- [x] Mark UNIT-CORE-004 ready after all source paths exist and its approved tests pass.
- [x] Implement UNIT-CORE-005 Runtime cleanup queue and shutdown drain; add focused unit test and validate.
- [x] Mark UNIT-CORE-005 ready after its source and test exist and pass.

## Validation

`cargo fmt --all`, `cargo test -p freemvvm-core` (14/14), and `.venv\Scripts\python.exe tools\dev.py check` passed. The full check includes documentation links, Rust format/clippy/build/test-harness build, C/C++ probes, C# build, and Python wheel build/install/import. `git diff --check` passed. Runtime cleanup queue and shutdown drain are implemented in `crates/core-runtime/src/runtime.rs`; shutdown processes accepted finalizers and tracked UI-state cleanup before publishing Closed, and a remaining public token can be released afterward without touching UI state.
