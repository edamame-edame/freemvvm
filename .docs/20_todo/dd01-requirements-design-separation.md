# D01 / DD-01 要求・設計・テスト境界の整理計画

- 文書版: 0.1
- 状態: DD-01境界整理を適用済み（2026-10-16）。要求境界候補は[別レビュー案](dd01-rq-alignment-review.md)で未承認
- 作成日: 2026-10-15
- 対象: [DD-01 Handle登録と検証](../13_detail_design/01-handle-registry.md)
- 親計画: [要求・基本設計・詳細設計の責務分離計画](requirements-design-separation.md)
- 調査台帳: [全BD/DD調査台帳 D01](requirements-design-separation-inventory.md)

## 目的

承認済みDD-HND-01～06の意味を維持しながら、外部保証候補とregistry・token生成・同期方式を分け、DD-01の役割を詳細設計に整える。要求変更の候補は別レビュー案に分離し、既承認RQを変更しない。

## 調査結果

- [RQ-04-004～007](../11_requirements/RQ-04_abi-boundary.md)はopaque handle、retain/release、status、error objectを定める。
- [RQ-05-001～003](../11_requirements/RQ-05_ownership-callbacks.md)は所有参照の識別、複数参照の独立破棄、破棄後の無効化とRuntime shutdownを定める。
- [RQ-02-001](../11_requirements/RQ-02_core-features.md)は対象の寿命終了後に無効参照を検出する結果を含む。[RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md)は複数threadから変更可能な共有状態の同期を要求する。
- [Runtime契約 §2/ R1](../12_design/runtime-contract.md)はtoken非再利用と枯渇時失敗、[同§3](../12_design/runtime-contract.md)は`retain` / `release`の任意thread実行を定める。
- DD-HND-01～06は単調64-bit発行器、登録表、検証順、線形化、lock範囲、枯渇時status/codeを具体化する。
- 要求test対応表にはTC-CORE-OBJ-01、TC-LIFE-01、TC-ABI-REP-01があり、DD-01 §5の多くの外部caseと重なる。競合・枯渇等の内部方式確認はunit-test側の対象候補となる。

## 実施結果

1. DD-01と承認済みRQ、Runtime契約、ABI仕様を照合した。
2. DD-01に上位根拠を各記述箇所から直接開けるリンクとして記載し、独立した要求対応表・出典一覧は追加しなかった。
3. token発行器・登録表・検証順・線形化・同期範囲をDDに残し、利用者向けerror分類と数値割当をDD-01から除いた。
4. DD-01の確認case一覧を除いた。既存の外部確認caseはTC-CORE-OBJ-01、TC-LIFE-01、TC-ABI-REP-01で対応を追跡する。内部競合・枯渇・部分登録不在のunit-testは各実装単位の計画で設計する。test成果物は変更していない。
5. DD-01から下流DDへの参照を除き、作業順・引継ぎは本計画で管理する。
6. DD-HND-01～06のIDと既存承認履歴を保持した。
7. DD-01、親計画、調査台帳、memoryを更新した。要求候補は[別レビュー案](dd01-rq-alignment-review.md)に分離した。

## 対象外

- 新しいtoken種別、token幅、status/code値、registry実装の採用。
- 既存RQの承認範囲の自動拡張。RQ変更は候補として提示し、個別承認を経る。
- 要求test matrixやunit-test strategyのcase追加・改訂。
- DD-02/03の詳細設計判断や着手。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
|---|---|---|---|
| 2026-10-15 | D01 / DD-01責務整理の開始 | 利用者 | 「次へ」に基づきD01を次作業として開始。具体的な改訂案は未承認 |
| 2026-10-16 | D01 / DD-01境界整理案 | 利用者 | 「ok 適用してください」に基づきDD-01改訂を承認。要求本文の改訂は別レビュー案へ分離 |

## 改訂履歴

| 版 | 日付 | 変更 | 承認状態 |
|---|---|---|---|
| 0.1 | 2026-10-15 | DD-01のRQ・DD・test境界を整理する計画を作成 | 改訂案レビュー待ち |
| 0.2 | 2026-10-16 | 独立した要求対応表・出典一覧を設けず、各根拠箇所に直接リンクするよう案を修正 | 改訂案レビュー待ち |
| 0.3 | 2026-10-16 | DD-01改訂を適用し、要求境界の追加候補を別レビュー案へ分離 | DD-01改訂承認済み。RQ候補は未承認 |
