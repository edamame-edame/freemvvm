# MVVM UIライブラリ 設計仕様骨格

- 文書種別: 設計仕様
- 状態: Draft
- 作成日: 2026-09-25
- 対象言語: C++ / C# / Rust / Python 3

## 1. 位置づけ

本書は、要求仕様、技術調査、ADR-0001をもとに、MVVM UIライブラリの設計仕様全体を分割するための骨格を定義する。

本書では、各サブシステムの責務、依存関係、設計文書の分割、実装スライスを定める。ABIの細則、Bindingの更新意味論、Rendererの具体方式などは後続の設計スライスで決定する。

## 2. 固定済みの前提

- 共通ランタイムはRustで実装する。
- 言語間の公開境界はversioned C ABIとする。
- C++、C#、Rust、Python 3に薄いwrapperを提供する。
- C++ ABI、Rust型、C++ class、STL、例外、Python object、C# objectは公開ABIに出さない。
- 組み込みLinuxを正式な対象に含める。
- Platform、Renderer、Text Engine、Input、AccessibilityはCoreから分離する。
- Core全体の別言語への置き換えは計画しない。
- 最初の縦断シナリオはWindow、StackPanel、TextBox、Text、Buttonに限定する。

## 3. 全体アーキテクチャ

```text
C++ / C# / Rust / Python
            │
      Language Wrappers
            │
      Versioned C ABI
            │
        Rust UI Core
            │
   Platform / Renderer Adapters
            │
 Windows / macOS / Linux / 組み込みLinux
```

### 3.1 Coreの責務

Rust UI Coreは、次の共通意味論を提供する。

- Objectとライフサイクル
- Propertyと変更通知
- Binding
- Command
- CollectionとSelection
- Logical TreeとVisual Tree
- Layout
- Input、Focus、IME
- Errorと診断
- UI threadとdispatcher

### 3.2 Adapterの責務

Coreから分離する対象は次のとおりとする。

- Renderer
- Platform backend
- Text engine
- Image codec
- Input backend
- Accessibility backend

GPU、入力デバイス、IME、画面回転、電源管理、Window Systemなどの実機依存要素はPlatform Adapterに閉じ込める。

## 4. 設計領域

| 領域 | 主な責務 | 詳細化の順序 |
|---|---|---:|
| 共通契約 | 用語、ID、エラー、ライフサイクル、スレッド | 1 |
| C ABI | handle、所有権、文字列、callback、ABI version | 1 |
| Core Runtime | Object、UI thread、dispatcher、diagnostics | 1 |
| MVVMモデル | Property、Binding、Command、Validation | 2 |
| データモデル | Collection、Selection、差分通知 | 3 |
| UIツリー | Element、Logical Tree、Visual Tree | 3 |
| Layout | Measure、Arrange、サイズ、配置 | 3 |
| Input | Pointer、Keyboard、Focus、IME | 4 |
| Control | Text、TextBox、Button、StackPanel、Window | 4 |
| Rendering | Scene、描画コマンド、Renderer抽象 | 5 |
| Platform | Linux、Windows、macOS、組み込みLinux | 5 |
| Accessibility | role、name、state、focus、platform API | 5 |
| Language Binding | C++、C#、Rust、Python wrapper | 横断 |
| Build / Package | target、CI、native artifact、配布 | 横断 |
| Verification | ABI、意味論、性能、実機検証 | 横断 |

## 5. 依存関係

```text
共通契約 / ABI
        │
   Core Runtime
     ┌──┴──┐
 MVVM基盤  UI Tree / Layout
     │        │
     └──┬─────┘
     基本Control
        │
 Renderer / Platform
```

次の要素を先に固定する。

1. API境界
2. 所有権とライフサイクル
3. UI threadとdispatcher
4. Property / Bindingの更新意味論
5. UI treeとlayoutの責務

## 6. 設計文書の分割

| 文書 | 内容 |
|---|---|
| `design-overview.md` | 全体構造、責務、依存関係、用語 |
| `runtime-contract.md` | Object、handle、thread、error、lifetime |
| `abi-spec.md` | versioned C ABIの型、関数、互換性 |
| `mvvm-semantics.md` | Property、Binding、Command、Validation |
| `ui-tree-layout.md` | Logical Tree、Visual Tree、layout |
| `input-focus-ime.md` | 入力、focus、keyboard、IME |
| `controls-v1.md` | 初期Controlの振る舞い |
| `platform-rendering.md` | Renderer、Platform Adapter、組み込みLinux |
| `language-bindings.md` | 4言語wrapperの規約 |
| `build-distribution.md` | target、CI、native artifact、配布 |
| `verification.md` | テスト、ABI検証、性能評価 |

## 7. 実装・設計スライス

### Slice 0: 全体骨格

- 責務分割
- モジュール境界
- 設計文書の構成
- 実装順序
- 未決事項の分類

### Slice 1: Runtime / ABI基盤

- opaque handle
- retain / release
- UTF-8文字列
- error object
- callback
- UI thread
- dispatcher
- ABI version

### Slice 2: MVVM基盤

- Object
- Property
- notification
- OneTime / OneWay / TwoWay
- Command
- validation
- binding解除

### Slice 3: 最小UI縦断

- Window
- StackPanel
- TextBox
- Text
- Button
- focus
- keyboard
- accessibility name
- 4言語のsmoke test

### Slice 4以降

- Collection / Selection
- Logical Tree / Visual Tree
- Layout
- Renderer
- Platform Adapter
- IME
- Accessibility
- Style / Template
- 高度なControl

## 8. 初期縦断シナリオ

```text
Window
└─ StackPanel
   ├─ TextBox  ← TwoWay  → ViewModel.Name
   ├─ Text    ← OneWay   ← ViewModel.Message
   └─ Button  ─ Command  → ViewModel.Save
```

最低限、次をC++、C#、Rust、Python 3で検証する。

- TextBoxの入力がViewModelへ反映される
- ViewModelの変更がTextへ反映される
- ButtonからCommandを実行できる
- Commandの実行可否を扱える
- focusが移動する
- キーボード操作ができる
- accessibility nameを取得できる

## 9. 骨格段階で保留する事項

- 対象OSの正式な優先順位
- DRM/KMS、Wayland、X11などの採用順
- native controlか独自描画か
- Bindingの循環検出方式
- Validationの標準モデル
- Text shaping、Unicode、RTLの対応範囲
- Python wrapperの最終方式
- ABIのmajor/minor互換性規則
- ライセンスと配布形式

## 10. 情報源との関係

| 情報源 | 本書での扱い |
|---|---|
| `mvvm-ui-library-requirements.md` | 要求、対象範囲、初期機能、未決事項の根拠 |
| `common-ui-library-research.md` | 既存技術の構成要素、比較、追加調査事項の根拠 |
| `adr-0001-rust-core.md` | Rust Core、C ABI、組み込みLinux対応に関する正式な決定 |

調査資料に含まれる旧C++ Core案は、Accepted状態のADR-0001により採用しない。

## 11. 次の設計スライス

次は`runtime-contract.md`と`abi-spec.md`の範囲を対象とし、以下を具体化する。

- handleの種類
- 所有権モデル
- retain / release規則
- 文字列・配列の借用とコピー
- error object
- callback解除と再入可能性
- UI thread違反の扱い
- ABI versionと互換性ポリシー

