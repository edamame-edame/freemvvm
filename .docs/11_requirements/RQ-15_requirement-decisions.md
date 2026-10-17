# RQ-15: 要求上の承認事項と残確認事項

- 文書版: 0.4
- 状態: 移管済み・旧ID廃止（2026-10-17、利用者承認）
- 更新日: 2026-10-17
- 入口: [要求仕様索引](reqirements-specification.md)

## 旧IDの移管

独立した要求の正本を各機能章へ統合した。旧IDは履歴参照用で再利用しない。次のリンク先が現行本文である。過去の承認履歴は当時の対象を示す。

| 旧ID（廃止） | 現行要求の正本 |
| --- | --- |
| <a id="rq-15-001"></a>RQ-15-001 | [RQ-12-010](RQ-12_build-distribution.md#rq-12-010)、[RQ-11-003](RQ-11_platform.md#rq-11-003)、[RQ-11-004](RQ-11_platform.md#rq-11-004) |
| <a id="rq-15-002"></a>RQ-15-002 | [RQ-01-002](RQ-01_purpose-architecture.md#rq-01-002)、[RQ-11-008](RQ-11_platform.md#rq-11-008) |
| <a id="rq-15-003"></a>RQ-15-003 | [RQ-11-008](RQ-11_platform.md#rq-11-008)、[RQ-11-009](RQ-11_platform.md#rq-11-009) |
| <a id="rq-15-004"></a>RQ-15-004 | [RQ-09-002](RQ-09_ui-scope.md#rq-09-002)、[RQ-09-003](RQ-09_ui-scope.md#rq-09-003) |
| <a id="rq-15-005"></a>RQ-15-005 | [RQ-09-005](RQ-09_ui-scope.md#rq-09-005)、[RQ-09-053](RQ-09_ui-scope.md#rq-09-053)、[RQ-09-054](RQ-09_ui-scope.md#rq-09-054) |
| <a id="rq-15-006"></a>RQ-15-006 | [RQ-09-005](RQ-09_ui-scope.md#rq-09-005)、[RQ-09-038](RQ-09_ui-scope.md#rq-09-038)、[RQ-09-040](RQ-09_ui-scope.md#rq-09-040)、[RQ-09-042](RQ-09_ui-scope.md#rq-09-042)、[RQ-09-055](RQ-09_ui-scope.md#rq-09-055)、[RQ-09-056](RQ-09_ui-scope.md#rq-09-056) |
| <a id="rq-15-007"></a>RQ-15-007 | [RQ-07-008](RQ-07_binding.md#rq-07-008)、[RQ-07-013](RQ-07_binding.md#rq-07-013)、[RQ-07-014](RQ-07_binding.md#rq-07-014)、[RQ-03-011](RQ-03_language-support.md#rq-03-011) |
| <a id="rq-15-008"></a>RQ-15-008 | [RQ-07-010](RQ-07_binding.md#rq-07-010)、[RQ-09-005](RQ-09_ui-scope.md#rq-09-005)、[RQ-02-021](RQ-02_core-features.md#rq-02-021)、[RQ-05-013](RQ-05_ownership-callbacks.md#rq-05-013)、[RQ-07-016](RQ-07_binding.md#rq-07-016) |
| <a id="rq-15-009"></a>RQ-15-009 | [RQ-06-004](RQ-06_threading-dispatch.md#rq-06-004)、[RQ-06-008](RQ-06_threading-dispatch.md#rq-06-008)、[RQ-06-009](RQ-06_threading-dispatch.md#rq-06-009)、[RQ-06-010](RQ-06_threading-dispatch.md#rq-06-010)、[RQ-06-011](RQ-06_threading-dispatch.md#rq-06-011) |
| <a id="rq-15-010"></a>RQ-15-010 | [RQ-07-005](RQ-07_binding.md#rq-07-005)、[RQ-07-006](RQ-07_binding.md#rq-07-006)、[RQ-07-009](RQ-07_binding.md#rq-07-009) |
| <a id="rq-15-011"></a>RQ-15-011 | [RQ-04-006](RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](RQ-04_abi-boundary.md#rq-04-007)、[RQ-04-022](RQ-04_abi-boundary.md#rq-04-022)、[RQ-04-023](RQ-04_abi-boundary.md#rq-04-023)、[RQ-04-027](RQ-04_abi-boundary.md#rq-04-027) |
| <a id="rq-15-012"></a>RQ-15-012 | [RQ-04-009](RQ-04_abi-boundary.md#rq-04-009)、[RQ-04-024](RQ-04_abi-boundary.md#rq-04-024)、[RQ-04-028](RQ-04_abi-boundary.md#rq-04-028) |
| <a id="rq-15-013"></a>RQ-15-013 | [RQ-12-007](RQ-12_build-distribution.md#rq-12-007)、[RQ-03-011](RQ-03_language-support.md#rq-03-011)、[RQ-03-012](RQ-03_language-support.md#rq-03-012) |
| <a id="rq-15-014"></a>RQ-15-014 | [RQ-12-004](RQ-12_build-distribution.md#rq-12-004)、[RQ-12-005](RQ-12_build-distribution.md#rq-12-005)、[RQ-12-006](RQ-12_build-distribution.md#rq-12-006)、[RQ-12-008](RQ-12_build-distribution.md#rq-12-008)、[RQ-12-016](RQ-12_build-distribution.md#rq-12-016)、[RQ-12-021](RQ-12_build-distribution.md#rq-12-021) |
| <a id="rq-15-015"></a>RQ-15-015 | [RQ-11-001](RQ-11_platform.md#rq-11-001)、[RQ-11-002](RQ-11_platform.md#rq-11-002)、[RQ-11-020](RQ-11_platform.md#rq-11-020)、[RQ-12-017](RQ-12_build-distribution.md#rq-12-017)、[RQ-12-018](RQ-12_build-distribution.md#rq-12-018)、[RQ-12-019](RQ-12_build-distribution.md#rq-12-019)、[RQ-12-020](RQ-12_build-distribution.md#rq-12-020) |
| <a id="rq-15-016"></a>RQ-15-016 | [RQ-12-004](RQ-12_build-distribution.md#rq-12-004)、[RQ-12-005](RQ-12_build-distribution.md#rq-12-005)、[RQ-12-006](RQ-12_build-distribution.md#rq-12-006)、[RQ-12-008](RQ-12_build-distribution.md#rq-12-008)、[RQ-12-011](RQ-12_build-distribution.md#rq-12-011)、[RQ-12-012](RQ-12_build-distribution.md#rq-12-012)、[RQ-12-013](RQ-12_build-distribution.md#rq-12-013)、[RQ-12-014](RQ-12_build-distribution.md#rq-12-014)、[RQ-12-015](RQ-12_build-distribution.md#rq-12-015) |
| <a id="rq-15-017"></a>RQ-15-017 | [RQ-12-010](RQ-12_build-distribution.md#rq-12-010) |
| <a id="rq-15-018"></a>RQ-15-018 | [RQ-10-001](RQ-10_accessibility.md#rq-10-001)、[RQ-10-002](RQ-10_accessibility.md#rq-10-002)、[RQ-10-003](RQ-10_accessibility.md#rq-10-003)、[RQ-10-004](RQ-10_accessibility.md#rq-10-004)、[RQ-10-005](RQ-10_accessibility.md#rq-10-005)、[RQ-10-006](RQ-10_accessibility.md#rq-10-006)、[RQ-10-007](RQ-10_accessibility.md#rq-10-007)、[RQ-10-008](RQ-10_accessibility.md#rq-10-008)、[RQ-10-009](RQ-10_accessibility.md#rq-10-009)、[RQ-10-011](RQ-10_accessibility.md#rq-10-011)、[RQ-10-012](RQ-10_accessibility.md#rq-10-012)、[RQ-10-013](RQ-10_accessibility.md#rq-10-013) |
| <a id="rq-15-019"></a>RQ-15-019 | [RQ-14-002](RQ-14_delivery-sequence.md#rq-14-002) |
| <a id="rq-15-020"></a>RQ-15-020 | [RQ-14-003](RQ-14_delivery-sequence.md#rq-14-003)、[RQ-14-005](RQ-14_delivery-sequence.md#rq-14-005) |
| <a id="rq-15-021"></a>RQ-15-021 | [RQ-14-010](RQ-14_delivery-sequence.md#rq-14-010)、[RQ-14-012](RQ-14_delivery-sequence.md#rq-14-012) |
| <a id="rq-15-022"></a>RQ-15-022 | [RQ-14-007](RQ-14_delivery-sequence.md#rq-14-007)、[RQ-14-008](RQ-14_delivery-sequence.md#rq-14-008) |

## 承認履歴

| 日付 | 対象版・ID / 範囲 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-09-26 | 旧要求仕様の該当範囲 | 利用者 | Acceptedを継承。後日追記の承認日・未決範囲は旧版で確認 |
| 2026-10-12 | 要求承認管理・章分割・RQ ID化 | 利用者 | 今回の指示による管理方針。旧本文に記載済みの承認事項を要求側で管理する |
| 2026-10-12 | [RQ-15-001](#rq-15-001) 対象OSの優先順位 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-002](#rq-15-002) 共通Rendererによる初期Control | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-003](#rq-15-003) Renderer責務の分離 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-004](#rq-15-004) 初期Controlの抽象 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-005](#rq-15-005) 初版textの対応範囲 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-006](#rq-15-006) IME compositionの外部動作 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-007](#rq-15-007) Binding循環の保護と静的警告 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-008](#rq-15-008) 初版validationの範囲 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-009](#rq-15-009) 非同期Commandの進捗と終端 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-010](#rq-15-010) 初版UI transactionの範囲 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-011](#rq-15-011) Error objectの安定性 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-012](#rq-15-012) ABI release versionと互換性 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-013](#rq-15-013) Python方式と解析支援 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-014](#rq-15-014) ライセンスと再配布 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-015](#rq-15-015) サポート期間 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-016](#rq-15-016) 配布・provenance・trust | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-017](#rq-15-017) CIと実機検証の分離 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| 2026-10-12 | [RQ-15-018](#rq-15-018) Accessibilityの共通意味情報 | 利用者 | 旧要求本文の承認済み結論を要求事項として継承・集約。残確認事項の所管は当該本文で管理 |
| — | RQ-15-019～022 優先確認項目 | — | 旧要求の管理項目を継承。個別の要求範囲・採否は本文と対応する要求承認記録で管理 |
| 2026-10-12 | RQ-15-001～022 / 再編集版0.3 | 利用者 | 要求条件、受入確認、未決範囲の整理を承認。RQ-15-001～018の個別要求承認履歴も維持 |

## 改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 旧版 | 2026-09-25 | 旧要求仕様を作成。2026-09-26にAccepted | 旧文書参照 | 旧要求仕様の承認記録 |
| 0.1 | 2026-10-12 | 章分割、RQ ID、受入確認案・設計/testリンク、要求承認履歴の集約 | Codex | 管理方針の指示に基づく再編集。版全体のレビューは未完了 |
| 0.2 | 2026-10-12 | 下流参照リンクを除去し、参照方向を明記 | Codex | 要求本文・承認状態は変更なし |
| 0.3 | 2026-10-12 | 既承認方針を要求条件・観測可能な受入確認・残確認境界に整理 | Codex | 要求の採否は変更せず再編集。利用者が同日承認 |

## 今回の承認履歴

| 日付 | 対象版・範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.4 / 全レビュー案の要求補足・意味単位移管 | 利用者 | 全レビュー案の反映指示により承認。実装・試験実行の完了とは区別する |

## 今回の改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 |
| --- | --- | --- | --- |
| 0.4 | 2026-10-17 | 要求と実現手段・検証・作業記録を分離し、正式IDと参照を対応付けた | Codex |
