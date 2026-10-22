# DD-12 Layout 作業計画

- 状態: 完了（DD-LAYOUT-01～08内容承認済み、2026-10-12）
- 着手日: 2026-10-11
- 対象: Core側Measure/Arrange、Sizer/Locationerの役割、StackPanel最小配置、Layout invalidation
- 上位出典: .docs/12_design/BD-04_ui-tree-lifetime-upper-design.md、BD-05_logical-visual-tree-upper-design.md、BD-06_layout-upper-design.md、BD-11_initial-control-contract-upper-design.md
- 先行詳細設計: DD-02、DD-03、DD-07、DD-08、DD-09、DD-11
- 対応する詳細設計: .docs/13_detail_design/12-layout.md

## 目的

承認済みBD-06のCore/Platform責務境界を守り、初期縦断シナリオのVisual Treeに対するMeasure/ArrangeとStackPanelの最小動作をレビュー案にする。Sizer/Locationer、制約、desired size、最終bounds、配置設定、変更時の再計算を分けて記述する。

## 作業手順

1. ADR-0001、Accepted要件/design overview、BD-04/05/06/11の本文と承認記録を確認する。BD-06で未決の単位系、scale、rounding、独自Sizer/Locationer拡張を別管理する。
2. DD-11のLogical/Visual Tree所有境界、DD-02のUI thread/lifetime、DD-03のdispatch順、DD-07/08のProperty/Binding通知契約を確認する。
3. Measure/Arrangeの入力・出力、desired sizeとfinal bounds、親子の処理順、制約の不正値と失敗時原子性を提案する。
4. StackPanelの順序、orientation、main/cross axis測定と配置、Align等の既定値・優先順位を初期動作として提案する。
5. Tree、root available size、Layout関連Propertyの変更から必要な再計算範囲を提案する。描画backend固有invalidationとの境界を分ける。
6. .docs/13_detail_design/12-layout.md に未承認のDD-LAYOUT判断、計算例、未決事項、検証ケースを記録する。
7. .docs/20_todo/detail-design-phases.md と .docs/01_memory/detail-design-planning-status.md を更新し、参照先と git diff --check を確認する。

## 承認境界と停止条件

利用者は2026-10-12にDD-LAYOUT-01～08を内容承認した。本計画はDD-12の承認反映までを扱う。ABI表現、単位系、Renderer/Platform契約など第7節の未決事項とDD-13以降の着手は本承認の対象外とする。

## 対象外

- boundsの数値単位、DPI/scale変換、pixel rounding、座標変換行列（BD-06で未決）
- Renderer API、scene/command、dirty region、Platform固有描画
- Input hit-test・Focus・IMEの詳細（DD-13/14/16）
- Text shapingに基づくText/TextBoxの自然サイズとControl別Property一覧（DD-15/適切なtext設計）
- virtualization、scrolling、gridなど後続ControlのLayout
- 利用者が独自Sizer/Locationerを登録するABI callback拡張

## 完了条件

- Measure/ArrangeとStackPanelの承認判断IDが根拠つきで記載されている。
- 正常系、境界寸法、制約不正、tree変更後の再計算、失敗時の状態不変を検証できる。
- DD-LAYOUT-01～08の承認結果を本書・フェーズ一覧・進捗記録へ反映し、BD-06の未決範囲と後続担当DDを区別する。
