- バージョンアップ毎に、変更要求の内容が適切に動作するか確認する。
- 影響範囲も特定し、それに応じた回帰テストも実施する。
## Traceabilityと実行証跡

変更影響と新旧互換の具体的な試験設計は[回帰計画](regression-plan.md)にまとめる。0.1全文は2026-10-17の実装開始指示に伴い承認済み。実行済みの回帰結果ではない。

変更RQと設計・unit依存をもとに影響回帰を選び、変更前後のversion、同一変更SHA、対象case/profile、結果・reportを記録する。前版の証跡を上書きしない。実test IDは `VER-` から始める。共通記録は [Requirements Test Manifest](../30_release_test/requirements-test-manifest.md)、要求別対応は [要求-test対応表](../30_release_test/requirement-test-matrix.md) に従う。

## 承認履歴（今回）

| 日付 | 版・対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1 / 要求分離に伴うtraceability | 利用者 | 全案反映指示により承認。実装・実行証跡のない項目は未実装・未実行を維持 |

## 改訂履歴（今回）

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | 要求分離に伴うtraceability | Codex |
