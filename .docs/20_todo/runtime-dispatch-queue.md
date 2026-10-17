# Runtime Dispatcher FIFO queue and wake coordination

- Updated: 2026-10-17
- Status: implementation and local validation complete; UNIT-CORE-006 ready
- Scope: `UNIT-CORE-006` implements the Runtime post queue, UI-thread-only single-item dispatch turns, asynchronous wake coalescing, and deterministic lost-wake regression coverage.

## Approved basis

- Requirements: [RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009), [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001), and [RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, especially “UI dispatcherとhost loop”, “Queueとwake-upの状態”, and “失敗と所有権”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-001 and UT-RUNTIME-002.
- The detailed design leaves host bridge registration and any new C ABI connection surface to the ABI design. This unit uses an internal asynchronous wake trait and does not choose that public connection.

## Slice boundary

- Implement FIFO queue insertion linearized under a mutex and dispatch at most one posted callback per UI-thread turn.
- A callback posted from a callback is appended for a later turn and never invoked recursively.
- Coalesce asynchronous wake requests while one is pending. Coordinate post, turn start, and turn-end recheck so accepted work cannot be stranded and wake is not duplicated.
- Use a controllable fake bridge and barriers/channels for deterministic thread ordering; do not use elapsed-time sleeps as a synchronization oracle.
- Exclude shutdown-priority control work, cancel/destroy ownership after shutdown, notifications/Binding, host registration, and public C ABI integration. These require later implementation slices and must keep their existing approved design boundaries.

## Steps

- [x] Confirm current Runtime DD v1.1 and core-runtime test-design v0.4 approval and select UT-RUNTIME-001/002.
- [x] Add `UNIT-CORE-006` as planned to `tools/units.yaml` before source implementation.
- [x] Implement the internal Dispatcher/FakeBridge seam and focused tests for FIFO/non-reentrant execution, wake coalescing, and the drain-end enqueue race.
- [x] Run focused tests, all `freemvvm-core` tests, Rust formatting/lint, full development checks, document-link checks, and whitespace validation.
- [x] Mark `UNIT-CORE-006` ready after its source exists and all registered tests pass.

## Validation

`cargo test -p freemvvm-core dispatcher::tests` passed both registered cases, and `cargo test -p freemvvm-core` passed all 16 tests. `cargo clippy -p freemvvm-core --all-targets --locked -- -D warnings`, `cargo fmt --all`, `.venv\Scripts\python.exe tools\dev.py check`, and `git diff --check` passed. The full check includes document links, Rust format/clippy/build/test-harness build, C/C++ probes, C# build, and Python wheel build/install/import. The public host-bridge connection remains outside this internal Dispatcher slice.
