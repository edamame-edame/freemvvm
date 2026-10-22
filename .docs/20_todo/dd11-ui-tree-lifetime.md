# DD-11 UI treeと要素寿命 作業計画

- 状態: 完了（DD-TREE-01～08内容承認済み、2026-10-11）
- 着手日: 2026-10-11
- 対象: Logical Treeの親子所属、所有参照、取り外し、Detached subtree、Property/Binding/Subscriptionとの寿命境界
- 上位出典: .docs/12_design/BD-04_ui-tree-lifetime-upper-design.md、BD-05_logical-visual-tree-upper-design.md、BD-11_initial-control-contract-upper-design.md、Runtime R1/R2、ABI A29～A32
- 先行詳細設計: DD-01 Handle登録、DD-02 所有権と終了、DD-07 Property、DD-08 Binding、DD-09 Validation、DD-10 Async Command
- 対応する詳細設計: .docs/13_detail_design/11-ui-tree-lifetime.md

## 目的

承認済みBD-04/05/11の範囲内で、UI tree構造変更と要素寿命の詳細レビュー案を作る。親子所有、tree変更の原子性、detach後のsubtree・handle・Property関係、最終破棄とshutdown時の責務を判断ID付きで示す。

## 作業手順

1. ADR-0001、Accepted要件、design overview、Runtime R1/R2、ABI A29～A32、BD-04/05/11の本文と承認記録を確認する。
2. DD-01/02/07/08/09/10からhandle、内部参照、Binding/Subscription、Validation/Commandの寿命契約を抽出する。
3. Logical TreeとVisual Treeを分け、Logical TreeだけがBD-04の親子所有参照を持つ前提で、所属・変更・detach・破棄の状態と失敗時動作をレビュー案にする。
4. 親子操作の制約、同Runtime検査、循環・複数親拒否、Window/StackPanelの既承認制約を照合する。
5. detach後のProperty/Binding/Subscription継続、Platform peer、focus/IME cleanupの責務境界を示し、未定義の上位判断や後続DDに返す事項を分離する。
6. .docs/13_detail_design/11-ui-tree-lifetime.md に未承認のDD-TREE判断、状態遷移図、正常系/失敗系の検証観点を記録する。
7. .docs/20_todo/detail-design-phases.md と .docs/01_memory/detail-design-planning-status.md を作業中に更新し、リンクと git diff --check を確認する。

## 承認境界と停止条件

利用者は2026-10-11にDD-TREE-01～08の内容を承認した。本計画はDD-11の承認反映までを扱い、ABI symbolや後続DDの承認・着手を含まない。第7節の未決事項は各後続設計で扱う。

## 対象外

- 新しい公開ABI symbolや配布用headerの確定
- Measure/Arrange、render invalidation、hit-test詳細
- detach時のfocus/IME遷移の詳細（DD-13/14）
- templateの生成・交換・Visual要素所有の完全設計
- DataContext、resource、name scope、event routing、accessibilityの継承
- wrapper固有APIと最終的なC++/C#/Rust/Pythonの型設計

## 完了条件

- DD-TREE判断案と上位出典・承認状況を追跡できる。
- 正常系、失敗時の非変更、detach/再所属、handle/内部参照、shutdownの観点を記載する。
- DD-TREE-01～08の承認と第7節の未決事項が区別され、承認結果が本書・フェーズ一覧・進捗記録へ反映されている。
