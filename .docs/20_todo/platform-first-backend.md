# 0.0.1 初期Platform backendの設計・実装計画

- 計画版: 0.1
- 状態: 計画全文を利用者が「ok」で承認（2026-10-26）。計画commit/push済み。DD・試験設計・実装開始は別承認
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
- 作業規則: [Git rule](../02_git/rule.md)、[Rust coding rule](../00_rule/coding_rule_Rust.md)、[implementation unit inventory](../32_unit_test/implementation-test-inventory.md)。

## 提案scope

- 最初の描画backendはRQ-11-014どおりLinux DRM/KMS + EGL/OpenGL ESとする。既存要求にないWayland/X11/software-renderer自動fallbackは追加しない。
- Rust Core、共通Renderer、Platform Adapter間の具体的trait/API、backend初期化と明示的unsupported/failure mapping、Window peer/surface所有、UI thread event/frame/shutdown handoffを承認済み抽象化へ落とす。
- まずWindow peerの生成・表示先接続・最小frame present・close/資源喪失処理を縦断可能にする設計とtest designを作る。実装unitは責務が独立して確認できる範囲に分ける。
- 対象profile候補はDebian 12 x64とRPi 4 AArch64実機。ARMv7は要求どおりcross-build確認に限り、他profileへsupportを拡張表示しない。
- Core fake adapter / mock deviceで境界・lifecycle・failureを自動検証し、Linux CI buildとRPi 4実機確認の合格条件・手順・必要な証跡を分ける。実機試験は対応機器上の実測が済むまで未実施と記録する。

## 対象外

入力routing、TextBox編集、keyboard/IME、Accessibility backend、text shaping、画像codec、Wayland/X11、software renderer、WebEngine、package/release、要求変更。これらは本unitの完了に含めず、後続計画で依存関係と承認状態を整理する。描画可能なVisual snapshotの形式やcontrol paint contractが既存承認文書で未確定なら、実装上の都合で補完せず設計差分としてレビューへ戻す。

## 実施順と承認境界

1. [x] Python wrapper完了ブランチから切り離し、親Core API branch `impl-rust/core-ui-controls-c-abi`から本計画レビュー用Rust/Core branchを作成した。別wrapperやWebEngineの差分は含めない。
2. [x] この計画0.1についてscope・profile・除外範囲・branch関係の利用者レビューを受け、「ok」により計画全文の承認を得た（2026-10-26）。設計・実装開始承認とは分けて記録する。
3. [ ] 実装前にRQ、ADR、BD、DD、test designと既存Core/C ABIの実装状態を照合し、未定義のCore/Adapter contract・Visual snapshot・frame presentation・resource failure semanticsを列挙する。
4. [ ] 必要なDD/test design文書の改訂案を作成し、各文書全体の利用者承認を得る。要求と設計に差異があれば、影響範囲と変更案の承認が先。新unit登録は承認済み設計・caseに対応させる。
5. [ ] 設計とunit-test contractの承認後、利用者の実装開始指示を得る。Core変更なので`impl-rust/platform-adapter-first-backend`でのみ実装し、他言語wrapper branchを親やmerge元にしない。
6. [ ] focused unit/consumer、Linux build、minimum toolchain、`tools/dev.py check`、document linksを実施し、同じrevisionへ結果を記録する。各実装unitを日本語commit/pushし、その実装SHAのWindows/Linux CI成功後にcompleteへ更新する。
7. [ ] Debian 12 x64とRPi 4 AArch64の実機/profile試験を実行し、実行環境、表示機器、driver/backend、report、SHAをrelease testへ記録する。CI build成功だけで実機確認を完了扱いしない。

## 完了条件

- Platform adapter/renderer/Coreの具体的契約と試験設計が文書全体で承認されている。
- Linux DRM/KMS + EGL/OpenGL ESの明示的backend初期化、Window peer/surface、minimum frame、close/resource failureの実装がCore UI意味論をOS resourceへ結合せず動作する。
- fake境界試験、Linux CI/build、最低Rust toolchainと同一実装revisionのWindows/Linux CIが成功し、実機確認はprofile別に記録される。
- software renderer等への暗黙fallback、未確認targetへのsupport拡大、実機未確認の合格扱いがない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-26 | 計画0.1 | 利用者承認 | 利用者「ok」によりscope・profile・除外範囲・branch関係を含む計画全文を承認。計画のcommit/pushを許可。DD承認・実装開始・release判断は別ゲート |
