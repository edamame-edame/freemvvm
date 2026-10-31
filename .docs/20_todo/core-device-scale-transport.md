# Window DeviceScaleをCoreへ渡す経路

- 計画版: 0.1 改訂案
- 状態: 計画0.1・Platform DD 1.7・UI test 1.0を利用者承認済み。UNIT-CORE-083完了、commit/push済み（`cf3ce05`）
- 作成日: 2026-10-31
- 対象version: `0.0.1`
- 対象branch案: `impl-rust/core-scene-rendering`
- 親branch: `impl-rust/core-apis-window-background`
- 関連計画: [Core UI Scene生成の次工程](core-scene-production.md)

## 背景

`SceneSnapshot`とLinux rendererは`DeviceScale`を扱える一方、実際のWindow open経路では`PlatformWindowPeer::open`が`WindowClientArea`のみを返し、`WindowLifecycle`もscaleを保持しない。現在のScene生成testはscale `(1.0, 1.0)`を直接指定するため、承認済み方針「CoreがScaleをrendererへ渡す」はRuntime/Platform統合経路でまだ成立していない。

Linux KMS adapterは現在、選択modeのpixel extentを同じ数値のDIP client areaとして報告する。これは現在のadapter profileのscale sourceを明示した仕様ではなく、実機の物理密度やHiDPI挙動の確認でもない。値の由来・profile上の意味を暗黙のまま実装に焼き付けない。

## 目的

Window peerが環境値として供給するDeviceScaleをCoreが検証・所有し、以後のSceneSnapshot生成へ渡せる内部経路を設計・実装する。公開C ABIやlanguage wrapper APIは増やさない。client areaのDIP値とscaleを別々に保ち、rendererはscaleを推定しない。

## 推奨案

### 文書化フェーズ（先行）

1. 承認済み[Platform internal layers 1.4/1.5](../13_detail_design/platform/internal-layers.md)を維持したまま改訂案を作り、open時の環境recordへ`WindowClientArea`と`DeviceScale`を別フィールドとして含める契約を記述する。
2. scaleの供給元をPlatform adapter/profileとし、Coreはfinite positive値を検証してWindow generationに結び付けて保持する案を提示する。resize時にscaleも変化しうる場合のgeneration付き更新とdirty範囲（LayoutかScene/frameか）は設計上明示する。
3. 現Linux KMS profileについて、mode pixel数値をDIP数値として返す現行契約に整合する候補として明示scale `(1.0, 1.0)`を提案する。ただし実機のDPI推定を意味せず、将来のprofile変更・HiDPI設定で別scaleを供給可能とする。このprofile値は利用者が設計改訂案を承認するまで確定扱いしない。
4. `.docs/32_unit_test/ui.md`へ、異方性scaleのopen供給・Core保存・Sceneへの受け渡し、invalid scaleによるopen拒否/既存状態保持、generation違いとclose/reopen、DIP寸法への二重適用なしを検証するtest caseを提案する。
5. 文書全体を提示してレビュー・承認を受ける。承認後に実装unitを開始する。

### 実装フェーズ（文書承認後）

`UNIT-CORE-083: WindowOpenEnvironmentからSceneSnapshotまでのDeviceScale transport`を提案する。

- Rust内部SPIにclient areaとscaleを分離したopen environment recordを追加し、Factory adapterとLinux adapterの経路を接続する。
- `WindowLifecycle`にgeneration付きDeviceScaleを保持し、無効値・古いgenerationでは状態をcommitしない。
- RuntimeのScene生成経路へCore保有scaleを供給する。ScaleをScene builderやrendererが推測・既定化しない。
- Linux profileのscale値は承認された設計値のみを供給する。現在のKMS hardware/device validationはこのunitの合格に含めない。
- fake peer/platform testで異方性値・invalid値・close/reopen/stale generationを検証し、既存renderer testでsceneのscaleが保持・消費されることをつなぐ。

## 対象外

- 実機EDID/physical DPI推定、ユーザー設定可能なUI scaling、display hotplug/profile-selection policy。
- 公開C ABI・wrapper APIの追加。
- Window client area単位の既承認意味（DIP）変更。
- Scene producerのTextService、Control appearance、dirty-to-frame scheduling、およびLinux hardware smokeの完了扱い。

## 検証案

設計承認後、unit manifestへ実装・設計・要求・test traceabilityを登録する。実装時はfocused Core test、Core test suite、Linux target check/clippy、`tools/dev.py check`、`tools/dev.py minimum`を同一revisionで実行する。テスト仕様・コード変更は実装と同じrevisionで記録し、Docker/CI結果を実機表示合格へ読み替えない。

## 承認境界

- 現時点で依拠する要求・設計: [RQ-09-067](../11_requirements/RQ-09_ui-scope.md#rq-09-067)、[UI Tree/Layout 1.4](../13_detail_design/ui/tree-layout.md)、[Platform internal layers 1.4/1.5](../13_detail_design/platform/internal-layers.md)、[UI unit tests 0.9](../32_unit_test/ui.md)。
- この計画案の承認は、Platform DD/UI test改訂案の作成許可と推奨案への合意を意味する。設計改訂全文の承認、実装開始、Linux実機確認、commit/push、PRを自動承認しない。
- 実装branchはRust Core/Platform作業中の`impl-rust/core-scene-rendering`を継続し、wrapper言語branchへ移らない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-10-31 | 計画0.1 | 利用者承認 | Scale transportの不足、Rust内部SPI、Linux KMS現profileの明示1:1案を承認 |
| 2026-10-31 | Platform DD 1.7 / UI UT 1.0 | 利用者承認 | open environmentからCore-owned Scaleまでの経路、Linux KMS現profile 1:1案、fake peer test oracleを承認 |
| 2026-10-31 | UNIT-CORE-083 | 実装・local検証・commit/push完了 | Core 175 tests、Linux Platform 10 tests、workspace check/minimum、Linux target check/clippy pass。`cf3ce05`。Linux実機未確認 |
| 2026-10-31 | UNIT-CORE-083 commit/push | 完了 | `cf3ce05 WindowのDeviceScaleをCoreへ伝搬`を`impl-rust/core-scene-rendering`へpush |
