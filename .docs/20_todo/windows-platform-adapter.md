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
