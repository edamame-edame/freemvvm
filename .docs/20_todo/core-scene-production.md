# Core UI Scene生成の次工程

- 2026-10-31 progress: `UNIT-CORE-080` (`ab2c273`) and `UNIT-CORE-081` (`6885f91`) are committed and pushed on `impl-rust/core-scene-rendering`. Plan 0.5's `UNIT-CORE-082` Runtime live tree capture is implemented; focused/Core/workspace/minimum checks pass locally. Production visual descriptors/TextService, CI and Linux hardware remain unverified for this unit.

- 計画版: 0.5 改訂案
- 状態: 計画0.3/0.4/0.5利用者承認済み。UNIT-CORE-080/081 commit/push済み。UNIT-CORE-082実装・local検証済み、commit/push待ち
- 作成日: 2026-10-29
- 対象version: `0.0.1`
- 対象branch案: `impl-rust/core-scene-rendering`（`impl-rust/core-apis-window-background`の子branch）
- 独立して保留する作業: [Linux Platform hardware smoke](linux-platform-hardware-smoke.md)。本作業は実機確認を含めない

## 目的

Platform Adapterの実機確認は保留し、既存のWindow/Control/Logical Tree/LayoutからCore-owned `SceneSnapshot`を生成して描画経路へ渡すため、要求・設計・test・実装の次の最小単位を確定する。既存backendのDocker仮passはrenderer単体のpixel確認であり、UI treeから画面表示までの縦断確認ではない。

## 監査結果

| 境界 | 現状 | 次に決めること |
|---|---|---|
| Control → Visual | Control treeから`LayoutSnapshot`を生成するが、`VisualNode`/Visual tree traversalやControl→DrawOp producerは未実装 | 描画エンジンに依存しないView/Visual契約とControl connectorの責務、走査順、所有/参照、snapshot時点 |
| View → renderer backend | 下流には共通`RenderBackend` seamがあるが、Controlとbackend-neutral View/Visualを結ぶ契約は未実装 | WinAPI/OpenGL/Vulkan等を個別実装・選択可能にする境界。Window/native surfaceの所有と描画API選択を分離し、backend差をCore UIへ漏らさない |
| Geometry / Clip | Layout boundsは親座標系のまま。絶対座標変換、祖先client clipの交差、draw orderのScene化なし | Control boundsからscene座標への変換とclip適用規則 |
| Text | 実Control用intrinsic measure provider/TextServiceなし。GlyphRun生成とfont asset解決なし。Linux GLESはGlyphRunを未対応として明示failure | RQ-09-072のparagraph表示を満たすTextService/font資源・profile・renderer対応範囲。Textを黙って落とさない |
| Appearance | 要求はText/Buttonの表示を含むが、Scene上の文字色や最小visual defaultsを実装都合だけで確定できない | Core-owned appearance source/defaultの要求・設計上の根拠。Style/Theme全面対応とは区別 |
| Scale / background | `SceneSnapshot`には`DeviceScale`とbackground RGBがあるが、Window/Runtimeが値を所有・供給する経路なし | CoreからScaleを渡す契約とbackground色の所有元・更新規則 |
| Frame connection | `window_submit_frame`は呼出元が用意した`SceneSnapshot`を受け取る。dirty→Layout→Scene→submitの自動経路なし | UI thread scheduling、dirty/revision/damage、frame resubmit条件 |
| Test mapping | Layout/Platform各単体testはあるが、Control→Scene producer unit/test mappingなし | 実装単位に対応するunitとtest設計・traceability |

### 要求上の制約

- RQ-09-072は単一paragraphのtext表示とclient clippingを要求する。soft-wrap、ellipsis、selection/copyは必須範囲ではない。Button labelとAccessibility nameは別データ。
- RQ-11-008/009/020/022/023/024に従い、Coreをsource of truthとしrendererを交換可能に保つ。backend failureを成功扱いせず、backend/profile/font識別を曖昧にしない。Text engineはRust Core外。
- DDに記載されたWindows DirectWrite、Linux Fontconfig/HarfBuzz/FreeType、Noto系候補は実装済み依存や確定profileとは限らない。候補をそのまま依存追加・配布仕様として確定しない。
- Control connectorが生成するView/Visual表現は、WinAPI/OpenGL/Vulkan等の具体backendから独立させる。共通`RenderBackend` seamは既に下流にあるため、既存seamを再定義するのではなく、Control→View→SceneSnapshot→選択backend間の責務・変換契約をつなぐ。
- OS Window/native surfaceの管理（例: WinAPI）と描画API/backend（例: OpenGL/Vulkan）は異なる選択軸になり得る。組合せ可能性、選択時点、profile/capability/failure通知は設計で決め、単一OS=単一rendererとは仮定しない。

## 次の作業案

1. DDおよびUI unit-test designの改訂案を文書単位で作る。上表の未決境界を埋め、RQとの対応・実装単位・pass条件・Docker仮passとhardware/profile確認の区別を明記する。既存承認の意味や範囲を黙って変更しない。
2. 提案する実装順は以下とする。順序・境界はDD/test文書レビューで確定し、承認前は実装しない。
   1. **View/renderer契約**: backend-neutral View/Visual modelとControl connector、および既存`RenderBackend`への接続。Window/native surfaceとrenderer選択を分離し、複数backendの登録・選択・capability/failure境界を定義。
   2. **Scene入力契約**: Core-owned `DeviceScale`/background供給元、TextService境界、font/profile資源とGlyphRunのappearance情報。
   3. **Scene producer**: Visual snapshot/traversal、absolute geometry、ancestor clipping、draw order、`SceneSnapshot`への変換。
   4. **frame接続**: dirty/revisionからLayout・Scene更新、UI thread上のsubmit/re-submitまで。
   5. **Text表示のbackend縦断**: 選定profileのGlyphRun描画。backend未対応時は明示failureを維持。
3. 文書全体のレビュー・利用者承認後、実装先言語が変わる場合は既定のbranch treeに従って実装branchを切り直し、承認済みunitごとに実装・test・同revisionの記録を進める。

## 検証方針

- Unit testはControl→Visual/Scene変換、geometry/clip、revision/damage、frame submit条件の実装単位に対応させ、同一revisionで確認する。
- Textの実表示は選定backend/profileで別unitとして検証する。未対応のGlyphRunを無視して通過させない。
- Docker検証は利用可能なbackendの仮確認に限り、実機Platform Adapter確認やprofile適合の合格へ昇格しない。
- 本計画・監査の文書チェックは実装testとは区別する。

## 最初の実装slice（計画0.3案）

### UNIT-CORE-080: ControlViewConnectorのgeometry/clip flattening

- 承認済み`UT-VIEW-GEOMETRY-001`を使い、ephemeral `VisualNodeSnapshot`と同revision `LayoutSnapshot`からrenderer-neutral `SceneSnapshot`を構築する。
- FinalBoundsは承認済みDD 1.4に従い親座標系として扱う。scene builderが親originを累積してabsolute DIPへ変換し、各nodeのclient clipと祖先clipの交差をClip operationとして出力する。Clipはlayout boundsを変更しない。
- child orderと各nodeのlocal FillRect/GlyphRun orderを保ち、background RGB、DeviceScale、layout revision、damageを出力へ渡す。geometry/build failureではpartial SceneSnapshotを公開しない。
- 完了した最小unitはVisual operation付きのephemeral inputからsceneを作るまで。live Runtime Control treeからvisual inputを組み立てる経路、TextService/font/profile/appearance、dirty-to-frame自動接続、renderer factory selection、Linux physical displayは本unitの完了に含めない。
- focused command: `cargo test -p freemvvm-core control_view_connector_ --locked`。Core crate全test、Clippy、fmtも同一revisionで確認する。

実装監査で、現行`LayoutEngine::arrange_node`が親座標を子FinalBoundsへ累積していたため、DD 1.4の「親座標系bounds」と不一致であることを発見した。UNIT-CORE-080の同revision修正で、FinalBoundsを親相対に保ち、scene generationでabsolute座標へ合成する。既存単層Layout testを維持し、nested geometry/clip testで新しい境界を検証する。

## 次の実装slice（計画0.4案）

### UNIT-CORE-081: Logical Control treeからVisualNodeSnapshotを構築

- attached Logical Control treeを表すrevision-stamped identity snapshotと明示Visual descriptor mapを受け、同じLayoutSnapshotに含まれるidentityだけからephemeral `VisualNodeSnapshot`を構築する。tree captureのRuntime/UI-thread接続とproduction descriptor供給元は後続unitに分ける。
- Visual descriptor（clip-child規則とlocal DrawOp列）は内部入力として明示的に渡し、Control treeの所有参照・Property callback・renderer/native handleをVisual snapshotへ持ち込まない。descriptorの欠落・余剰、layout/tree revision不一致、重複identityはSceneを部分公開せず失敗する。
- 本unitはrevision-stamped tree snapshotからのtraversal / identity / ordering / all-or-nothing接続まで。Text shaping、font profile、Text/Button appearance、Clip policyのControl別既定値、Runtime live tree capture、dirty-to-frame schedulingは追加しない。descriptor供給元がないControlを空描画としてPresented成功させない。
- 承認済み`UT-VIEW-CONNECTOR-001`からtree attach/detach/order・同revision・operation ordering・atomic failureの確認を切り出す。実装はtest専用fake descriptorを注入し、製品appearanceを創作しない。
- focused command案: `cargo test -p freemvvm-core control_tree_visual_snapshot_ --locked`。Core tests、Clippy/fmt、`tools/dev.py check`、`tools/dev.py minimum`を同revisionで確認する。

計画0.4は次sliceの責務を示す。Runtime live tree captureと、Control内容から実描画operationを供給するProduction Visual descriptor/TextServiceは後続設計課題であり、UNIT-CORE-081をLinux上のUI表示完了とは扱わない。

## 次の実装slice（計画0.5案）

### UNIT-CORE-082: Runtimeのattached Control treeをclean LayoutSnapshotと同revisionでcapture

- UI thread上で開いているWindowのcontent root以下をLogical Treeから順序付き・非所有の`ControlTreeSnapshot`へcaptureする。
- Window generation、現在Windowにpublish済みのLayoutSnapshot identity、tree revision、全Controlのlayout-dirty状態を照合する。いずれかが古い/dirty/不一致ならsnapshotを返さない。空contentは空sceneとして成功扱いせず、tree snapshotなしを明示する。
- identity snapshotはControl/Tree ownerやProperty callbackを保持せず、DrawOp/Visual descriptorは含めない。UNIT-CORE-081の`build_from_control_tree`へ渡せる入力をCore Runtimeから供給するところまで。
- attach/detach/order更新、未再計算layout、Window close/generation不一致、成功後のsame-revision captureを既存fake Platform test harnessで検証する。新しいC ABI/APIは追加しない。
- focused command案: `cargo test -p freemvvm-core runtime_control_tree_snapshot_ --locked`。Core tests、Clippy/fmt、`tools/dev.py check`、`tools/dev.py minimum`を同revisionで確認する。

UNIT-CORE-082はRuntime tree/layout snapshotの整合接続であり、production Visual descriptorやLinux上のControl paintingを完了とは扱わない。

## 対象外

- Debian/RPi実機DRM/KMS操作、Platform Adapter hardware smoke、`/dev/dri` passthrough。
- 新しい公開C ABI signatureまたはlanguage wrapper API。Scene producerに必要と判明した場合は別の設計/API unitとして波及を提示する。
- OS keyboard/pointer routing、TextBox IME、Accessibility、package/publication、0.0.1 release判断。
- RQにないvisual default、placeholder text/color、software fallback、未対応GlyphRunを無視する挙動。

## 承認境界

- 計画0.1で承認された範囲（要求・設計・実装差分の読取監査と次の設計提案）は完了した。
- 本計画0.2は利用者承認済み。承認範囲はDD/UI test改訂案の作成までであり、設計本文の承認・実装開始を含まない。
- 承認済み正本: [UI Tree/Layout 1.4](../13_detail_design/ui/tree-layout.md)、[Platform internal layers 1.6](../13_detail_design/platform/internal-layers.md)、[UI unit tests 0.8](../32_unit_test/ui.md)。implementation-test inventoryへunit mappingを登録し、承認済み範囲で実装する。
- 実装着手前の契約確認: `DeviceScale`のWindowPeer→Core供給/変更経路とWindow background RGBのCore所有・初期値供給元、Text/Button等ControlからのDrawOp生成（TextService依存を含む）を確定し、要求のないplaceholder/defaultを作らない。
- DD/UI test本文の改訂は文書全体をレビュー・承認してから実装する。計画承認、設計承認、実装開始、unit completion、commit/pushは別状態として記録する。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-10-29 | 計画0.1 | 利用者承認・監査完了 | Layout不足を先に解消する改訂計画を経て、UNIT-CORE-076/077/078を完了。 |
| 2026-10-29 | 計画0.2 | 利用者承認 | Scene producer、TextService、scale/background供給元、Visual geometry/clip、frame接続に加え、backend-neutral View契約と複数renderer選択軸を反映。DD/UI test改訂案の作成を承認。 |
| 2026-10-30 | DD/UI test改訂案 | review draft | ControlViewConnector→backend-neutral SceneSnapshot→選択可能RenderBackendの責務とfake-test proposalを3文書に作成。文書全体レビュー待ち。 |
| 2026-10-30 | DD 1.4 / DD 1.6 / UT 0.8 | 利用者承認 | 「ok」により3文書全体を承認。実装・試験合格とは別管理。 |
| 2026-10-31 | 計画0.3 / UNIT-CORE-080 | 利用者承認・実装済み | UNIT-CORE-080をcommit `ab2c273`でpush。focused 2件・Core 171件・`tools/dev.py check`・`tools/dev.py minimum` pass。CI・Linux実機未確認 |
| 2026-10-31 | 計画0.4 / UNIT-CORE-081 | 利用者承認 | revision-stamped tree identity snapshotとfake descriptor mapからSceneを構造生成。focused 2件・Core 173件・`tools/dev.py check`・`tools/dev.py minimum` pass。Runtime live tree capture、production appearance/TextService/Control別clip policyは後続 |
| 2026-10-31 | 計画0.5 / UNIT-CORE-082 | 利用者承認 | Runtime UI-thread tree captureを現行clean LayoutSnapshotへ照合。focused test pass。dirty/stale/closed stateを返さずsceneを部分公開しない |
