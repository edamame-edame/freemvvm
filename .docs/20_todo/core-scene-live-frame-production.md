# Core ControlからPlatform frameまでのproduction接続計画

- 計画版: 0.1
- 状態: 計画0.1利用者承認済み（2026-10-31）。UNIT-CORE-086に着手、UNIT-CORE-087は後続
- 作成日: 2026-10-31
- 対象version: `0.0.1`
- 対象branch案: `impl-rust/core-scene-rendering`（Rust Core / Platform feature）
- 上位計画: [Core UI Scene生成の次工程](core-scene-production.md)、[0.0.1利用可能UIライブラリ計画](release-0.0.1.md)
- 前提: UNIT-CORE-085、UNIT-PLATFORM-004は実装・unit test・Windows/Linux CIまで完了。Linux DRM実機確認は別途保留

## 目的

現状のCoreはControl tree/Layout/Text snapshotとScene/frameの各契約を持つが、production `VisualNodeDescriptor`の生成と、dirtyなWindowからLayout→Scene→`window_submit_frame`までを自動でつなぐ経路がない。fake descriptorや呼出元作成Sceneだけでなく、実Control stateからPlatformへframeを提出する内部経路を、承認済み要求・設計内で実装可能なunitへ分ける。

この計画の承認は設計文書の改訂、実装開始、公開C ABI追加、Windows renderer選定、Linux実機合格を意味しない。

## 現状監査

| 境界 | 現状 | この計画で埋める差分 |
|---|---|---|
| Control → Visual | `ControlViewConnector`は明示descriptor mapを受け取るが、production Controlsからmapを供給する経路がない。現行integration testsはfake descriptorを使う | Core Control stateと同revisionのLayout/Text snapshotから、承認済み初期表示内容だけを内部Visual operationへ変換する |
| dirty → frame | `window_submit_frame`は呼出元作成済み`SceneSnapshot`を受け取る。dirty/layout revisionからの自動submit経路はない | Window generation単位でdirtyを集約し、complete Layout/Visual/Sceneを作って既存Platform peerへsubmitする |
| failure/retry | Layout/Text/Scene/frame individual failure contracts exist; end-to-end automatic production path has no test | failureではpartial snapshotを公開せずdirtyを保持し、stale completionを無視してretry可能にする |
| hardware | Linux実機DRM/KMSでのWindow open/displayは保留 | 対象外。fake Platformとheadless software testまでをhardware passへ読み替えない |

## 根拠・対応範囲

- 要求: [RQ-09 UI scope](../11_requirements/RQ-09_ui-scope.md) RQ-09-001/004/006/007/008/067/072/075、[RQ-11 Platform](../11_requirements/RQ-11_platform.md) RQ-11-008/009/020/021/023。
- 設計正本: [UI tree/layout](../13_detail_design/ui/tree-layout.md)、[UI controls](../13_detail_design/ui/controls.md)、[Platform internal layers](../13_detail_design/platform/internal-layers.md)。各unit着手時に対象版と承認履歴を再確認する。
- test設計: [UI unit test](../32_unit_test/ui.md) `UT-VIEW-CONNECTOR-001` / `UT-TEXT-PLATFORM-DEFAULT-001` / `UT-CORE-SCENE-FRAME-001`、[release system scenarios](../30_release_test/system-scenarios.md)のlayout/frame/error cases。
- Core実装はRust/C ABI親feature branch内とし、既存public C ABI surfaceを増やさない案。必要性が判明した場合は理由と波及範囲を別レビューに戻す。

## 実装unit案

### UNIT-CORE-086 — Production Visual descriptor供給

- attached Control tree snapshot、同revision `LayoutSnapshot`、同revision resolved Text snapshotからVisual operationを生成し、既存`ControlViewConnector`へ接続する。
- Text/TextBox/Button label等は承認済みControl semanticsにある表示値だけを利用する。Control chrome、placeholder、caret/selection、IME、hover/pressed/focus paint等の未確定appearanceを追加しない。描画内容が確定できないControlは空operationで成功扱いせず、明示的な非描画/unsupported境界を設計根拠とともに示す。
- stale tree/layout/text revision、欠落descriptor, allocation/TextService failureではpartial Visual/Sceneを返さない。
- focused test案: `cargo test -p freemvvm-core production_visual_ --locked`。`UT-VIEW-CONNECTOR-001`の実Control descriptor・同revision・失敗atomicityを検証する。

### UNIT-CORE-087 — dirty Windowからのframe orchestration

- Window open generationごとに既存のLayout/Text/Visual invalidationを集約し、UI thread上でMeasure/Layout→production Visual→Scene→既存Platform submitを同一revisionで行う。
- 連続更新は未開始frameへ集約し、submit successのみではPresentedとしない。対応completionでのみframe完了とし、failureはdirty/last complete snapshotを維持する。closed/reopened generationや古いrevisionのcompletionを捨てる。
- Platform Adapter実装選択や新renderer、公開frame APIは追加しない。既存Linux peerへ渡せる共通Sceneをfake backendで検証する。
- focused test案: `cargo test -p freemvvm-core dirty_window_frame_ --locked`。`UT-CORE-SCENE-FRAME-001`のcoalescing、same-revision、retry/failure atomicity、stale completionを検証する。

各unitの実際のtest名・source/config mappingは着手前に`tools/units.yaml`とimplementation-test inventoryへ登録する。各unitの対応testを同じrevisionで実行し、pass後に日本語commit・working branchへのpushを行う。

## 対象外

- Linux DRM/KMS、GBM、EGLの物理機器上でのopen/display、実機GPU/HiDPI確認。別hardware profile evidenceとして保留を維持する。
- Windows OS Window/renderer実装、GDI/Direct2D/OpenGL/Vulkanの選定。RQ-11-004を満たすWindows縦断計画として別途設計・レビューする。
- 公開C ABI・言語wrapper API変更、Style/Theme、Button/TextBox chrome、入力routing、IME、Accessibility。
- 新しいfont/color/default、表示方式またはControl意味論。設計未確定事項は実装都合で決めず、影響と差分を先に提示する。
- release合格、tag、公開。

## 完了条件

- production Controlsから同revisionのSceneSnapshotが生成され、fake Platformへ提出できる。
- open/resize/content/tree/display-property更新がframe invalidationを起こし、同じUI turn内の更新を適切に集約する。
- Text/Layout/Scene/submit failure後に以前のcomplete stateを維持し、成功Presentedを偽らない。generation/revision stale completionは現行stateを変えない。
- focused/unit suite、workspace required gates、minimum Rust、format、document link/manifest checksが各implementation revision上でpassする。
- hardware実機確認は未実施のまま別扱いし、release profile passへ読み替えない。

## 承認境界

- 利用者は計画0.1を承認し、UNIT-CORE-086から順に実装を進めることを承認した。計画とunit/test mappingを先行してcommit/pushし、各実装unitは対応test pass後に別途日本語commit/pushする。
- UNIT-CORE-087は計画承認済みの後続unitだが、086の完了後にsource/test mappingを再確認して着手する。
- 設計の未決事項が実装結果を変えると判明した場合は、その項目を解決する文書単位の設計提案を先に出す。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-10-31 | 計画0.1 | 利用者承認・UNIT-CORE-086開始 | 「承認」によりproduction Visual供給、dirty-to-frame unit splitとtest mappingを承認。計画commit/push後、UNIT-CORE-086から実装する |
