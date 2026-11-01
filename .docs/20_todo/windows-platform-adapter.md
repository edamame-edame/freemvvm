# 0.0.1 Windows Platform Adapter 実装計画

- 計画版: 0.2
- 状態: 0.1承認済み（利用者、2026-10-31）。0.2改訂計画承認済み（利用者、2026-11-01）。unitごとの実装・試験状態は別管理
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

## UNIT-PLATFORM-WINDOWS-001 実装進捗

- 実装メモ: [Windows Platform Adapter / Win32 peer](../01_memory/platform-windows-adapter.md)
- 状態: UNIT-001/002は実装・local検証・同revision Windows/Linux CI完了。UNIT-003、Windows GUI実機確認は未着手。
- 実装: `crates/platform-windows`にWin32 peerと専用message workerを追加し、`Window.open()`時だけnative Windowを生成する。初期client areaと`WM_SIZE`をDIP・generation付きでCoreへ通知し、明示close後の再open、open失敗後のretry、shutdown時のworker/HWND解放を実装した。Windows C ABI artifactにはWindows factoryを接続し、Linux/その他targetの構成を維持した。
- 境界: rendererはUNIT-002で追加するため、このunitの`submit_frame`は未実装を成功扱いせずfailureを返す。OS `WM_CLOSE`はCore close-request連携が未実装なので抑止する。このunitではtitlebar close・input動作を完了扱いしない。C ABI unit testは可視Windowを生成しないようtest build時にOS factoryから分離した。
- 検証: `cargo fmt --all -- --check`、workspace Clippy/build/tests、`cargo check --workspace --all-targets --target x86_64-unknown-linux-gnu --locked`、Windows x64 target check、Rust 1.85 Windows target check、`tools/dev.py check`、`tools/dev.py minimum`がlocal pass。Windows peer 7 testsもpass。
- 同revision CI: `72f47f66418384b03eadaa1dfe6e79fd6757c676` の[Windows/Linux CI run 41263860362](https://github.com/edamame-edame/freemvvm/actions/runs/41263860362)がsuccess。直前の`c048d95`ではLinux host向けWindows backend stubにconstructorがなくclippy compileが失敗したため、`72f47f6`で修正し、Linux target cross-checkと再CI passを確認した。
- GUI: 可視Windowを起動する実機smokeは未実施。今回のlocal/CI検証で置き換えない。

### UNIT-PLATFORM-WINDOWS-002 実装進捗

- 状態: Direct2D renderer実装・local検証・同revision Windows/Linux CI完了。GUI実機smokeは未実施。
- 実装: Windows専用private依存`windows` 0.62.2を追加し、Direct2D HWND render targetをWin32 worker上で所有する。96 DPI targetへCore DeviceScaleをrect edge単位で適用し、background clear→SceneSnapshot順のClip/FillRect→EndDrawを行う。WM_SIZEでtarget sizeを更新し、WM_PAINTで直近成功sceneを再描画する。
- 完了境界: DirectWrite未対応のGlyphRunは描画開始前に拒否し、Presented扱いにしない。同期render成功後にだけ対象FrameTicketのCore completion sinkを呼ぶ。公開C ABI・Core SPI・scene producer・WM_CLOSE連携に変更なし。
- 検証: `cargo test -p freemvvm-platform-windows platform_windows_render_ --locked` 5 tests pass、`cargo test -p freemvvm-platform-windows --all-targets --locked` 12 tests pass、workspace Clippy pass、Windows x64 target check pass、Rust 1.85 Windows target check pass、`tools/dev.py check`と`tools/dev.py minimum` pass。
- 同revision CI: 実装SHA `aac97d553168c1c6845221fb7dda7cf4020fa332` の[Windows/Linux CI run 41266800908](https://github.com/edamame-edame/freemvvm/actions/runs/41266800908) success。Linux `-D warnings`で未使用renderer helperが検出された先行SHA `7439860` は、Windows production/test-only module cfgを分離して修正。Windows CI専用stepもpeer限定からAdapter全unit実行へ拡張した。
- Windows GUI実機上のvisible frame presentationは未確認で、別の手動smoke/release qualificationに残す。OS `WM_CLOSE`→Core close-request連携も0.0.1残課題として後段に残す。

## 実装unit詳細案（利用者承認済み）

既存のCore Platform SPIを使うWindows固有の変更に限定し、Core scene producerやLinux backendをこのbranchへ取り込まない。scene producerが別branchで進行中のため、backend unitは承認済み`SceneSnapshot`を直接入力する決定的試験とし、Controlからの自動scene生成をWindows Adapterの完了条件に混ぜない。

| Unit | 責務・主な変更範囲 | 対応試験案 | 完了境界 |
|---|---|---|---|
| `UNIT-PLATFORM-WINDOWS-001` | Windows target向けfactory compositionとWin32 Window peer。明示open、初期client area、generation付きresize、close/reopen、失敗・shutdown順序を実装する。公開C ABIは追加しない | `UT-PLATFORM-WINDOWS-PEER-001`。Win32操作をfake可能な境界に隔離し、open失敗後retry、stale generation、close後再open、shutdown時の解放を検証 | 決定的fake試験とWindows target buildが成功。可視Windowの動作は未判定のまま |
| `UNIT-PLATFORM-WINDOWS-002` | Win32 peerのsurfaceへDirect2D rendererを接続し、共通Clip/FillRect、背景色・scale、submitとPresented completionを実装する | `UT-PLATFORM-WINDOWS-RENDER-001`。fake可能なsurface/render境界で順序、pixel/command相当のoracle、completion/failure、stale ticketを検証 | headless/fake試験とWindows target buildが成功。実画面表示は別smoke |
| `UNIT-PLATFORM-WINDOWS-003` | Core `GlyphRun`をDirectWriteで描画し、text/font resource lifecycleとrenderer close時の解放を接続する | `UT-PLATFORM-WINDOWS-TEXT-001`。入力GlyphRunとDirectWrite adapter呼出し、失敗時のFrame結果、資源解放を検証 | DirectWrite経路のfake/target試験が成功。OS既定font解決や文字品質のprofile確認は別途記録 |

### UNIT-CORE-080 実装進捗

- 状態: Core実装・local検証・同SHA Windows/Linux CI完了。
- 変更: `GlyphRunSnapshot`へ不透明黒のRGBA defaultを追加。C ABI、wrapper、任意色設定APIは変更なし。
- 試験: focused test 1件、Core 170件、workspace全test、workspace Clippy、Rust 1.85 workspace check、format、文書link検査がpass。

### UNIT-003着手前のフォント契約（計画0.2承認済み）

RQ-09 v0.7、`internal-layers.md` 1.7、`ui.md` 1.1の承認を受け、以下をUNIT-CORE-080/Windows UNIT-003の実装oracleとする。text color追加はCore snapshot型の変更を伴うため、Windows描画unitより先に同じRust/Core feature branchの独立unitとして実装・検証する。別言語wrapper branchへの切替は発生しない。

- text colorの省略時値をopaque black (`#000000`) とする。Window背景色の既定opaque whiteとは独立したGlyphRun属性とする。
- Windowsのsystem-font profileは、固定の`MS UI Gothic`名やフォントフォルダーの固定パスに依存させず、DirectWriteのsystem font collectionから実行環境にインストールされたfontを解決する。日本語UI向けのfamily選択・fallbackはOSのfont/profile情報に委ね、選ばれた具体的font face identityをshaping済みGlyphRunのdigestへ反映する。DirectWrite rendererはdigestに対応するfont faceでglyph IDを描画し、描画時に別fontへ黙って置換しない。
- アプリ指定font assetは明示した`.ttf` fileをDirectWriteのfont-file/custom-font collection経由で読み込めるようにする。OS Fontsディレクトリを独自走査せず、インストール済みfontはsystem collection、アプリ指定fileはfile reference/custom collectionを使う。未指定fontと明示file指定を区別する。
- digest/font faceの解決不能、明示fileの不在・不正は描画成功として扱わず、frame failureへ伝える。system fallbackはshaping時のprofile解決に限り、renderer内の暗黙fallbackを設けない。
- `UT-PLATFORM-WINDOWS-TEXT-001`ではfont resolverとDirectWrite boundaryをfakeし、色、選択されたfont identity/glyph forwarding、欠損・不正assetのfailure、resource releaseを決定的に検証する。OSの実フォント選択や文字品質はWindows profile smokeへ分離する。

#### 実装順（承認済み）

1. 完了: RQ-09 v0.7、Platform internal layers v1.7、unit test ui v1.1の全文承認を記録。
2. 現行Rust/Core feature branchで`UNIT-CORE-080`を実装する。`GlyphRunSnapshot`にopaque black defaultのRGBA colorを追加し、`UT-CORE-GLYPH-RUN-001`を同じrevisionで検証して日本語commit/pushする。公開C ABI・wrapper APIは変更しない。
3. 同branch上でCore unit完了後に`UNIT-PLATFORM-WINDOWS-003`を実装する。DirectWriteはsystem font collection/custom TTF sourceから作られたfont faceをdigestで解決し、描画・failure・close lifecycleを`UT-PLATFORM-WINDOWS-TEXT-001`で検証する。branchの親Core API branchとの依存関係は維持し、別言語branchを親にしない。
4. 両unitのCI完了後、font profile/system family選択と可視文字品質をWindows 11 GUI実機smokeで別途記録する。

本提案を受けて`RQ-09_ui-scope.md` 0.7、`platform/internal-layers.md` 1.7、`ui.md` 1.1の改訂案を作成した。文書全体レビュー・承認後にCore unitからsource実装へ進む。Windows 11ではMicrosoftのTypography資料がSegoe UI VariableをWindowsアプリの既定UIフォントとして推奨し、日本語UI familyとしてYu Gothic UIを挙げているため、`MS UI Gothic`を固定の「現行Windows既定」とは扱わない。

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
| 2026-11-01 | 実装unit詳細案 | 利用者承認 | 利用者「ok」により3 unit分割とtest登録準備を承認。計画更新 `a77fac4` をpush |
| 2026-11-01 | UI test design 1.0 / UNIT-PLATFORM-WINDOWS-001 | 利用者承認・実装開始 | 利用者「承認」によりUI test design 1.0全文を承認。その後「実装を開始」の指示によりUNIT-001の実装を開始。試験・CI合格およびGUI実機確認は別状態 |
| 2026-11-01 | 計画0.2、RQ-09 v0.7、DD 1.7、UI test design 1.1 | 利用者承認 | 利用者「ok」によりopaque black default、OS system-font resolution、明示TTF source、Core UNIT-080先行とWindows UNIT-003の順序を承認。実装・test passは別状態 |
