# 0.0.1 Windows Platform Adapter 実装計画

- 計画版: 0.1
- 状態: 計画承認済み（利用者、2026-10-31）。実装開始・試験合格は別ゲート
- 対象version: `0.0.1`
- 対象: Windows x64 の Win32 Window peer と Direct2D/DirectWrite 描画接続
- 実装branch案: `impl-rust/platform-adapter-windows`
- 親Core APIs branch: `impl-rust/core-apis-window-background`
- 対象外branch: 各言語wrapper、Linux backend、Rust inner WebEngine

## 目的

[0.0.1 release計画](release-0.0.1.md)のWindows x64 UI縦断対象を満たすため、Windows側でCore Windowを明示openし、承認済み共通SceneをOS Windowへ描画できるPlatform Adapterを実装する。Linux DRM/KMS backendから実装を流用・mergeせず、共通Core/Renderer SPIを使う別backendとして構成する。

実Windowの生成・表示・close/reopen・frame presentationはWindows実機または実行可能なWindows GUI環境の手動確認とする。CIでは可視Window起動を要求せず、target build、fake境界試験、headless consumer等の安全な検証を継続する。Windows実機未確認をCI結果で代替しない。

## 根拠

- 要求: [UI機能範囲](../11_requirements/RQ-09_ui-scope.md)、[Platform・target](../11_requirements/RQ-11_platform.md)、[thread・dispatch](../11_requirements/RQ-06_threading-dispatch.md)。特にWindows x64の初期targetおよび初期UI縦断確認を定めるRQ-11-004、Windows 11 x64を含むprofileのRQ-11-024に従う。
- 上位計画: [0.0.1 release計画](release-0.0.1.md)の完了条件および「Platform、描画、入力、Accessibility」段階。
- 設計: [Platform abstraction](../13_detail_design/platform/abstraction.md)、[Platform internal layers](../13_detail_design/platform/internal-layers.md) 1.6。Windows peerはWin32、描画はDirect2D/DirectWriteを使い、Core Window/Sceneをsource of truthとする。既存のCore/Renderer SPI、peer generation、client-area、resize、frame completion、close/shutdown orderingを維持する。
- 試験: [UI単体試験](../32_unit_test/ui.md) 0.9、[system scenarios](../30_release_test/system-scenarios.md)、[requirement-test matrix](../30_release_test/requirement-test-matrix.md)。fake adapterのunit境界とWindows実profileの縦断確認を分ける。
- Git・実行規則: [Git branch rule](../02_git/rule.md)、[unit-test strategy](../32_unit_test/unit-test-strategy.md)、[implementation inventory](../32_unit_test/implementation-test-inventory.md)。

## 実施範囲案

1. 実装着手時にRQ/ADR/承認済みDD/test designとCore APIs branchを再確認し、Windows実装が既存SPIで満たせるかを監査する。新しい外部保証・公開ABI・OS選択は追加しない。承認済み設計と実装境界に差があれば、実装を止めて差分とレビュー対象文書を提示する。
2. target-specific factory compositionからWindows peerをRuntimeへ注入し、`Window.open()`、初期client area、resize通知、明示closeと再open、generation・失敗・shutdown境界を実装する。Runtime生成時に可視Windowを暗黙起動しない。
3. 共通SceneSnapshotからWindows surfaceへ描画するrendererを接続する。承認済みDrawOpと共通Clip/FillRect/GlyphRun契約の対応を保ち、frame submitとPresented completionを区別する。新規software fallbackやWindows固有のControl意味論は追加しない。
4. Core SPI/factory/rendererの失敗・lifecycleをfake可能な境界で単体検証する。通常CIではWindows targetのbuildと決定的fake試験を実施するが、可視Window生成や実機描画の成功をCI gateにしない。
5. Windows 11 x64実行環境で手動profile smokeを行い、実Window open、初期surface/frame presentation、close/reopen、失敗時の再試行可能性を確認する。OS/build/profile、GPU/driver、SHA、log/reportを記録し、release scenarioへの対応を明示する。
6. 試験設計とunit台帳への登録を実装前に行う。具体unit分割・TC assertion・コマンドは現行sourceと承認済みtest design照合後に確定し、Windows手動確認はCI unitとは別の実行記録にする。

## CIと実機確認の境界

| 検証 | 実施場所 | 扱い |
| --- | --- | --- |
| Windows target compile/link、Core/fake-adapter unit、headless-safe consumer | 通常CI | 自動実行する。Windowが実際に表示された証拠とはしない |
| Win32 visible peerのopen/close、Direct2D surface上のframe presentation | Windows GUI実機profile | 専用の手動smokeとして実行・記録する。CIから切り離す |
| Windows x64 UI縦断とpackage consumer | 0.0.1 release qualification | 対象profileで実施し、unit/CI結果で代替しない |

## 実装unit詳細案（利用者承認済み）

既存のCore Platform SPIを使うWindows固有の変更に限定し、Core scene producerやLinux backendをこのbranchへ取り込まない。scene producerが別branchで進行中のため、backend unitは承認済み`SceneSnapshot`を直接入力する決定的試験とし、Controlからの自動scene生成をWindows Adapterの完了条件に混ぜない。

| Unit | 責務・主な変更範囲 | 対応試験案 | 完了境界 |
|---|---|---|---|
| `UNIT-PLATFORM-WINDOWS-001` | Windows target向けfactory compositionとWin32 Window peer。明示open、初期client area、generation付きresize、close/reopen、失敗・shutdown順序を実装する。公開C ABIは追加しない | `UT-PLATFORM-WINDOWS-PEER-001`。Win32操作をfake可能な境界に隔離し、open失敗後retry、stale generation、close後再open、shutdown時の解放を検証 | 決定的fake試験とWindows target buildが成功。可視Windowの動作は未判定のまま |
| `UNIT-PLATFORM-WINDOWS-002` | Win32 peerのsurfaceへDirect2D rendererを接続し、共通Clip/FillRect、背景色・scale、submitとPresented completionを実装する | `UT-PLATFORM-WINDOWS-RENDER-001`。fake可能なsurface/render境界で順序、pixel/command相当のoracle、completion/failure、stale ticketを検証 | headless/fake試験とWindows target buildが成功。実画面表示は別smoke |
| `UNIT-PLATFORM-WINDOWS-003` | Core `GlyphRun`をDirectWriteで描画し、text/font resource lifecycleとrenderer close時の解放を接続する | `UT-PLATFORM-WINDOWS-TEXT-001`。入力GlyphRunとDirectWrite adapter呼出し、失敗時のFrame結果、資源解放を検証 | DirectWrite経路のfake/target試験が成功。OS既定font解決や文字品質のprofile確認は別途記録 |

### 実装開始前の登録

上記はunit/testの提案であり、承認済みDDを超える外部契約を確定しない。実装開始前に次を行う。

1. `.docs/32_unit_test/ui.md`へ各unitの前提、action、observable、oracle、cleanup、commandを追加する。unit testは可視Windowを生成しない。
2. `.docs/32_unit_test/implementation-test-inventory.md`と`tools/units.yaml`へ各unit、要求/設計参照、test path、変更path、依存関係を登録する。
3. dependency選択とWindows SDK/COM boundaryをsource/Cargo workspaceと照合し、Linux/macOSでのcross-target compileを維持できない選択や新しい公開依存が必要なら、実装を始めず差分を提示する。
4. 各unitの対応試験が通ったrevisionごとに日本語commit・pushする。Windows GUI実機smokeと0.0.1 release qualificationは全unit完了後も別記録とする。

### branch依存の注意

この計画branchは`impl-rust/core-apis-window-background`配下のCore SPIを利用する。別途進行中のCore scene producer (`impl-rust/core-scene-rendering`) はこのbranchの親・merge元にせず、backend検証には既知の`SceneSnapshot`を使う。後日、release縦断でproducerとbackendを接続して評価する際は、両変更を含む統合revisionを明記し、片方のCI結果を他方へ転記しない。

## 対象外

- Linux DRM/KMS backendの変更とDebian/RPi実機確認
- OS keyboard/pointer routing、TextBox IME、Accessibility providerのWindows接続
- Windows ARM64、別renderer/API、software fallback、WebEngine
- 新しい公開C ABI、言語wrapper API、package publication、0.0.1 release/tag
- 設計未決事項を実装都合だけで確定すること

## 完了条件案

- Windows target artifactにWindows factoryが接続され、実行時に明示した`Window.open()`がWindows peerを生成できる。Unsupportedのままの境界を意図せず残さない。
- open成功後に共通Sceneのframe presentation completionを確認でき、明示close後にCore Windowを保持したまま再openできる。
- failure、stale generation、close/shutdown、resize境界に対応するfake単体試験が同一実装revisionで成功する。
- 通常CIはvisible Window起動に依存せず、Windows target buildとfake試験を通過する。
- Windows 11 x64手動実機profile smokeを未実施/成功/失敗のいずれかで正確に記録し、未実施なら0.0.1 release gateを未完了とする。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-31 | 計画0.1 | 利用者レビュー待ち | 0.0.1のWindows x64 target/UI縦断scopeを個別Platform実装計画へ具体化。承認・commit・実装開始は未実施 |
| 2026-10-31 | 計画0.1 | 利用者承認 | 利用者「計画はokです」により計画全体を承認。計画commit/pushを許可。実装開始・試験合格・0.0.1 release判断は別ゲート |
| 2026-11-01 | 実装unit詳細案 | 利用者承認 | 利用者「ok」により3 unit分割とtest登録準備を承認。計画更新 `a77fac4` をpush。test design全文承認と製品コード実装開始は別ゲート |
