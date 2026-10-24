# Runtime/Core/ABI: asynchronous Command implementation

## Scope

Rust Core and the versioned C ABI own Command state, operation identity, read-only state Properties, cancellation tokens, failure snapshots, and terminal report application. C# wrappers must use the exported C ABI and keep worker lifetime separate from Command-handle lifetime.

## Implemented contract

- `AsyncCommand` stores open/eligible state, current operation, atomic cooperative cancel token, terminal state, and immutable failure details. `OperationIdIssuer` is shared by the Runtime so IDs are unique across Commands and stop before wrapping.
- `command_execute` commits `RUNNING` before invoking the synchronous start callback. A start callback failure keeps the accepted operation ID and records a `FAILED` snapshot with `UI_COMMAND_EXECUTION_FAILED` status.
- `command_cancel` only requests cancellation and sets `CANCELLING`; success, failure, or cancellation is committed later by a UI-thread completion call. Old IDs and duplicate terminal reports do not overwrite the current state.
- Core-owned state I64 and can-execute BOOL Properties are publicly read-only. Command transitions update both Properties within the same host-event scope; changing the eligibility BOOL updates can-execute before that scope delivers Property notifications.
- `handle_release` on a Command owner does not close the Command. Explicit `command_close` or Runtime shutdown prevents new execution, requests cooperative cancellation, and destroys the start callback once without waiting for worker completion.
- Workers retain the borrowed Command/cancel-token handles they need. Terminal reports return to the UI thread through `runtime_post`; the C ABI does not run UI completion directly on a worker.

## Paths and verification

- Core state machine: `crates/core-runtime/src/command.rs`; Runtime and Property integration: `crates/core-runtime/src/runtime.rs`, `crates/core-runtime/src/property.rs`, `crates/core-runtime/src/lib.rs`.
- Versioned API declarations/exports: `crates/c-abi/include/freemvvm/abi.h` and `crates/c-abi/src/lib.rs`.
- Focused tests: `UT-CORE-COMMAND-001` and `UT-ABI-COMMAND-001`; linked C smoke: `UT-ABI-COMMAND-SMOKE-001`.
- On 2026-10-24, Core command tests (7), C ABI command tests (2), C11 command smoke, `tools/dev.py check`, Rust 1.85 minimum check, and all Rust workspace tests passed locally. Windows/Linux CI run [40072910815](https://github.com/edamame-edame/freemvvm/actions/runs/40072910815) passed on `impl-rust/runtime-async-command`.

## Follow-up boundary

C# Command facade and callback-root integration are a separate unit. The C# wrapper DD v1.4 was approved on the C# branch; C# source changes should wait until this Core/C ABI commit passes Windows/Linux CI and the wrapper's fake ABI/consumer test mapping is registered.
