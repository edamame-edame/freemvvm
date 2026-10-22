# 配布・保守の運用記録計画

- 文書版: 1.0
- 状態: 所管分離承認済み（2026-10-17）。実運用の担当・値は未確定

| 対象 | 処理・記録 | 要求根拠 |
| --- | --- | --- |
| license inventory | first-party/依存componentとpackage内容を対応させ、必要license/notice/変更表示を収録してrelease検査へ渡す | [RQ-12-016](../11_requirements/RQ-12_build-distribution.md#rq-12-016) |
| support registry | ABI major×profileをkeyに状態・開始/終了日・告知と最終artifact参照を更新する | [RQ-12-017](../11_requirements/RQ-12_build-distribution.md#rq-12-017)、[RQ-12-018](../11_requirements/RQ-12_build-distribution.md#rq-12-018)、[RQ-12-019](../11_requirements/RQ-12_build-distribution.md#rq-12-019)、[RQ-12-020](../11_requirements/RQ-12_build-distribution.md#rq-12-020) |
| artifact release | 検証済みidentity/digest、target/profile、build/runtime/hardwareの証拠を紐付けて公開する | [RQ-11-021](../11_requirements/RQ-11_platform.md#rq-11-021)、[RQ-12-011](../11_requirements/RQ-12_build-distribution.md#rq-12-011)、[RQ-12-012](../11_requirements/RQ-12_build-distribution.md#rq-12-012)、[RQ-12-013](../11_requirements/RQ-12_build-distribution.md#rq-12-013) |
| 未確定の運用入力 | 担当者、公式URL/registry、鍵保管、保管権限・期限、security response手順を具体化する | RQ-16-004に従って新規保証を別承認する |

## 承認履歴（今回）

| 日付 | 版・対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 1.0 / SEP-OPS-01～03の所管移管 | 利用者 | 全案反映指示により承認。実装・実行証跡のない項目は未実装・未実行を維持 |

## 改訂履歴（今回）

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 1.0 | 2026-10-17 | SEP-OPS-01～03の所管移管 | Codex |
