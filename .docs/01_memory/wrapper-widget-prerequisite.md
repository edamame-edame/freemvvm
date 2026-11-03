# Scope: Input wrapper branchに必要なWidget object基盤

- カテゴリ: 実装記憶 / language wrappers / UI Controls
- 対象: `.docs/20_todo/input-focus-event-path-implementation.md`、`UNIT-WRAP-*-INPUT-001`

## 学んだこと

- Input DDが「各具体Widget wrapperにsubscribe_inputを提供」としていても、対象言語の公開Widget型が存在することを別途確認する。C ABIのControl APIだけでは、利用者向けwrapperから購読対象を作れない。
- unit登録の予定pathは実ファイルの存在だけでなく、既存公開APIで前提オブジェクトを生成・接続できるかも確認する。前提が欠けている場合はAPIを推測実装せず、上流設計・試験・unit dependencyを追加する。
- `impl-rust/core-input-focus`のworktreeだけを見ると公開Widget型がないが、repoには完了済みControl wrapper branchが別に存在する。Rust `impl-rust/ui-controls-wrapper`、C++ `impl-cpp/ui-controls-wrapper`、C# `impl-csharp/ui-controls-wrapper`、Python `impl-python/runtime-wrapper`を監査し、APIを再設計せずInput wrapper childへ統合する。
- branchで分離された成果物がcheckoutに見えない場合、repo全体のbranchとunit台帳も調べる。欠落と判断する前に、完了済みwrapper branch・CI記録・merge baseを確認する。
