# 単体試験：Tree・Layout・Control・Input・TextBox

- 文書版: 0.2
- 状態: 0.2改訂案レビュー待ち。0.1承認内容を維持し、Tree内部ownerの寿命試験を追加。製品testは未実装・未実行
- 更新日: 2026-10-24

候補単位はTreeCoordinator、LayoutEngine、具象Control、InputRouter/FocusManager、TextBox編集transaction。OS pixel一致や実IME接続はsystem/platform試験へ分離する。

上位根拠: [単体試験方針](unit-test-strategy.md)、[実装unit台帳契約](implementation-test-inventory.md)、[共通fixture・実行記録](README.md)。

設計根拠: [ui/tree-layout.md](../13_detail_design/ui/tree-layout.md) 1.2案「Tree所属 / クラス設計の補足」、[runtime-objects/objects-values.md](../13_detail_design/runtime-objects/objects-values.md) 1.3案「Tree内部ownerとpublic参照の独立管理」、[ui/controls.md](../13_detail_design/ui/controls.md) 1.1「Control API / Window / 機能委譲」、[ui/input-focus.md](../13_detail_design/ui/input-focus.md) 1.1「Focus状態 / 入力・IME接続」、[ui/text-editing.md](../13_detail_design/ui/text-editing.md) 1.1「入力検証 / IME / 編集候補と確定値」。

## Tree・Layout・Control

Window、StackPanel、Text、TextBox、Buttonを同Runtimeで構成し、別RuntimeのControlも用意。fake peerの生成・resize・close失敗を個別に制御する。layoutはCore論理座標とDDの制約式で判定する。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-tree-001"></a>UT-TREE-001 | 子のinsert/remove/moveと別Runtime・二重所属・循環を試す | 成功時だけ所属commit。invalidとpeer準備失敗で旧Treeを保持、親子強参照循環を作らない | [RQ-09-002](../11_requirements/RQ-09_ui-scope.md#rq-09-002)、[RQ-09-003](../11_requirements/RQ-09_ui-scope.md#rq-09-003)、[RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)<br/>[TC-TREE-01](../30_release_test/requirement-test-matrix.md#case-tc-tree-01)、[TC-VERT-01](../30_release_test/requirement-test-matrix.md#case-tc-vert-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |
| <a id="ut-tree-003"></a>UT-TREE-003 | Attached childの全public handleを解放し、親からのchildren列挙、detach、owner解放後のcleanupを追う。attach時のcleanup予約失敗も注入 | Attached中はchild stateを保持し、TreeOwnerから新しい独立public handleを発行可能。detach後にpublic/internal ownerが全て0ならcleanupを一度だけ実施。予約失敗時はTree・参照数不変。Runtime shutdown後のowner解放でticketを再発行しない | [RQ-02-007](../11_requirements/RQ-02_core-features.md#rq-02-007)、[RQ-02-013](../11_requirements/RQ-02_core-features.md#rq-02-013)、[RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)、[RQ-09-003](../11_requirements/RQ-09_ui-scope.md#rq-09-003)、[RQ-09-043](../11_requirements/RQ-09_ui-scope.md#rq-09-043)<br/>[TC-TREE-01](../30_release_test/requirement-test-matrix.md#case-tc-tree-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01) |
| <a id="ut-tree-002"></a>UT-TREE-002 | detach/re-attach時に表示資源生成を失敗させ、後で明示再試行する | Detached構造・確定値・登録を保持。所属だけ成功と公開せず、再試行で正しく接続 | [RQ-09-069](../11_requirements/RQ-09_ui-scope.md#rq-09-069)<br/>[TC-TREE-01](../30_release_test/requirement-test-matrix.md#case-tc-tree-01) |
| <a id="ut-layout-001"></a>UT-LAYOUT-001 | StackPanelの両orientation、空/複数child、constraint変更を与え、計算途中失敗も注入 | Sizerは下から、Locationerは上から計算。同revisionのboundsを公開し、表現不能時は旧公開配置を保つ | [RQ-09-067](../11_requirements/RQ-09_ui-scope.md#rq-09-067)<br/>[TC-LAYOUT-01](../30_release_test/requirement-test-matrix.md#case-tc-layout-01) |
| <a id="ut-control-001"></a>UT-CONTROL-001 | ButtonのCommandを交換、異Runtime設定、Button破棄後に共有ownerからexecuteする | 失敗交換は旧参照維持。Button破棄だけでCommandをcloseせず、共有ownerの利用を壊さない | [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-06-010](../11_requirements/RQ-06_threading-dispatch.md#rq-06-010)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01)、[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-window-001"></a>UT-WINDOW-001 | open失敗後再試行、close後再open、古いgenerationのCloseRequest resolveを試す | Core content/Propertyを保持し、stale requestを新peerへ誤適用しない。OS closeでRuntime終了しない | [RQ-09-071](../11_requirements/RQ-09_ui-scope.md#rq-09-071)、[RQ-11-020](../11_requirements/RQ-11_platform.md#rq-11-020)<br/>[TC-PLAT-01](../30_release_test/requirement-test-matrix.md#case-tc-plat-01)、[TC-SUPPORT-01](../30_release_test/requirement-test-matrix.md#case-tc-support-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-window-002"></a>UT-WINDOW-002 | close要求にAccept/Decline/Defer、未解決release、Runtime終了を別fixtureで与える | 既定Accept、未解決破棄Decline、一回解決。Runtime終了は個別vetoで停止しない | [RQ-09-071](../11_requirements/RQ-09_ui-scope.md#rq-09-071)<br/>[TC-PLAT-01](../30_release_test/requirement-test-matrix.md#case-tc-plat-01) |
## Input・Focus

active Windowとfocusable/enabledなTextBox二つ、非focusable Containerを用意する。共通eventをUI queueへ投入し、native focus同期をfakeで折返す。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-input-001"></a>UT-INPUT-001 | target→Logical祖先の配送中にhandled指定とdetachをそれぞれ起こす | handled後の配送/既定動作を止め、各段の所属を再検証。再帰入力を開始しない | [RQ-09-068](../11_requirements/RQ-09_ui-scope.md#rq-09-068)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)<br/>[TC-INPUT-01](../30_release_test/requirement-test-matrix.md#case-tc-input-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01) |
| <a id="ut-focus-001"></a>UT-FOCUS-001 | A→BのFocus要求と、disabled/Detached/非active Windowへの無効要求を別々に与える | 有効時は新Focus commit後A.Lost→B.Gained。無効時は旧Focusとイベント/IME状態を維持 | [RQ-02-010](../11_requirements/RQ-02_core-features.md#rq-02-010)、[RQ-09-046](../11_requirements/RQ-09_ui-scope.md#rq-09-046)、[RQ-09-049](../11_requirements/RQ-09_ui-scope.md#rq-09-049)<br/>[TC-FOCUS-01](../30_release_test/requirement-test-matrix.md#case-tc-focus-01)、[TC-ACC-07](../30_release_test/requirement-test-matrix.md#case-tc-acc-07)、[TC-INPUT-01](../30_release_test/requirement-test-matrix.md#case-tc-input-01) |
| <a id="ut-focus-002"></a>UT-FOCUS-002 | active focusなしでkeyboard/textを投入し、その後Focusを得る | 対象なし入力を後で再送しない。Window/StackPanelのfocusable=trueを受け付けずTab割当を創作しない | [RQ-09-070](../11_requirements/RQ-09_ui-scope.md#rq-09-070)、[RQ-09-049](../11_requirements/RQ-09_ui-scope.md#rq-09-049)<br/>[TC-FOCUS-01](../30_release_test/requirement-test-matrix.md#case-tc-focus-01)、[TC-INPUT-01](../30_release_test/requirement-test-matrix.md#case-tc-input-01) |
## 編集候補・Validation・IME

初期確定値と編集snapshotを取得。Validatorの戻り値、session ID、copy/feedback確保failpointを手動制御。日本語・結合文字・絵文字の代表列はbyte位置検査用で、全Unicode網羅とは扱わない。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-text-001"></a>UT-TEXT-001 | 通常入力をAccept/Incomplete/Rejectに分岐させ、Property拒否も別に発生 | Acceptだけ確定へ進み、Incompleteは編集のみ、通常Rejectは旧本文/位置、Property拒否は候補を編集領域へ保持 | [RQ-09-038](../11_requirements/RQ-09_ui-scope.md#rq-09-038)、[RQ-09-039](../11_requirements/RQ-09_ui-scope.md#rq-09-039)<br/>[TC-IME-01](../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-VERT-03](../30_release_test/requirement-test-matrix.md#case-tc-vert-03) |
| <a id="ut-text-002"></a>UT-TEXT-002 | IME Start/Update、Commit Reject、明示Cancel、終了sessionの遅延eventを個別に投入 | preeditはValidator/Property/Bindingへ伝播しない。IME Rejectは候補保持、Cancelは開始組へ復元、staleは不適用 | [RQ-09-038](../11_requirements/RQ-09_ui-scope.md#rq-09-038)、[RQ-09-040](../11_requirements/RQ-09_ui-scope.md#rq-09-040)、[RQ-09-056](../11_requirements/RQ-09_ui-scope.md#rq-09-056)<br/>[TC-IME-01](../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-VERT-03](../30_release_test/requirement-test-matrix.md#case-tc-vert-03) |
| <a id="ut-text-003"></a>UT-TEXT-003 | 未確定preedit中にFocusを離し、再Focusする | 編集本文とcaret/selectionを保持しsessionを終了。Focus移動だけでcommit/cancel/Validatorを起こさない | [RQ-09-055](../11_requirements/RQ-09_ui-scope.md#rq-09-055)<br/>[TC-IME-01](../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-text-004"></a>UT-TEXT-004 | UTF8途中byte、stale TextIndexMap、selectionのみ変更、候補copy失敗を別々に与える | 無効位置と古い変換を拒否。selectionのみはValidatorなし。copy失敗した候補を保存済みrecoveryとしない | [RQ-09-065](../11_requirements/RQ-09_ui-scope.md#rq-09-065)、[RQ-09-041](../11_requirements/RQ-09_ui-scope.md#rq-09-041)、[RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)<br/>[TC-IME-01](../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |
| <a id="ut-text-005"></a>UT-TEXT-005 | 同一編集のBinding伝播、Property通知、編集snapshot通知に順序ログを付ける | RQ-09-066の順序とsnapshot整合をoracleにする。DDの通知順保留記述解消まで実装入口はblocked | [RQ-09-066](../11_requirements/RQ-09_ui-scope.md#rq-09-066)<br/>[TC-IME-01](../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01) |
| <a id="ut-text-006"></a>UT-TEXT-006 | preedit中に外部から異なる確定値を設定し、その後旧sessionのupdate/commitを投入 | 編集値を新確定値へ置換しfeedback/reasonを初期化。旧sessionを無効化し遅延入力で新値を上書きしない | [RQ-09-042](../11_requirements/RQ-09_ui-scope.md#rq-09-042)<br/>[TC-IME-01](../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01) |

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1全文 | — | レビュー案・未承認。作成指示を試験設計の承認や試験合格とは扱わない |
| 2026-10-17 | 0.1全文 | 利用者 | 「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」により既存試験案を承認し環境整備を開始。製品test実装・合格を意味しない |
| 2026-10-24 | 0.2改訂案 | — | A案に基づきTreeOwner / public handle lifetimeのUT-TREE-003を追加。文書全体レビュー待ち |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
| 0.2 | 2026-10-24 | CoreObjectControlのTree internal ownerとcleanup reservationの寿命・失敗境界をUT-TREE-003で具体化 | Codex | 文書全体のレビュー待ち。未実装・未実行 |
