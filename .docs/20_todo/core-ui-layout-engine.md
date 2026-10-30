# Core UI Layout実行基盤

- 計画版: 0.1
- 状態: 承認済み。UNIT-CORE-076/077/078完了。Layout計算基盤は完了し、Scene producerは別計画へ移行
- 作成日: 2026-10-29
- 対象version: `0.0.1`
- 対象branch案: `impl-rust/core-ui-layout-engine`（現Core APIs branch `impl-rust/platform-adapter-first-backend` の子。Rust Core/C ABIだけを対象）
- 前提: Platform Adapter実機確認は別todoへ保留。本計画はAdapter実機・wrapper変更を含めない

## 目的

承認済みUI要求の配置結果を、Coreが実際にMeasure/Arrangeして同一revisionの描画・入力結果へ渡せる状態にする。Scene producerより先に、root利用可能領域・各Controlの測定入力・公開設定とLayout結果の受け渡し契約を決め、既存設計の概念記述と現状コードの間を埋める。

## 監査結果

### 要求・設計・テスト

- RQ-09-007/008/045/057/067は配置方向、位置・サイズの決定、Align既定値、数値拒否、祖先client clipを要求する。RQ-09-004/072はText表示の外部範囲を定めるが、intrinsic size/font-metrics計算は定めない。
- 承認済み[UI Tree/Layout DD](../13_detail_design/ui/tree-layout.md) 1.2にはMeasure/Arrange、StackPanel計算、dirty伝播、原子的LayoutSnapshotの意味論と`LayoutEngine`/`Sizer`/`Locationer`案がある。
- 承認済み[UI controls DD](../13_detail_design/ui/controls.md) 1.1にはWindow/StackPanel/Textの型・操作案があるが、Textの実測値算出とroot領域をLayoutへ渡す実行契約は具体化されていない。
- [unit test design](../32_unit_test/ui.md) 0.6にはUT-LAYOUT-001/UT-LAYOUT-ABI-001がある。想定oracleはあるが、現コード上で呼ぶMeasure/Arrange実装がない。0.7案では計算kernel、root area、Text measurement、atomicityのcaseへ具体化した。

### 現状実装

- `ui_controls.rs`にはControl kind、StackPanel orientation/child Align、`layout_dirty`がある。Tree操作と設定変更はdirty化する。
- `runtime.rs`/`lib.rs`にあるLayout関係APIはdirty query/cleanとorientation/Align設定であり、Measure/Arrange、DesiredSize、FinalBounds、LayoutSnapshotは未実装。
- C ABI headerにもroot client size、Control size constraints、Measure/Arrange結果を受け渡す関数は見当たらない。公開APIを増やすか、host/Platform内部protocolに閉じるかは未決。
- Window peerはframeにSceneSnapshotを受け取る一方、現状のpeer contractからroot client領域をCoreへ返す接点は確認できない。
- Textのfont shapingはDD上のTextService案にとどまり、Coreにmeasurement実装はなく、font asset delivery/glyph rendererは未完成。GlyphRunを含むsceneをLinux rendererは拒否する。

## 設計改訂で決める範囲

既承認RQの外部保証は変更せず、既承認Tree/Layout DD 1.2、Control DD 1.1、Platform internal-layers DD 1.4とUT 0.6を文書全体改訂案として更新した。作成案はTree/Layout 1.3、Control 1.2、Platform internal-layers 1.5、UT 0.7。要求にない外部挙動が必要になればRQ差分を先行提案する。

1. **Root領域の供給**: Window client areaのDIP値のsource、open/resize時の更新、未確定・zero・invalid値、閉じたWindowと再open時の扱い。PlatformからCoreへの責務境界とテスト可能なinternal entry point。
2. **Leaf測定値**: Control constraintsの意味と初期利用者設定範囲。Text/Button/TextBoxのDesiredSizeを何のfont/text metricsから得るか。TextService導入前のplaceholder/fallback寸法は作らない。
3. **Layout契約**: Measure/ArrangeのUI-thread入口、dirty消費、候補結果のatomic commit、失敗時の旧snapshotとretry、revisionとLogical/Visual node対応。
4. **C ABI境界**: 初期案ではclient area、任意constraint setter、bounds観測を公開C ABIに追加せず、root領域とLayoutSnapshotをCore/Platform内部に閉じる。RQにないサイズ設定APIが必要なら要求差分・ABI改訂を別提案する。
5. **描画接続との順序**: layout済みbounds/祖先clipをscene producerへ渡す契約。Text/GlyphRun生成と実描画はfont identity/color/package未決のためLayout単位に混ぜず、別依存として扱う。
6. **Test oracle**: 空/複数children、両orientation、constraint、Align/maximum、root領域変更、overflow/invalid input、部分失敗atomicity、dirty消費と同revision boundsをfake root/leaf providerで検証する範囲。

## 実装の想定分割（設計承認後に確定）

1. Core内部の制約・寸法・配置計算kernelと原子的LayoutSnapshot。必要ならleaf measurementをtest seamとして注入し、製品のfont fallbackを仮定しない。
2. Window/Control treeとのLayoutEngine接続、dirty propagation/consume、root領域変更からbounds snapshot確定まで。要求/設計でC ABI surfaceが必要と確定した場合だけこのunitまたは独立C ABI unitに含める。
3. scene producerはLayout snapshot接続後の別unit。Text表示を含むsceneを完結とする前にTextService/glyph asset/render contractを先行承認・実装する。

各unitのsources、dependencies、同一revisionのunit test/cargo check/Clippy/CIを`.docs/32_unit_test/implementation-test-inventory.md`と`tools/units.yaml`へ登録してから着手する。実装開始・commitはこの設計案のレビュー承認とは別の承認状態として扱う。

## 参照

- 要求: [RQ index](../11_requirements/reqirements-specification.md)、[RQ-09 UI](../11_requirements/RQ-09_ui-scope.md)、[RQ-11 Platform](../11_requirements/RQ-11_platform.md)。要求の承認履歴と状態は作業開始時に再確認する。
- 設計: [UI Tree/Layout DD](../13_detail_design/ui/tree-layout.md)、[UI controls DD](../13_detail_design/ui/controls.md)、[Platform abstraction DD](../13_detail_design/platform/abstraction.md)、[Platform internal-layers DD](../13_detail_design/platform/internal-layers.md)、[ADR-0001](../04_approval/adr-0001-rust-core.md)。
- 試験: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[UI unit tests](../32_unit_test/ui.md)、[implementation-test inventory](../32_unit_test/implementation-test-inventory.md)。
- 先行監査: [Scene production audit](core-scene-production.md)。

## 対象外

- Linux/RPi実機DRM/KMS確認、Adapter hardware smoke、OS input/IME/accessibility、wrapper言語の変更。
- 未承認のC ABI追加、font fallback/placeholder size、GlyphRun未対応時の描画成功扱い。
- TextService/font package/GlyphRun renderer全体の実装。Layout改訂時に依存条件を記録し、別計画へ切り出す。

## 承認境界

- 本文はLayout基盤の作業計画であり、0.1全文承認済み。DD/UT改訂案の承認、各unitの実装承認はそれぞれ別の履歴として管理する。
- DD/UT改訂案は文書全体のレビュー・承認後に実装へ進む。C ABI追加やRQ改訂が必要と判明した場合は根拠と差分を同じレビューへ提示し、設計だけで要求を拡張しない。
- 計画のcommit/pushは利用者レビュー後。実装commitは承認済みunit完了後に既定運用で行う。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-29 | 計画0.1 | 承認済み | Layout実行本体とroot/leaf入力契約をScene producerに先行させる計画。利用者承認後にDD/UTを全文改訂し、実装開始が承認された |

## 実施状況

- 2026-10-29: 承認済みDD 1.3 / Control DD 1.2 / Platform internal-layers DD 1.5 / UT 0.7に基づき、計画unit 1のCore内純粋Layout kernel（`UNIT-CORE-076`）を実装・検証した。
- immutable tree snapshotを入力にMeasure/Arrangeし、成功時にのみ新しいimmutable LayoutSnapshotを返す。失敗時は呼出し側が保持する既存snapshotを変更しない。
- focused 4 tests、Core全164 tests、Clippy、Rust 1.85 check、format checkがpass。実行SHAと詳細はunit完了記録に登録する。
- 次はWindow client-area入力を独立unit化し、その後に実Control tree snapshot構築・LayoutSnapshot生成をつなぐ。TextService計測とScene producerは別境界とし、Platform Adapter実機確認は保留を維持する。
- 2026-10-29: 大きな統合範囲を実装確認可能な単位に分け、`UNIT-CORE-077`ではWindow client-area初期値とgeneration付き更新のCore input protocolを先行する。実Control tree snapshot統合はこの単位の外とし、設計範囲は変えない。
- 2026-10-29: `UNIT-CORE-078`で実Control treeからのLayout入力構築、Window単位のimmutable snapshot公開、測定失敗/測定中入力変更時の旧snapshot保持を実装・検証した。TextService/production leaf metricsとScene producerは未実装で、別計画の依存として維持する。
