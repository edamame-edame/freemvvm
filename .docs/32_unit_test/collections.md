# 単体試験：Collection・batch・Selection・snapshot

- 文書版: 0.1
- 状態: 0.1全文承認済み（利用者、2026-10-17）。製品testは未実装・未実行。接続gapは別管理
- 更新日: 2026-10-17

候補単位はCollectionModel/ModelState、BatchBuilder/PreparedState、SelectionValidator。次段階Collectionの試験設計であり、初期releaseの未提供featureを必須passに含めない。

上位根拠: [単体試験方針](unit-test-strategy.md)、[実装unit台帳契約](implementation-test-inventory.md)、[共通fixture・実行記録](README.md)。

設計根拠: [runtime-objects/collections.md](../13_detail_design/runtime-objects/collections.md) 1.1「batchの原子性 / 候補・現行state・読取りsnapshot」。

## 一括更新と不変snapshot

固定Schemaに五値種別の列を用意し、代表入力として3行A/B/CとSingle/Multiple Selectionを作る。3行は順序・削除・移動を判別する最小fixtureで製品容量制限ではない。IDは取得値で照合し連番を仮定しない。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-collection-001"></a>UT-COLLECTION-001 | 同batchでinsert、move、同cellへ複数setを順序付き実行しcommit | 仮state上の順序を反映し、最後の有効setを保存。Model/Selection/通知は同revisionを示す | [RQ-02-005](../11_requirements/RQ-02_core-features.md#rq-02-005)、[RQ-09-062](../11_requirements/RQ-09_ui-scope.md#rq-09-062)、[RQ-09-063](../11_requirements/RQ-09_ui-scope.md#rq-09-063)<br/>[TC-COL-01](../30_release_test/requirement-test-matrix.md#case-tc-col-01) |
| <a id="ut-collection-002"></a>UT-COLLECTION-002 | batchの各prepare段階へOOM、invalid ID、異kind、重複remove、stale revisionを注入 | Model値・順序・Selection・revision・通知の全てが操作前と一致。成功した独立batchを巻き戻さない | [RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)、[RQ-04-026](../11_requirements/RQ-04_abi-boundary.md#rq-04-026)、[RQ-09-063](../11_requirements/RQ-09_ui-scope.md#rq-09-063)<br/>[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-COL-01](../30_release_test/requirement-test-matrix.md#case-tc-col-01) |
| <a id="ut-collection-003"></a>UT-COLLECTION-003 | insert予約後にbatchを失敗させ、次batchで新規insertする | 予約済みitem IDを再利用せず、古いIDを新itemへ誤適用しない | [RQ-02-005](../11_requirements/RQ-02_core-features.md#rq-02-005)<br/>[TC-COL-01](../30_release_test/requirement-test-matrix.md#case-tc-col-01) |
| <a id="ut-collection-004"></a>UT-COLLECTION-004 | range snapshot取得後にmodelを更新・解放し、別threadでIDsとtyped列をcopyする | 取得revisionと値が不変。Schema/segment保持によりmodel寿命に依存しない。容量不足は部分copyなし | [RQ-09-062](../11_requirements/RQ-09_ui-scope.md#rq-09-062)、[RQ-04-021](../11_requirements/RQ-04_abi-boundary.md#rq-04-021)<br/>[TC-COL-01](../30_release_test/requirement-test-matrix.md#case-tc-col-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01) |
| <a id="ut-collection-005"></a>UT-COLLECTION-005 | 選択中itemを削除し、ModelChangedとSelectionChanged内でsnapshotを読む | 削除後のSelectionをModelと一括反映し、ModelChangedが先。同revisionを観測、currentを自動補充しない | [RQ-09-063](../11_requirements/RQ-09_ui-scope.md#rq-09-063)、[RQ-09-064](../11_requirements/RQ-09_ui-scope.md#rq-09-064)<br/>[TC-COL-01](../30_release_test/requirement-test-matrix.md#case-tc-col-01)、[TC-SEL-01](../30_release_test/requirement-test-matrix.md#case-tc-sel-01) |
| <a id="ut-selection-001"></a>UT-SELECTION-001 | 複数selectedを持つ状態でSingleへ縮退、別Collection ID指定、current削除を試す | 無効縮退・所属不一致は旧選択維持。current削除後は空で、Focusを選択と同一化しない | [RQ-02-006](../11_requirements/RQ-02_core-features.md#rq-02-006)、[RQ-09-064](../11_requirements/RQ-09_ui-scope.md#rq-09-064)<br/>[TC-SEL-01](../30_release_test/requirement-test-matrix.md#case-tc-sel-01) |
| <a id="ut-collection-006"></a>UT-COLLECTION-006 | 通知中に同期batch commitを試し、同処理をpostで次turnへ依頼する | 同期再入を拒否。次turnは適格性再検証後に実行可能、旧通知snapshotを途中変更しない | [RQ-09-063](../11_requirements/RQ-09_ui-scope.md#rq-09-063)<br/>[TC-COL-01](../30_release_test/requirement-test-matrix.md#case-tc-col-01) |

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1全文 | — | レビュー案・未承認。作成指示を試験設計の承認や試験合格とは扱わない |
| 2026-10-17 | 0.1全文 | 利用者 | 「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」により既存試験案を承認し環境整備を開始。製品test実装・合格を意味しない |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
