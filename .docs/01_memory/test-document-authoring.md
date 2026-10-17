# テスト文書 / 単体6分野・release縦断・version回帰

- 更新日: 2026-10-17
- スコープ: `.docs/30_release_test/system-scenarios.md`、`.docs/31_version_test/regression-plan.md`、`.docs/32_unit_test/README.md`と分野別6文書
- 計画・結果: [テスト文書作成](../20_todo/test-document-authoring.md)
- 入口: [試験設計](../32_unit_test/README.md)

利用者の「OK,テスト文書作りましょうか」を、既存DD1.1全文の承認とテスト文書作成開始として扱った。DD19本文と索引へ承認履歴を追記し、1.0原文の保持を検査した。本文の既存未決記述を承認だけで確定へ読み替えない。

要求test正本の280RQ・92論理TCは変更していない。追加した9文書は0.1レビュー案・未承認。単体はCore/runtime、events、collections、UI、wrappers、platform/buildの6分野70項目、releaseは9scenario、versionは6項目。各分野のfixture・操作・期待値・RQ/TC・後始末を記し、予定test IDと設計IDを区別する。TCリンクは関連付けであり、一行でTC全体を充足したことにしない。

source/test path・UNIT境界・実command・SHA・reportは実装時に登録する。現段階はplanned / 未実装 / not-run。Column単位のassertion結果を保持し、unit fakeと実OS/package、cross-buildとboard実行を分ける。Collectionは次段階。coverage閾値・正式support範囲をこの作業で追加しない。

接続gapは3点。Command本文は専用進捗なしだがRQ-06-013は単一channelを要求。TextBox本文に通知順保留が残るがRQ-09-066は順序を規定。Focus状態図のinvalid/no-change矢印がActiveFocusedからActiveNoFocusへ向く。該当文書へリンクして差異と変更対象を入口に記録した。製品DDは今回の承認記録以外は変更していない。UT-COMMAND-005とUT-TEXT-005はoracle/API接続待ちであり、対応unitのready化前にDDを別改訂する。

静的検査は追加9文書・予定test ID85件の重複なし、BD/DDと案内を含む1983リンク/anchor・277表でエラー0件。DD1.0本文保持、今回の変更対象外312ファイルhash一致。製品test・build・CI・Mermaid描画・commit/pushは未実施。検査資料はGit除外のDDローカルarchiveへ保存した。

## 実装開始時点の承認

2026-10-17の「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」により既存9文書の0.1全文を承認済みへ更新し、各承認履歴へ追記した。上記の未承認は初回作成時点の記録。製品test70+9+6項目は未実装・未実行を維持する。開発基盤のprobeは別のUNIT-BUILD-001として扱い、製品test合格へ換算しない。
