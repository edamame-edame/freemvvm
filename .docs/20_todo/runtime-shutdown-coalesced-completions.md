# Complete every coalesced shutdown request

- Updated: 2026-10-18
- Status: implemented and verified
- Scope: `UNIT-CORE-018` makes one priority shutdown and cleanup commit every accepted completion record that coalesced onto it.

## Approved basis

- Requirements: [RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006) and [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012).
- Detailed design: [Runtime and shutdown](../13_detail_design/runtime-objects/runtime.md) v1.1, “Host境界・終了要求・完了観測” and “CompletionRecord”.
- Test design: [Core Runtime tests](../32_unit_test/core-runtime.md) v0.4, UT-RUNTIME-005.

## Slice boundary

- Keep each accepted completion alive under HostRegistration until cleanup completes or the host stops.
- Coalesce Dispatcher shutdown, but complete every distinct accepted completion record after one Runtime cleanup drain.
- On abrupt host stop, mark all live accepted records HostUnavailable without claiming cleanup completed.
- Deduplicate repeated admission of the same completion record; leave public ShutdownRequest handles and ABI wiring for later units.

## Steps

- [x] Confirm approved RQ, DD v1.1, and UT-RUNTIME-005.
- [x] Register `UNIT-CORE-018` as planned before editing source.
- [x] Retain accepted completion records until their terminal transition.
- [x] Fan out one cleanup commit to each distinct accepted request.
- [x] Extend UT-RUNTIME-005 for distinct coalesced requests and dropped observers.
- [x] Run focused/core tests, full development checks, documentation links, and whitespace validation.
- [x] Mark the unit ready after source and registered tests passed.

## Validation

The exact test command is recorded in `tools/units.yaml`; readiness follows the [implementation inventory contract](../32_unit_test/implementation-test-inventory.md).

- Focused UT-RUNTIME-005: passed.
- `cargo test -p freemvvm-core`: 19 passed.
- `cargo clippy -p freemvvm-core --all-targets -- -D warnings`: passed.
- `.venv\Scripts\python.exe tools\dev.py check`: passed, including workspace Clippy/build, C/C++ probes, C# build, and Python checks.
- `.venv\Scripts\python.exe tools\check_docs.py`: 76 active documents and 5,853 local links passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
