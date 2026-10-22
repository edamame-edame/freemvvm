# C++ Runtime wrapper API and cleanup boundary

- Updated: 2026-10-22
- Status: review proposal; no implementation started
- Scope: settle the first public C++ Runtime ownership surface and the lifetime boundary required for failed native releases before implementing the wrapper or CMake package.

## Why this decision is next

The C++17 raw C ABI consumer smoke is complete in `f3c3f78`. Requirement [RQ-12-004](../11_requirements/RQ-12_build-distribution.md#rq-12-004) requires a CMake install/export package containing a public C++ wrapper and the common native library. The approved [C++ wrapper design](../13_detail_design/wrappers/cpp.md) establishes move-only ownership, explicit shutdown, non-throwing result paths, and preservation of release failures, but leaves public type/result names and cleanup registry lifetime/library-unload rules open. Packaging a stable wrapper before those decisions would freeze an accidental API.

## Approved basis

- Requirements: [RQ-03-001](../11_requirements/RQ-03_language-support.md#rq-03-001), [RQ-03-005](../11_requirements/RQ-03_language-support.md#rq-03-005), [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006), [RQ-04-016](../11_requirements/RQ-04_abi-boundary.md#rq-04-016), [RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012), [RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002), and [RQ-12-004](../11_requirements/RQ-12_build-distribution.md#rq-12-004).
- Architecture: [ADR-0001](../04_approval/adr-0001-rust-core.md), Accepted Rust Core with versioned C ABI.
- Detailed design: [C++ wrapper](../13_detail_design/wrappers/cpp.md) v1.1, [Common C ABI](../13_detail_design/runtime-objects/abi.md) v1.2, and [Build/package](../13_detail_design/build/toolchain-packages.md) v1.1.
- Existing oracle: [UT-WRAP-CPP-001](../32_unit_test/wrappers.md#ut-wrap-cpp-001) and [TC-WRAP-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01).

## Review proposal

Keep the first public API slice limited to Runtime creation, UI-thread query, explicit shutdown/request-shutdown, and explicit token close. Use C++17-compatible non-throwing result types as the base API; any throwing convenience layer can follow separately. Runtime destruction must not implicitly initiate shutdown. Public names and exact result layout are intentionally not fixed by this proposal.

For ownership, preserve the approved rule that an owned token is not discarded when `handle_release` fails. A cleanup node must be reserved before an ABI call can create an owned token, and a failed destructor release must transfer the token and reserved node without allocation. A runtime-scoped cleanup owner is preferred over a process-global registry so unrelated Runtime instances do not share cleanup state. Its concrete lifetime must keep the C ABI library loaded until all pending releases have been drained or explicitly reported as unresolved.

### Recommended direction for approval

- Expose a move-only `freemvvm::Runtime` first. Keep the public operation set to `create`, `is_ui_thread`, `shutdown`, `request_shutdown`, and explicit `close`; destruction releases only and never starts shutdown.
- Use a small C++17 `Result<T>`/`Result<void>` base path that preserves raw `ui_status` and, when returned, an owned Error snapshot. Defer throwing conveniences, Property wrappers, and callback adapters.
- Give every Runtime a shared internal control block that owns a preallocated intrusive pending-release list and pins the loaded native module while any owned token or pending release remains. Runtime-bound wrappers share this control block. Destructors transfer failed releases to the list without allocating; explicit close reports the immediate failure. Safe-point/shutdown drains retry the list, and unresolved nodes keep the control block and native module pinned rather than losing a callable release path.

This is a proposal, not a decision already present in the approved design. Approval should confirm this public slice and cleanup-owner model, or replace either with a chosen alternative.

No new behavior beyond approved C ABI and wrapper requirements is proposed. The library-unload rule is an implementation boundary that must be specified before a destructor can safely hand off a failed release.

## Work after review

- [ ] Record the selected public names, result convention, explicit close/shutdown distinction, and cleanup-owner lifetime in the approved C++ wrapper design.
- [ ] Register the first wrapper implementation and focused tests in `tools/units.yaml` and `.docs/32_unit_test/` before changing source.
- [ ] Implement and verify the selected minimal Runtime wrapper.
- [ ] Add the wrapper and native runtime to a relocatable CMake install/export package and exercise it from a separate downstream consumer.
- [ ] Record Windows and Linux CI evidence separately from local host verification.

## Revision history

| Date | Change | Approval |
| --- | --- | --- |
| 2026-10-22 | Proposed the first C++ Runtime surface and cleanup-owner model after the raw ABI smoke | Review proposal; no API or behavior approved by this document |
