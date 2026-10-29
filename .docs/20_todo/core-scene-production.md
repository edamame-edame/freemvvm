# Core UI Scene生成の次工程

- 計画版: 0.1 review draft
- 状態: 利用者レビュー待ち。新しいDD/test全文レビューと実装は未開始。未commit
- 作成日: 2026-10-29
- 対象version: `0.0.1`
- 対象branch案: `impl-rust/core-scene-producer`（現Platform/Core branchにあるSceneSnapshot/Renderer seamへ依存する子branch）
- 独立して保留する作業: [Linux Platform hardware smoke](linux-platform-hardware-smoke.md)。本作業は実機確認を含めない

## 目的

Platform Adapterの実機確認は保留し、既存のWindow/Control/Logical Tree/LayoutからCore-owned `SceneSnapshot`を生成して描画経路へ渡すための、要求・設計・test・実装の次の最小単位を確定する。既存backendのDocker仮passはrenderer単体のpixel確認であり、UI treeから画面表示までの縦断確認ではない。

## 現状の差分

- CoreにはWindow/StackPanel等のControl state、Logical Tree、Layout/Focus、`SceneSnapshot`、FrameTracker、`PlatformWindowPeer::submit_frame`接続がある。
- `SceneSnapshot`はテストから直接作られており、実際のControl/Visual tree traversalから自動生成する実装はない。Platform DD v1.3もscene producerを後続unitとしている。
- 承認済みUI DDには`VisualNode::build_visual_children()`、Textの`build_visual(text_snapshot)`等がある一方、Control/VisualからDrawOpへの完全な対応、scene revision/damage生成の実装境界がunitとして未登録。
- Linux GLES backendの実装subsetはClip/FillRectで、GlyphRunを含むsceneは明示failureとなる。Text表示要求を満たすにはTextService/glyph data/rendererの依存と順序を監査する必要がある。
- 従って、描画不能なTextを黙って捨てる、暫定色/文字描画を創作する、未対応sceneをPresented扱いする方式は採用しない。

## 根拠

- 要求: [RQ総目次](../11_requirements/reqirements-specification.md)、[RQ-09 UI](../11_requirements/RQ-09_ui-scope.md) RQ-09-001/003/004/006/007/067、[RQ-11 Platform](../11_requirements/RQ-11_platform.md) RQ-11-008/020/022/023。
- 設計: [UI Tree/Layout](../13_detail_design/ui/tree-layout.md)、[UI Controls](../13_detail_design/ui/controls.md)、[Platform internal layers](../13_detail_design/platform/internal-layers.md)。承認履歴と未決節を再確認し、既存承認範囲を変えない。
- 試験: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[UI unit tests](../32_unit_test/ui.md)、[implementation-test inventory](../32_unit_test/implementation-test-inventory.md)。新unit/testは承認後に登録する。
- 先行状態: [0.0.1 release plan](release-0.0.1.md)、[Platform backend plan](platform-first-backend.md)、Docker仮検証は`970752b`でcontainer内`provisional_pass`。これは実機・full UI合格ではない。

## 作業案

1. RQ本文・承認履歴、該当BD/DD全文、既存Core/API実装、test designとunit mappingを再確認し、RQ・設計・実装の差分表を作る。
2. 次を実装前に明示する。
   - Logical ControlとVisualNodeの対応、走査順、所有/参照とsnapshot時点
   - arrange済みboundsと祖先client clipの適用、Window contentのscene root
   - revision、damage、background color、`DeviceScale`の取得元とframe submit条件
   - Text/GlyphRun生成に必要なTextService/font/glyph資源と、Linux GLESで未対応の描画境界
3. 既承認DDだけで決められない実現仕様があれば、該当DDとUI unit-test designの文書全体改訂案を作成し、利用者レビューを経てから実装する。外部observableなText表示や他の要求自体が曖昧ならRQ差分を先に提示する。
4. design/testが承認されてから、独立して検証できる最小unitに分ける。想定順は(a) Visual traversal/幾何・Clip/scene snapshot、(b) TextService/GlyphRun生成と実描画のいずれか必要な側、(c) Control updateからframe再提出までの接続。境界を監査後に確定し、依存を飛ばさない。
5. 各unitについてRust Core testを同じrevisionで実行し、Docker仮検証を影響確認として再実行する。Docker仮PASSはhardware/profile PASSに昇格しない。

## 対象外

- Debian/RPi実機DRM/KMS操作、Platform Adapter hardware smoke、`/dev/dri` passthrough。
- 新しい公開C ABI signatureまたはlanguage wrapper API。Scene producerに必要と判明した場合は別の設計/API unitとして波及を提示する。
- OS keyboard/pointer routing、TextBox IME、Accessibility、package/publication、0.0.1 release判断。
- 要求されていないvisual default、placeholder text/color、software fallback、未対応GlyphRunを無視する挙動。

## 承認境界

- この計画0.1の承認は、差分監査と次のDD/test proposal作成を開始する許可とする。
- 新しいDD/test design本文は文書全体のレビュー・承認後に実装へ進む。実装開始指示、unit completion、commit/pushは別状態として記録する。
- 計画または設計案は利用者レビュー前にcommit/pushしない。承認済み実装unitの完了commitは既存repo運用に従う。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-10-29 | 計画0.1 | review draft | Platform Adapter確認を後回しにし、次の非Platform製品課題としてCore UI scene producer/Text描画依存を提案。利用者レビュー待ち |
