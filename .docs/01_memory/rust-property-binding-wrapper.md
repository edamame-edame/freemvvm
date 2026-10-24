# Rust wrapper / Property Binding

- 状態: Rust Property Bindingの詳細設計v1.5・試験設計v0.8・実装計画は利用者承認済み。実装とlocal gates完了。Windows/Linux CI結果待ち。
- 現状: versioned C ABIには`binding_create`、`binding_cancel`、`binding_error_snapshot`、`binding_subscribe_error`がある。Rust wrapperにはBinding facadeがまだない。
- 既存根拠: C++ wrapperの`Binding<T>`/`BindingError<T>`はmove-only owner、明示cancelとhandle releaseの分離、error snapshot、`Subscription`再利用を採用している。Rust API案はRust `UiContext`と既存`Owned`/`Subscription`へ対応づける。
- 作業計画: [Rust Property Binding wrapper](../20_todo/rust-property-binding.md)。Rust wrapperの`NativeApi`はRuntime APIだけを抽象化しProperty/Bindingは直接ABIを呼ぶ。したがってhost-loop付きlinked C ABI integration testを利用し、別crate compile testで公開APIを確認する。添付`dev.py`は読み取り専用として扱い既存`rust-wrapper-runtime-test`を利用する。
- 注意: C ABI/CoreのBinding graphやpropagationをRustで再実装しない。OneTime成功はownerなし、OneWay/TwoWayは明示cancelまでRuntimeが接続を保持する。計画承認前に`tools/units.yaml`を変更したり実装を開始しない。
