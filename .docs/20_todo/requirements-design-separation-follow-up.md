# 要求分離後の具体化課題と移管記録

- 文書版: 1.2
- 状態: 分離・未設計レビュー全件の正本適用済み。実装・運用採取は未完了

## 全件適用の入口

[適用結果と全200判断の正式ID](remaining-design-application-result.md)を参照する。要求280件、方式169判断を正本へ反映した。元の394箇所は適用前の走査記録として保持し、現在の未設計件数として数えない。

## 具体化先

| 範囲 | 正本 | 状態 |
| --- | --- | --- |
| F32の提供経路・数値変換 | [DD-24](../13_detail_design/24-numeric-conversion.md) | 方式承認・適用済み。実装/実行は未完了 |
| Collection・Selectionの所有、batch、snapshot | [DD-25](../13_detail_design/25-collection-selection.md) | 方式承認・適用済み。実装/実行は未完了 |
| Host境界・shutdown完了観測・cleanup drain | [DD-26](../13_detail_design/26-host-shutdown-api.md) | 方式承認・適用済み。実装/実行は未完了 |
| 通知・非収束・Command・エラーcatalog | [DD-27](../13_detail_design/27-event-error-catalog.md) | 方式承認・適用済み。実装/実行は未完了 |
| 編集API・Unicode位置・IME回復 | [DD-28](../13_detail_design/28-text-edit-api.md) | 方式承認・適用済み。実装/実行は未完了 |
| Layout・座標・Input route・Tree所属 | [DD-29](../13_detail_design/29-geometry-input-tree-api.md) | 方式承認・適用済み。実装/実行は未完了 |
| Control API・Window close・描画backend | [DD-30](../13_detail_design/30-control-surface-renderer.md) | 方式承認・適用済み。実装/実行は未完了 |
| Text engine・IME OS接続・Accessibility | [DD-31](../13_detail_design/31-os-text-accessibility.md) | 方式承認・適用済み。実装/実行は未完了 |
| C++公開型・handler・cleanup同期 | [DD-32](../13_detail_design/32-cpp-api-ownership.md) | 方式承認・適用済み。実装/実行は未完了 |
| C# token owner・interop・Dispose・load | [DD-33](../13_detail_design/33-csharp-api-ownership.md) | 方式承認・適用済み。実装/実行は未完了 |
| Rust公開型・receiver・registry・panic | [DD-34](../13_detail_design/34-rust-api-ownership.md) | 方式承認・適用済み。実装/実行は未完了 |
| Python API・通常終了handshake・特殊method | [DD-35](../13_detail_design/35-python-lifecycle-api.md) | 方式承認・適用済み。実装/実行は未完了 |
| callback識別・禁止検知・lint/MCP | [DD-36](../13_detail_design/36-callback-analyzer.md) | 方式承認・適用済み。実装/実行は未完了 |
| 署名envelope・schema・parser | [SD-06](../14_secure_design/06-manifest-wire-verifier.md) | 方式承認・適用済み。実装/実行は未完了 |
| 鍵管理・失効・offline・staging | [SD-07](../14_secure_design/07-trust-key-staging.md) | 方式承認・適用済み。実装/実行は未完了 |
| 最低版・target matrix・toolchain lock・保守 | [DD-37](../13_detail_design/37-build-support-profiles.md) | 方式承認・適用済み。実装/実行は未完了 |
| package名・loader・artifact取得・catalog | [DD-38](../13_detail_design/38-package-loader-tool.md) | 方式承認・適用済み。実装/実行は未完了 |
| 実装unit台帳・test ID・CI・scenario | [UT-INVENTORY](../32_unit_test/implementation-test-inventory.md) | 方式承認・適用済み。実装/実行は未完了 |
| 次段階UI・将来候補の範囲と拡張点 | [DD-39](../13_detail_design/39-next-stage-ui.md) | 方式承認・適用済み。実装/実行は未完了 |
| 共通API表・feature・status優先順・互換移行 | [DD-40](../13_detail_design/40-api-inventory.md) | 方式承認・適用済み。実装/実行は未完了 |

SDK/鍵/実機/担当者/action pin/実test構成の取得は[実環境入力](remaining-design-review/external-inputs.md)に従う。将来候補と長期airgapの別profileは、初期必須へ追加せず個別の採用・条件承認を経る。

## 元文書から移管した検討・進行記録

以下は適用前の計画・未決記録であり、今回解決した選択肢は上表と現行RQを優先する。

### BD-01_value-types-upper-design.md / CollectionとGridの設計の進め方（BD-33承認済み）

- 元文書: [BD-01_value-types-upper-design.md](../12_design/BD-01_value-types-upper-design.md)

Gridの全セルをUI ControlやPropertyとして常時生成する方式を必須としない。推奨する分割は次のとおり。BD-01はCollection ABIやGridの性能保証を決定していない。

1. CoreのCollectionModelは行数、安定した行ID、必要な行・列の値の取得、挿入・削除・移動・変更の差分通知を提供する。大量の行をC ABIの一セル一往復で取得させないため、可視範囲のまとめ取得を検討する。
2. Gridは可視行と周辺だけを生成・再利用し、スクロールで表示を差し替える。表示セルごとに永久のTwoWay Bindingを張らず、表示用snapshotを使う。編集中のセルだけ編集状態とValidatorを保持し、確定時にModelへ書き戻す。
3. 行IDと列IDで選択と編集中の対象を追跡する。並べ替え・フィルター・削除・非同期差分によって対象が消えたときの確定失敗、入力保持、エラー表示を規定する。workerからの差分はUI threadへ投稿する。
4. 4言語から同じModelを作れる高水準adapterを用意し、利用者が差分通知やControl再利用を毎回手作業で実装しなくてよいようにする。小規模配列から始められる既定adapterを検討する。

この方向なら、利用者はModelのデータと更新方法を指定し、Grid内部の可視セル生成・再利用はライブラリが担当できる。BD-33はCollection/Selectionを要求仕様第4節のCore機能として維持し、値型・Widget treeから独立したCore object、stable item/column ID、差分通知、可視cell virtualizationの上位境界を承認した。項目schema、具体API、Grid編集規則、性能目標は後続判断とする。

### BD-01_value-types-upper-design.md / 承認結果と後続

- 元文書: [BD-01_value-types-upper-design.md](../12_design/BD-01_value-types-upper-design.md)

BD-01は2026-09-26に承認された。初期公開Property値型をBOOL/I64/UTF8/F64/独自8バイトDECIMALへ拡張し、F64のNaN・±0、DECIMALの5ビットscaleと範囲拒否、View入力とModel検証の関係を上記のとおり定める。A5の初期種類数をこの判断で改訂する。カスタム、一般的なnull許容値、OBJECTは後続、Collectionは別Core機能とする。Grid/Collectionの具体案、十進型のABI署名・演算・converter APIは今後レビューする。

### BD-04_ui-tree-lifetime-upper-design.md / 判断範囲と後続項目

- 元文書: [BD-04_ui-tree-lifetime-upper-design.md](../12_design/BD-04_ui-tree-lifetime-upper-design.md)

- 本判断はtree所属と寿命の上位ルールであり、Logical TreeとVisual Treeの役割、Control templateが作る要素、ツリー変更API、layout・描画・hit testへの反映を確定しない。
- Detached状態のAPIごとの可否、Property値の保持範囲、再所属時の状態復元、native resource再生成の失敗処理は詳細設計に送る。
- 親子リンクの追加・削除・付け替えの具体API、失敗時の原子性、Window close時の通知順、detach時のfocus/IME cleanupは各後続判断に送る。
- A29～A32のTextBox生成・値接続・外部確定値の扱い・破棄規則を変更しない。TextBox固有のProperty寿命を一般のtree ownershipへ広げない。
- Treeの所有参照はRuntime契約の「循環する内部参照を作らない」に従う。Logical/Visual関係を混在させたときの全体参照グラフと、template由来要素の所有者は詳細なtree設計で明示する。

### BD-07_input-event-boundary-upper-design.md / 判断範囲と後続項目

- 元文書: [BD-07_input-event-boundary-upper-design.md](../12_design/BD-07_input-event-boundary-upper-design.md)

- 本判断はraw inputから共通Core Inputへの入口を定める。pointer target選択、keyboard focus、capture、event bubbling／tunneling、default action、gesture、keyboard navigation、IME compositionの状態遷移は定めない。
- どの種類の入力eventを初期公開するか、eventの順序・時刻・キャンセル、native controlとCoreの編集状態同期、Platformごとの対応機能は後続のInput/Focus/IME判断で扱う。
- callbackを呼ぶスレッドはR2～R4を維持する。workerからUI objectを直接操作しない規則、長時間処理は非同期へ委譲する規則も変更しない。

### BD-08_focus-upper-design.md / 判断範囲と後続項目

- 元文書: [BD-08_focus-upper-design.md](../12_design/BD-08_focus-upper-design.md)

- Focusを移せるControlの条件（focusable、visible、enabled等）、Tab順、keyboard navigation、FocusScope、window deactivation時の復帰候補保持・解除は別途定める。
- FocusChanged callback/eventの公開、FocusLost／FocusGainedの順序、拒否可能なFocus移動、detach時の再Focus先は詳細設計に送る。
- IME compositionがFocus移動中にどう確定・取消されるか、TextBoxのselection/caret、A25～A28のValidator callbackとの順序はIME／TextBox設計で定める。
- Accessibility API上のfocus/state写像はAccessibility判断で扱う。
- 本案はRuntimeごとに一つのactive keyboard-focus domainを仮定する。複数seatから独立入力を同時に扱う要件が出た場合は、active focusをseatごとに持つ拡張を別判断する。

### BD-36_runtime-shutdown-dispatch-upper-design.md / 4. DDフェーズへの反映

- 元文書: [BD-36_runtime-shutdown-dispatch-upper-design.md](../12_design/BD-36_runtime-shutdown-dispatch-upper-design.md)

- DD-02 DD-LIFE-07は、通常shutdownの状態遷移とcleanupを定める。callback内要求の優先dispatchは本判断に従い、戻り値/完了観測はDD-06とwrapper設計で決める。
- DD-03は、shutdown requestを通常workより優先して現在のevent直後に実行するdispatcher契約とdispatch context検出を詳細化する。
- DD-04は、現在実行中callbackの終了後にclosure destroyをUI threadで一度だけ行う順序を詳細化する。
- DD-06/16およびwrapper各文書は、wait、pending/accepted status、完了通知のABI・言語別表現を同じ契約に合わせる。

### 05-boundary-values-errors.md / 6. 未決境界と後続担当

- 元文書: [05-boundary-values-errors.md](../13_detail_design/05-boundary-values-errors.md)

| 未決事項 | 後続 |
|---|---|
| API全体のstatus優先順位、ABI symbolと正式なstatus/domain/code数値catalog | DD-06 |
| F64/DECIMAL kind-specific Property create/get/set/Validator署名、DECIMAL固定8-byteのABI符号化 | DD-07、その後DD-18～21 |
| Property setter・Binding・Validationのcommit/notification/error状態 | DD-07～09 |
| Command result/error snapshotとoperation statusの具体API | DD-10 |
| 配列・Collection data、batch、snapshot consistency | Collection専用設計と上位判断 |
| C++/C#/Rust/Pythonの具体的なerror型、例外/Result変換、文字列runtime表現 | DD-18～21 |
| cause取得、diagnostic payload/size/depth、message localization | DD-06または対象機能設計 |

### 11-ui-tree-lifetime.md / 7. 未決事項と後続設計

- 元文書: [11-ui-tree-lifetime.md](../13_detail_design/11-ui-tree-lifetime.md)

- 公開C ABIの正確なchild add/remove/get/count symbol、child index型、iterator/snapshot API: DD-15およびABI設計。レビュー案の意味論承認だけでsymbolは確定しない。
- 親子の付け替えはatomic reparent APIを設けず、明示removeの後に追加する（DD-TREE-03承認済み）。追加に失敗した場合はDetached状態を維持する（DD-TREE-06承認済み）。
- Detached Widgetで許す各APIの一覧とProperty値保持範囲: DD-15およびwrapper設計。
- Property/Binding/SubscriptionがWidget/Control内部Propertyを所有する具体グラフと、Widgetの公開handle release後にそれらを保持するowner: DD-15。A29～A32を優先する。
- Logical変更からVisual tree更新へ反映する時点、描画resource再生成の失敗・rollback: DD-12/16、Renderer基本設計。
- detach/Window close時のfocus、capture、IME composition cleanupとevent routing: DD-13/14およびInput/Focus/IME基本設計。
- template-generated Visual elementのowner、replace/destroy順: Template/Control設計。
- shutdown時の全Logical/Visual object処理順: DD-02/12/16の整合レビュー。

### 11-ui-tree-lifetime.md / 7. 未決事項と後続設計

- 元文書: [11-ui-tree-lifetime.md](../13_detail_design/11-ui-tree-lifetime.md)

- 公開C ABIの正確なchild add/remove/get/count symbol、child index型、iterator/snapshot API: DD-15およびABI設計。レビュー案の意味論承認だけでsymbolは確定しない。
- 親子の付け替えはatomic reparent APIを設けず、明示removeの後に追加する（DD-TREE-03承認済み）。追加に失敗した場合はDetached状態を維持する（DD-TREE-06承認済み）。
- Detached Widgetで許す各APIの一覧とProperty値保持範囲: DD-15およびwrapper設計。
- Property/Binding/SubscriptionがWidget/Control内部Propertyを所有する具体グラフと、Widgetの公開handle release後にそれらを保持するowner: DD-15。A29～A32を優先する。
- Logical変更からVisual tree更新へ反映する時点、描画resource再生成の失敗・rollback: DD-12/16、Renderer基本設計。
- detach/Window close時のfocus、capture、IME composition cleanupとevent routing: DD-13/14およびInput/Focus/IME基本設計。
- template-generated Visual elementのowner、replace/destroy順: Template/Control設計。
- shutdown時の全Logical/Visual object処理順: DD-02/12/16の整合レビュー。

### 12-layout.md / 7. 未決事項と後続設計

- 元文書: [12-layout.md](../13_detail_design/12-layout.md)

- Layout寸法の単位、scale/DPI変換、原点、座標変換行列、pixel rounding: BD-06が後続判断へ送る。DD-LAYOUT-01～08の承認では数値表現を決めていない。
- NaN、infinity、overflow、許容誤差、比較閾値のABI表現・error status: 単位/値型/error catalog設計。DD-LAYOUT-02の制約意味論承認は表現を確定しない。
- Text/TextBox/Button等のControl別Sizer、font metrics、content padding、minimum size: DD-15とText/Renderer設計。
- Margin、padding、spacing、均等Stretch、最大制約を越えるDesiredSize、overflow/clipping: DD-15/後続Layout判断。
- Window/rootへavailable sizeを渡す具体Platform hookとscale連携、layout invalidationから描画frameまでの接続: DD-16。
- Hit-testがbounds、clip、transformを使う細則: DD-13/16。
- 頻繁なProperty更新時のlayout coalescing budget/priority、large tree incremental algorithm: 性能検証後の後続設計。
- 独自Sizer/Locationer登録拡張API: 上位設計で再判断する場合のみ対象にする。

### 12-layout.md / 7. 未決事項と後続設計

- 元文書: [12-layout.md](../13_detail_design/12-layout.md)

- Layout寸法の単位、scale/DPI変換、原点、座標変換行列、pixel rounding: BD-06が後続判断へ送る。DD-LAYOUT-01～08の承認では数値表現を決めていない。
- NaN、infinity、overflow、許容誤差、比較閾値のABI表現・error status: 単位/値型/error catalog設計。DD-LAYOUT-02の制約意味論承認は表現を確定しない。
- Text/TextBox/Button等のControl別Sizer、font metrics、content padding、minimum size: DD-15とText/Renderer設計。
- Margin、padding、spacing、均等Stretch、最大制約を越えるDesiredSize、overflow/clipping: DD-15/後続Layout判断。
- Window/rootへavailable sizeを渡す具体Platform hookとscale連携、layout invalidationから描画frameまでの接続: DD-16。
- Hit-testがbounds、clip、transformを使う細則: DD-13/16。
- 頻繁なProperty更新時のlayout coalescing budget/priority、large tree incremental algorithm: 性能検証後の後続設計。
- 独自Sizer/Locationer登録拡張API: 上位設計で再判断する場合のみ対象にする。

### 13-input-focus.md / 8. 未決事項と後続設計

- 元文書: [13-input-focus.md](../13_detail_design/13-input-focus.md)

- Focus traversal、shortcut/action binding（例: Tab/Shift+Tab、Ctrl+Z）、key mapの既定値・変更方法: 本詳細設計の対象外。付加価値候補として `.docs/20_todo/keyboard-actions-value-memo.md` に記録。
- visibility/enabled/focusableの完全なControl model: DD-15。
- FocusLost/Gainedの公開event/callback、正確なABI symbol、event handled/cancel/default-action契約: DD-15とABI設計。
- pointer hit-test結果をFocus requestへ結ぶ順序、capture、gesture、座標transform: DD-16。
- Platformごとのraw key identity、modifier/IME/text event順、activationとnative focus同期のsequence/suppression: DD-16。
- TextBox key editing、selection/caret、Undo/Redo、明示IME cancel、composition詳細: DD-14。
- Accessibility focus/state写像: DD-17。
- multi-seat/focus seatごとの状態: 上位要件追加後に判断。

### 13-input-focus.md / 8. 未決事項と後続設計

- 元文書: [13-input-focus.md](../13_detail_design/13-input-focus.md)

- Focus traversal、shortcut/action binding（例: Tab/Shift+Tab、Ctrl+Z）、key mapの既定値・変更方法: 本詳細設計の対象外。付加価値候補として `.docs/20_todo/keyboard-actions-value-memo.md` に記録。
- visibility/enabled/focusableの完全なControl model: DD-15。
- FocusLost/Gainedの公開event/callback、正確なABI symbol、event handled/cancel/default-action契約: DD-15とABI設計。
- pointer hit-test結果をFocus requestへ結ぶ順序、capture、gesture、座標transform: DD-16。
- Platformごとのraw key identity、modifier/IME/text event順、activationとnative focus同期のsequence/suppression: DD-16。
- TextBox key editing、selection/caret、Undo/Redo、明示IME cancel、composition詳細: DD-14。
- Accessibility focus/state写像: DD-17。
- multi-seat/focus seatごとの状態: 上位要件追加後に判断。

### 14-textbox-ime.md / 7. 未決事項と後続境界

- 元文書: [14-textbox-ime.md](../13_detail_design/14-textbox-ime.md)

- Unicode internal offset単位、grapheme navigation、selection affinity、logical/visual arrow policy（BD-23）。本書は候補比較まで。
- TextBox確定Property自身のValidator拒否時の入力状態、IME candidate退避失敗時の回復/status（DD-09 §7、ABI §20.2）。
- TextBox編集操作APIの全一覧、公開caret/selection API、ABI symbol、各言語wrapper signature（DD-15、DD-18～21）。
- キーとcaret movement/selection操作のbinding、Tab traversal、Ctrl+Z/Undo/Redo shortcut、その他action binding。本書では定めない。候補: `.docs/20_todo/keyboard-actions-value-memo.md`。
- Composition attributes/複数segmentの見た目、候補windowの選択・paging・caret座標配置、font shaping/rendering（DD-16、DD-15）。
- OS固有IME API、session ID生成器、IMEごとのevent ordering差とretry可能性（DD-16）。
- accessibility経由の編集操作の共通意味論（DD-17）。

### 14-textbox-ime.md / 7. 未決事項と後続境界

- 元文書: [14-textbox-ime.md](../13_detail_design/14-textbox-ime.md)

- Unicode internal offset単位、grapheme navigation、selection affinity、logical/visual arrow policy（BD-23）。本書は候補比較まで。
- TextBox確定Property自身のValidator拒否時の入力状態、IME candidate退避失敗時の回復/status（DD-09 §7、ABI §20.2）。
- TextBox編集操作APIの全一覧、公開caret/selection API、ABI symbol、各言語wrapper signature（DD-15、DD-18～21）。
- キーとcaret movement/selection操作のbinding、Tab traversal、Ctrl+Z/Undo/Redo shortcut、その他action binding。本書では定めない。候補: `.docs/20_todo/keyboard-actions-value-memo.md`。
- Composition attributes/複数segmentの見た目、候補windowの選択・paging・caret座標配置、font shaping/rendering（DD-16、DD-15）。
- OS固有IME API、session ID生成器、IMEごとのevent ordering差とretry可能性（DD-16）。
- accessibility経由の編集操作の共通意味論（DD-17）。

### 15-controls-v1.md / 5. 縦断シナリオ案

- 元文書: [15-controls-v1.md](../13_detail_design/15-controls-v1.md)

1. RuntimeにWindowを作り、StackPanelを一つのcontent rootとして所属させる。
2. StackPanelにTextBox、Text、Buttonを順序付きで追加する。
3. ViewModelのUTF-8 `Name`をTextBoxの確定`Value`へTwoWay、`Message`をTextの`Content`へOneWayで接続する。
4. TextBoxで入力candidateをDD-09/14経路に通し、Acceptのみ確定Property/Bindingへ伝播する。IME preeditは未確定編集状態に留まる。
5. Button activationでassigned Commandがありかつeffective enabledならDD-10 executeを一度依頼し、実行中はeffective executable=falseとなる。
6. Window/StackPanel/leaf controlsからAccessibleName、role相当、state/valueを共通Core意味情報として得る。具体OS bridgeはテスト対象にしない。

このシナリオはcontrol semanticsのレビュー例であり、具体コンストラクタ名、ABI symbol、UI source-code DSL、keyboard shortcutを定めない。

### 15-controls-v1.md / 7. 未決事項と後続設計

- 元文書: [15-controls-v1.md](../13_detail_design/15-controls-v1.md)

- Common `IsEnabled`のancestor effective semantics、Control別Focusable既定値、Focusable Containerの操作: 利用者確認対象。本判断でTab traversalは決めない。
- visibility propertyが初期Controlに必要か、その表示/measure/focus/accessibility意味: 要求追加または別判断まで保留。
- Window title/close event/native decoration、Window surface open/closeとRuntime/Command screen ownerの接続: DD-16およびlifetime/wrapper設計。
- Button label property名とText/AccessibleNameのfallback、Command associationのAPI/Binder surface、Command strong-referenceとclose owner。特に複数Button共有時に一Button破棄がCommandをcloseしないことを確認。
- Text overflow/wrapping/selection、TextBox public selection/caret API、button visual states/style/theme: Renderer/Control follow-up。
- OS Accessibility mapping・event payload・AT実機検証: DD-17。C ABI symbolsと4言語の具体型: DD-18～21。
- Tab/Shift+Tab、Ctrl+Z、その他key/action bindingsとUndo/Redo: `.docs/20_todo/keyboard-actions-value-memo.md`。本承認案に含めない。

### 15-controls-v1.md / 7. 未決事項と後続設計

- 元文書: [15-controls-v1.md](../13_detail_design/15-controls-v1.md)

- Common `IsEnabled`のancestor effective semantics、Control別Focusable既定値、Focusable Containerの操作: 利用者確認対象。本判断でTab traversalは決めない。
- visibility propertyが初期Controlに必要か、その表示/measure/focus/accessibility意味: 要求追加または別判断まで保留。
- Window title/close event/native decoration、Window surface open/closeとRuntime/Command screen ownerの接続: DD-16およびlifetime/wrapper設計。
- Button label property名とText/AccessibleNameのfallback、Command associationのAPI/Binder surface、Command strong-referenceとclose owner。特に複数Button共有時に一Button破棄がCommandをcloseしないことを確認。
- Text overflow/wrapping/selection、TextBox public selection/caret API、button visual states/style/theme: Renderer/Control follow-up。
- OS Accessibility mapping・event payload・AT実機検証: DD-17。C ABI symbolsと4言語の具体型: DD-18～21。
- Tab/Shift+Tab、Ctrl+Z、その他key/action bindingsとUndo/Redo: `.docs/20_todo/keyboard-actions-value-memo.md`。本承認案に含めない。

### 16-platform-rendering.md / 7. 未決事項と後続設計

- 元文書: [16-platform-rendering.md](../13_detail_design/16-platform-rendering.md)

- **利用者判断が必要**: Window close intentをclose event/requestとしてCoreへ渡した後、Core側にaccept/decline/deferを持たせるか、OS closeを即時受理するか。Windowを閉じた後にCore Windowを保持して再openできるか。DD-PLAT-06に案を提示した。
- Window open/closeを誰が要求・所有するか、Runtime/Command screen ownerとの接続、複数Window/複数backend support（DD-15 §7、BD-12）。
- DD-03 `UiLoopBridge`の登録API、wakeup failure before/after queue acceptance、host loop停止時の同期worker shutdown timeout/cancel、status/完了観測（DD-03 §7、DD-06/18～21）。
- 座標の数値単位、原点、DPI/scale変換、transform、pixel rounding、unbounded layout size encoding（BD-06、DD-12）。
- Renderer API/scene representation、damage region/partial redraw、frame clock/vsync、GPU thread/context/fence、device lost時のfallbackとresource recreation（BD-12、backend follow-up）。
- Window title/native decoration、具体Windows/Linux windowing・graphics/IME API/library・driver/versionと対応機能差（BD-12、Platform profile）。
- Pointer hit-test/clip/transformとnative/Core focus event ordering、OS-specific event timestamp/capture semantics（DD-13）。
- IME candidate window anchor、font/text shaping/Unicode位置変換、composition rendering detail/session ID width/API（BD-23/24、DD-14）。
- OS Accessibility peer/event/action mappingとAT実機検証（DD-17、BD-13）。
- backend capability queryと失敗status catalog・retry policy、formal target/CI/release gate（DD-06/22/23、BD-12）。

### 16-platform-rendering.md / 7. 未決事項と後続設計

- 元文書: [16-platform-rendering.md](../13_detail_design/16-platform-rendering.md)

- **利用者判断が必要**: Window close intentをclose event/requestとしてCoreへ渡した後、Core側にaccept/decline/deferを持たせるか、OS closeを即時受理するか。Windowを閉じた後にCore Windowを保持して再openできるか。DD-PLAT-06に案を提示した。
- Window open/closeを誰が要求・所有するか、Runtime/Command screen ownerとの接続、複数Window/複数backend support（DD-15 §7、BD-12）。
- DD-03 `UiLoopBridge`の登録API、wakeup failure before/after queue acceptance、host loop停止時の同期worker shutdown timeout/cancel、status/完了観測（DD-03 §7、DD-06/18～21）。
- 座標の数値単位、原点、DPI/scale変換、transform、pixel rounding、unbounded layout size encoding（BD-06、DD-12）。
- Renderer API/scene representation、damage region/partial redraw、frame clock/vsync、GPU thread/context/fence、device lost時のfallbackとresource recreation（BD-12、backend follow-up）。
- Window title/native decoration、具体Windows/Linux windowing・graphics/IME API/library・driver/versionと対応機能差（BD-12、Platform profile）。
- Pointer hit-test/clip/transformとnative/Core focus event ordering、OS-specific event timestamp/capture semantics（DD-13）。
- IME candidate window anchor、font/text shaping/Unicode位置変換、composition rendering detail/session ID width/API（BD-23/24、DD-14）。
- OS Accessibility peer/event/action mappingとAT実機検証（DD-17、BD-13）。
- backend capability queryと失敗status catalog・retry policy、formal target/CI/release gate（DD-06/22/23、BD-12）。

### 17-accessibility.md / 8. 未決事項と後続判断

- 元文書: [17-accessibility.md](../13_detail_design/17-accessibility.md)

- **利用者判断が必要**: DD-15で承認対象外のまま残した effective `IsEnabled` / `Focusable` defaults / Container focus、TextBox accessibility valueのcommitted/editing/preedit境界をDD-17でどこまで決めるか。推奨はFocus/EnabledはDD-15のCore state decisionを待ち、TextBox providerは現在画面上で編集されているediting textを読み取りvalueに使い、committed `Value`はViewModel/Binding valueとして混同せず別情報に出す案。preeditの読み上げはOS profile実測後に決める。
- **利用者判断が必要**: LinuxでAT-SPI 2 D-Bus providerを必須にする範囲。Linux x64 desktopでは候補として採用し、headless/組み込みKMS profileでAccessibility busがない場合のunsupported/optional behaviorは上位Platform profileで定める案。
- StackPanelのWindows UIA Pane vs Group、Linux roleとcontrol/content view inclusion、Window root mappingを確定する適合profile。
- OS provider query/action callbackのthread/apartment、UI threadとの同期返答、timeout、shutdown時error mapping。Windows providerとAT-SPI D-Bus actionの各仕様に沿ってDD-06/16/wrapperと整合させる。
- Windows UIA provider interface/host fragment、COM identity/refcount/thread options、event registration lifecycleと具体C++/Rust binding strategy。
- Linux AT-SPI registry/bus availability detection、DBus object path stability, Accessible/Action/Component/Text interfaces、event signal/lifetime implementationと必要依存。GTK/ATK経由か直接D-Bus providerかの選択。
- TextBox selection/caret/text edit operation public surface、Validation state/error message露出、hidden/offscreen/visibility state、label association/help/localization/name fallback（DD-15/17）。
- screen reader/switch access実機、具体AT product/OS version、formal conformance/accessibility certification範囲とrelease gate（BD-13は未決）。
- macOS provider profileとLinux embedded imageでのAT-SPI bus packaging/version support。

### 17-accessibility.md / 8. 未決事項と後続判断

- 元文書: [17-accessibility.md](../13_detail_design/17-accessibility.md)

- **利用者判断が必要**: DD-15で承認対象外のまま残した effective `IsEnabled` / `Focusable` defaults / Container focus、TextBox accessibility valueのcommitted/editing/preedit境界をDD-17でどこまで決めるか。推奨はFocus/EnabledはDD-15のCore state decisionを待ち、TextBox providerは現在画面上で編集されているediting textを読み取りvalueに使い、committed `Value`はViewModel/Binding valueとして混同せず別情報に出す案。preeditの読み上げはOS profile実測後に決める。
- **利用者判断が必要**: LinuxでAT-SPI 2 D-Bus providerを必須にする範囲。Linux x64 desktopでは候補として採用し、headless/組み込みKMS profileでAccessibility busがない場合のunsupported/optional behaviorは上位Platform profileで定める案。
- StackPanelのWindows UIA Pane vs Group、Linux roleとcontrol/content view inclusion、Window root mappingを確定する適合profile。
- OS provider query/action callbackのthread/apartment、UI threadとの同期返答、timeout、shutdown時error mapping。Windows providerとAT-SPI D-Bus actionの各仕様に沿ってDD-06/16/wrapperと整合させる。
- Windows UIA provider interface/host fragment、COM identity/refcount/thread options、event registration lifecycleと具体C++/Rust binding strategy。
- Linux AT-SPI registry/bus availability detection、DBus object path stability, Accessible/Action/Component/Text interfaces、event signal/lifetime implementationと必要依存。GTK/ATK経由か直接D-Bus providerかの選択。
- TextBox selection/caret/text edit operation public surface、Validation state/error message露出、hidden/offscreen/visibility state、label association/help/localization/name fallback（DD-15/17）。
- screen reader/switch access実機、具体AT product/OS version、formal conformance/accessibility certification範囲とrelease gate（BD-13は未決）。
- macOS provider profileとLinux embedded imageでのAT-SPI bus packaging/version support。

### 18-wrapper-cpp.md / 9. 未決事項と後続担当

- 元文書: [18-wrapper-cpp.md](../13_detail_design/18-wrapper-cpp.md)

| 未決事項 | 担当・次の判断 |
|---|---|
| C++ pending-cleanup registryのnode確保、同期、Runtime pinning、UI safe point wake、shutdown後drain、共有library unload時の寿命 | DD-CPP-15のAPI/implementation design。DD-02のtoken未消費契約は変更しない |
| Runtime wrapperをshutdown前に破棄した場合のdiagnostic、debug assertion、owner state | DD-02/06との整合を確認し、必要ならC++公開API判断 |
| `Result`/status型、throwing convenience層の具体型と最低C++標準 | 後続API設計章。Buildの対象標準はDD-22とも整合 |
| Error messageのUTF-8からC++ string/wide stringへの表示変換 | DD-06/22、application localization boundary |
| 利用者exception handlerの設定scope、default handler、handler自身の例外/再入、callback failure visibility | 後続API設計章。DD-06/07/10/14、追加候補DD-24と整合 |
| Native APIが登録関数から戻る前にcallbackを呼ぶか | owner APIごとの契約。userdata transfer順を確定する前に検証 |
| DECIMAL専用C++ valueとABI encoding、F64/DECIMAL property API | DD-07およびABI上位判断後 |
| Window/UI peer closeとAccessibility providerを含むplatform object寿命 | DD-16/17未決事項。C++ wrapperから補完しない |
| package、DLL/SOロード、ABI symbol解決、static/dynamic配布 | DD-22 |
| C++/C#/Rust/Pythonで揃えるcallback/error/lifetime verification suite | DD-19～23 |

### 18-wrapper-cpp.md / 9. 未決事項と後続担当

- 元文書: [18-wrapper-cpp.md](../13_detail_design/18-wrapper-cpp.md)

| 未決事項 | 担当・次の判断 |
|---|---|
| C++ pending-cleanup registryのnode確保、同期、Runtime pinning、UI safe point wake、shutdown後drain、共有library unload時の寿命 | DD-CPP-15のAPI/implementation design。DD-02のtoken未消費契約は変更しない |
| Runtime wrapperをshutdown前に破棄した場合のdiagnostic、debug assertion、owner state | DD-02/06との整合を確認し、必要ならC++公開API判断 |
| `Result`/status型、throwing convenience層の具体型と最低C++標準 | 後続API設計章。Buildの対象標準はDD-22とも整合 |
| Error messageのUTF-8からC++ string/wide stringへの表示変換 | DD-06/22、application localization boundary |
| 利用者exception handlerの設定scope、default handler、handler自身の例外/再入、callback failure visibility | 後続API設計章。DD-06/07/10/14、追加候補DD-24と整合 |
| Native APIが登録関数から戻る前にcallbackを呼ぶか | owner APIごとの契約。userdata transfer順を確定する前に検証 |
| DECIMAL専用C++ valueとABI encoding、F64/DECIMAL property API | DD-07およびABI上位判断後 |
| Window/UI peer closeとAccessibility providerを含むplatform object寿命 | DD-16/17未決事項。C++ wrapperから補完しない |
| package、DLL/SOロード、ABI symbol解決、static/dynamic配布 | DD-22 |
| C++/C#/Rust/Pythonで揃えるcallback/error/lifetime verification suite | DD-19～23 |

### 19-wrapper-csharp.md / 10. 未決事項と後続担当

- 元文書: [19-wrapper-csharp.md](../13_detail_design/19-wrapper-csharp.md)

| 未決事項 | 後続 |
|---|---|
| `SafeHandle` marshallingと`ui_handle` uint64固定幅の互換、in-flight lease方法 | API設計章・DD-22のTFM/interop matrix |
| cleanup managerの同期・preallocated ticket・Runtime pinning・AppDomain/AssemblyLoadContext unload・process exit drain | DD-CPP-15共通論点/API設計章、DD-22 |
| C#最低language version、TFM、`LibraryImport`/`DllImport`、source generationの採用 | BD-18/要求の未決、DD-22 |
| public exception/result type名、Error message encoding policy、unknown code extension | API設計章。DD-06/07/09/10/18との整合 |
| callback exception handler registration scope、default handler、handler failure reporting | API設計章とDD-24候補 |
| `NativeLibrary.SetDllImportResolver`かdefault RID probingか、consumer assembly resolverとの統合 | DD-22。BD-18 package/RID decisionは維持 |
| ARM/Yocto RID・libc・native dependency compatibility | BD-18/BD-16/27、DD-22 |
| UI loop停止中worker shutdown wait解除 | DD-03/06/16とwrapper共通判断 |
| Platform Accessibility UIA/AT-SPI provider wrapper | DD-17未決事項。ここでC# bindingを追加しない |

### 19-wrapper-csharp.md / 10. 未決事項と後続担当

- 元文書: [19-wrapper-csharp.md](../13_detail_design/19-wrapper-csharp.md)

| 未決事項 | 後続 |
|---|---|
| `SafeHandle` marshallingと`ui_handle` uint64固定幅の互換、in-flight lease方法 | API設計章・DD-22のTFM/interop matrix |
| cleanup managerの同期・preallocated ticket・Runtime pinning・AppDomain/AssemblyLoadContext unload・process exit drain | DD-CPP-15共通論点/API設計章、DD-22 |
| C#最低language version、TFM、`LibraryImport`/`DllImport`、source generationの採用 | BD-18/要求の未決、DD-22 |
| public exception/result type名、Error message encoding policy、unknown code extension | API設計章。DD-06/07/09/10/18との整合 |
| callback exception handler registration scope、default handler、handler failure reporting | API設計章とDD-24候補 |
| `NativeLibrary.SetDllImportResolver`かdefault RID probingか、consumer assembly resolverとの統合 | DD-22。BD-18 package/RID decisionは維持 |
| ARM/Yocto RID・libc・native dependency compatibility | BD-18/BD-16/27、DD-22 |
| UI loop停止中worker shutdown wait解除 | DD-03/06/16とwrapper共通判断 |
| Platform Accessibility UIA/AT-SPI provider wrapper | DD-17未決事項。ここでC# bindingを追加しない |

### 20-wrapper-rust.md / 10. 未決事項と後続担当

- 元文書: [20-wrapper-rust.md](../13_detail_design/20-wrapper-rust.md)

| 未決事項 | 後続 |
|---|---|
| crate名、最低Rust version/edition、公開型・関数名、trait boundの細部 | 後続API設計章、BD-20の未決事項 |
| cleanup registryの同期、lock poisoning、Runtime pin、wake-up、process exit/library unloadでのdrain | API/実装設計。DD-LIFE-06とDD-CPP-15/DD-CS-12を共通契約にする |
| `panic=unwind` callback panicの機能別failure、診断payload、利用者handler | callback owner設計・後続API設計章。DD-07/09/10/14およびDD-18/19と整合 |
| error型・文字列decode/invalid UTF-8 policy・Error clone/ownership API | API設計章、DD-05/06、BD-27 |
| UI thread identityをRust markerとnative checkでどう組み合わせるか、async facade | API設計章。完了待ち機構は承認前に追加しない |
| Cargo feature、artifact acquire、build.rs metadata、link/load、crate publishing | BD-20/DD-22 |
| Rust target matrix、ARM Linux/sysroot/libc/board profile | BD-16/DD-22、hardware verification gate |
| loop停止中のworker同期shutdown解除とshutdown完了観測 | DD-03/06/16およびwrapper共通判断 |

### 20-wrapper-rust.md / 10. 未決事項と後続担当

- 元文書: [20-wrapper-rust.md](../13_detail_design/20-wrapper-rust.md)

| 未決事項 | 後続 |
|---|---|
| crate名、最低Rust version/edition、公開型・関数名、trait boundの細部 | 後続API設計章、BD-20の未決事項 |
| cleanup registryの同期、lock poisoning、Runtime pin、wake-up、process exit/library unloadでのdrain | API/実装設計。DD-LIFE-06とDD-CPP-15/DD-CS-12を共通契約にする |
| `panic=unwind` callback panicの機能別failure、診断payload、利用者handler | callback owner設計・後続API設計章。DD-07/09/10/14およびDD-18/19と整合 |
| error型・文字列decode/invalid UTF-8 policy・Error clone/ownership API | API設計章、DD-05/06、BD-27 |
| UI thread identityをRust markerとnative checkでどう組み合わせるか、async facade | API設計章。完了待ち機構は承認前に追加しない |
| Cargo feature、artifact acquire、build.rs metadata、link/load、crate publishing | BD-20/DD-22 |
| Rust target matrix、ARM Linux/sysroot/libc/board profile | BD-16/DD-22、hardware verification gate |
| loop停止中のworker同期shutdown解除とshutdown完了観測 | DD-03/06/16およびwrapper共通判断 |

### 21-wrapper-python.md / 11. 未決事項と後続担当

- 元文書: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)

| 未決事項 | 後続 |
|---|---|
| Python minimum version、CPython/PyPy/free-threaded support、GIL matrix | BD-14/DD-22 |
| public owner/callback/error type、method名、`close`のexception chaining/context semantics | API設計章 |
| `__del__`/weakref.finalize/managerの選択、ticket registry同期、worker finalize、permanent release failure | API/implementation design。DD-LIFE-06、DD-CPP-15、DD-CS-12、DD-RS-05/06と整合 |
| interpreter/runtimeごとのlifecycle coordinator登録、main/host thread終了通知の移植可能な提供元、normal-exit補助hook、UI wake-up経路、in-flight callback/root drain handshake、main/UI相互待ちがない証明 | DD-22/API・implementation設計およびCore lifecycle連携。DD-PY-20の要求を満たす具体方式 |
| Qt `aboutToQuit()`相当のhost loop adapter、`sys.excepthook`/`threading.excepthook`の導入・合成・解除と複数Runtimeの所有規則、main未捕捉例外とwrapper所有worker例外の終了policy | API/implementation設計およびDD-22。既存host hookを保存・委譲し、thread/contextに適したdispatchでdeadlockを作らないこと |
| userdata registry key/cellとcallback registration中の同期invocation可否 | ABI owner API契約、DD-06～10/14、callback共通設計 |
| destroy後のPython root release safe pointとplatform UI loop integration | DD-16/22。C destroy内でPython finalizerを起動しない制約を維持 |
| `BaseException`捕捉、`KeyboardInterrupt`/`SystemExit`の再通知、handler default/diagnostics | 後続API設計章。callback固有failureはowner設計に従う |
| special method overrideごとのrequired/optional/no-chain-up一覧、base-call guard実装、operator dispatchの実行時検出範囲 | BD-15 analyzerと後続API設計。event callback/event method規則はDD-24候補と重ならないよう範囲確認 |
| free-threaded Python version/support matrix、`ctypes`/native-library/thread-stateの適合 | BD-14/DD-22。安全性設計はGIL有無に依存させない |
| `ctypes` symbol/prototype generation/header parity tooling | BD-14/DD-22、BD-15 type stub/analyzer境界 |
| Python package/index、pure wheel metadata、native binary wheel tags、library search/loading | BD-17/DD-22 |
| ARM/Yocto wheel compatibilityとnative artifact acquisition | BD-16/17/DD-22 |
| UI loop停止時の同期worker shutdown待機解除/completion observation | DD-03/06/16とwrapper共通設計 |
| dynamic Python codeのevent method静的解析とmarker/decorator仕様 | BD-15および追加候補DD-24。DD-21では決めない |

### 21-wrapper-python.md / 11. 未決事項と後続担当

- 元文書: [21-wrapper-python.md](../13_detail_design/21-wrapper-python.md)

| 未決事項 | 後続 |
|---|---|
| Python minimum version、CPython/PyPy/free-threaded support、GIL matrix | BD-14/DD-22 |
| public owner/callback/error type、method名、`close`のexception chaining/context semantics | API設計章 |
| `__del__`/weakref.finalize/managerの選択、ticket registry同期、worker finalize、permanent release failure | API/implementation design。DD-LIFE-06、DD-CPP-15、DD-CS-12、DD-RS-05/06と整合 |
| interpreter/runtimeごとのlifecycle coordinator登録、main/host thread終了通知の移植可能な提供元、normal-exit補助hook、UI wake-up経路、in-flight callback/root drain handshake、main/UI相互待ちがない証明 | DD-22/API・implementation設計およびCore lifecycle連携。DD-PY-20の要求を満たす具体方式 |
| Qt `aboutToQuit()`相当のhost loop adapter、`sys.excepthook`/`threading.excepthook`の導入・合成・解除と複数Runtimeの所有規則、main未捕捉例外とwrapper所有worker例外の終了policy | API/implementation設計およびDD-22。既存host hookを保存・委譲し、thread/contextに適したdispatchでdeadlockを作らないこと |
| userdata registry key/cellとcallback registration中の同期invocation可否 | ABI owner API契約、DD-06～10/14、callback共通設計 |
| destroy後のPython root release safe pointとplatform UI loop integration | DD-16/22。C destroy内でPython finalizerを起動しない制約を維持 |
| `BaseException`捕捉、`KeyboardInterrupt`/`SystemExit`の再通知、handler default/diagnostics | 後続API設計章。callback固有failureはowner設計に従う |
| special method overrideごとのrequired/optional/no-chain-up一覧、base-call guard実装、operator dispatchの実行時検出範囲 | BD-15 analyzerと後続API設計。event callback/event method規則はDD-24候補と重ならないよう範囲確認 |
| free-threaded Python version/support matrix、`ctypes`/native-library/thread-stateの適合 | BD-14/DD-22。安全性設計はGIL有無に依存させない |
| `ctypes` symbol/prototype generation/header parity tooling | BD-14/DD-22、BD-15 type stub/analyzer境界 |
| Python package/index、pure wheel metadata、native binary wheel tags、library search/loading | BD-17/DD-22 |
| ARM/Yocto wheel compatibilityとnative artifact acquisition | BD-16/17/DD-22 |
| UI loop停止時の同期worker shutdown待機解除/completion observation | DD-03/06/16とwrapper共通設計 |
| dynamic Python codeのevent method静的解析とmarker/decorator仕様 | BD-15および追加候補DD-24。DD-21では決めない |

### 22-build-distribution.md / 10. 未決事項と後続担当

- 元文書: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)

| 未決事項 | 後続 |
|---|---|
| Rust/Cargo/C/C++/platform SDK/compiler/linker exact version、Linux libc baseline、Yocto/sysroot lock、board/CPU/FPU/GPU/input profile、CI provider/jobs/cache | Build/test implementation plan。正式support matrixは利用者の上位判断が必要 |
| package IDs、wrapper version scheme、Python support/tag、.NET TFM/RID additions、CMake version/targets、Cargo registry/crate names | 後続API/distribution/release design |
| canonical catalog URL/API/auth/proxy、resolver command/plugin、explicit fetch workflow、cache placement/GC/offline bundle format | Artifact distribution implementation design |
| exact manifest field names/types/schema version/critical flags/size limits and multiple-major reader policy | Manifest schema implementation/verification design |
| signature envelope/key-id bytes、trusted root provisioning, key custody, rotation/revocation/stale metadata intervals | 利用者承認済みのJCS/Ed25519/SHA-256を維持し、crypto/trust implementation designで確定。安定版公開・fetch実装前に確定 |
| release approval workflow、CI provenance attestation、SBOM format/aggregation/signing、reproducible-build requirement | Release/security design |
| toolchain profileをsupport matrix/EOLへ登録する手順と最低12か月deprecation通知の公開channel | BD-22 support policy実装、release process |
| full 4-language vertical scenario、system release gate、version-impact regression suite、ABI compatibility tests | Requirements test case catalog、Release/Version test規約、test implementation plan |

### 22-build-distribution.md / 10. 未決事項と後続担当

- 元文書: [22-build-distribution.md](../13_detail_design/22-build-distribution.md)

| 未決事項 | 後続 |
|---|---|
| Rust/Cargo/C/C++/platform SDK/compiler/linker exact version、Linux libc baseline、Yocto/sysroot lock、board/CPU/FPU/GPU/input profile、CI provider/jobs/cache | Build/test implementation plan。正式support matrixは利用者の上位判断が必要 |
| package IDs、wrapper version scheme、Python support/tag、.NET TFM/RID additions、CMake version/targets、Cargo registry/crate names | 後続API/distribution/release design |
| canonical catalog URL/API/auth/proxy、resolver command/plugin、explicit fetch workflow、cache placement/GC/offline bundle format | Artifact distribution implementation design |
| exact manifest field names/types/schema version/critical flags/size limits and multiple-major reader policy | Manifest schema implementation/verification design |
| signature envelope/key-id bytes、trusted root provisioning, key custody, rotation/revocation/stale metadata intervals | 利用者承認済みのJCS/Ed25519/SHA-256を維持し、crypto/trust implementation designで確定。安定版公開・fetch実装前に確定 |
| release approval workflow、CI provenance attestation、SBOM format/aggregation/signing、reproducible-build requirement | Release/security design |
| toolchain profileをsupport matrix/EOLへ登録する手順と最低12か月deprecation通知の公開channel | BD-22 support policy実装、release process |
| full 4-language vertical scenario、system release gate、version-impact regression suite、ABI compatibility tests | Requirements test case catalog、Release/Version test規約、test implementation plan |

### design-overview.md / 7. 実装・設計スライス

- 元文書: [design-overview.md](../12_design/design-overview.md)



### design-overview.md / 8. 初期縦断シナリオ

- 元文書: [design-overview.md](../12_design/design-overview.md)

```text
Window
└─ StackPanel
   ├─ TextBox  ← TwoWay  → ViewModel.Name
   ├─ Text    ← OneWay   ← ViewModel.Message
   └─ Button  ─ Command  → ViewModel.Save
```

最低限、次をC++、C#、Rust、Python 3で検証する。

- TextBoxの入力がViewModelへ反映される
- ViewModelの変更がTextへ反映される
- ButtonからCommandを実行できる
- Commandの実行可否を扱える
- focusが移動する
- キーボード操作ができる
- accessibility nameを取得できる

### design-overview.md / 11. 次の設計スライス

- 元文書: [design-overview.md](../12_design/design-overview.md)

次は`runtime-contract.md`と`abi-spec.md`の範囲を対象とし、以下を具体化する。

- handleの種類
- 所有権モデル
- retain / release規則
- 文字列・配列の借用とコピー
- error object
- callback解除と再入可能性
- UI thread違反の扱い
- ABI versionと互換性ポリシー

### mvvm-semantics.md / 16. 次に設計する項目

- 元文書: [mvvm-semantics.md](../12_design/mvvm-semantics.md)

- Propertyと各言語で定義されたViewModelの接続契約、値型とABIの型表現。
- 値変換・非同期検証、依存Property、通知callbackが連続更新する場合の上限と診断、Binding handleの最終所有者。
- 非同期Commandの具体的なABI・wrapper、workerの所有と生成失敗、進捗の間引きと順序保証。
- 言語側ViewModelの検証callback、エラー表現と4言語wrapperの契約。
- View入力状態とBindingエラー状態の表示優先順位、IME・Undo/Redoの詳細。

### language-bindings.md / 8. 次に設計する項目

- 元文書: [language-bindings.md](../12_design/language-bindings.md)

- C ABIのProperty値型、入力・出力buffer、Validation結果とBinding errorの表現。
- 既存の言語側Observableクラスを双方向同期するadapterの通知・エラー・lifetime規則。
- 4言語の具体的なAPI、Commandの非同期タスク橋渡し、パッケージと各targetの検証。


## 承認履歴（今回）

| 日付 | 対象版・範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 1.0 / 計画と未設計事項の所管移管 | 利用者 | 全案反映を承認。本文に未設計と記した項目の実装方式・実装完了は含まない |

## 改訂履歴（今回）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 1.0 | 2026-10-17 | 要求正本への対応、層に応じた責務・flow・所有・stateの本文へ改訂 | Codex |
| 1.1 | 2026-10-17 | 全件レビュー案・対応表への入口を追加。方式採用は未承認 | Codex |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 1.2 / 未設計課題全件の正式ID・所管と実環境残作業へ更新 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 1.2 | 2026-10-17 | 未設計課題全件の正式ID・所管と実環境残作業へ更新を正本へ適用し、旧判断との対応を保持 | Codex |
