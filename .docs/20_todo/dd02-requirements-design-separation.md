# D02 / DD-02 要求・設計・検証境界の整理計画

- 文書版: 0.2
- 状態: D02境界整理を適用済み（2026-10-16）。要求上の補足候補は[D02-RQレビュー](dd02-rq-alignment-review.md)に分離
- 作成日: 2026-10-16
- 対象: [DD-02 所有権と終了](../13_detail_design/02-runtime-lifetime.md)
- 親計画: [要求・基本設計・詳細設計の責務分離計画](requirements-design-separation.md)
- 調査台帳: [全BD/DD調査台帳 D02](requirements-design-separation-inventory.md)
- 要求境界レビュー: [D02-RQレビュー案](dd02-rq-alignment-review.md)

## 目的

DD-LIFE-01～08およびDD-LIFE-03～05の2026-10-13承認済み改訂の意味を保ちながら、DD-02を所有関係・データフロー・状態遷移・cleanup順を記す詳細設計として整える。利用者に見える結果は既存RQとの対応を確認し、要求候補は別案で扱う。検証ケースと後続DDへの作業割当はDD本文から分ける。

## 調査結果

- DD-LIFE-01～08は2026-09-29に承認済み。DD-LIFE-03～05 v0.3も2026-10-13に承認済みである。文書冒頭と各判断表の状態はこの承認履歴と整合しているが、導入段落には「改訂本文はレビュー待ち」とする古い記載が残る。
- DD-02 §3～4はhandle token、Runtime anchor、active registration、弱参照index、shutdown、finalizerの参照・状態・cleanup flowを具体化しており、DDの記載対象である。
- DD-02 §5「レビュー時の確認ケース」は検証内容であり、設計本文から外す候補。現行要求test matrixにはTC-LIFE-01、TC-OWN-01、TC-CALLBACK-01、TC-THREAD-01が割り当てられている。caseの追加・削除は別途対応表の確認を経る。
- DD-02 §3～6にはDD-03/04/07/08/11/16など後続文書への参照がある。仕様根拠の参照は上流向きに揃え、後続作業の引継ぎは本計画または親計画で管理する。
- §6は「上位設計へ確認が必要」とするが、callback/event中shutdownの方針（BD-36-01/02/04）は既に利用者確認済みである。未決の戻りstatus・完了観測・UI loop停止時の扱いはBD-36がDD-06/16とwrapper詳細へ送っており、DD-02内で再決定しない。
- Detached objectのAPI別可否はBD-04が後続詳細設計へ委ねている。DD-02では決定せず、UI tree lifetimeを対象とするDD-11の作業計画に渡す候補とする。

## 適用結果

1. DD-02の導入段落とDD-LIFE-03～05の状態を承認履歴に合わせ、v0.4として記録した。承認済み設計意味は変更していない。
2. 上流RQ・Runtime・BD・ABIと先行DD-01への参照を該当箇所に置き、DD-02から後続DDへの参照を除いた。
3. owner graph、registration cleanup、shutdown、worker finalizerの処理・状態設計を維持した。§6はBD-36の確定済みshutdown方針を前提とする境界説明に改め、未決のAPI status・完了観測・loop停止時の扱いを本書の対象外とした。
4. §5の確認ケース一覧をDD本文から除いた。要求test対応の既存caseは[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)。requirements-test matrixとunit-test計画は変更していない。
5. Detached objectのAPI別動作はDD-02の対象から外し、DD-11の要求境界確認項目として調査台帳に残した。
6. RQ-06-002だけではfinalizer queueのenqueue失敗時にtokenを消費しない結果を直接判断できず、shutdown冪等性・停止後release/UI操作の外部結果もRQ本文で明示されていないため、要求候補として[D02-RQレビュー](dd02-rq-alignment-review.md)に記録した。RQ本文・ID・承認履歴は変更していない。
7. 親計画、調査台帳、`.docs/01_memory/detail-design-planning-status.md`を更新した。

## 対象外

- DD-LIFE-01～08およびDD-LIFE-03～05 v0.3の承認済み意味の変更。
- 新しいshutdown同期API、完了status、timeout/cancel、UI loop停止時の動作の決定。
- Detached objectに対する個別APIの可否、Property状態保持、再所属処理の決定。
- requirements-test matrixやunit-test strategyのcase改訂。
- DD-03以降の設計判断。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
|---|---|---|---|
| 2026-10-16 | D02 / DD-02境界整理の着手 | 利用者 | 「次に行きましょう」に基づきレビュー案を作成。本文改訂は未承認 |
| 2026-10-16 | D02 / DD-02境界整理案とD02-RQの分類 | 利用者 | 「OK 適応して」に基づき承認。DD-02 v0.4へ適用し、外部保証候補は要求本文を変更せず別追跡 |

## 改訂履歴

| 版 | 日付 | 変更 | 承認状態 |
|---|---|---|---|
| 0.1 | 2026-10-16 | DD-02の要求・設計・検証境界と承認状態の不整合を調査し、改訂範囲を計画 | レビュー案 |
| 0.2 | 2026-10-16 | 承認済みD02境界整理をDD-02と進捗台帳へ適用。RQ本文は変更せず、要求候補を別追跡 | 適用済み |
