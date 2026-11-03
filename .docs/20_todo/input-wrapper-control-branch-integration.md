# Input wrapperへ既存UI Control wrapperを統合する計画案

- 計画版: 0.1
- 状態: 利用者承認済み（2026-11-03）、Rust Control branch統合およびRust Input facade実装・local検証済み。残りの各言語統合は後続
- 対象version: `0.0.1`
- 関連計画: [Input / Focus event path実装計画](input-focus-event-path-implementation.md)
- 旧案: [UI Control wrapper基盤の追加計画案](ui-control-wrappers-prerequisite.md)（branch監査で撤回）

## 目的

承認済みInput wrapper APIは各言語の具体Widget型への登録操作として設計されている。既に完了しているUI Control wrapper branchをCore Input branch上の各Input wrapper branchへ統合し、Control APIを重複実装せずにInput event facadeへ進む。

## branch監査結果

| 言語 | 既存Control wrapper branch | unit状態 | 最終記録commit |
| --- | --- | --- | --- |
| Rust | `impl-rust/ui-controls-wrapper` | `UNIT-WRAP-RS-UI-CTRL-001` complete | `c6ed2d4` |
| C++ | `impl-cpp/ui-controls-wrapper` | `UNIT-WRAP-CPP-009` complete | `393bee8` |
| C# | `impl-csharp/ui-controls-wrapper` | `UNIT-WRAP-CS-CONTROLS-001` complete | `bc47e97` |
| Python | `impl-python/runtime-wrapper` | `UNIT-WRAP-PY-CONTROLS-001` complete | `34cc265` |

これらbranchは各言語の公開`Window`・`StackPanel`・`Text`・`TextBox`・`Button`等を実装し、対応するDD・test design・consumer試験を登録済み。各branchと最新Core Input branchとの共通merge baseは`dfb86cf`。現在の`impl-rust-wrapper/input-focus`は`4fdbea1`から作成され、実装変更はない。

## 作業順

| 順 | 作業 | 完了条件 |
| --- | --- | --- |
| 1 | 各Control wrapper branchの設計承認、unit complete状態、最終commit、実際の公開型・source/test pathを確認 | branch単位の証跡と最新Core Input branchとの差分を記録。別機能のbranchを暗黙に混ぜない |
| 2 | [x] Rust Input wrapper branchへ`impl-rust/ui-controls-wrapper`を統合 | Input DDとControl DD、source・consumer・unit登録を同一treeへ統合。commitは検証完了後に行う |
| 3 | [x] 統合後のRust Control wrapper試験を再実行 | `cargo test --locked -p freemvvm`と両consumer exampleがpass（2026-11-03）。この環境にPython 3.13 / `.venv`がないため`tools/dev.py`経由ではなく同じCargo操作を直接実行 |
| 4 | [x] `UNIT-WRAP-RS-INPUT-001`の依存を更新 | `UNIT-ABI-022`、`UNIT-WRAP-RS-UI-CTRL-001`、`UNIT-WRAP-RS-CMD-001`を依存に登録 |
| 5 | [x] Rust Input facadeとconsumerを実装 | 承認済み`wrappers/rust.md`・試験設計の5 event種、callback-scoped borrow、cancel/drop、panic境界を実装。unit tests、Clippy、Rust 1.85 workspace check、format、文書リンク検査がpass。commit/push後にCIを確認 |
| 6 | C++・C#・Pythonも同じ順で各言語wrapper branchを統合 | 各言語Input branchへ対応する完了済みControl wrapper branchを統合し、既存Control試験を再検証してからInput APIを実装 |

## 対象外

本計画ではControl公開APIを再設計・再実装しない。`window-background-config`などInput購読の前提でない別機能branchも含めない。統合時に現在Core APIとの実装不整合が見つかった場合は、差分と対象unitを記録してから修正範囲をレビューする。

## branch形

```text
version/0.0.1
└─ impl-rust/core-input-focus
   ├─ impl-rust-wrapper/input-focus
   │  └─ merge impl-rust/ui-controls-wrapper → Rust Input facade
   ├─ <C++ Input wrapper branch>
   │  └─ merge impl-cpp/ui-controls-wrapper → C++ Input facade
   ├─ <C# Input wrapper branch>
   │  └─ merge impl-csharp/ui-controls-wrapper → C# Input facade
   └─ <Python Input wrapper branch>
      └─ merge impl-python/runtime-wrapper → Python Input facade
```

各Input wrapper branchはCore Input APIsを親として切り、その後、同言語の完了済みControl wrapper branchだけを統合する。wrapper言語間のbranchは親子にしない。

## 承認境界

Control branch統合、既存Control試験の再実行、Input unitの依存更新をこの計画の承認範囲とする。APIの追加・変更が判明した場合は設計文書を別途改訂・レビューする。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-11-03 | 計画0.1 | 利用者承認 | 「ok」により既存の完了済みControl wrapper branch統合、再検証、Input unit依存更新を承認 |
