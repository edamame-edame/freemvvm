# 0.0.1 初期Platform backendの設計・実装計画

- 計画版: 0.2
- 状態: 計画0.2 / DD v1.4 / UI test design 0.6は2026-10-27に利用者承認済み。UNIT-PLATFORM-002実装・local検証済み、同revision CI未実施
- 作成日: 2026-10-26
- 対象version: `0.0.1`
- 対象branch案: `impl-rust/platform-adapter-first-backend`
- 親branch: `impl-rust/core-ui-controls-c-abi`（`version/0.0.1`由来）。別言語wrapper branchやRust inner WebEngine branchはmergeしない
- 現branch: 本計画のレビュー用に上記親branchから作成済み

## 目的

承認済み要求が最初に検証する描画backendとして定めるLinux DRM/KMS + EGL/OpenGL ESを、既存Rust Core UI Tree/C ABIから独立したPlatform Adapterとして具体化し、Window peerと最小の表示frameへつなぐ。UI libraryとして使える状態を目指す0.0.1計画のPlatform基盤unitであり、PyPI等での公開やrelease判定は行わない。

## 根拠と着手時の再確認

- 要求: [RQ-09 UI範囲](../11_requirements/RQ-09_ui-scope.md)、[RQ-11 Platform](../11_requirements/RQ-11_platform.md) RQ-11-001～004、008～009、014、019～024、[RQ-06 thread/dispatch](../11_requirements/RQ-06_threading-dispatch.md)、[RQ-12 build/distribution](../11_requirements/RQ-12_build-distribution.md)。要求本文・承認履歴は実装開始前に再確認する。
- 上位設計: [Platform abstraction](../13_detail_design/platform/abstraction.md) v1.1、[Platform internal layers](../13_detail_design/platform/internal-layers.md) v1.1、[Runtime](../13_detail_design/runtime-objects/runtime.md)、[UI Tree/Layout](../13_detail_design/ui/tree-layout.md)、[共通C ABI](../13_detail_design/runtime-objects/abi.md)。文書全体の承認状態と具体化済み/未決事項を再確認する。
- 試験: [UI単体試験](../32_unit_test/ui.md)、[wrapper単体試験](../32_unit_test/wrappers.md)、[Linux profile integration](../30_release_test/system-scenarios.md)、[unit-test strategy](../32_unit_test/unit-test-strategy.md)。Core fake adapter試験と実機profile確認を別結果として記録する。
- 作業規則: [Git rule](../02_git/rule.md)、[Rust coding rule](../00_rule/coding_rule_Rust.md)、[implementation unit inventory](../32_unit_test/implementation-test-inventory.md)。実装上の注意は[Platform Linux backend memory](../01_memory/platform-linux-backend.md)を参照。

## 提案scope

- 最初の描画backendはRQ-11-014どおりLinux DRM/KMS + EGL/OpenGL ESとする。既存要求にないWayland/X11/software-renderer自動fallbackは追加しない。
- Rust Core、共通Renderer、Platform Adapter間の具体的trait/API、backend初期化と明示的unsupported/failure mapping、Window peer/surface所有、UI thread event/frame/shutdown handoffを承認済み抽象化へ落とす。
- UNIT-PLATFORM-002ではCoreが正の有限`DeviceScale`と各成分0.0～1.0の正規化RGB背景色をSceneSnapshotへ渡し、Linux GLES rendererはDIP座標をpixelへ変換し、各frameの未描画領域を指定色でclearする。Clipは可視範囲だけを制限し子のlayout/PAINT semanticsを変えない。Clipは現在のeffective clipを置換し、Scene側が祖先clipを合成して渡す提案。
- まずWindow peerの生成・表示先接続・最小frame present・close/資源喪失処理を縦断可能にする設計とtest designを作る。実装unitは責務が独立して確認できる範囲に分ける。
- 対象profile候補はDebian 12 x64とRPi 4 AArch64実機。ARMv7は要求どおりcross-build確認に限り、他profileへsupportを拡張表示しない。
- Core fake adapter / mock deviceで境界・lifecycle・failureを自動検証し、Linux CI buildとRPi 4実機確認の合格条件・手順・必要な証跡を分ける。実機試験は対応機器上の実測が済むまで未実施と記録する。

## 要求・設計・現行Coreの差分監査

2026-10-26に、承認済みRQ-11 v0.5、Platform DD v1.1、共通C ABI DD v1.3、現行`impl-rust/core-ui-controls-c-abi`のCore実装を照合した。

- 要求はWindows 11 x64、Debian 12 x64、ARMv7/AArch64 cross-build、代表RPi 4 AArch64をsupport profileとし、Linux描画backendはDRM/KMS + EGL/OpenGL ESを最初に検証する（RQ-11-014、RQ-11-024）。DD v1.1はLinux経路をGBM/EGL/OpenGL ES 3として具体化済みで、新backendの選定から始める必要はない。
- 共通C ABI DDは`window_open/close`と失敗時のCore Window保持を定義する。Rust Coreにはprivate `WindowPeerAdapter`の`open/close` lifecycle seamとfake peer試験がある一方、production peer factory/Runtimeへのadapter接続はなく、`window_install_test_peer`はtest-only。peer未登録の`window_open`はUnsupportedとなる。
- Platform DDはSceneSnapshot（Clip/FillRect/GlyphRun）、frame/surface state、Renderer/Adapter所有境界を定義するが、該当するScene/DrawOp型、production Renderer、frame present経路はCore sourceにまだない。
- Runtime DDはhost `UiLoopBridge`とUI-thread dispatchを定義する一方、DRM/KMS event collection・frame completionをそのUI turnへ届ける具体的integration seam、Runtime起動時profile/factoryとdevice接続失敗の境界はPlatform/Coreをまたぐ詳細提案が必要。
- よってDD案では、既承認のbackend/profile選択を再判断せず、Runtime/WindowからLinux adapter、scene/frame、UI-thread event/present/shutdownへの接続契約を具体化する。新しいC ABIや言語wrapper変更が必要となる場合は差分と波及先を別に示し、承認前に実装しない。

## 対象外

入力routing、TextBox編集、keyboard/IME、Accessibility backend、text shaping、画像codec、Wayland/X11、software renderer、WebEngine、package/release、要求変更。これらは本unitの完了に含めず、後続計画で依存関係と承認状態を整理する。描画可能なVisual snapshotの形式やcontrol paint contractが既存承認文書で未確定なら、実装上の都合で補完せず設計差分としてレビューへ戻す。

## 実施順と承認境界

1. [x] Python wrapper完了ブランチから切り離し、親Core API branch `impl-rust/core-ui-controls-c-abi`から本計画レビュー用Rust/Core branchを作成した。別wrapperやWebEngineの差分は含めない。
2. [x] この計画0.1についてscope・profile・除外範囲・branch関係の利用者レビューを受け、「ok」により計画全文の承認を得た（2026-10-26）。設計・実装開始承認とは分けて記録する。
3. [x] 実装前にRQ、ADR、BD、DD、test designと既存Core/C ABIの実装状態を照合。未定義のRuntime/profile/factory接続、production WindowPeer、Visual snapshot/Renderer、UI-thread event/frame completion integrationを列挙した（監査結果は本節）。
4. [x] Platform DD v1.2とUI unit-test design 0.4を作成し、2026-10-26に利用者が文書全体を承認。要求の変更はなく、新unit 3件を承認済み設計・caseに対応させて`tools/units.yaml`へplanned登録した。実装・実行は未着手。
5. [x] DD/test design承認後、利用者の「ok」（2026-10-26）を実装開始指示として受けた。Core変更は`impl-rust/platform-adapter-first-backend`で実施し、他言語wrapper branchを親やmerge元にしない。
6. [x] UNIT-CORE-073/074はそれぞれfocused unit、`tools/dev.py check`、`tools/dev.py minimum`、同一実装SHAのWindows/Linux CIを完了し記録した。Linux platform crate/buildはまだ存在せず、この結果には含まれない。
7. [x] 未確定のunit split・Core render/completion SPI・DRM policy・native dependency・GlyphRun inputをDD v1.3とUI test design 0.5へ反映。`tools/units.yaml`とimplementation inventoryも同じ提案にそろえた。
8. [x] Platform DD v1.3とUI test design 0.5を文書全体で承認。利用者の別指示「ok」（2026-10-26）でUNIT-CORE-075の実装を開始。
9. [x] UNIT-CORE-075を実装し、focused test・`tools/dev.py check`・`tools/dev.py minimum`をlocal pass。実装SHA `e3d3acdd56d501c208505719b7d7d340e6314454` の[Windows/Linux CI run 40326835471](https://github.com/edamame-edame/freemvvm/actions/runs/40326835471)もsuccessしunitをcompleteへ更新。
10. [x] UNIT-PLATFORM-001を実装。DRM card/connector/mode選択、GBM/EGL surface lifecycle、Linux専用C ABI factory composition、fake boundary testsを追加。Windows local `cargo check/clippy`とfocused fake lifecycle test pass。Linux CIのfocused 5 tests、`tools/dev.py check`、`tools/dev.py minimum`とWindows CI gateが同一SHA `b53f0f59cdc3569094ab770d719dff20b00ae6ed` でpass（[CI run 40330710176](https://github.com/edamame-edame/freemvvm/actions/runs/40330710176)）。Linux CIに必要な`libdrm-dev`/`libgbm-dev`を追加。描画/presentと実機確認は後続。
11. [ ] UNIT-PLATFORM-002はCore DeviceScale/背景色、GLES Clip/FillRect、DRM CRTC/page-flip、worker completion/lossの範囲で実装中。計画0.2 / DD v1.4 / UI test design 0.6は全文承認済み。Core 160 tests、Linux adapter 10 tests、Linux target cross-check/Clippy、`tools/dev.py check`、`tools/dev.py minimum`がlocal pass。同一revision CIを実施してからunitを完了へ更新する。実機profile確認は後続条件として残す。
12. [ ] Debian 12 x64とRPi 4 AArch64の実機/profile試験を実行し、実行環境、表示機器、driver/backend、report、SHAをrelease testへ記録する。CI build成功だけで実機確認を完了扱いしない。

## 完了条件

- Platform adapter/renderer/Coreの具体的契約と試験設計が文書全体で承認されている。
- Linux DRM/KMS + EGL/OpenGL ESの明示的backend初期化、Window peer/surface、minimum frame、close/resource failureの実装がCore UI意味論をOS resourceへ結合せず動作する。
- fake境界試験、Linux CI/build、最低Rust toolchainと同一実装revisionのWindows/Linux CIが成功し、実機確認はprofile別に記録される。
- software renderer等への暗黙fallback、未確認targetへのsupport拡大、実機未確認の合格扱いがない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-26 | 計画0.1 | 利用者承認 | 利用者「ok」によりscope・profile・除外範囲・branch関係を含む計画全文を承認。計画のcommit/pushを許可。DD承認・実装開始・release判断は別ゲート |
| 2026-10-26 | Platform DD 1.2 / UI unit-test design 0.4 | 利用者承認 | 利用者「ok」により両文書全体を承認。planned unit 3件を台帳へ登録。実装開始・試験実装/実行・実機確認は別ゲート |
| 2026-10-26 | 実装開始 / UNIT-CORE-073 | 利用者承認 | DD/test design承認後の利用者「ok」を実装開始指示として扱う。対象はCore factory/Window peer unit。後続unitの実装承認範囲を先取りしない |
| 2026-10-26 | UNIT-CORE-073 / `153a0d07` | 実装・試験完了 | focused 2 tests、`tools/dev.py check`、`tools/dev.py minimum`、[Windows/Linux CI run 40314950652](https://github.com/edamame-edame/freemvvm/actions/runs/40314950652) success。Linux target artifact/backend本体と実機profile確認は後続 |
| 2026-10-26 | UNIT-CORE-074 | 実装・local検証済み、CI待ち | Immutable SceneSnapshotとgeneration/frame/revision付きframe trackerを追加。focused 5 tests、Clippy、`tools/dev.py check`、`tools/dev.py minimum` local pass。Layout/ControlからのScene生成・production Renderer・device presentは対象外 |
| 2026-10-26 | UNIT-CORE-074 / `5837a44c` | 実装・試験完了 | [Windows/Linux CI run 40316650420](https://github.com/edamame-edame/freemvvm/actions/runs/40316650420) success。Layout/ControlからのScene生成・production Renderer・device presentと実機profile確認は後続 |
| 2026-10-26 | Linux backend next-step proposal | 利用者承認（draft作成のみ） | 利用者の「ok」により未確定Platform backend契約をDD/test design review draftへ具体化する作業を承認。新draft全文承認・unit implementation開始は別ゲート |
| 2026-10-26 | Platform DD 1.3 / UI unit-test design 0.5 | レビュー待ち | review draftを作成し、inventory・unit ledgerへplanned提案を反映。利用者のdraft作成承認は文書内容の承認や実装開始を意味しない |
| 2026-10-26 | Platform DD 1.3 / UI unit-test design 0.5 | 利用者承認 | 利用者の「ok」により両文書全体を承認。planned unit/test mappingを確定。実装開始・試験合格・実機確認は別ゲート |
| 2026-10-26 | UNIT-CORE-075 実装開始 | 利用者承認 | 文書承認後の別の「ok」を実装開始指示として扱い、Core render submission/completion seamへ着手 |
| 2026-10-26 | UNIT-CORE-075 / `e3d3acdd` | 実装・試験完了 | focused 1 test、Core Clippy、`tools/dev.py check`、`tools/dev.py minimum` local pass。[同SHA Windows/Linux CI run 40326835471](https://github.com/edamame-edame/freemvvm/actions/runs/40326835471) success。scene producer・Linux renderer・実機描画は後続 |
| 2026-10-26 | UNIT-PLATFORM-001 / `b53f0f59` | 実装・試験完了 | Linux focused 5 testsとWindows/Linux bootstrap gateが同SHA CI run 40330710176でsuccess。DRM/GBM/EGL surface初期化・close/explicit retry境界まで。描画/presentとDebian/RPi実機確認は対象外 |
| 2026-10-27 | 計画0.2改訂案 / DD 1.4 / UI test design 0.6 | review draft | CoreからScale/背景色を渡す指示とClipの意味を反映。変更したplan/design/test全文はcommit前レビュー待ち。利用者の実装継続指示に基づきコード作業中 |
| 2026-10-27 | 計画0.2 / DD 1.4 / UI test design 0.6 | 利用者承認 | 利用者の「ok」によりCore Scale/背景色、Clip境界、GLES subset、DRM page-flip lifecycle、試験oracleを含む文書全体を承認。UNIT-PLATFORM-002実装の同revision CI・実機確認は別管理 |
