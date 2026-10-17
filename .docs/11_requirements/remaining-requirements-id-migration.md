# 具体化レビューの要求ID対応

- 文書版: 1.0
- 状態: 要求変更承認済み（2026-10-17）

暫定判断番号は要求変更の由来として記録し、現行本文は正式RQを参照する。既存IDを廃止・再利用していない。

| 暫定番号 | 正式RQ | 内容 |
| --- | --- | --- |
| REM-RQ-01 | [RQ-04-029](RQ-04_abi-boundary.md#rq-04-029) | F32変換の外部結果 |
| REM-RQ-02 | [RQ-02-022](RQ-02_core-features.md#rq-02-022) | 文字列と数値の明示converter |
| REM-RQ-03 | [RQ-09-062](RQ-09_ui-scope.md#rq-09-062) | Collectionの型とsnapshot |
| REM-RQ-04 | [RQ-09-063](RQ-09_ui-scope.md#rq-09-063) | Collection通知と再入 |
| REM-RQ-05 | [RQ-09-064](RQ-09_ui-scope.md#rq-09-064) | Selection縮退・差替え |
| REM-RQ-06 | [RQ-06-012](RQ-06_threading-dispatch.md#rq-06-012) | 終了要求の完了観測 |
| REM-RQ-07 | [RQ-03-013](RQ-03_language-support.md#rq-03-013) | 初期unload範囲 |
| REM-RQ-08 | [RQ-07-017](RQ-07_binding.md#rq-07-017) | WatchDog設定適用時点 |
| REM-RQ-09 | [RQ-06-013](RQ-06_threading-dispatch.md#rq-06-013) | Command進捗値 |
| REM-RQ-10 | [RQ-09-065](RQ-09_ui-scope.md#rq-09-065) | TextBox編集と位置の操作 |
| REM-RQ-11 | [RQ-09-066](RQ-09_ui-scope.md#rq-09-066) | TextBox通知の観測順 |
| REM-RQ-12 | [RQ-09-067](RQ-09_ui-scope.md#rq-09-067) | 座標・数値範囲・clip |
| REM-RQ-13 | [RQ-09-068](RQ-09_ui-scope.md#rq-09-068) | Input配送とpointer範囲 |
| REM-RQ-14 | [RQ-09-069](RQ-09_ui-scope.md#rq-09-069) | 再所属失敗と再試行 |
| REM-RQ-15 | [RQ-09-070](RQ-09_ui-scope.md#rq-09-070) | Containerのfocusable範囲 |
| REM-RQ-16 | [RQ-09-071](RQ-09_ui-scope.md#rq-09-071) | Window開閉とclose判断 |
| REM-RQ-17 | [RQ-11-022](RQ-11_platform.md#rq-11-022) | 描画backendとfallback |
| REM-RQ-18 | [RQ-09-072](RQ-09_ui-scope.md#rq-09-072) | 初期text表示の範囲 |
| REM-RQ-19 | [RQ-11-023](RQ-11_platform.md#rq-11-023) | font・IME・Accessibility profile |
| REM-RQ-20 | [RQ-10-014](RQ-10_accessibility.md#rq-10-014) | OS同期actionの待機境界 |
| REM-RQ-21 | [RQ-03-014](RQ-03_language-support.md#rq-03-014) | 言語minimumとbuild基準 |
| REM-RQ-22 | [RQ-03-015](RQ-03_language-support.md#rq-03-015) | Python正常cleanupの協調条件 |
| REM-RQ-23 | [RQ-03-016](RQ-03_language-support.md#rq-03-016) | Python基底初期化とoverride範囲 |
| REM-RQ-24 | [RQ-03-017](RQ-03_language-support.md#rq-03-017) | callback markerと検知能力 |
| REM-RQ-25 | [RQ-12-022](RQ-12_build-distribution.md#rq-12-022) | manifest資源上限 |
| REM-RQ-26 | [RQ-12-023](RQ-12_build-distribution.md#rq-12-023) | offline期限・rollback/失効 |
| REM-RQ-27 | [RQ-11-024](RQ-11_platform.md#rq-11-024) | 初期support profile |
| REM-RQ-28 | [RQ-04-030](RQ-04_abi-boundary.md#rq-04-030) | 初回stableとwrapper互換 |
| REM-RQ-29 | [RQ-12-024](RQ-12_build-distribution.md#rq-12-024) | 保守・証拠の保存 |
| REM-RQ-30 | [RQ-12-025](RQ-12_build-distribution.md#rq-12-025) | 初期link形式 |
| REM-RQ-31 | [RQ-09-073](RQ-09_ui-scope.md#rq-09-073) | 次段階Style/Theme/Resource/Template |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 1.0 / 31要求変更の正式ID対応 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 1.0 | 2026-10-17 | 31要求変更の正式ID対応を正本へ適用し、旧判断との対応を保持 | Codex |
