# MVVM UIライブラリ 要求仕様（移行入口）

- 状態: 旧パス互換入口。現行要求の正本は章別RQ文書
- 移行日: 2026-10-12

現行要求は [要求仕様の総目次](reqirements-specification.md) と各RQ章で管理する。旧版Acceptedの本文・追記承認は [移行前版](archive/mvvm-ui-library-requirements-20261012.md) に保存した。旧章番号で参照する際は以下から該当RQ章へ進む。

## 1. 文書の位置づけ

[要求・設計文書の管理と承認](RQ-16_document-governance.md)

## 2. 情報源の整理

[要求・設計文書の管理と承認](RQ-16_document-governance.md)

## 3. 目的と基本アーキテクチャ

[目的と共通アーキテクチャ](RQ-01_purpose-architecture.md)

## 4. 必須要求

[Core機能](RQ-02_core-features.md)、[対応言語と共通意味論](RQ-03_language-support.md)、[公開ABI境界](RQ-04_abi-boundary.md)、[所有権とcallback](RQ-05_ownership-callbacks.md)、[Threadと非同期処理](RQ-06_threading-dispatch.md)、[Binding](RQ-07_binding.md)

## 5. 最初の縦断シナリオに含まれていた要求

独立した要求章は設けず、既存要求に統合した。Bindingは[RQ-07](RQ-07_binding.md)、初期Controlとfocus・keyboardは[RQ-09](RQ-09_ui-scope.md)、Accessibilityは[RQ-10](RQ-10_accessibility.md)、複数言語での共通利用は[RQ-01](RQ-01_purpose-architecture.md)と[RQ-03](RQ-03_language-support.md)を参照する。

## 6. UI機能要求

[UI機能と段階的な対象範囲](RQ-09_ui-scope.md)

## 7. Accessibility要求

[Accessibility](RQ-10_accessibility.md)

## 8. Platform要求

[Platformと組み込みLinux](RQ-11_platform.md)

## 9. ビルドと配布

[ビルドと配布](RQ-12_build-distribution.md)

## 10. 非機能要求

[非機能要求と実測](RQ-13_nonfunctional-measurement.md)

## 11. 開発順序

[開発・検証の実施順序](RQ-14_delivery-sequence.md)

## 12. 未決事項と後続の判断状況

[要求上の承認事項と残確認事項](RQ-15_requirement-decisions.md)

## 13. 文書管理方針

[要求・設計文書の管理と承認](RQ-16_document-governance.md)

[暫定REQ IDから正式RQ IDへの対応](requirements-id-migration.md)

## 承認履歴

| 日付 | 対象版・ID / 範囲 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-09-26 | 旧要求仕様の該当範囲 | 利用者 | Acceptedを継承。後日追記の承認日・未決範囲は旧版で確認 |
| 2026-10-12 | 要求承認管理・章分割・RQ ID化 | 利用者 | 今回の指示による管理方針。旧本文に記載済みの承認事項を要求側で管理する |
| 2026-10-12 | 参照方向・secure-design追加 | 利用者 | 今回指示による構造更新。要求本文・セキュリティ判断の承認ではない |
| — | 旧パス互換入口 / 再編集版0.2 | — | 継承要求の再編集版はレビュー待ち |

## 改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 旧版 | 2026-09-25 | 旧要求仕様を作成。2026-09-26にAccepted | 旧文書参照 | 旧要求仕様の承認記録 |
| 0.1 | 2026-10-12 | 章分割、RQ ID、受入確認案・設計/testリンク、要求承認履歴の集約 | Codex | 管理方針の指示に基づく再編集。版全体のレビューは未完了 |
| 0.2 | 2026-10-12 | 参照方向を下流から上流へ是正し、secure-designを並列成果物として追加 | Codex | 入口の更新。要求本文・セキュリティ判断は未承認 |
