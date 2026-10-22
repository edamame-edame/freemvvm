# D02-RQ / DD-02由来の要求境界レビュー案

- 文書版: 0.4
- 状態: 全案反映承認済み・責務分離の正本適用完了（2026-10-17）。未設計課題は別計画へ移管
- 作成日: 2026-10-16
- 対象: [DD-02 所有権と終了](../13_detail_design/02-runtime-lifetime.md)
- 作業計画: [D02 / DD-02境界整理](dd02-requirements-design-separation.md)
- 関連要求: [RQ-02](../11_requirements/RQ-02_core-features.md)、[RQ-05](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06](../11_requirements/RQ-06_threading-dispatch.md)
- 関連上位判断: [Runtime契約](../12_design/runtime-contract.md)、[BD-03](../12_design/BD-03_runtime-notification-upper-design.md)、[BD-04](../12_design/BD-04_ui-tree-lifetime-upper-design.md)、[BD-36](../12_design/BD-36_runtime-shutdown-dispatch-upper-design.md)
- 関連要求test: [要求test対応表](../30_release_test/requirement-test-matrix.md)
- 一括レビューへの対応: [全案件索引](requirements-design-separation-review/index.md)、[要求文案SEP-RQ-005～010](requirements-design-separation-review/rq-candidates.md#candidate-005)、[DetachedのD11案](requirements-design-separation-review/d11.md)

2026-10-17の一括作成で、破棄失敗、停止後操作、shutdown再要求と呼出しcontext別結果の要求文案を上記へ集約した。本書の分類方針承認を維持し、要求本文案とcase改訂の採用は別に追跡する。

## 現在の適用結果

[全案適用結果](requirements-design-separation-application-result.md)と[未設計の後続課題](requirements-design-separation-follow-up.md)を参照する。以下の走査・案作成時点の差異や未承認表示は当時の記録であり、現在の承認状態は本節と承認台帳を正本とする。

## 目的と承認境界

DD-02の利用者可視の条件が既存RQで十分表現されているかを分類する。2026-10-16に分類方針が承認された。要求本文・ID・承認履歴は変更しておらず、外部保証の不足候補はRQ改訂案として別途具体化する。

## 論点の分類

| DD-02の論点 | 現行要求・上位根拠 | 分類案 | 対応案 |
|---|---|---|---|
| 公開参照の取得・個別破棄、参照寿命 | RQ-05-002/003、RQ-02-001/013 | 外部契約はRQ、tokenと内部参照の関係はDD | 現行RQを根拠として参照グラフをDDに残す。DD-01とのtoken整合を維持する |
| active Subscription等は公開handle破棄だけで解除されない | RQ-05-002/006～008、RQ-07-011、承認済みDD-02 v0.3 | 外部契約はRQ、registry owner・参照方向・cleanup sequenceはDD | DD-LIFE-03～05の承認済み内容を維持。要求再定義は行わない |
| callback解除後のin-flight処理とuserdata破棄 | RQ-05-007/008、RQ-05-009、Runtime R3/R4 | 外部保証はRQ、CancelPending・safe point・強参照解放順はDD | 状態図とcleanup flowをDDに残し、確認ケースはtest成果物に委ねる |
| workerから最後のUI-affine参照をreleaseした場合 | [RQ-02-013/014](../11_requirements/RQ-02_core-features.md)、[RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06-001～003](../11_requirements/RQ-06_threading-dispatch.md)、Runtime R2 | UI thread破棄の保証は要求/Runtime、finalizer参照とqueue handoffはDD | DD-LIFE-06を維持する。RQ-06-002は一般dispatch失敗の所有・破棄結果を扱うが、finalizer handoff失敗とtoken保持を直接規定しない。RQ改訂候補として別途提示する |
| shutdown後の残存handle release、UI操作拒否 | [RQ-02-013/014](../11_requirements/RQ-02_core-features.md)、[RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md)、Runtime R2、BD-36 | 外部可否はRQ/Runtime契約、Closed shellとanchor処理はDD | DD-LIFE-07/08の実現設計を維持する。停止後release可能・UI操作不可の外部結果はRQ本文で明確化する候補とする |
| shutdownの冪等性と二重cleanup防止 | [RQ-02-013](../11_requirements/RQ-02_core-features.md)、Runtime R2、BD-36 | 複数回呼び出した際の結果は外部契約候補、state guard・一回限りcleanupはDD | DD-LIFE-08の設計を維持する。shutdown再呼出し成功・追加副作用なしをRQで明記する候補とする |
| enqueue失敗時のtoken消費 | [RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md)、Runtime R2 | 失敗時の外部結果はRQ、finalizer確保・token消費順はDD | enqueue失敗でtokenを保持しreleaseを失敗させる利用者可視結果をRQに追記する候補とする。DD-LIFE-06の順序はDDに残す |
| Detached objectの存続・操作可否 | [RQ-02-013](../11_requirements/RQ-02_core-features.md)、BD-04 | Detached存続の上位方針はBD、API別の結果はRQ/DD-11 | DD-02の対象外とし、DD-11のRQ境界確認へ送る。DD-02では新しい契約を決めない |
| 状態遷移・caseごとの観測結果 | RQ-02-013、RQ-05/06、[requirements-test matrix](../30_release_test/requirement-test-matrix.md) | 状態と処理順はDD、合否判定・試験caseはrequirements-test | DDの状態図とflowは残し、旧§5のcase列挙を外した。既存caseは[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)で計画上対応づける。case自体は変更しない |

## 承認済みの分類結果と後続

1. RQ-06-002の一般的なdispatch失敗契約だけではfinalizer handoff失敗時のtoken保持を直接追跡できないため、要求追記候補として残す。
2. shutdown再呼出しとshutdown後のrelease/UI操作結果は、Runtime契約・承認済みDDに記載がある一方、要求本文の利用者可視条件として明確化する候補に残す。
3. Detached objectのAPI別動作はDD-02から除き、DD-11の要求境界確認に渡す。
4. DD-02の確認caseは現行TC-LIFE-01、TC-OWN-01、TC-CALLBACK-01、TC-THREAD-01に計画上対応づける。要求test matrix・unit-test計画のcaseは変更しない。
5. 上記1～2のRQ ID選定、文言、要求test mappingは別のRQ改訂レビューで決める。このレビュー承認だけで要求本文を変更しない。

## 変更しない事項

- RQ本文、要求ID、各要求の承認履歴。
- DD-LIFE-01～08およびDD-LIFE-03～05 v0.3の承認済み意味。
- DD-06のstatus/error catalog、BD-36の未決API表現。
- requirement-test matrixとunit-test計画のcase。
- handle/token、Runtime anchor、registry ownership、weak index、finalizer queueの内部方式。ただし本文から明らかな不整合が見つかった場合は別判断として提示する。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
|---|---|---|---|
| 2026-10-16 | D02-RQ / DD-02由来の要求境界分類 | 利用者 | 「OK 適応して」に基づき分類方針を承認。要求本文と要求test caseは変更せず、RQ不足候補を別レビューへ送る |

## 改訂履歴

| 版 | 日付 | 変更 | 承認状態 |
|---|---|---|---|
| 0.1 | 2026-10-16 | DD-02の外部保証・設計・検証内容を現行RQと上位判断に照合 | レビュー案 |
| 0.2 | 2026-10-16 | 承認済み分類結果、要求候補、既存test case割当とDD-11引継ぎを反映 | 分類方針承認済み。RQ改訂は別途 |
| 0.3 | 2026-10-17 | 全案件レビューの要求文案・D11改訂案への入口を追加 | 分類承認を維持。RQ採用状態は変更なし |

## 2026-10-17 適用結果

全66台帳項目の分離を適用した。D23は要求testへの既存移管を維持し、廃止文書を再作成していない。

### 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 全レビュー案の反映 | 利用者 | 「すべて反映して下さい。」により採用・正本適用を承認 |

### 改訂履歴

| 版 | 日付 | 内容 |
| --- | --- | --- |
| 0.4 | 2026-10-17 | 承認と正本適用結果を追記。作成時点の比較・引用を保持 |
