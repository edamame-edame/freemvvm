- リリース毎に、毎回システム自体が用途に合わせて動作するか確認する。
- 全体の要求が変更されるたび、内容を適切に見直す。
- 本テストの通過なしにリリースすることは不可。
## Traceabilityと実行証跡

具体的な用途適合の試験設計は[共通UI縦断・profile・配布物](system-scenarios.md)にまとめる。0.1全文は2026-10-17の実装開始指示に伴い承認済み。既存必須gateの代替やrelease合格の記録ではない。

releaseごとの用途適合と必須gateを確認し、対象RQ/case、commit SHA、artifact identity/digest、target/profile、結果・reportを対応付ける。今回のcase割当だけでrelease合格としない。実test IDは `REL-` から始める。共通記録は [Requirements Test Manifest](requirements-test-manifest.md)、要求別対応は [要求-test対応表](requirement-test-matrix.md) に従う。

## 承認履歴（今回）

| 日付 | 版・対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1 / 要求分離に伴うtraceability | 利用者 | 全案反映指示により承認。実装・実行証跡のない項目は未実装・未実行を維持 |

## 改訂履歴（今回）

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | 要求分離に伴うtraceability | Codex |
