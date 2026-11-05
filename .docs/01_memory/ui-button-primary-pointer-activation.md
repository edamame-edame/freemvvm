# freemvvm: Core UI Button primary pointer activation

- カテゴリ: 実装知見 / Core UI / Input・Button
- 対象: `UNIT-CORE-091` / `UT-INPUT-005`

## 学んだこと

- Buttonの押下状態はRuntimeごとに一件だけ保持し、`ObjectIdentity`とWindow generationだけを記録する。downで前の状態を置き換え、未handledのupで必ず消費するため、target不一致・handled up・detach・disable・Window close/reopenで誤発火しない。
- dispatch後にも元routeが有効かを確認してから既定動作を適用する。ButtonとWindowまでの有効状態を検証し、pointer captureは行わない。
- Button Commandは既存の`command_execute` lifecycleを使う。同期callback用の一時handleを作って実行後に解放し、CommandのCanExecute・実行中拒否・operation stateを既存経路に委ねる。
- 対応試験は`.docs/32_unit_test/ui.md`のUT-INPUT-005。今回のCore suiteは202件passし、workspace Clippy、format、Rust 1.85 check、文書リンク検査もpassした。

参照: [Input/Focus実装計画](../20_todo/input-focus-event-path-implementation.md)、[UI試験設計](../32_unit_test/ui.md)、[詳細設計](../13_detail_design/ui/input-focus.md)。
