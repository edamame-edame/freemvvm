# 具体化適用 / 旧未決記述の置換記録

- 状態: 適用前の記録（2026-10-17）
- 現行: [全件適用表](remaining-design-application-result.md)

以下は既存DD本文の未決表記を置換した監査記録で、現行仕様ではない。過去の承認履歴自体は正本に保持した。

## 1. 05-boundary-values-errors.md / 適用前対応行 8

```text
本書のDD-VAL-01～07は利用者承認済み（2026-10-07）。既承認のA2/A5～A8/A15/A18、BD-01/27、DD-01～04を変更しない。後続欄に記した未決事項は各担当設計へ送る。C宣言、擬似コード、例示statusは設計レビュー用であり、配布用ヘッダまたは実装済みAPIを意味しない。
```

置換先: [05-boundary-values-errors.md](../13_detail_design/05-boundary-values-errors.md)の具体化分冊。

## 2. 05-boundary-values-errors.md / 適用前対応行 81

```text
- nullabilityは値kindとは別の未決事項であり、handle 0または空文字列からnullを推定しない。既存A7のnullable OBJECT規則以外に一般null契約を追加しない。
```

置換先: [05-boundary-values-errors.md](../13_detail_design/05-boundary-values-errors.md)の具体化分冊。

## 3. 05-boundary-values-errors.md / 適用前対応行 87

```text
| 未決事項 | 後続 |
```

置換先: [05-boundary-values-errors.md](../13_detail_design/05-boundary-values-errors.md)の具体化分冊。

## 4. 06-abi-surface.md / 適用前対応行 58

```text
| DD-ABI-09 | `runtime_shutdown`はRuntimeがClosedになる同期操作とする。UI thread callback/event内の呼出しは実行中処理を中断せず、優先control taskとして受付後に`UI_PENDING`で戻る。workerからの呼出しはDD-03に従いUI threadの完了を待ち、完了後に`UI_OK`を返す。shutdown開始前のUI thread外の呼出しはbridgeへdispatchする。 | A4、BD-36、DD-LIFE-07/08、DD-DSP-03/05/08。停止loopの同期waitはhost停止通知で未完了として解除する。公開API/resultの具体形は未設計。 | 承認済み |
```

置換先: [06-abi-surface.md](../13_detail_design/06-abi-surface.md)の具体化分冊。

## 5. 06-abi-surface.md / 適用前対応行 131

```text
- host loopがwakeを受け入れた後に停止する競合、wakeの失敗通知、bridge交換、複数Runtimeのhost登録は未決またはDD-16へ送る。
```

置換先: [06-abi-surface.md](../13_detail_design/06-abi-surface.md)の具体化分冊。

## 6. 06-abi-surface.md / 適用前対応行 142

```text
- `UI_PENDING`を返した後に完了を待つticket/future/通知APIは追加しない。Runtime closed状態や既存の終了通知で完了を観測できるか、API上の完了確認が必要かは未決。
```

置換先: [06-abi-surface.md](../13_detail_design/06-abi-surface.md)の具体化分冊。

## 7. 06-abi-surface.md / 適用前対応行 164

```text
DD-ABI-01～10は2026-10-07に承認済みである。未決境界とDD-07以降の機能固有API、DD-16のplatform loop実装、DD-18～23のwrapper/配布設計は各フェーズで確認する。
```

置換先: [06-abi-surface.md](../13_detail_design/06-abi-surface.md)の具体化分冊。

## 8. 07-property-notification.md / 適用前対応行 8

```text
本書のDD-PROP-01～10は2026-10-08に承認済みである。承認済みの上位意味論を変更しない。C宣言は署名と境界の設計資料であり、配布用headerまたは実装済みAPIではない。未決境界は後続設計の担当として記録する。
```

置換先: [07-property-notification.md](../13_detail_design/07-property-notification.md)の具体化分冊。

## 9. 07-property-notification.md / 適用前対応行 212

```text
利用者は2026-10-08にDD-07を承認し、DD-PROP-01～10を内容承認済みとして記録した。未決境界とDD-08以降のBinding/Validation/Command意味論、wrapper固有API、C宣言の配布header化は各フェーズで確認する。 X01レビュー案は2026-10-12に承認され、DD-PROP-08の内部owner・cleanup改訂案を作成した。
```

置換先: [07-property-notification.md](../13_detail_design/07-property-notification.md)の具体化分冊。

## 10. 02-runtime-lifetime.md / 適用前対応行 121

```text
受付済みと完了済みを表すAPI上のstatus・完了観測、およびUI loop停止時の同期worker動作はBD-36の未決範囲であり、本書では決めない。DD-LIFE-07/08の承認済みshutdown状態遷移とcleanup内容は変更しない。
```

置換先: [02-runtime-lifetime.md](../13_detail_design/02-runtime-lifetime.md)の具体化分冊。

## 11. 11-ui-tree-lifetime.md / 適用前対応行 48

```text
| DD-TREE-06 | Detached subtreeの再所属は、追加検証を全て通過したときに新parentが所属参照を取得して行う。追加失敗時はDetached状態と既存のProperty値・Binding/Subscription状態を保つ。再所属時のPlatform資源再生成はVisual/Platform側の責務であり、その詳細と失敗復旧はDD-12/16へ送る。 | BD-04は再所属時のpeer再生成/再接続を要求するが、具体peer modelは未決。 | 承認済み（2026-10-11） |
```

置換先: [11-ui-tree-lifetime.md](../13_detail_design/11-ui-tree-lifetime.md)の具体化分冊。

## 12. 11-ui-tree-lifetime.md / 適用前対応行 86

```text
この案では、Logical Tree変更のcommitと、Property notification/Binding deliveryを同一のcallback再入処理として混在させない。外部通知の正確なタイミング・順序はDD-07/08の既承認規則に従う。Visual tree更新のatomicityはRenderer契約が未確定のため追加しない。
```

置換先: [11-ui-tree-lifetime.md](../13_detail_design/11-ui-tree-lifetime.md)の具体化分冊。

## 13. 03-dispatcher.md / 適用前対応行 8

```text
本書のDD-DSP-01～08は2026-10-05に利用者承認済みである。R2～R4、P1～P4、BD-03/26、DD-01/02の承認済み契約を変更しない。未決境界は§7に記し、後続設計へ送る。C宣言や擬似コードは配布用ヘッダ、実装済みAPIを意味しない。
```

置換先: [03-dispatcher.md](../13_detail_design/03-dispatcher.md)の具体化分冊。

## 14. 03-dispatcher.md / 適用前対応行 37

```text
- DD-02はshutdownを`Running → Closing → Closed`とし、shutdown後のUI操作を`UI_CLOSED`とした。callback/event中のshutdown要求は現在の処理を完了してから優先実行する。非同期呼出しは完了を待たず、同期worker呼出しはUI thread上の完了を待つ。ただしcallback/event中は同期指定でも要求受付後に戻る。statusと完了観測の詳細は未決のため、本書ではAPI表現まで確定しない。
```

置換先: [03-dispatcher.md](../13_detail_design/03-dispatcher.md)の具体化分冊。

## 15. 03-dispatcher.md / 適用前対応行 49

```text
| DD-DSP-06 | CoreはOS/hostのevent loopを所有・実行せず、Runtimeに紐づく`UiLoopBridge`を介してUI threadへ非同期wakeを依頼する。bridgeは通常workのwakeとshutdown優先taskのwakeを区別でき、後者を現在のevent終了後の次処理機会へscheduleする。wakeは任意threadから安全に呼べ、呼出し中にRuntime/user callbackやcleanupを実行しない。 | 設計骨格のPlatform adapter分離とR2を反映する抽象契約。現ABIに接続hookがないため、bridgeの生成・登録方法とC ABIの追加有無はDD-06/16で決める。bridgeを用意できないhost loopでのRuntime作成/postの扱いは本案では未確定。 |
```

置換先: [03-dispatcher.md](../13_detail_design/03-dispatcher.md)の具体化分冊。

## 16. 03-dispatcher.md / 適用前対応行 50

```text
| DD-DSP-07 | post queueに固定件数上限は設けず、資源確保に失敗したpostは`UI_OUT_OF_MEMORY`で拒否する。拒否時はcallback等の所有権を移さず、受理済みqueueを変えない。BD-26の中間進捗は別途最新値へ集約できるため、全progress sampleを個別postしない。 | BD-26はqueue容量・progress集約場所を未確定としている。上限やbackpressure statusを導入せず、ABI既存statusで割当失敗を扱う提案。 |
```

置換先: [03-dispatcher.md](../13_detail_design/03-dispatcher.md)の具体化分冊。

## 17. 03-dispatcher.md / 適用前対応行 89

```text
BD-36-01/02/04で確認済みのとおり、callback/event中のshutdown要求は現在の処理完了後に優先実行する。非同期呼出しは完了を待たず、同期worker呼出しは完了を待つ。ただしcallback/event中の同期呼出しは待機できないため受付後に戻る。停止loopではhost停止通知で同期waitを未完了として解除する。受付/完了statusはDD-06に従い、公開完了観測APIは未設計でありDD-06/16とwrapper設計へ送る。
```

置換先: [03-dispatcher.md](../13_detail_design/03-dispatcher.md)の具体化分冊。

## 18. 03-dispatcher.md / 適用前対応行 101

```text
本書の共通機構は維持し、以前API/パラメータ/実装方式を未決としていた事項は次の同層分冊へ反映した。過去の承認記録中の「未決」は記録日時点の範囲を示す。現在の具体化状態は本節と分冊本文で確認する。実装・実機・採取値は設計完了と別に追跡する。
```

置換先: [03-dispatcher.md](../13_detail_design/03-dispatcher.md)の具体化分冊。

## 19. 04-callback-lifecycle.md / 適用前対応行 50

```text
| DD-CB-08 | callback/event entry pointを識別する言語別markerを設け、診断ツールがそのメソッドと到達可能な処理を調べる案を検討する。共通化する場合も識別対象や診断分類の範囲にとどめ、解析・計測の実装能力は言語ごとに異なってよい。 | BD-03のevent callbackに対する静的警告を拡張する候補。検知ルール、構文、全言語で必須にするかは未決。 |
```

置換先: [04-callback-lifecycle.md](../13_detail_design/04-callback-lifecycle.md)の具体化分冊。

## 20. 04-callback-lifecycle.md / 適用前対応行 91

```text
| 警告 | event method内の長い同期処理、sleep、blocking wait/join、同期I/O、長い反復 | UI threadを占有・停止させる可能性。ただし現行上位設計に一律禁止規則はないためwarning候補であり、しきい値・検出範囲は未決 |
```

置換先: [04-callback-lifecycle.md](../13_detail_design/04-callback-lifecycle.md)の具体化分冊。

## 21. 04-callback-lifecycle.md / 適用前対応行 97

```text
この候補表は調査範囲を示し、個々の未決候補を禁止規則に昇格させるものではない。エラーにする規則には承認済み根拠または新しい設計判断を付け、警告には危険の程度とfalse positiveの扱いを定める必要がある。特に長時間処理・blocking・captureの検査は言語ごとの構文・解析能力が異なる。
```

置換先: [04-callback-lifecycle.md](../13_detail_design/04-callback-lifecycle.md)の具体化分冊。

## 22. 04-callback-lifecycle.md / 適用前対応行 118

```text
属性の具体構文、対象とするcallback種別、annotationを必須にするか、動的なlambda/closureや既存メソッドをどう扱うか、entry point計測とmethod-body instrumentationのどこまで必要かは未決である。C++ compiler attribute、C# attribute、Rust proc-macro、Python decorator/JITはそれぞれ異なるため、検知対象の分類は共有しつつ、各言語で実現可能な解析・計測の範囲を後続のwrapper/tooling設計で定める。
```

置換先: [04-callback-lifecycle.md](../13_detail_design/04-callback-lifecycle.md)の具体化分冊。

## 23. 04-callback-lifecycle.md / 適用前対応行 132

```text
各言語wrapperの具体的な例外/Result表現、診断payload、callback/destroy thunkのC ABI実装はDD-06およびDD-18～21で定める。Property購読とValidationの個別処理順はDD-07/09、Commandのoperation lifecycleはDD-10、TextBox input callbackはDD-14で補う。検知ルール全体と各言語のparser/analyzer/tool integrationは範囲が広いため、DD-04では候補整理までとした。DD-24「イベントメソッド制約と事前検知」は計画上の追加候補であり、未着手・未承認である。未決事項を確定仕様や実装指示として扱わない。
```

置換先: [04-callback-lifecycle.md](../13_detail_design/04-callback-lifecycle.md)の具体化分冊。

## 24. 04-callback-lifecycle.md / 適用前対応行 136

```text
本書の共通機構は維持し、以前API/パラメータ/実装方式を未決としていた事項は次の同層分冊へ反映した。過去の承認記録中の「未決」は記録日時点の範囲を示す。現在の具体化状態は本節と分冊本文で確認する。実装・実機・採取値は設計完了と別に追跡する。
```

置換先: [04-callback-lifecycle.md](../13_detail_design/04-callback-lifecycle.md)の具体化分冊。

## 25. 16-platform-rendering.md / 適用前対応行 49

```text
| DD-PLAT-02 | ひとつのRuntimeは初期構成で一つのPlatform Adapter/backendに接続する。開かれたCore Windowごとに0または1つのAdapter-owned Window peerを対応させ、peerがnative window/surfaceとgraphics resourceを保持する。Core Windowとpeerは別object/寿命とし、WindowのDetached状態だけではpeerを暗黙破棄しない。 | BD-04/11/12、DD-11/15。複数backend同時表示、Windowのscreen owner、openの公開APIは未確定。 |
```

置換先: [16-platform-rendering.md](../13_detail_design/16-platform-rendering.md)の具体化分冊。

## 26. 16-platform-rendering.md / 適用前対応行 52

```text
| DD-PLAT-05 | Window peerはCore Windowへ関連付けた状態でsurfaceを作成し、Platformから来る利用可能領域・scale等のenvironment変更をCoreへ通知する。Core Windowは意味状態を維持し、surface resize/device recreation時にLayout dirtyとframe dirtyを立てて再描画する。OS peer喪失時もCore Tree/Property/Binding/Focus意味状態をOS resourceと一緒に破棄しない。 | BD-06/12、DD-11/12/15。peer作成を明示操作とするか、Runtime/window ownerが開始するかは未決。 |
```

置換先: [16-platform-rendering.md](../13_detail_design/16-platform-rendering.md)の具体化分冊。

## 27. 16-platform-rendering.md / 適用前対応行 53

```text
| DD-PLAT-06 | OSのclose intentはPlatform AdapterからCoreへのclose requestとして通知する。OS eventを受けたことだけでRuntime shutdownやCommand/screen ownerのcloseを実行しない。Window close requestのaccept/veto/defer、Windowを再表示する意味、close後のCore Window状態は未決選択としてレビューに提示する。 | DD-15 §7、DD-02/03、BD-12。close policyをこの案だけで決めない。 |
```

置換先: [16-platform-rendering.md](../13_detail_design/16-platform-rendering.md)の具体化分冊。

## 28. 16-platform-rendering.md / 適用前対応行 54

```text
| DD-PLAT-07 | Adapterはraw pointer/keyboard/window eventをBD-07/DD-13の共通Input eventへ正規化し、Core UI threadへ順序を保って渡す。AdapterはOS座標・device情報を保持し、Core Layout座標への変換点を一箇所に置く。Coreのroute/Focus/activationはDD-13に従い、backendごとに再実装しない。 | BD-07/08、DD-12/13、DD-15。event ABI、座標単位、原点、rounding、pointer hit-test詳細は未確定。 |
```

置換先: [16-platform-rendering.md](../13_detail_design/16-platform-rendering.md)の具体化分冊。

## 29. 16-platform-rendering.md / 適用前対応行 57

```text
| DD-PLAT-10 | Runtime shutdown開始時はAdapterの新規OS event受信とframe schedulingを停止し、DD-03に従いUI thread上で未開始のworkをcancelする。AdapterはIME sessionを閉じ、native peers/graphics resourcesを破棄してからCoreがそれらの参照を解放する。既に実行中のUI callback/render turnは戻るまで完了させ、nested dispatchを始めない。 | R2～R4、BD-36、DD-02/03/04/11/14。shutdown worker待機はhost停止通知で未完了として解除する。timeout値は未設計。 |
```

置換先: [16-platform-rendering.md](../13_detail_design/16-platform-rendering.md)の具体化分冊。

## 30. 16-platform-rendering.md / 適用前対応行 84

```text
この案はCore Windowとnative peerの寿命を分ける。`CoreOnly`はCore objectが存在するがnative surfaceを持たない状態、`SurfaceReady`はOS表示面が利用可能な状態を表す。`CloseRequested`以降のaccept/defer/decline semanticsはDD-PLAT-06の未決事項であり、この図だけでWindow close contractは確定しない。Runtime shutdownは個々のclose requestとは別の全peer teardown経路とする。
```

置換先: [16-platform-rendering.md](../13_detail_design/16-platform-rendering.md)の具体化分冊。

## 31. 16-platform-rendering.md / 適用前対応行 133

```text
- worker同期shutdownはDD-03のhost停止通知で未完了と理由を返し、cleanup完了と区別する。公開停止通知API・result表現は未設計として追跡する。
```

置換先: [16-platform-rendering.md](../13_detail_design/16-platform-rendering.md)の具体化分冊。

## 32. 08-binding-graph.md / 適用前対応行 134

```text
| 循環証拠の状態記録 | BD-03は証拠のある因果連鎖だけを停止するが、任意callbackの状態履歴には有限上限が必要で、fingerprint衝突による誤停止も避ける必要があった。 | DD-BIND-10はhash索引 + 意味的snapshot照合、上限超過は判定不能とする案を承認済み。具体的な履歴上限値は未決。 |
```

置換先: [08-binding-graph.md](../13_detail_design/08-binding-graph.md)の具体化分冊。

## 33. 08-binding-graph.md / 適用前対応行 143

```text
型変換は未決の実装方式ではなく、承認済みBinding意味論が同kind接続に限定しているため、このDDの対象外である。将来parse/formatや異種kind変換を提供する場合は、先に要求・BDの変更を行い、その承認後に詳細設計へ追加する。
```

置換先: [08-binding-graph.md](../13_detail_design/08-binding-graph.md)の具体化分冊。

## 34. 10-async-command.md / 適用前対応行 9

```text
本書のDD-CMD-01～12は2026-10-11に利用者承認済みである。第9節の未決事項は本承認の対象外として残す。C1～C4、A21～A24、BD-26の承認済み意味論を具体化し、Command専用の進捗・結果値API、worker scheduler、screen ownership APIは追加しない。C宣言や状態表は設計資料であり、配布用header・実装済みAPIを意味しない。
```

置換先: [10-async-command.md](../13_detail_design/10-async-command.md)の具体化分冊。

## 35. 10-async-command.md / 適用前対応行 53

```text
| DD-CMD-03 | operation IDはRuntime内で単調に発行する非zero `uint64_t`とし、終端・過去operationを含め一度使ったIDを再利用しない。採番・operation record確保の失敗ではID出力を0、Command状態を変更せず、workerを起動しない。ID空間枯渇時は周回再利用せず新規実行を拒否する。 | A22/A23、DD-VAL-06。枯渇時statusと実現可能な採番方式は未決であり、追加statusや数値を本案で定めない。 | 承認済み（2026-10-11） |
```

置換先: [10-async-command.md](../13_detail_design/10-async-command.md)の具体化分冊。

## 36. 10-async-command.md / 適用前対応行 161

```text
| workerが完了をpost成功 | UI反映は未確定 | Coreがpost callback closureを所有し、実行かshutdown cancel時にdestroy |
```

置換先: [10-async-command.md](../13_detail_design/10-async-command.md)の具体化分冊。

## 37. 10-async-command.md / 適用前対応行 169

```text
- operation IDの最大値到達時の返却statusおよびRuntime/sessionをまたぐ一意性は未決。Runtime内でのID再利用は禁止する。
```

置換先: [10-async-command.md](../13_detail_design/10-async-command.md)の具体化分冊。

## 38. 10-async-command.md / 適用前対応行 180

```text
本書の共通機構は維持し、以前API/パラメータ/実装方式を未決としていた事項は次の同層分冊へ反映した。過去の承認記録中の「未決」は記録日時点の範囲を示す。現在の具体化状態は本節と分冊本文で確認する。実装・実機・採取値は設計完了と別に追跡する。
```

置換先: [10-async-command.md](../13_detail_design/10-async-command.md)の具体化分冊。

## 39. 09-validation-input.md / 適用前対応行 9

```text
本書のDD-VALID-01～10は2026-10-11に利用者承認済みである。第7節の未決事項は本承認の対象外として残す。V1～V8およびA17/A18・A25～A28・A30～A32の承認済み意味論を変更しない。Cの型や処理例は設計資料であり、配布用headerや実装済みAPIを意味しない。
```

置換先: [09-validation-input.md](../13_detail_design/09-validation-input.md)の具体化分冊。

## 40. 09-validation-input.md / 適用前対応行 114

```text
- TextBoxの確定値Property自体にProperty Validatorを登録した場合の、ユーザー入力Acceptからsetter拒否となった際の編集状態・reasonへの反映はABI §20.2がInput/Control詳細設計へ送っているため未決とする。
```

置換先: [09-validation-input.md](../13_detail_design/09-validation-input.md)の具体化分冊。

## 41. 09-validation-input.md / 適用前対応行 116

```text
- TextBox input subscriptionとProperty/Binding subscriberの厳密な相対順、state/error/editingの一体snapshot、表示優先順位は未決。
```

置換先: [09-validation-input.md](../13_detail_design/09-validation-input.md)の具体化分冊。

## 42. 09-validation-input.md / 適用前対応行 123

```text
DD-VALID-01～10は承認済みV/A意味論とDD-07/08の接点を具体化した。未決事項のうち上位契約と整合しない点が見つかった場合は本書だけで確定せず、該当する基本設計またはABI判断へ戻す。特にTextBox自身のProperty Validator拒否時の編集処理とIME退避失敗は、ABI §17/20およびInput/Control設計との接続確認を要する。 X01レビュー案は2026-10-12に承認され、Validator registration ownerとcleanup flowを補足する改訂案を作成した。
```

置換先: [09-validation-input.md](../13_detail_design/09-validation-input.md)の具体化分冊。

## 43. 14-textbox-ime.md / 適用前対応行 8

```text
本書は上位判断とDD-09/13をTextBox編集状態・IME sessionへ接続する詳細設計である。DD-TEXT-01～12は2026-10-12に利用者承認済み。承認済みのIME event意味論は変更しない。DD-TEXT-06/09の結果は今回の要求承認に合わせて具体化した。IME回復経路、内部位置単位、通知細部は未設計として別途追跡する。Undo/Redoおよびkey/modifierと編集操作・UI機能を結ぶ規則は本書に含めず、`.docs/20_todo/keyboard-actions-value-memo.md` の候補範囲とする。
```

置換先: [14-textbox-ime.md](../13_detail_design/14-textbox-ime.md)の具体化分冊。

## 44. 14-textbox-ime.md / 適用前対応行 32

```text
    T -.-> Q["自身Property拒否・OOM結果はRQ未決"]
```

置換先: [14-textbox-ime.md](../13_detail_design/14-textbox-ime.md)の具体化分冊。

## 45. 14-textbox-ime.md / 適用前対応行 57

```text
| DD-TEXT-11 | caret/selectionの内部位置単位を未確定として、UTF-8 byte offset、Unicode scalar boundary、extended grapheme boundaryと変換層の有無を比較する。全位置は少なくとも有効UTF-8境界を守り、A25 callbackには承認済みUTF-8 byte offsetで変換して渡す。 | A6/A25、BD-23はgrapheme単位移動・selection affinity・visual/logical policyを後続へ残す。内部単位は利用者確認を求める。 |
```

置換先: [14-textbox-ime.md](../13_detail_design/14-textbox-ime.md)の具体化分冊。

## 46. 14-textbox-ime.md / 適用前対応行 58

```text
| DD-TEXT-12 | 編集状態の一つのlogical transaction内で本文、selection/caret、composition、input state/reason、確定値Propertyを原子的に確定する。通知はDD-09のProperty/Binding順を守る。TextBox input subscriptionと他callback群の相対順、composition update通知の統合は未決とする。 | DD-VALID-08、DD-VALID §6、DD-PROP-07、DD-BIND-05。 |
```

置換先: [14-textbox-ime.md](../13_detail_design/14-textbox-ime.md)の具体化分冊。

## 47. 14-textbox-ime.md / 適用前対応行 102

```text
| Editing text | 表示・編集対象となるUTF-8全文。未確定入力を含みうる | Incomplete/Reject-preeditでは更新しても伝播しない |
```

置換先: [14-textbox-ime.md](../13_detail_design/14-textbox-ime.md)の具体化分冊。

## 48. 12-layout.md / 適用前対応行 8

```text
本書のDD-LAYOUT-01～08は2026-10-12に利用者承認済みである。第7節の未決事項は本承認の対象外として残す。Measure/ArrangeはRust CoreがVisual Tree上で実行し、Platformから与えられるroot表示領域等を入力としてboundsを確定する。公開ABIの完全なC宣言や配布用headerを定義しない。
```

置換先: [12-layout.md](../13_detail_design/12-layout.md)の具体化分冊。

## 49. 12-layout.md / 適用前対応行 41

```text
- Boundsの単位、scale/DPI変換、座標原点・変換行列、pixel roundingはBD-06の未決事項であり、本書では数値表現を定めない。
```

置換先: [12-layout.md](../13_detail_design/12-layout.md)の具体化分冊。

## 50. 12-layout.md / 適用前対応行 48

```text
| DD-LAYOUT-02 | Layout関連操作と測定結果のcommitはUI thread専用とする。各要素のサイズ制約は横・縦それぞれ最小値と最大値を持つ。寸法は非負でNaNを許さず、最大値には「上限なし」を表す意味値を許す。上下限が逆転する制約は無効入力として拒否し、既存制約・DesiredSize・FinalBoundsを変更しない。ABI上の符号化は未決のままにする。 | R2、A2、BD-06。無限値のABI表現や具体statusはABI/単位設計へ送る。 | 承認済み（2026-10-12） |
```

置換先: [12-layout.md](../13_detail_design/12-layout.md)の具体化分冊。

## 51. 12-layout.md / 適用前対応行 54

```text
| DD-LAYOUT-08 | Measure/Arrangeは決定的かつ冪等とし、入力制約、Visual構造、配置Propertyが同一なら同じDesiredSize/FinalBoundsを返す。制約の不正、破棄済みVisual、Runtime shutdownでは既存の公開結果を部分更新せず失敗する。独自callbackを呼ばないためlayout pass中の利用者コード再入は発生させない。 | DD-02/03、BD-06。浮動小数点の許容誤差・rounding・エラー診断codeは未決のまま残す。 | 承認済み（2026-10-12） |
```

置換先: [12-layout.md](../13_detail_design/12-layout.md)の具体化分冊。

## 52. 12-layout.md / 適用前対応行 86

```text
Horizontalでは横幅合計・最大高さで対称に扱う。実寸の単位、fractional値のpixel mapping、roundingはBD-06の未決事項に留める。
```

置換先: [12-layout.md](../13_detail_design/12-layout.md)の具体化分冊。

## 53. 15-controls-v1.md / 適用前対応行 8

```text
本書は初期Controlの内部状態と機能間の接続を具体化する。利用者は2026-10-12にDD-CONTROL-01～12を記載どおり承認した。これは公開API/ABIや実装済みControlを意味しない。§7に列挙した事項は未決のまま後続へ送る。承認済みTree、Layout、Property、Binding、Command、Validation、Focus、TextBox/IME意味論を再定義しない。OSへのAccessibility写像・描画・key bindingの既定は後続または別候補とする。
```

置換先: [15-controls-v1.md](../13_detail_design/15-controls-v1.md)の具体化分冊。

## 54. 15-controls-v1.md / 適用前対応行 46

```text
| DD-CONTROL-02 | 初期Control共通の意味情報として`AccessibleName`（UTF-8、空欄可、既定空）を持たせ、表示文字列やWindow titleと独立させる。表示Text/Labelからのfallbackとlabel関係は加えない。 | BD-13は明示的name設定/取得、空欄およびfallback未決を認める。OS-specific name propertyへの写像はDD-17。 |
```

置換先: [15-controls-v1.md](../13_detail_design/15-controls-v1.md)の具体化分冊。

## 55. 17-accessibility.md / 適用前対応行 8

```text
本書のDD-ACC-01～10は利用者承認済み（2026-10-12）。承認範囲は本書の判断表に記載した契約に限り、§8の未決事項を確定しない。BD-13のPlatform中立なAccessibility意味情報とLogical TreeをCoreのsource-of-truthとする境界を維持し、OS providerはその意味を各APIへ写像する。DD-15 §7の未決事項を本書で暗黙に確定しない。Windows UI AutomationとLinux AT-SPI 2は初期mapping profile候補として承認されたが、未決欄に残したAPI version・依存条件・OS固有詳細まで確定しない。擬似protocolは設計案であり、ABI宣言や実装済みAPIではない。
```

置換先: [17-accessibility.md](../13_detail_design/17-accessibility.md)の具体化分冊。

## 56. 17-accessibility.md / 適用前対応行 50

```text
| DD-ACC-06 | OSのsupported actionはCoreがそのControlで実際に扱えるsemantic actionのみ公開する。Button invokeはDD-CONTROL-12の同じactivation eligibility/Command経路へ接続し、Focus actionはDD-13 requestへ接続する。TextBox edit/value/selection actionはDD-14のediting transactionを通す。OS key eventを合成してshortcutとして実装しない。 | BD-13、DD-10/13/14/15。public TextBox selection/caret API未決のため、個別Text action setはレビュー対象・必要に応じ保留。 |
```

置換先: [17-accessibility.md](../13_detail_design/17-accessibility.md)の具体化分冊。

## 57. 17-accessibility.md / 適用前対応行 54

```text
| DD-ACC-10 | 初期mapping profile候補はWindows UI Automation server-side provider、Linux desktop profileはAT-SPI 2 Accessible/Action等のD-Bus providerとする。初期mappingは標準role/property/pattern/interfaceで表現し、独自role/event/patternは既存標準で意味を表せない場合に限り追加レビューする。 | BD-12/13と公式API資料。API version、Linux AT-SPI busの必須/任意条件、C++/Rust binding/library選択、embedded profile適用は未決。 |
```

置換先: [17-accessibility.md](../13_detail_design/17-accessibility.md)の具体化分冊。

## 58. 17-accessibility.md / 適用前対応行 63

```text
| TextBox | Edit control type、Text/Value patternの必要機能 | Entry role、Text/EditableText等の必要interface | DD-14のediting text/preedit/committed ValueのどれをOS valueとして表示するか、caret/selection APIが未決。 |
```

置換先: [17-accessibility.md](../13_detail_design/17-accessibility.md)の具体化分冊。

## 59. 17-accessibility.md / 適用前対応行 79

```text
- `enabled/focusable`はControlのCore Accessibility stateを参照する。AdapterはLogical ancestor、native peer、OS defaultから独自に再計算しない。DD-15 §7のeffective enabled/focusable未決が解決するまでは、その細則に依存するmappingを承認済みと扱わない。
```

置換先: [17-accessibility.md](../13_detail_design/17-accessibility.md)の具体化分冊。

## 60. 17-accessibility.md / 適用前対応行 86

```text
- Window: Core active/focus情報の取得。Window activation/close actionはDD-16未決境界を越えて追加しない。
```

置換先: [17-accessibility.md](../13_detail_design/17-accessibility.md)の具体化分冊。

## 61. 17-accessibility.md / 適用前対応行 87

```text
- StackPanel: child tree navigationのみ。Focusability/keyboard operationsはDD-15の未決範囲に従う。
```

置換先: [17-accessibility.md](../13_detail_design/17-accessibility.md)の具体化分冊。

## 62. 17-accessibility.md / 適用前対応行 89

```text
- TextBox: Coreが確定したtext/value/edit操作のみ。selection/caret公開詳細が未決の機能はsupported actionとして出さない。ATによる入力候補はDD-09/14 Validator transactionへ通す。
```

置換先: [17-accessibility.md](../13_detail_design/17-accessibility.md)の具体化分冊。

## 63. 18-wrapper-cpp.md / 適用前対応行 54

```text
| DD-CPP-08 | Property kindごとに明示したC++型変換を行い、範囲外整数の切り詰め、型間の暗黙変換、`std::string`のNUL終端依存を避ける。UTF-8はdata pointer+byte lengthで渡す。 | DD-VAL-01～03、DD-PROP-01～03、BD-01。F64/DECIMALの具体ABIが未確定な部分は型trait等の拡張点に留める。 |
```

置換先: [18-wrapper-cpp.md](../13_detail_design/18-wrapper-cpp.md)の具体化分冊。

## 64. 18-wrapper-cpp.md / 適用前対応行 90

```text
| Runtime | token releaseのみ。shutdownを暗黙開始しない | shutdown完了後にhandleをrelease | Runningのまま破棄された場合のdiagnostic/防止策は未決 |
```

置換先: [18-wrapper-cpp.md](../13_detail_design/18-wrapper-cpp.md)の具体化分冊。

## 65. 18-wrapper-cpp.md / 適用前対応行 102

```text
- wrapperが親Runtimeのshared pointerを保持する案は、Runtime object寿命を延ばす意味がDD-LIFE-02のCore anchorと一致しない可能性がある。採用は未決とし、Runtime C++ ownerの寿命モデルと併せて決める。
```

置換先: [18-wrapper-cpp.md](../13_detail_design/18-wrapper-cpp.md)の具体化分冊。

## 66. 18-wrapper-cpp.md / 適用前対応行 170

```text
非同期shutdownで完了待ちfutureを付加する根拠はないため、本案では追加しない。callback/event中の`UI_PENDING`後にcompletion APIが必要かはDD-ABI-09で未決のため別判断を要する。
```

置換先: [18-wrapper-cpp.md](../13_detail_design/18-wrapper-cpp.md)の具体化分冊。

## 67. 19-wrapper-csharp.md / 適用前対応行 59

```text
| DD-CS-11 | Runtime shutdownは明示APIとし、`UI_PENDING`による受付と`UI_OK`によるUI-thread完了を別の結果として表す。callback/event内のshutdownで同期待ちしない。worker同期shutdownはloop稼働時のみnative契約どおり完了を待つ。 | BD-36、DD-ABI-06/09、DD-DSP-08、DD-CPP-10。loop停止時wait解除は未決。 |
```

置換先: [19-wrapper-csharp.md](../13_detail_design/19-wrapper-csharp.md)の具体化分冊。

## 68. 19-wrapper-csharp.md / 適用前対応行 60

```text
| DD-CS-12 | C# cleanup managerは各owned tokenに対応するpreallocated cleanup ticketを持つ。Ticket確保後にだけcreate/retain/out-handle ownershipを取得する。明示DisposeまたはGC cleanupでnative releaseが失敗したとき、token所有をticketへ移し、UI thread safe pointでretryする。Runtime shutdownはUI-affine cleanup後に残存ticketをClosed shell releaseとしてdrainする。 | DD-LIFE-06/08、DD-CPP-15。Registryの同期、Runtime pinning、process shutdown順、assembly unload時の処理は未決。 |
```

置換先: [19-wrapper-csharp.md](../13_detail_design/19-wrapper-csharp.md)の具体化分冊。

## 69. 19-wrapper-csharp.md / 適用前対応行 129

```text
| Runtime Dispose | shutdownを暗黙に開始しない。未shutdown時の診断/拒否方法は未決 |
```

置換先: [19-wrapper-csharp.md](../13_detail_design/19-wrapper-csharp.md)の具体化分冊。

## 70. 19-wrapper-csharp.md / 適用前対応行 139

```text
DD-19ではloader implementation、resolver導入、NuGet package filenameを決定しない。開発時build/run、RID publish、self-contained/single-file、ARM/Yocto profileはDD-22とBD-18未決条件に残す。
```

置換先: [19-wrapper-csharp.md](../13_detail_design/19-wrapper-csharp.md)の具体化分冊。

## 71. 20-wrapper-rust.md / 適用前対応行 59

```text
| DD-RS-15 | UI-affine操作はUI threadでのみ呼ぶ。workerからの変更は値snapshotとpost closureをdispatcherへ渡す。非同期shutdownは完了を待たず受付結果で戻り、callback中要求もevent終了後に実行する。worker同期shutdownはUI loop稼働時にUI cleanup完了を待つ。 | L3/L4、BD-36、DD-DSP-08、DD-ABI-06/09、DD-CPP-10/ DD-CS-11。loop停止通知でwaitを未完了として解除する。公開完了観測APIは未設計を維持する。 | 承認済み（2026-10-12） |
```

置換先: [20-wrapper-rust.md](../13_detail_design/20-wrapper-rust.md)の具体化分冊。

## 72. 20-wrapper-rust.md / 適用前対応行 66

```text
// 説明用。実際の名前、generic構造、公開範囲は未決。
```

置換先: [20-wrapper-rust.md](../13_detail_design/20-wrapper-rust.md)の具体化分冊。

## 73. 20-wrapper-rust.md / 適用前対応行 147

```text
このregistryのlock/poison処理、Runtime pin、wake-up、library unloadまでの寿命は未決である。解放失敗tokenを捨てるfallback、Drop中にpanicするfallback、確保不能なnodeへ後から登録する実装は本案に含めない。
```

置換先: [20-wrapper-rust.md](../13_detail_design/20-wrapper-rust.md)の具体化分冊。

## 74. 21-wrapper-python.md / 適用前対応行 52

```text
| DD-PY-07 | cleanup ticket確保ができないときはRuntime/object create、retain、owned out-tokenを返すnative callを始めない。release failure時はtokenを捨てず、事前確保ticketでpending registryへ移してUI safe pointでretryする。成功後はticket、Runtime anchorの順で解放する。 | DD-LIFE-06/08、DD-VAL-06、DD-CPP-15、DD-CS-12、DD-RS-05/06。registry同期・wake・永続failureの扱いは未決。 | 利用者承認済み |
```

置換先: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)の具体化分冊。

## 75. 21-wrapper-python.md / 適用前対応行 56

```text
| DD-PY-11 | `ctypes` callback trampoline内でPython callback由来の例外をC ABI境界へ伝播させない。`Exception`/`BaseException`の捕捉範囲を明示し、callback種別に沿って診断/失敗処理へ渡す。`KeyboardInterrupt`/`SystemExit`もC callbackからPython callerへそのままunwindできない。 | R4、DD-CB-05～09、BD-14。default handling・診断payload・各callback固有の結果は未決。 | 利用者承認済み |
```

置換先: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)の具体化分冊。

## 76. 21-wrapper-python.md / 適用前対応行 65

```text
| DD-PY-20 | RuntimeはPython main/host threadの終了を検知してRuntime単位のlifecycle coordinatorへ通知し、全Runtimeへshutdown要求を送り、Python側callbackへの新規entryを止めたうえでnative UI loopを起床・終了させる経路を持つ案とする。`atexit`は通常interpreter終了時の補助signalとし、残存non-daemon UI threadがinterpreter shutdown開始を妨げる場合にも対応できるよう、それだけに依存しない。coordinator/finalizerはUI completionを同期待ちせず、shutdown taskはBD-36どおりactive event終了後に優先処理する。 | BD-36、DD-DSP-01～08、DD-CB-01～07、DD-LIFE-07/08、Python `atexit`/threading lifecycle。main/host thread終了通知の移植可能な提供元、in-flight callback barrier、Python rootをinterpreter finalization前に外す手順、異常終了時の保証限界は未決。 | 利用者承認済み |
```

置換先: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)の具体化分冊。

## 77. 21-wrapper-python.md / 適用前対応行 108

```text
- finalizerを`__del__`、`weakref.finalize`、または両者を包むmanagerのどれで実装するかは未決。callback失敗をstderr出力だけで報告しない。
```

置換先: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)の具体化分冊。

## 78. 21-wrapper-python.md / 適用前対応行 124

```text
Python callback rootをいつ安全に解放し、in-flight callbackが残る間にinterpreter teardownをどう遅らせるかは重要な未決事項である。Python側rootを解放するsafe pointとnative側callback gate/barrierを協調させる必要がある。user callbackがmain thread上の処理を同期待ちする場合など、終了hookからの単純なjoin/waitはdeadlockし得るため、完了待ち方式を本案では決めない。`atexit`は通常終了時の補助signal候補であり、fatal error、未処理signalによるkill、`os._exit()`等までのcleanup保証を意味しない。
```

置換先: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)の具体化分冊。

## 79. 21-wrapper-python.md / 適用前対応行 145

```text
destroy thunkはuserdata keyをregistryからatomicにdetachし、新しいcallback invocationを拒む。その後、Python callable/CFUNCTYPEへの最終参照をC destroy stackやregistry lockの中で落とさず、wrapper safe-point queueへ渡す。任意Python objectの`__del__`やweakref callbackがreference dropに伴って実行され得るため、Core destroy thunk内のPython object destructionを避ける。queueをdrainするsafe pointのhost event-loop接続とinterpreter終了前drainは未決である。
```

置換先: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)の具体化分冊。

## 80. 21-wrapper-python.md / 適用前対応行 180

```text
実行時guardは、method metadataで指定されたoverrideだけをclass creation時のhook/metaclass/decorator候補で包み、guard状態を呼出しごとに持つ構成が考えられる。要求された親実装側がそのguardへchain-up到達を記録し、override終了時に未到達なら分かりやすい例外を出す。検査対象は`super()`という構文文字列ではなく、契約上の親実装呼出しである。Python演算子構文が直接special methodを呼ぶ場合も、class creation時に対象methodをwrapperで包めれば実行時に検査できる。dynamic class変更や検査外の継承ではruntime検査を保証せず、BD-15 analyzerのdiagnosticまたはunknownへ送る。missing chain-up exceptionのmethod一覧、severity、継承先の抑制方法、対象operatorは未決である。
```

置換先: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)の具体化分冊。

## 81. 21-wrapper-python.md / 適用前対応行 231

```text
図の`CallbackGateClosed`から`InterpreterFinalized`までにin-flight callback/rootを安全にdrainする具体handshakeは未決である。終了中の同期待ちがmain threadとUI/callback間の依存を作らない条件もレビュー対象とする。
```

置換先: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)の具体化分冊。

## 82. 21-wrapper-python.md / 適用前対応行 248

```text
| Python main/host thread終了時にUI threadが生存 | lifecycle coordinatorがcallback gateを閉じてnative priority shutdownをsignalし、UI loopを停止する。検知方式とcallback/root drain handshakeは未決 |
```

置換先: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)の具体化分冊。

## 83. 21-wrapper-python.md / 適用前対応行 251

```text
| Python main threadの未捕捉例外 | `sys.excepthook`からBD-36/DD-03のthread/context dispatchでshutdownを要求し、保存した既存hookでtracebackを報告する。hook差し替え競合・既存hookとの合成は未決 |
```

置換先: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)の具体化分冊。

## 84. 22-build-distribution.md / 適用前対応行 39

```text
| Build profile | host/target、sysroot/toolchain、target profile、ABI major、feature setを指定する | Build設定とprofile記録。exact値は未決 |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 85. 22-build-distribution.md / 適用前対応行 42

```text
| Canonical catalog | 公式manifestとimmutable identityの権威ある保管・取得点になる | service/URL/APIは未決 |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 86. 22-build-distribution.md / 適用前対応行 52

```text
| DD-BUILD-02 | build profileはbuild hostとtargetを独立して明示し、少なくともtarget OS/architecture/triple、board/toolchain profile ID、sysroot identity、ABI major、要求featureを持つ。ABI・linkに影響するcompiler/linker/sysroot/CPU featureの値はprofileに束縛し、host machineからtargetを推測しない。 | BD-16、BD-20、BD-29/30。compiler/Rust/CMake/SDK/sysrootのexact version、CPU/FPU/float ABI、libc baseline、board名は未決。 | 利用者承認済み |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 87. 22-build-distribution.md / 適用前対応行 54

```text
| DD-BUILD-04 | 初期target順はWindows x64 native build/smoke、Linux x64 native build/test、Linux x64 hostからARMv7/AArch64 cross-build、選定した一つのARM実機profile gate、その後にWindows ARM64/macOS等とする。未検証targetへのfallbackやsupport表示拡張をしない。 | BD-12/16/22。実機board、exact compiler/sysroot、CI provider/runner、正式release matrixは未決。 | 利用者承認済み |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 88. 22-build-distribution.md / 適用前対応行 56

```text
| DD-BUILD-06 | 組み込みLinux ARMv7/AArch64 native artifactはcross-build対象にするが、libc・sysroot・GPU/input依存とprofile compatibilityを確かめるまでdesktop Linux向けmanylinux/musllinux wheelまたは一般NuGet RIDとして扱わない。互換性がないprofileはYocto/system packageまたはtarget-specific archiveなどの別導入経路とし、形式選定は後続判断とする。 | BD-16/17/18/19。正式なARM package形式、board-specific recipe、wheel/RID適合判定は未決。 | 利用者承認済み |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 89. 22-build-distribution.md / 適用前対応行 57

```text
| DD-BUILD-07 | native artifactのcanonical identityは一つの公式catalogをsource of truthにする。language packageへ同梱・参照するbinaryは同じmanifest identity/digestを保つ。selectorはbuild/runtime targetと明示profileから決め、host/target混同や推測fallbackを拒否する。 | BD-29/30。catalog service/URL/API、package managerごとの取得pluginは未決。 | 利用者承認済み |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 90. 22-build-distribution.md / 適用前対応行 58

```text
| DD-BUILD-08 | manifestはruntime versionとは独立したschema versionを持つ。payloadにはruntime version、ABI major、OS/architecture/target triple、profile/compatibility、feature、filename、byte size、artifact SHA-256 digest、immutable release identity、schema version、signing key identityを含める。 | BD-28～32。完全なfield名・型・critical/optional表現は未決。 | 利用者承認済み |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 91. 22-build-distribution.md / 適用前対応行 59

```text
| DD-BUILD-09 | manifest payloadはUTF-8 JSONのRFC 8785 JCS canonical formを候補とし、manifest payload内すべての意味情報（signature containerを除く）をEd25519 signatureで束縛する候補とする。artifact file digestにはSHA-256を候補とする。duplicate keys、不正Unicode、範囲外数値、曖昧なparseは拒否し、large integer-like valuesは文字列表現にする。algorithm/canonicalization/schema test vectorsは利用者承認前に確定しない。 | BD-29/31/32、RFC 8785、RFC 8032、NIST FIPS 180-4。RFC 8785はInformational RFC。algorithm approval、key encoding、signature envelope、field wire schemaは未決。 | 利用者承認済み |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 92. 22-build-distribution.md / 適用前対応行 60

```text
| DD-BUILD-10 | manifest readerは署名を検証した上でmanifest全体を一意にparseする。unknown schema major、duplicate/ambiguous field、unknown critical/required field/feature、署名対象の不一致はfail closed。same-majorのunknown optional fieldは署名検証後に限り無視可能とし、target/ABI/trust/security判定に影響する情報をoptionalにしない。 | BD-29/31/32。readerの対応schema major数、resource size/depth limitは未決。 | 利用者承認済み |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 93. 22-build-distribution.md / 適用前対応行 61

```text
| DD-BUILD-11 | artifact取得・import・package assemblyでは、runtime versionとdigestをlock可能な値として記録し、cache keyにもimmutable identityとdigestを含める。catalog fetchは明示操作とし、signature/trust/expiry/revocation、digest、target/profile、ABI major、required featureを検証してからlink/loadに渡す。署名・信頼がunknown/revoked/invalid、target mismatch、metadata解析不能ならfail closed。offline時は運用者が別経路から信頼したpublic trust bundleと失効情報を使う。 | BD-29～32。trust rootはcatalogと独立してprovisionする。key custody、rotation時期、revocation propagation SLA、stale offline metadataの期限と詳細responseは未決。 | 利用者承認済み |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 94. 22-build-distribution.md / 適用前対応行 126

```text
| offlineでtrust/revocation metadataがない、またはstale | 未決の有効期限policyに従う。未知keyや署名検証不能を自動受入れしない |
```

置換先: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)の具体化分冊。

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 1.0 / 94箇所の旧未決表記の記録 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 1.0 | 2026-10-17 | 94箇所の旧未決表記の記録を正本へ適用し、旧判断との対応を保持 | Codex |
