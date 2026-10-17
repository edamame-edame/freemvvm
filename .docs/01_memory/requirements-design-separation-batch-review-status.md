# 要求・設計分離: 全案件レビュー案とcoverage

- 最新状態: [2026-10-17 全案適用](requirements-design-separation-application-status.md)

以下はそれ以前の進行記録。現在の承認・適用状態は上記を参照する。


- 更新日: 2026-10-17
- スコープ: requirements-design-separationのSEP-01～07、X01～10、全66文書
- 入口: [全案件レビュー索引](../20_todo/requirements-design-separation-review/index.md)
- 作成計画: [一括作成計画](../20_todo/requirements-design-separation-batch-review.md)

利用者は個別に作っていた残案件のレビュー案を一括作成するよう指示した。98 Markdownを作成し、残る文書別62件、横断10件、全15RQ章、要求文案63件とtest/API/SD/運用/ID移行案を用意した。D00/D01/D02/D23の4適用済み記録は維持し、全66台帳項目の未掲載は0件。案作成完了と採用・正本適用の完了を区別する。

現行RQ定義と要求test表の集合は187件。旧移行時の195 IDには廃止済みRQ-08-001～008が含まれるため、現行件数と旧履歴の件数を混同しない。旧RQ08を再利用していない。仮候補SEP-RQ-*とレビューID SEP-*は正式仕様IDではなく、採否・意味単位の承認後に必要な正式番号だけを発番する。

要求本文は能力・条件・最終結果、BDは責務/data/ownership flow、DDは具体call/state/lifecycleへ分けた。各文書案に原文箇所と作成時hash、承認済み範囲、具体改訂本文とMermaidを置いた。DD間の依存リンクは定義元への参照と後続作業割当を区別し、文書番号の大小だけで削除しない。

F32の利用API、停止loopでの同期待機、TextBox自身のProperty拒否、cancel理由、IME OOM、Control公開既定/未接続Button、Python host終了検知等は選択要/設計gapとして保持。JCS/Ed25519/SHA-256の既承認提案範囲を維持し、未定envelope/鍵運用等はSDへ割り当てる。DD-24、Collection/Selection専用DD、具体公開APIを今回完成扱いにしない。

現行BD/DD65ファイルは生成前後hash一致。新レビュー内3,546リンク/anchor、277表、62 Mermaid fence、63候補、166承認行とRQ/test集合を検査しエラー0件。検査は文書構造とcoverageであり、Mermaid描画・製品test実行・設計の実装を証明しない。
