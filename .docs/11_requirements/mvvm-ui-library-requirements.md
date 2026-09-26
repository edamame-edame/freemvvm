# MVVM UIライブラリ 要求仕様ドラフト

- 作成日: 2026-09-25
- 対象言語: C++ / C# / Rust / Python 3
- ステータス: Draft

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
- Element
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
- CollectionModel
- SelectionModel
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

List、ComboBox、Tree、Tableに共通するModel、Selection、Item template、Collection差分通知、virtualizationの契約を、個別Controlより先に設計する。

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
- C#はRIDごとのnative assetをパッケージに含める。
- Pythonは当初ctypesまたはcffiを優先する。
- Yocto SDK、sysroot、linker、C library設定をCIで固定する。

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

## 12. 未決事項

次の項目は要求仕様として追加決定が必要である。

- 対象OSの正式な優先順位
- native controlか独自描画か
- RendererのAPIと描画モデル
- text shaping、Unicode、RTL対応範囲
- IME compositionの詳細
- binding循環の扱い
- validationの標準モデル
- async Commandのキャンセル仕様
- UI transactionの有無
- error objectの詳細構造
- ABI互換性ポリシー
- Python wrapperの最終方式
- ライセンス
- 配布形式
- CIで検証するコンパイラとtarget matrix
- accessibility APIの対象範囲

特に優先して決めるべきなのは、以下の4点である。

1. ABIの所有権・callback・thread契約
2. Bindingの更新意味論
3. 初期対象OSと組み込みLinuxの描画方式
4. Text / IME / Accessibilityの対応範囲

## 13. 文書管理方針

- ADRはアーキテクチャ上の決定を記録する。
- 調査資料は外部根拠、比較、代替案、未決事項を記録する。
- 本書は要求仕様のドラフトとして管理する。
- ADRと要求仕様が矛盾する場合、Accepted状態のADRを優先し、必要に応じてADRを更新する。
