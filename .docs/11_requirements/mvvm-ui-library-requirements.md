# MVVM UIライブラリ 要求仕様

- 作成日: 2026-09-25
- 対象言語: C++ / C# / Rust / Python 3
- ステータス: Accepted（利用者確認、2026-09-26）

## 1. 文書の位置づけ

本書は、以下の2文書をもとに、情報源・設計判断・要求仕様・未決事項を整理したものである。

| 文書 | 位置づけ |
| --- | --- |
| `adr-0001-rust-core.md` | 正式なアーキテクチャ決定。Rust Coreとversioned C ABIを採用する。 |
| `common-ui-library-research.md` | 既存技術の調査、設計候補、比較、追加調査事項。仕様決定そのものではない。 |

調査資料にはC++ Coreを推奨する旧案が含まれるが、現在の正式な決定はADR-0001のRust Coreである。

## 2. 情報源の整理

### 2.1 公式資料の分類

| 分野 | 主要な情報源 | 参考にする内容 |
| --- | --- | --- |
| Property / Binding | WPF、Slint | 通知、OneWay / TwoWay binding、依存関係の再評価 |
| Model / Collection | Qt Quick、Slint | Model / View / Delegate、Collection変更通知 |
| Control / Template | WPF | Control、Style、ControlTemplate、DataTemplate |
| Accessibility | W3C WAI-ARIA APG | role、name、state、focus、keyboard操作 |
| UIアーキテクチャ | Slint、Avalonia、Flutter、GTK | Core、描画、Platform、言語境界の分離 |
| ABI / FFI | .NET P/Invoke、Python ctypes、Rust FFI | C ABI、callback、所有権、例外の境界 |
| Binary互換性 | Qt Shared Libraries | 公開ABIと内部実装の分離 |
| クロスコンパイル | Rust Platform Support、Cargo | target triple、target別native artifact |

外部資料は設計の根拠・比較材料であり、本プロジェクトの要求を直接決定するものではない。URL一覧は `common-ui-library-research.md` の公式資料一覧を参照する。

## 3. 目的と基本アーキテクチャ

C++、C#、Rust、Python 3から共通のMVVMモデルで利用できる、クロスプラットフォームUIライブラリを構築する。

共通ランタイムはRustで実装し、各言語にはversioned C ABIを介した薄いwrapperを提供する。

```text
C++ / C# / Rust / Python
          │
     Thin Wrappers
          │
   Versioned C ABI
          │
      Rust UI Core
```

## 4. 必須要求

### 4.1 Core

Rust UI Coreは、少なくとも以下を提供する。

- Object
- Property
- Binding
- Command
- Collection
- Selection
- Visual Tree
- Layout
- Input
- Focus
- IME
- Error handling
- Lifetime管理
- UI thread管理

### 4.2 対応言語

以下の4言語から、同じUI機能とMVVM意味論を利用できること。

- C++
- C#
- Rust
- Python 3

### 4.3 C ABI

公開境界はversioned C ABIとする。ABIには以下だけを置く。

- 固定幅整数
- `float` / `double`
- UTF-8の `(pointer, length)`
- opaque handle
- 明示的な `retain/release`
- `ui_status`
- error object
- `callback + userdata + destroy`
- ABI version
- feature query

以下はABIに公開しない。

- Rust型、trait、generic、borrow
- C++ class、STL、RTTI、exception
- Python object
- C# object / delegate
- Rust panic
- C++ exception
- Python exception

### 4.4 所有権とcallback

次の契約をAPIごとに明文化する。

- handleの所有者
- retain/releaseの規則
- 破棄順序
- 文字列・配列の所有権
- pointerの借用期間
- callbackの保持と解除
- callback解除後の再呼び出し禁止
- userdataの破棄責任
- callbackの再入可能性

### 4.5 Thread

- UI操作はUI threadに限定する。
- worker threadからのUI操作はdispatcher経由とする。
- UI threadとworker間のデータ競合を防止する。
- 非同期Commandの完了、エラー、キャンセルを定義する。

### 4.6 Binding

最低限、次のBindingを提供する。

- OneTime
- OneWay
- TwoWay

仕様として確定する項目：

- 更新方向
- 更新タイミング
- 更新順序
- 循環Bindingの扱い
- 再入時の扱い
- transactionの有無
- validation errorの伝播
- binding解除時の挙動

## 5. 最初の縦断シナリオ

以下をRust Core、C ABI、C++ / C# / Rust / Python wrapperの全てで動作させる。

```text
Window
└─ StackPanel
   ├─ TextBox  ← TwoWay  → ViewModel.Name
   ├─ Text    ← OneWay   ← ViewModel.Message
   └─ Button  ─ Command  → ViewModel.Save
```

検証項目：

- TextBoxの入力がViewModelへ反映される
- ViewModelの変更がTextへ反映される
- ButtonからCommandを実行できる
- Commandの実行可否を扱える
- focusが移動する
- キーボード操作ができる
- accessibility nameを取得できる
- 4言語で同じテストを実行できる

## 6. UI機能要求

### 6.1 初期実装

- Window
- Widget
- Container
- Text
- TextBox
- Button
- StackPanel
- 基本layout
- focus
- keyboard input
- 最小限のaccessibility name

### 6.2 次段階

- CheckBox
- ToggleButton
- ScrollViewer
- Image
- Border
- Separator
- ProgressBar
- CollectionModel（BD-33承認済み: Value kind／Widget treeと独立したCore object。stable item ID、構造化change set、visible range snapshot。BD-35承認済み: batch atomicityはModel整合性に限り、独立入力の値／業務validationをまとめて待たせない）
- SelectionModel（BD-33／BD-34承認済み: stable item ID参照。Single/Multiple cardinalityを持ち、current itemはselection・keyboard focusと独立。削除itemのselection/current状態は消去する）
- ListView
- virtualized list
- Style
- Theme
- Resource
- Template

### 6.3 将来候補

- ComboBox
- TreeView
- TableView
- Dialog
- Popup
- Menu
- ToolBar
- TabView
- binding診断
- UIテスト支援

List、ComboBox、Tree、Tableに共通するModel、Selection、Item template、Collection差分通知、virtualizationの契約を、個別Controlより先に設計する。BD-33は共通CollectionModelの境界とGrid virtualization方針、BD-34はSelection cardinalityとcurrent itemの上位意味論を承認済み。BD-35承認済み（2026-09-27）: change batchはCollection Model整合性について全件適用／全件拒否とし、独立入力欄の有効なcommitを別入力のvalidation待ちにしない。各Controlの既定mode・gesture/navigationは後続。

## 7. Accessibility要求

Accessibilityは後付け機能ではなく、基盤仕様に含める。

各Controlについて、最低限以下を定義する。

- role
- name
- state
- value
- focus状態
- keyboard操作
- enabled / disabled状態
- platform accessibility APIへの写像

W3C ARIA APGは参考資料として利用する。ただし、Web向け指針をdesktopや組み込みLinuxへ適用する部分は、本プロジェクト側で設計を確定する。

## 8. Platform要求

### 8.1 対象

組み込みLinuxを正式対象に含める。初期対象候補は以下とする。

- Linux ARMv7
- Linux AArch64
- Linux x64
- Windows x64 / ARM64
- macOS x64 / ARM64

bare metalやMCUは、通常のLinux targetとは別のplatform profileとして扱う。

### 8.2 Platform adapter

以下はRust Coreから分離する。

- renderer
- platform backend
- text engine
- image codec
- input backend
- accessibility backend

組み込みLinuxの描画backendは、次の順で検証する。

1. DRM/KMS + EGL/OpenGL ES
2. Wayland
3. X11
4. framebuffer
5. software renderer

GPU、input device、IME、display rotation、power management、window systemなどの実機依存要素はplatform adapterに閉じ込める。

## 9. ビルドと配布

- target tripleごとにnative artifactを生成する。
- OS・CPU architectureごとにライブラリを用意する。
- 同一targetのC ABI artifactを4言語で共有する。
- BD-19承認により、C++向け標準integrationをCMake install/export packageとし、public C++ wrapperと対象targetの共通C ABI native libraryを含める。初期targetはWindows x64／Linux x64とし、downstream projectは`find_package(... CONFIG)`を使う。vcpkg／Conan等のregistryは未決。
- BD-20承認により、Rust wrapperをraw FFIの`-sys` crateとsafe API crateに分け、Rust Coreは同梱せず共通C ABI runtimeを別artifactとして利用する。別環境で生成したtarget artifactを取得して取り込む機能を備え、featureで有効化する。Cargo `--target`でtarget triple、artifact profileでboard/toolchain差を選び、target/profile/ABIの不一致を検出する。registry、fetch command、cache、signature等は未決。
- C#はRIDごとのnative assetをパッケージに含める。
- BD-18承認により、C# managed wrapperとRID別native C ABI libraryを同一NuGet packageの`runtimes/{rid}/native/`に含め、初期RIDは`win-x64`／`linux-x64`とする。組み込みARM向けRID packageはprofile互換性確認後に判断する。NuGet feed、package ID、TFM等は未決。
- Python wrapperはBD-14に従い標準ライブラリ`ctypes`を使い、compiled extensionを必須としない。
- BD-17承認により、Python wrapper distributionとtarget別native runtime distributionを分け、初期native wheelはWindows x64／Linux x64から始める。組み込みLinux ARM向けは、対象profileの互換性確認前にmanylinux/musllinux wheelとみなさない。package名、index、exact tag等は未決。
- Yocto SDK、sysroot、linker、C library設定をCIで固定する。

BD-16承認により、初期CI targetはWindows x64、Linux x64、組み込みLinux ARMv7/AArch64の順に追加する。Linux x64 hostからARM向けartifactをcross-compileし、Build/link verifiedと実機Runtime verifiedを区別する。ARM実機のhardware profile検証は別gateとする。exact compiler、sysroot、board、正式release matrixは後続判断で固定する。

## 10. 非機能要求

代表的なARM boardを一つ選び、同一機能で以下を実測する。

- 起動時間
- バイナリサイズ
- メモリ使用量
- 描画FPS
- layout処理時間
- Property更新コスト
- Collection差分処理コスト
- callback / FFI crossingコスト
- UI threadの最大処理時間
- 実機での安定性

実装言語だけで性能を判断せず、描画、layout、text、画像、GPU同期、FFI境界を個別にプロファイルする。

## 11. 開発順序

1. C ABI仕様を確定する。
2. ABI versioning、error、ownership、thread規約を確定する。
3. RustのObject / Property / Binding / Commandを実装する。
4. C++ / C# / Rust / Python wrapperを作成する。
5. MVVM smoke testを4言語で実行する。
6. Window / StackPanel / TextBox / Text / Buttonを追加する。
7. focus、keyboard、IMEの基礎を追加する。
8. accessibility nameとroleを追加する。
9. Collection / Selection / Templateを追加する。
10. rendererとplatform adapterを実装する。
11. ABI compatibility testをCIに追加する。
12. 実機性能を測定する。
13. 必要な箇所だけbatch APIや最適化を追加する。

## 12. 未決事項と後続の判断状況

次の項目は本書の作成時に追加決定が必要とされた論点である。承認済みの判断は項目に追記し、それ以外は関連する上位設計で確認する。

- 対象OSの正式な優先順位（BD-12承認済み、2026-09-26: 対応・検証はWindows→Linux x64→組み込みLinux ARMv7/AArch64→その他の順。Linux x64の共通資産を使いARM向けはcross-compileする。正式な公開順位・CI matrixは後続判断）
- native controlか独自描画か（BD-12承認済み、2026-09-26: 初期Controlは共通Rendererで描画し、native controlを意味・状態のsource-of-truthにしない）
- RendererのAPIと描画モデル（BD-12承認済み範囲: Core Visual Tree／LayoutとPlatform描画backendを分離。API、scene/command、更新領域は未決）
- 初期Controlの共通抽象と責務（BD-11承認済み、2026-09-26: `Element`を`Widget`に改め、子を持ちうる抽象分類`Container`を追加。Window／StackPanelはContainer、TextBox／Text／Buttonは葉Widgetとする。個別PropertyとContainer APIは後続判断）
- text shaping、Unicode、RTL対応範囲（BD-23承認済み、2026-09-27: UTF-8 encodingと初版Latin＋日本語UI文字列の表示・編集を採用。Windows／Linux x64／組み込みLinuxごとにfont discovery/fallback、shaping/rasterization、font assets/dependencies、IME、実機verificationを管理する。Windows DirectWrite、Linux Fontconfig＋shaper/rasterizer、Noto Sans CJK JP等は候補。exact Japanese corpus/font package・version、Chinese/Korean/RTL/emoji coverage、caret/navigation細則は未決）
- IME compositionの詳細（BD-09／BD-10／BD-24承認済み、2026-09-27: Focus離脱時はpreeditと位置を保持してIME sessionを終了。IME session識別、明示cancel時はcomposition開始snapshotへ復元、commitとFocusLostはAdapter受信順で処理し終了済みsessionの遅延eventを破棄。Undo/Redo・OS APIは後続）
- binding循環の扱い（BD-03／BD-15承認済み、2026-09-26: 静的解析でControlとPropertyを解決できる言語・箇所は同一ControlのPropertyアクセスを警告し、Pythonもlibrary compilerのlint/type checkで解決できる範囲を含む。Runtimeは全言語で有限作業枠・保留・診断・非収束連鎖の停止により保護。dynamic Python全件検出や作業枠・復旧の詳細は後続設計）
- validationの標準モデル（一部判断済み。V1～V8承認済み。BD-25承認済み: 初版のCore Property/View Input Validatorは短時間同期とし、Core-managed async Validator/pending stateは設けない。remote/long-running checkはViewModel AsyncCommand＋状態Propertyで表現し、stale result識別はアプリ側で行う。複数Property、共通表示・再評価、将来のCore async APIは後続判断）
- async Commandのキャンセル・進捗仕様（一部判断済み。C1～C4・BD-26承認済み: 中間進捗は最新値へ集約でき、終端結果は進捗と独立して一度だけ反映する。operation IDとUI thread上の処理順で古い／終端後通知を破棄する。集約方式、頻度・時間閾値、進捗型・複数channelは後続詳細）
- UI transactionの有無（BD-02承認済み、2026-09-26: 初期版では明示transactionを設けない。外側のUI処理後にBindingと通知をまとめ、複数setterの自動rollbackは保証しない）
- error objectの詳細構造（一部判断済み。A2／A15／A18およびBD-27承認済み: status、domain/code/message、snapshotと寿命を定義。機械判定はdomain/code/status、messageは説明用とし、causeとdiagnosticを分離する。完全なcode catalog、ABI表現、診断payloadは未決）
- ABI互換性ポリシー（一部判断済み。A3およびBD-28承認済み: 安定版native runtimeはMAJOR.MINOR.PATCH、MAJORをABI symbol majorと一致させ、minorを後方互換追加、patchを互換fixとする。初回stable番号・pre-release policy・各wrapper package制約は後続）
- Python wrapperの最終方式（BD-14／BD-15承認済み、2026-09-26: 初期Python 3 wrapperは標準ライブラリ`ctypes`でversioned C ABIを呼び、compiled extensionを必須としない。強いlint/type check、MCP/CLI/CI共通diagnostics、Temp cache増分解析を承認。checker選定・package形式等は後続判断）
- ライセンス（BD-21承認済み、2026-09-26: first-party source、C ABI/header、language wrapper、source package、target-specific binary SDKにApache-2.0を適用し、source／改変source／改変binary／app binaryの再配布を許可する。配布時のlicense/notice条件とthird-party dependencyの個別license維持を含む。repository公開、CLA/DCO、docs/assets license、SBOM等の運用詳細は未決）
- サポート期間（BD-22承認済み、2026-09-27: 最新安定ABI majorを通常保守し、直前majorは新major安定版公開後24か月security fix／重大障害修正のみ保守する。サポート状態はABI majorとtarget profileの組で公表し、build verifiedとruntime verifiedを区別する。profile非推奨は原則12か月前通知とし、EOLは公式保守終了とする。security response SLA、LTS、exact target matrixは未決）
- 配布形式（PythonはBD-17、C# NuGetはBD-18、C++ CMake packageはBD-19、Rust Cargo wrapper/native runtime境界と別環境artifactの取得featureはBD-20で承認済み。BD-29～BD-30でimmutable signed artifact、公式catalog、各言語package manager、target明示とversion/digest pinningを承認。BD-31承認済み: 独立provision済みtrust root、認証済みkey rotation/revocation、明示trust bundleによるoffline verificationを要求する。BD-32はmanifest schemaのversioning・互換規則を承認済み。暗号形式、registry実装、fetch/cache詳細は後続）
- CIで検証するコンパイラとtarget matrix（BD-16承認済み、2026-09-27: 初期順はWindows x64→Linux x64→組み込みLinux ARMv7/AArch64。Linux x64 hostから両ARM targetをcross-compileし、実機hardware verificationとは区別する。exact compiler/sysroot/board/release matrixは後続判断）
- accessibility APIの対象範囲（BD-13承認済み、2026-09-26: Core Widgetが共通role/name/state/focusability/focus/enabled/keyboard意味情報を持ち、Platform AdapterがOS APIへ写像する。初期Accessibility treeはLogical Treeを基礎とし、template内Visual要素は既定で公開しない。OS別写像、name fallback、ABI/API詳細は後続判断）

特に優先して決めるべきなのは、以下の4点である。

1. ABIの所有権・callback・thread契約
2. Bindingの更新意味論
3. 初期対象OSと組み込みLinuxの描画方式
4. Text / IME / Accessibilityの対応範囲

## 13. 文書管理方針

- ADRはアーキテクチャ上の決定を記録する。
- 調査資料は外部根拠、比較、代替案、未決事項を記録する。
- 本書はAcceptedの要求仕様として管理する。変更は承認済み判断との整合を確認してレビューする。
- ADRと要求仕様が矛盾する場合、Accepted状態のADRを優先し、必要に応じてADRを更新する。
