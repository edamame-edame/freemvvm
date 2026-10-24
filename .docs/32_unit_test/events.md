# 単体試験：callback・Property・Binding・Command

- 文書版: 0.2
- 状態: 0.1全文承認済み（利用者、2026-10-17）。0.2でCore/C ABI非同期Command試験を追加。focused Rust testsとC consumer smokeはlocal pass。Windows/Linux CIは別管理
- 更新日: 2026-10-24

候補単位はcallback実行guard、Property/通知queue、BindingGraph/PropagationChain、Command/operation。workerの業務処理はfakeで結果を制御し、実thread poolの性能はここで判定しない。

上位根拠: [単体試験方針](unit-test-strategy.md)、[実装unit台帳契約](implementation-test-inventory.md)、[共通fixture・実行記録](README.md)。

設計根拠: [events/callbacks.md](../13_detail_design/events/callbacks.md) 1.1「登録とinvoke / クラス設計の補足」、[events/notifications-binding.md](../13_detail_design/events/notifications-binding.md) 1.1「通知slice / Binding graph / 診断」、[events/commands.md](../13_detail_design/events/commands.md) 1.1「状態遷移 / operation報告のUI反映」。

## 登録と通知

invokeログにcallback種別・開始終了・UI thread・destroy回数を記録する。通知が実際に変わる異値setterを使い、同値coalesceと誤認しない。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-callback-001"></a>UT-CALLBACK-001 | Subscription成功後に異なる値を二回更新。public ownerだけreleaseして再更新、その後cancelする | cancel前は再登録なしで繰返しinvoke、owner releaseだけで解除しない。cancel後の新invokeなし | [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01) |
| <a id="ut-callback-002"></a>UT-CALLBACK-002 | callback内で自己cancelし、未配信更新も残す | 現在invoke中userdataは有効、終了後destroy一回。後続invokeとreleaseによる再destroyなし | [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)<br/>[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01) |
| <a id="ut-callback-003"></a>UT-CALLBACK-003 | Validator内で同Runtime setter・自己cancel・終了要求をそれぞれ試す | 禁止操作は変更前に拒否。終了要求は受付でき、現在callback終了後の終了規則に従う | [RQ-05-013](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-013)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01) |
| <a id="ut-callback-004"></a>UT-CALLBACK-004 | 通知例外と戻り値callback異常を別fixtureで注入 | 通知例外を診断し他購読者を継続。判定callback異常は通常入力拒否と別結果、ABIへ例外を通さない | [RQ-05-014](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-014)、[RQ-05-015](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-015)<br/>[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01) |
| <a id="ut-notify-001"></a>UT-NOTIFY-001 | 購読者A内で後続BをcancelしCを追加、Propertyを再更新する | Bは解除後呼ばれず、Cは既存配信snapshotへ割り込まない。再更新は次回配信、再帰なし | [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-07-008](../11_requirements/RQ-07_binding.md#rq-07-008)<br/>[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
## Bindingと有限作業枠

独立source/targetを三つ以上用意し、外側UI scopeの退出を手動制御する。graph・Property値・queue・エラーsnapshotを操作前後で観測する。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-binding-001"></a>UT-BINDING-001 | OneTime/OneWay/TwoWayを別fixtureで作成し、両端へ直接異値を設定 | 各modeの初期同期と方向を保持。TwoWayの同scope競合は最後に成功した直接変更が勝つ | [RQ-07-001](../11_requirements/RQ-07_binding.md#rq-07-001)、[RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-003](../11_requirements/RQ-07_binding.md#rq-07-003)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-VERT-04](../30_release_test/requirement-test-matrix.md#case-tc-vert-04)、[TC-VERT-03](../30_release_test/requirement-test-matrix.md#case-tc-vert-03) |
| <a id="ut-binding-002"></a>UT-BINDING-002 | 自己循環・間接循環・入力元重複・初期target拒否・作成OOMを個別に与える | 接続作成拒否後の値・graph・queueは旧状態。拒否で既存接続を壊さない | [RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)、[RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-BIND-INIT-REJECT-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-init-reject-01)、[TC-BIND-PROPAGATION-REJECT-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-propagation-reject-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |
| <a id="ut-binding-003"></a>UT-BINDING-003 | 作成済みBindingのtarget検証を拒否後、受理値をsourceへ設定 | source確定値とtarget旧値を保ちBinding errorを取得可能。後の成功で現在errorをclear、保持snapshotは不変 | [RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)、[RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)<br/>[TC-BIND-INIT-REJECT-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-init-reject-01)、[TC-BIND-PROPAGATION-REJECT-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-propagation-reject-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-binding-004"></a>UT-BINDING-004 | 同scopeでsourceを複数回変更し、そのscope内と通知callbackでtargetを読む | sourceは直ちに新値、target反映はscope退出後。Bindingは最終値を伝播し外部通知前に完了 | [RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-binding-005"></a>UT-BINDING-005 | 完了したdelivery itemが128/129件になるfixtureを作り、yield前後とshutdownを観測 | 128完了境界でyieldし残件を保持。Property購読者snapshotやcallback途中で切らず、終了優先を維持 | [RQ-07-008](../11_requirements/RQ-07_binding.md#rq-07-008)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-binding-006"></a>UT-BINDING-006 | hash衝突する非同値履歴と意味snapshot一致履歴を注入し、履歴予算枯渇を別に試す | hash一致だけで停止証拠にしない。証拠成立時は当該連鎖だけ停止し、情報不足を循環証明にしない | [RQ-07-014](../11_requirements/RQ-07_binding.md#rq-07-014)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-binding-007"></a>UT-BINDING-007 | 同一保留chainでfake clockを既定5秒の前後へ進め、閾値変更・無効化を別fixtureで試す | 同じ保留状態の診断は一回。無効化は診断だけを止め、yieldや証拠に基づく停止を無効化しない | [RQ-07-013](../11_requirements/RQ-07_binding.md#rq-07-013)、[RQ-07-014](../11_requirements/RQ-07_binding.md#rq-07-014)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |

## Command operation

UI実行要求とstart callback、fake worker、ID付きpostを別の制御点にする。全testで最後にCommandを明示close、workerを解放してからRuntimeを終了する。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-command-001"></a>UT-COMMAND-001 | can_execute false、実行中、record確保失敗でexecuteし、その後有効executeを試す | 受付前拒否はID=0・worker開始なし・旧状態維持。受理後は新ID/RUNNINGをcommitしてstart一回 | [RQ-06-008](../11_requirements/RQ-06_threading-dispatch.md#rq-06-008)、[RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)<br/>[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |
| <a id="ut-command-002"></a>UT-COMMAND-002 | start起動失敗とworker失敗を別々に報告し、終端後に再executeする | 受付成功と業務FAILEDを区別。終端から新IDで再実行でき、再登録やIDLEへ戻す操作を要求しない | [RQ-02-004](../11_requirements/RQ-02_core-features.md#rq-02-004)、[RQ-06-004](../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)、[RQ-06-008](../11_requirements/RQ-06_threading-dispatch.md#rq-06-008)<br/>[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-command-003"></a>UT-COMMAND-003 | 同IDを重複cancelし、workerから成功・失敗・cancelledを別fixtureで報告 | cancel flag/CANCELLINGと実際の終端を区別。重複取消は追加効果なし、終端反映一回 | [RQ-06-004](../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)、[RQ-06-009](../11_requirements/RQ-06_threading-dispatch.md#rq-06-009)<br/>[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-command-004"></a>UT-COMMAND-004 | 新operation開始後に旧ID報告、終端後に同ID進捗、close後に完了報告を投入 | 現在operationとUIを上書きしない。closeは新実行を止め取消を要求しUIでworker joinしない | [RQ-06-010](../11_requirements/RQ-06_threading-dispatch.md#rq-06-010)、[RQ-06-011](../11_requirements/RQ-06_threading-dispatch.md#rq-06-011)<br/>[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-core-command-001"></a>UT-CORE-COMMAND-001 | fake start closureとworker completionを制御して、eligible/disabled、連続execute、cancel、success/failure/cancelled、stale ID、close/shutdownを実行 | ID発行、RUNNING/CANCELLING/終端、state/can-execute同時commit、failure snapshot clear/immutable、close後更新抑止、worker非join、closure destroy一回を確認する | [RQ-02-004](../11_requirements/RQ-02_core-features.md#rq-02-004)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-06-004](../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)、[RQ-06-008](../11_requirements/RQ-06_threading-dispatch.md#rq-06-008)、[RQ-06-009](../11_requirements/RQ-06_threading-dispatch.md#rq-06-009)、[RQ-06-010](../11_requirements/RQ-06_threading-dispatch.md#rq-06-010)、[RQ-06-011](../11_requirements/RQ-06_threading-dispatch.md#rq-06-011)<br/>[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01) |
| <a id="ut-abi-command-001"></a>UT-ABI-COMMAND-001 | C ABIでCommand create/execute/cancel/close、state/can-execute Property取得、failure snapshot、callback/userdata/destroyを操作し、wrong-thread/runtime/type、post failureを注入 | out値初期化、status/error mapping、owned/borrowed token、create/post時のclosure ownership、retain後のworker query/report、C ABI境界でのpanic/例外越境なしを確認する | [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)、[RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-ABI-COMP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-comp-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01), [TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-abi-command-smoke-001"></a>UT-ABI-COMMAND-SMOKE-001 | C consumerをpublic headerでcompile/linkし、UI loop bridge上でCommand作成・実行・post完了・state取得・closeを行う | 配布headerとRust ABI exportsが一致し、C consumerからworker終端をUIへ一度だけ反映できる | [RQ-02-004](../11_requirements/RQ-02_core-features.md#rq-02-004)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)、[RQ-06-004](../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)、[RQ-06-008](../11_requirements/RQ-06_threading-dispatch.md#rq-06-008)<br/>[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)、[TC-ABI-COMP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-comp-01) |
| <a id="ut-command-005"></a>UT-COMMAND-005 | 進捗未定・0・1・複数sampleと終端を報告する試験を計画する | RQ-06-013に基づく専用channel oracleはDDとの不整合解消待ち。APIや範囲外値の結果をtestで創作せずblocked項目として保持 | [RQ-06-013](../11_requirements/RQ-06_threading-dispatch.md#rq-06-013)<br/>[TC-CMD-01](../30_release_test/requirement-test-matrix.md#case-tc-cmd-01) |

## 実行記録

- `UT-CORE-COMMAND-001`: `cargo test -p freemvvm-core command::tests` — 7 passed。Runtime共有operation ID issuerはwrapせず、同一Runtimeの複数Command間でIDを再利用しない。
- `UT-ABI-COMMAND-001`: `cargo test -p freemvvm-c-abi command_` — 2 passed。Command create/start failure/cancel/complete/snapshot/close、read-only Property、eligibility反映、retain/release境界を含む。
- `UT-ABI-COMMAND-SMOKE-001`: `tools/dev.py abi-command-smoke` — C11 consumerのnative link/load、`runtime_post`完了、state/can-execute、closeがpass。
- `tools/dev.py check`、`tools/dev.py minimum`、`cargo test --workspace --all-targets --locked` — pass。Windows/Linux CIは実装revision push後に確認する。

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1全文 | — | レビュー案・未承認。作成指示を試験設計の承認や試験合格とは扱わない |
| 2026-10-17 | 0.1全文 | 利用者 | 「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」により既存試験案を承認し環境整備を開始。製品test実装・合格を意味しない |
| 2026-10-24 | 0.2追加 | — | 承認済みCommand DDと基盤実装計画に基づきCore/C ABI Command試験設計を登録。実装・合格は別管理 |
| 2026-10-24 | 0.2実行記録 | — | `UNIT-CORE-066`のCore 6 tests、C ABI 2 tests、C11 consumer smoke、repository check、minimum、全Rust testsがlocal pass。Windows/Linux CI pending |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
| 0.2 | 2026-10-24 | Rust Core/C ABI Command unitの試験IDとC consumer smokeを追加 | Codex | 承認済み要求/DDおよび`.docs/20_todo/abi-async-command.md`に基づく。実装・試験結果は別管理 |
