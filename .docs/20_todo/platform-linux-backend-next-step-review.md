# Linux Platform backend実装unit分割・契約具体化案

- 提案版: 0.1
- 状態: DD v1.3 / test design 0.5全文承認済み。レビュー案・計画差分は承認済み文書として未コミット
- 対象計画: [0.0.1 初期Platform backend](platform-first-backend.md)
- 対象branch: `impl-rust/platform-adapter-first-backend`
- 作成日: 2026-10-26

## 背景

親計画とPlatform DD 1.2 / UI test design 0.4は承認済み。`UNIT-CORE-073`と`UNIT-CORE-074`は実装・local検証・同一SHAのWindows/Linux CIが完了した。次に登録されている`UNIT-PLATFORM-001`はLinux DRM/KMS・GBM・EGL/GLES present/resource lossを一つのunitとしている。

着手前の実装照合で、承認済み要求/DD/test designの意図は明確だが、動作を決める具体的な契約が未確定であることを確認した。

| 未確定の境界 | 現状 | 実装上の影響 |
| --- | --- | --- |
| Runtimeへのbackend注入 | Coreに`PlatformRuntimeFactory`とWindow peer open/closeはある。Linux target用factoryをC ABI Runtime作成へ注入する接続はない | Linux crate単体を作っても実RuntimeのWindowから起動されない |
| frame submission/completion | `SceneSnapshot`と`FrameTracker`はあるが、Runtime/LayoutからRendererへsceneを渡すSPI、非同期completionをUI dispatcherへ戻すAPIがない | GPU submit/presentをCoreのPresented stateへ安全に反映できない |
| glyph描画データ | `GlyphRunSnapshot`はfont digest/size/glyph id/placementを保持するだけで、font face・glyph bitmap/atlas・texture等の描画資源を持たない | 初期DrawOpのGlyphRunを実際のGLES rendererが描画できない。layout/text shapingとの接続も未実装 |
| DRM device/connector選択 | `/dev/dri/card*`の走査、接続済みconnector、mode選択の規則がない | 複数GPU/display、接続なし、権限不足時の結果が環境ごとに不定になる |
| native dependency/link | Cargo/system libraryの選択、必要system package、runtime依存、cross-build sysroot条件が未登録 | build成功と実行時load可否の境界を定義できない |
| failure観測 | 公開Platform SPIの`PlatformOperationError`はopaqueで、Window peer failureは既存Coreで`UI_INTERNAL`に集約される | device loss/recoverabilityと診断詳細をどの境界で保持するか未確定 |

## 推奨分割

大きな既承認目標は維持し、契約確定と実装検証を独立した小unitへ分ける。以下は提案であり、承認前の確定仕様・実装ではない。

1. **Core Platform render/event seam**: Runtime-owned submission API、SceneSnapshotとFrameTicketの受渡し、backend completionをCore dispatcher経由で処理する契約、failure/stale completion testsを定義・実装する。公開C ABI signatureは変更しない。
2. **Linux device/surface lifecycle**: Linux target向けfactory注入、DRM node/connector/mode選択、GBM/EGL surface作成・close・再試行・resource lossをfake境界で検証する。描画可能なsceneとの接続前にhardware rendering完了を主張しない。
3. **Linux renderer/present**: GLES resource、対応DrawOpの描画、page-flip/present completionとFrameTicketの統合、failure recoveryをdevice fake testsとLinux buildで検証する。GlyphRun対応はTextServiceが消費可能なglyph representationが確定してから実装する。
4. **Hardware profile evidence**: Debian 12 x64 / RPi 4 AArch64の実画面・device access結果は既承認system/profile testで別に記録する。

unit ID、source path、依存順、test designを確定する際は`tools/units.yaml`、`.docs/32_unit_test/ui.md`、implementation inventoryを同じレビューで更新する。

## 技術案（レビュー用）

### DRM/GBM/EGL/GLES bindings

- DRM/KMS Rust binding候補: `drm` crate 0.14.0。`gbm` 0.18.0のoptional `drm-support`依存と版を合わせる。Linux DRM crateはdevice-node/ioctlを直接扱いlibdrmを経由しない。
- GBM候補: `gbm` crate 0.18.0。`gbm-sys`を介するため、必要なsystem development libraryとtarget sysroot条件をCI/profileへ宣言する。
- EGL候補: `khronos-egl` 6.0.0のdynamic feature。`libEGL.so.1`をruntime loadできるが、GBMを含むすべてのnative dependenciesがWindow open時まで遅延されるという意味ではない。
- GLES function loading、Cargo target cfg、Debian/ARM sysrootの具体的組合せは、依存packageとCI build結果を確認して採用する。

参照: [drm crate 0.14](https://docs.rs/drm/0.14.0/drm/)、[gbm crate 0.18](https://docs.rs/gbm/0.18.0/gbm/)、[khronos-egl crate 6](https://docs.rs/khronos-egl/6.0.0/khronos_egl/)。`gbm` 0.18.0のmanifestにあるoptional `drm`依存も確認した。これらの版・feature情報は2026-10-26時点で、Cargo.lock確定時に再確認する。

### 推奨する初期device policy

- 初期profileはsingle active outputと明記する。
- node候補は`/dev/dri/card*`をnumeric suffix順で調べ、DRM/KMS resourceが利用可能で、少なくとも一つのconnected connectorとmodeset可能なmodeを持つ最初のnodeを選ぶ。
- connectorはconnectedのうちpreferred modeを持つ最小connector IDを選ぶ。preferred modeがなければ選択できる最大pixel area、同面積なら最小connector IDとする。
- 選択できるnode/outputがない、またはpermission deniedの場合はWindow openを失敗させる。Wayland/X11/softwareへの暗黙fallbackはしない。
- この自動単一output policyは初期実装案に限り、複数Window/複数display/明示選択configは対象外とする。

### 推奨するfailure境界

- 未接続profile/factoryは既存規則どおり`UI_UNSUPPORTED`。
- 接続済みLinux factoryでnative library/device/permission/output/surface初期化に失敗した場合はWindow open failure（現Core mappingでは`UI_INTERNAL`）。Core Window/Tree/Propertyは維持し、同Windowで明示retry可能。
- Backend内ではfailure kindとnative diagnosticを保持し、公開C ABI statusを増やさない。
- present failure/device lossではFrameをPresentedにせずCore stateを巻き戻さない。surface generationを更新し旧generation completionを破棄する。

## 確認した実装・環境制約

- 開発hostにはWindows target toolchainのみがinstalled。WSL commandはDocker Desktop管理下のdistributionだけを公開し、一般用途のLinux build environmentは用意されていない。
- GitHub ActionsにはUbuntu runnerがあり、現bootstrap `check`はRust/Linux smoke相当を実行するが、DRM/GBM/EGL development librariesのinstallやLinux platform crate buildはまだ行わない。
- よって次unitは少なくともUbuntu CIへnative development dependencyを追加し、Rust minimum/toolchainとLinux crate buildを同revisionで確認する必要がある。RPi実画面動作はCI passと区別する。

## レビュー承認

- [x] 2026-10-26: 利用者は本レビュー案の作成を承認。これは下記文書の内容承認、unit計画の確定、実装開始の承認ではない。
- [x] [Platform DD v1.3](../13_detail_design/platform/internal-layers.md)と[UI test design 0.5](../32_unit_test/ui.md)を全文レビューし、利用者の「ok」（2026-10-26）で承認。
- [x] 推奨案を採用: Core render/completion seamを先行し、device policy・failure境界・bindings候補とunit splitをunit計画へ反映。
- [ ] 別案: device selection、native dependency/link、GlyphRun/rendererの対象範囲を指定してから両文書を改訂する。

この承認は設計・試験設計の文書全体およびunit/test/source mappingを確定する。実装開始は別の明示指示を受けてからとし、承認前のDD 1.2 / test design 0.4は履歴上のbaselineとして保持する。
