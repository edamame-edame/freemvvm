# 共通 MVVM UI ライブラリの構成要素と設計調査

調査日: 2026-09-25  
対象: C++ / C# / Rust / Python 3 から共通の MVVM モデルで利用する UI ライブラリ

## 位置づけ

本資料は既存フレームワークの公式資料から確認できる構成要素と、本プロジェクトへの設計提案を分けて記録する。提案は仕様決定ではない。対象 OS、描画方式、宣言的 UI 記法、ライセンス要件は未決定。

## 確認できた共通要素

| 領域 | 既存実装・指針から得られる知見 | 本プロジェクトで検討する要素 |
| --- | --- | --- |
| Property / Binding | WPF はデータソースと UI の同期、コレクションの表示、入力値の更新を binding で扱う [1]。Slint は依存 property の変更時に binding を再評価し、双方向 binding も提供する [5][6]。 | `Property<T>`、通知、OneWay / TwoWay / OneTime、更新タイミング、検証エラー |
| Command / Event | Slint は UI callback をホスト言語の処理へ接続する [7]。 | `execute`、`can_execute`、変更通知、購読解除、非同期処理の方針 |
| Collection | Qt Quick は model / view / delegate を分離する [3]。Slint Python API も UI 用 Model を公開する [8]。 | 増減・移動・更新通知、選択モデル、item template、大量件数の仮想化 |
| Control / Layout | WPF は control、layout、描画、style、template を提供する [2]。 | Element、レイアウト、基本 control、入力と focus、window |
| Style / Template | WPF の Style はプロパティ値、ControlTemplate は control の構造と外観、DataTemplate はデータの表示方法を扱う [4]。 | style、theme、resource、control / data template、状態 |
| Accessibility | W3C APG は role、状態、名前、キーボード操作、focus を含む widget の指針を提供する [9][10]。 | 共通 accessibility tree と platform adapter、各 control の操作規約 |

## 推奨する構成と優先度

1. **基礎契約**: Object の所有権、Property、変更通知、Binding、Command、エラー、UI スレッド規約を定義する。
2. **画面基盤**: logical / visual tree、layout、描画、pointer / keyboard / IME、focus、window を定義する。テンプレートで内部要素が増えるため、二つの tree の用途を明記する。
3. **基本 control**: Text、Button、TextBox、CheckBox、StackPanel、ScrollViewer を縦断的に実装する。
4. **外観と集合**: Style / Theme / Template、CollectionModel、SelectionModel、ListView、virtualization を追加する。
5. **高度な control と開発支援**: ComboBox、TreeView、TableView、Dialog、Menu、binding 診断、UI テスト支援を追加する。

Accessibility は最後に仕様を足すのではなく、role / name / state / focus / keyboard 操作の契約を基礎段階で決め、control ごとに実装と検証を進める。描画や OS 連携の完成順は対象 platform の決定後に調整する。

## 標準 control の候補

| 区分 | 候補 |
| --- | --- |
| 表示 | Text、Image、Icon、Border、Separator、ProgressBar |
| 入力 | Button、ToggleButton、CheckBox、RadioButton、Switch、TextBox、PasswordBox、Slider、SpinBox |
| 選択・データ | ComboBox、ListBox、ListView、TableView、TreeView |
| レイアウト | Panel、StackPanel、Grid、WrapPanel、DockPanel、ScrollViewer |
| コンテナ・画面 | TabView、GroupBox、Expander、Window、Dialog、Popup、Menu、ToolBar |

これは網羅的な実装約束ではない。List / ComboBox / Tree / Table に共通する Model、Selection、Template の契約を先に設計する。

## 4 言語間で先に決める契約

共通のネイティブ実装を 4 言語から呼ぶ構成は**提案**であり、既存資料から直接導かれる必然ではない。採用する場合は ABI または言語バインディングの境界に、値型・文字列の符号化と所有権・object lifetime・callback の解除・例外とエラー・UI スレッド・async・コレクションの変更通知を明文化する。Slint の複数言語対応 [11] は比較対象だが、C# を含む本プロジェクトの ABI をそのまま決める根拠にはならない。

最小の縦断シナリオは、`Window > StackPanel > TextBox / Text / Button` とし、TextBox と ViewModel の双方向 binding、Text の片方向 binding、Button の Command、focus とキーボード操作、accessibility name を C++ / C# / Rust / Python 3 から同じ意味で利用できることを確認する。

## 追加調査が必要な論点

- 対象 OS と native control / 独自描画の選択
- binding の循環、更新順序、トランザクション、validation の表示
- FFI の参照所有権、GC と参照カウントの相互作用、callback 破棄
- UI スレッドと worker 間の dispatch、非同期 command のキャンセル
- IME の composition、Unicode、RTL、高 DPI
- platform 別 accessibility API への写像と自動テスト
- ライセンス、配布サイズ、起動時間、実測性能

## 実装言語と FFI の追加調査

### 既存ライブラリの例

| ライブラリ | 主な実装 | 複数言語への境界 | 示唆 |
| --- | --- | --- | --- |
| Qt | C++ | C++ API、メタオブジェクト、各種言語バインディング | C++ で property / signal / slot / runtime type を作る実績がある。ただし同一 major 内の binary compatibility を維持するため、private implementation などの API 規律が必要 [12][13]。 |
| Slint | Rust | C++ / Rust / Python / JavaScript | Rust の UI ランタイムをホスト言語へ公開する構成が実用化されている [14][15]。C++ が内部実装の必須条件ではない。 |
| Avalonia | C# | C# / F# と native platform backend | コントロールと scene graph は managed 側、Skia と platform backend は native 側に分離している [16][17]。 |
| Flutter | Dart + C++ engine | Dart API (`dart:ui`) と C++ engine / embedder | C++ は rendering / engine / platform 境界に置かれ、アプリの高レベル UI と分担している [18][19]。 |
| GTK / GObject | C | GObject の型情報と introspection | C ABI、参照規則、型情報を中心に多数の言語バインディングを成立させている [20][21]。 |

### 速度について

C++ と Rust はどちらもネイティブコードを生成するため、UI ライブラリの実用性能を「C++ だから速い」とは決められない。フレーム処理では描画、レイアウト、テキスト、画像、GPU 同期が支配的であり、言語境界を細かく往復する設計では marshalling、割り当て、callback、スレッド切り替えの方が問題になりやすい。C# も native interop を使える [22]。したがって、実測前に実装言語だけで速度を選ばず、API をバッチ化し、描画とモデル更新の境界をプロファイルする。

### 推奨判断

本プロジェクトの第一段階では、**C++ で UI core と platform/rendering adapter を実装し、公開境界は C ABI に固定する構成**を推奨する。理由は次のとおり。

1. C++ 自体が対象言語であり、UI / graphics / OS SDK との統合が直接できる。
2. C ABI は C++、C# P/Invoke、Rust `extern "C"`、Python `ctypes` / cffi の共通の最小境界になる [22][23][24]。
3. Qt と Flutter に、C++ を長期運用する UI / engine の実例がある [12][18]。
4. Rust core を選んでも各言語の境界は必要であり、C++ を利用する利点は「FFI が不要」ではなく「C++ 側の wrapper が薄くできる」点にある。

これは C++ ABI を公開するという意味ではない。`std::string`、`std::vector`、C++ exception、RTTI、template 型、仮想クラス、コンパイラ依存の layout を境界に出さない。Qt が binary compatibility を保つために private implementation を使うように、公開 API の ABI と内部 C++ class を分離する [12][13]。

Rust core は、メモリ安全性と並行処理の安全性を優先する場合の有力な**初期選択肢**である。Slint が示すように成立するが、本プロジェクトでは Rust から C++ platform / graphics API へ接続する層と、4 言語の binding 生成を先に整備する必要がある。UI core 全体を後から C++ から Rust へ置き換える計画は、property、layout、rendering、input、accessibility、thread、lifetime の意味論を再実装することになるため、通常は現実的な保守計画にならない。C ABI は wrapper とアプリケーションの再コンパイル範囲を抑えるが、core の置換作業そのものを小さくするものではない。実装言語は早期に決め、置換可能性は renderer、platform adapter、text engine などの独立 subsystem に限定する。

Rust を選ぶ場合も、各 platform target 向けの native artifact をビルド・配布する必要がある。Rust の target は target triple で識別され、Cargo は `--target` により対象 architecture / OS / ABI ごとにビルドする [26]。同じ C ABI なら C# と Python が同じ platform artifact を利用できるが、Windows x64、Windows ARM64、Linux x64、macOS ARM64 などの組み合わせごとにライブラリを用意する。これは C++ でも同じであり、C ABI は言語間の境界を共通化するが、platform binary の生成を不要にはしない。

### 実装境界の仕様案

```text
ui-core (private C++ classes)
        │
        ├─ ui_abi.h : versioned C ABI, opaque handles
        │      ├─ C++ typed wrapper
        │      ├─ C# LibraryImport / P-Invoke wrapper
        │      ├─ Rust safe wrapper
        │      └─ Python cffi or extension wrapper
        │
        └─ platform / renderer adapters
```

ABI の初版では、固定幅整数、`float` / `double`、UTF-8 の `(pointer, length)`、opaque handle、`ui_status`、明示的な `retain/release`、`callback + userdata + destroy` を使う。文字列・配列をどちら側が解放するかを API ごとに定義し、例外・panic・Python exception は ABI を越えない。Rust も FFI の unwind と ABI の一致を要求するため、境界で panic を捕捉して status に変換する [25]。

UI 操作は UI thread に限定し、worker からは dispatcher 経由で処理する。callback を細かく呼ぶ設計を避け、property 更新・collection 差分・描画 command は可能な範囲でまとめる。C# delegate、Python callable、Rust closure は wrapper が保持し、解除時に必ず native callback を無効化する。

### 開発順序

1. `ui_abi.h` と ABI versioning、所有権、thread、error の規約を文書化する。
2. C++ の `Object / Property / Binding / Command` を opaque handle で実装する。
3. C++、Rust、C#、Python の薄い wrapper で同じ MVVM smoke test を動かす。
4. Element / layout / input / focus と最小の TextBox / Text / Button を追加する。
5. property 更新、collection 差分、描画をプロファイルし、必要な箇所だけバッチ API を追加する。
6. ABI compatibility test を CI に追加し、各 OS・各コンパイラ・各言語の matrix を検証する。

## 追加情報源（公式資料）

12. Qt, [Meta-Object System](https://doc.qt.io/qt-6/metaobjects.html) — signal / slot、runtime type、dynamic property。
13. Qt, [Creating Shared Libraries](https://doc.qt.io/qt-6/sharedlibrary.html) — 同一 major 内の binary compatibility。
14. Slint, [Repository architecture](https://github.com/slint-ui/slint#architecture) — Rust 実装と複数ホスト言語。
15. Slint, [C++ API README](https://github.com/slint-ui/slint/tree/master/api/cpp) — Rust 実装を C++ から利用する構成。
16. Avalonia, [Cross-platform architecture](https://docs.avaloniaui.net/docs/fundamentals/cross-platform-architecture) — managed UI と描画の分担。
17. Avalonia, [Architecture](https://docs.avaloniaui.net/docs/fundamentals/architecture) — scene graph、compositor、platform backend。
18. Flutter, [Architectural overview](https://docs.flutter.dev/resources/architectural-overview) — Dart framework と C++ engine。
19. Flutter, [How Flutter works](https://docs.flutter.dev/learn/pathway/how-flutter-works) — engine が C++ である理由と描画。
20. GTK, [Documentation](https://docs.gtk.org/) — GLib / GObject / introspection。
21. GTK, [GObject type system concepts](https://docs.gtk.org/gobject/concepts.html) — language binding を考慮した型・所有権。
22. Microsoft, [Platform Invoke](https://learn.microsoft.com/en-us/dotnet/standard/native-interop/pinvoke) — managed code から native functions、structs、callbacks。
23. Python, [ctypes](https://docs.python.org/3/library/ctypes.html) — C-compatible data types と DLL 呼び出し。
24. Rust, [FFI](https://doc.rust-lang.org/nomicon/ffi.html) — C calling convention と unwind。
25. Rust, [Unwinding](https://doc.rust-lang.org/nomicon/unwinding.html) — panic と FFI 境界。
26. Rust, [Platform Support](https://doc.rust-lang.org/rustc/platform-support.html) / [Cargo target option](https://doc.rust-lang.org/cargo/commands/cargo-rustc.html) — target triple と対象別ビルド。

## 情報源（公式資料）

1. Microsoft, [WPF Data binding overview](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/data/) — binding、通知、コレクション。
2. Microsoft, [WPF overview](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/overview/) — UI 機能の全体像。
3. Qt, [Models and Views in Qt Quick](https://doc.qt.io/qt-6/qtquick-modelviewsdata-modelview.html) — model / view / delegate。
4. Microsoft, [WPF Styles and templates](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/controls/styles-and-templates) — Style / ControlTemplate / DataTemplate。
5. Slint, [Custom Controls](https://docs.slint.dev/latest/docs/slint/guide/development/custom-controls/) — property の依存追跡とホスト API。
6. Slint, [Two-Way Bindings](https://docs.slint.dev/latest/docs/slint/reference/language/two-way-bindings/) — 双方向 binding。
7. Slint, [Functions and Callbacks](https://docs.slint.dev/latest/docs/slint/guide/language/coding/functions-and-callbacks/) — callback とホスト言語。
8. Slint, [Python API](https://docs.slint.dev/latest/docs/python/) — Model と callback。
9. W3C WAI, [ARIA Authoring Practices Guide](https://www.w3.org/WAI/ARIA/apg/) — widget pattern と意味論。Web 向け指針なので desktop API への適用は設計上の類推。
10. W3C WAI, [Developing a Keyboard Interface](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/) — focus とキー操作。
11. Slint, [Language Integrations](https://docs.slint.dev/latest/docs/slint/language-integrations/) — 複数言語への公開形態。
