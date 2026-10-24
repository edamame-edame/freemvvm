# Scope: C# Property Binding design review

- The approved implementation plan is `.docs/20_todo/csharp-property-binding.md`; plan commit: `fde7a98`.
- `.docs/13_detail_design/wrappers/csharp.md` v1.3 Binding addition received full-document user approval on 2026-10-24. The approved implementation has started; keep its test registration and implementation status separate from the design approval.
- Binding semantics are already specified by approved RQ-07 and the common Binding C ABI DD. The C# proposal reuses `UiResult<T>`, `UiErrorSnapshot`/`PreparedError`, `TokenOwner`, `UiSubscription`, static callback trampolines, and the Runtime callback failure handler.
- OneTime C ABI success has no Binding token; the proposal represents it as successful `UiResult<UiBinding<T>?>` with a null value. OneWay/TwoWay return an owned Binding wrapper.
- A Binding error snapshot copies the immutable native error details into managed `UiErrorSnapshot` and separately owns the returned rejected-Property token. Error callback registration has no initial callback and reuses the existing Subscription lifecycle.
- `csharp-property-test` passed with fake ABI coverage for all five managed value types and three Binding modes, failure/error ownership, cancellation/close retry, immutable error capture, and callback kind/lifetime.
- `csharp-property-consumer-test` passed through the host C ABI for all five types, OneTime/OneWay/TwoWay propagation, close-without-cancel, and cancel behavior. The consumer registers a UI loop bridge and dispatches turns for queued delivery.
- `tools/dev.py check`, `tools/dev.py minimum` (Rust 1.85.0), `tools/check_docs.py`, and `git diff --check` passed locally. Windows/Linux CI run [40060773110](https://github.com/edamame-edame/freemvvm/actions/runs/40060773110) passed; static docs validation is not runtime test evidence.
