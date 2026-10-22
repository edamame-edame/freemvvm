# 未設計レビュー / 設計判断と実環境の採取値

- 文書版: 0.2
- 状態: 方式採用・正本適用済み（2026-10-17）。元課題は走査時点の記録、実採取値は未取得

今回の案で、採取/固定/検証する手順と責務を定めた。以下は実在する運用環境や成果物から得る値であり、架空の値で埋めた設定を有効としない。採取未了と方式未設計を区別する。

| 実入力 | 今回定めた方式 | 採取・確定条件 |
| --- | --- | --- |
| compiler/SDK patch、sysroot/image digest | profile lock生成・必須field gate | 実際のSDK/imageでversion commandとfile inventoryを採取。空fieldではrelease不可 |
| font/graphics/crypto library exact版・digest | 適合library選定と同一profile lockへ固定 | license/ABI/RFC vectorsを検証し依存lockとaudit evidenceを生成 |
| 代表board image/kernel/device能力 | RPi4 AArch64候補とARMv7別build gate | 実機接続後に現物profileを採取。未接続実機のpassを作らない |
| 公式catalog origin、registry namespace所有 | 静的署名catalog、設定origin、名称候補 | 運用者が管理可能なorigin/namespaceをprovisionして署名trustと結ぶ |
| root/release鍵と鍵ID | role分離・2/3 root・KMS/HSM release | 運用者が生成。秘密鍵はchat/文書/lockへ出さない |
| 担当者・組織の実名 | ReleaseMaintainer/SecurityMaintainerのroleとrecord更新手順 | 運用主体がrole assignmentを記録する |
| action full SHA/CI runner image digest | full SHA pin・same checkout revision evidence | workflow実装時に選定actionのrepo/commitを検証しlockする |
| test path/symbol/command/report | planned/ready台帳と実行schema | 実装単位着手時に実在pathへ結び、実行前は未実装/未実行を維持 |

## 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 実環境入力の境界 | 利用者 | 一括生成を指示。採用・正本反映・実装は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | 未設計事項の一括レビュー案を作成 | Codex |

## 現在の適用状態

[全件適用表](../remaining-design-application-result.md)に200判断の正式IDと適用先を記録した。元の課題引用は適用前の記録として保持する。

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.2 / 全件適用状態への接続 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.2 | 2026-10-17 | 全件適用状態への接続を正本へ適用し、旧判断との対応を保持 | Codex |
