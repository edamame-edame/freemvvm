# Rust wrapper / Property Binding

- 状態: Rust Property Bindingの詳細設計v1.5・試験設計v0.8・実装計画は利用者承認済み。実装中。
- 現状: versioned C ABIには`binding_create`、`binding_cancel`、`binding_error_snapshot`、`binding_subscribe_error`がある。Rust wrapperにはBinding facadeがまだない。
- 既存根拠: C++ wrapperの`Binding<T>`/`BindingError<T>`はmove-only owner、明示cancelとhandle releaseの分離、error snapshot、`Subscription`再利用を採用している。Rust API案はRust `UiContext`と既存`Owned`/`Subscription`へ対応づける。
- 作業計画: [Rust Property Binding wrapper](../20_todo/rust-property-binding.md)。`tools/dev.py`は読み取り専用添付として提供されたため、このunitでは編集せず既存`rust-wrapper-runtime-test`を利用する案。
- 注意: C ABI/CoreのBinding graphやpropagationをRustで再実装しない。OneTime成功はownerなし、OneWay/TwoWayは明示cancelまでRuntimeが接続を保持する。計画承認前に`tools/units.yaml`を変更したり実装を開始しない。
