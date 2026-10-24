# WebView エンジンプラグイン 設計・実装計画

- 文書版: 1.0
- 状態: 実装・単体検証完了（Phase 0 ～ Phase 4 完了）
- 作成日: 2026-10-24
- 対象: Rust Core (`crates/webview` / `crates/core-runtime`)、versioned C ABI (`crates/c-abi`)、4言語ラッパー (`C++` / `C#` / `Rust` / `Python 3`)、将来の View 用言語コンパイラ埋め込み境界

## 1. 目的と前提（利用者合意事項）

1. **リポジトリ一体統合**: `freemvvm` リポジトリ内の拡張コンポーネントおよび Core Control（`WebView`）として設計・実装し、`.docs/` の要求・設計・テスト仕様と 4言語ラッパーを一体管理する。
2. **純粋 Rust フルスクラッチ（外部 crate 依存ゼロ）**: `crates/core-runtime` / `crates/c-abi` と同様、サードパーティ製 crate に依存せず Rust 標準ライブラリ（`std`）のみで Markdown / HTML / CSS パーサ、Web レイアウトエンジン、`SceneSnapshot` 描画生成、および軽量スクリプト / JavaScript エンジン VM を構築する。これにより Apache-2.0 ライセンス純度、サプライチェーン安全性、組み込み Linux（ARMv7 / AArch64）移植性を維持する。
3. **対象機能とスクリプト対応**:
   - Markdown（CommonMark + GFM 相当: 見出し、段落、強調、リスト、表、コードブロック、引用、リンク等）および HTML / CSS のパースと描画
   - FreeMVVM の `Property`（OneWay / TwoWay Binding）および `Command`（リンク遷移・ボタン操作・フォーム送信・スクリプト起動アクション）との統合
   - 軽量スクリプト / JavaScript エンジンによる DOM 操作・イベントハンドリング・ViewModel バインディング連携（決定論的な実行ステップ上限・メモリ上限・サンドボックス境界を伴う）
   - 4言語ラッパー（C++ / C# / Rust / Python 3）および将来の View 用言語からの共通埋め込み API
4. **文書ガバナンス（`AGENTS.md` 準拠）**:
   - `RQ → BD → (DD + SD) → テスト設計 (RT/UT) → 実装単位 (UNIT) ごとの実装・単体テスト` の順序を厳守する。
   - 上流文書（`RQ`）から下流文書（`BD` / `DD` / `SD` / テスト / 実装）へのリンクを持たせない。
   - 未実装の C ABI 宣言を `crates/c-abi/include/freemvvm/abi.h` に先行追加しない（`abi.h` は `crates/c-abi/src/lib.rs` の実エクスポートと完全一致させる）。

---

## 2. 段階別作業計画（Phase 0 ～ Phase 4）

### Phase 0: 要求定義・基本設計・詳細設計・セキュア設計・テスト設計

- [x] **Step 0-1: 要求仕様（RQ）の策定とレビュー**
  - `.docs/11_requirements/RQ-17_webview-plugin.md`（および [要求仕様索引](../11_requirements/reqirements-specification.md) の目次更新案）を作成し、以下の要求IDを定義する:
    - Markdown / HTML / CSS 入力からの DOM 構築と Core 描画（OS ネイティブブラウザ非依存）
    - View 言語および 4言語ラッパーからの `WebView` Control 埋め込みと共通意味論
    - MVVM `Property<Utf8>` 等によるソース文字列・DOM 属性/フォーム値の OneWay / TwoWay Binding、および `Command` 連携
    - 軽量スクリプト / JavaScript エンジンのサンドボックス実行、DOM / ViewModel ブリッジ、決定論的リソース上限（ステップ数・メモリ・再入防止）
    - セキュリティ・オフライン既定動作（暗黙の外部ネットワーク通信禁止、リソース解決ポリシー、入力サイズ・ネスト深さ上限）
    - Accessibility（`AxSnapshot` へのセマンティック DOM 要素投影: 見出し・リンク・ボタン・入力欄・テキスト）
- [x] **Step 0-2: 基本設計（BD）およびセキュア設計（SD）の策定とレビュー**
  - `.docs/12_design/ui/webview-plugin.md`: パーサ層（Markdown / HTML / CSS）、DOM & スタイル計算層、Web レイアウト層（Block / Inline / Flex / Table と Core `Measure`/`Arrange` の統合）、描画層（`SceneSnapshot` / `GlyphRun` 出力）、スクリプト VM 層、MVVM ブリッジ層の責務分担と主要状態遷移を定義する。
  - `.docs/14_secure_design/08-webview-sandbox-parser-limits.md`: 非信頼の Markdown / HTML / CSS / Script 入力に対する信頼境界、パーサのトークン数・DOM ノード数・ネスト深さ上限、CSS セレクタ複雑度上限、スクリプト VM の命令ステップ・ヒープ上限、外部 URI / リソース解決の無害化を定義する。
- [x] **Step 0-3: 詳細設計（DD）の策定とレビュー**
  - `.docs/13_detail_design/ui/webview.md`: Rust Core / `crates/webview` のモジュール・型設計（`DomArena`, `HtmlParser`, `MarkdownParser`, `CssRuleset`, `ComputedStyle`, `WebLayoutEngine`, `ScriptVm`, `WebViewState`）、データフロー、エラー契約を定義する。
  - `.docs/13_detail_design/runtime-objects/abi.md` および各言語ラッパー DD（`cpp.md`, `csharp.md`, `rust.md`, `python.md`）への `ui_abi_1_webview_*` API 契約の追加案を定義する。
- [x] **Step 0-4: 要求テスト設計（RT）・単体テスト設計（UT）・実装 Unit 台帳更新**
  - `.docs/30_release_test/requirement-test-matrix.md`、`requirements-test-assertions.md` への検証ケース追加。
  - `.docs/32_unit_test/webview.md` の新設と `tools/units.yaml` への `UNIT-WEBVIEW-*` 登録。

---

### Phase 1: WebView コアエンジン基盤（DOM / Markdown / HTML / CSS パーサ & スタイル解決）

- [x] **`UNIT-WEBVIEW-001` (`webview-dom-arena-and-budget`)**
  - `crates/webview/src/dom.rs`: アリーナ方式の `DomDocument` / `DomNode`（`Element`, `Text`, `Comment`）、属性マップ、ノード数・深さ・文字列長のリソース上限ガード（SD 準拠）。
- [x] **`UNIT-WEBVIEW-002` (`webview-markdown-parser`)**
  - `crates/webview/src/markdown.rs`: CommonMark + GFM サブセット（見出し `h1`–`h6`、段落、強調・コードスパン、箇条書き・番号付きリスト、GFM テーブル、コードブロック、引用、リンク・画像）から `DomDocument` への変換器。
- [x] **`UNIT-WEBVIEW-003` (`webview-html-parser`)**
  - `crates/webview/src/html.rs`: HTML トークナイザ・ツリービルダ（void 要素、暗黙閉じタグ、文字実体参照、未知タグ・不正ネストの安全な正規化、`<style>` / `<script>` 抽出）。
- [x] **`UNIT-WEBVIEW-004` (`webview-css-cascade`)**
  - `crates/webview/src/css.rs`: CSS トークナイザ・宣言パーサ、User-Agent 既定スタイルシート、セレクタマッチング（タグ、`#id`、`.class`、子孫 ` `、子 `>`、擬似クラス `:hover` / `:focus` / `:disabled`）、詳細度（Specificity）計算とカスケード解決（`ComputedStyle` 生成）。

---

### Phase 2: Web レイアウトエンジン & `SceneSnapshot` 描画生成

- [x] **`UNIT-WEBVIEW-005` (`webview-layout-engine`)**
  - `crates/webview/src/layout.rs`:
    - Block Formatting Context（BFC: margin collapse, padding, border, width/height）
    - Inline Formatting Context（IFC: テキスト計測・行折り返し・ベースライン整列・インライン背景/境界）
    - Flexbox（`flex-direction`, `justify-content`, `align-items`, `flex-grow`）および GFM Table レイアウト
    - FreeMVVM Core の `Measure(LayoutConstraint)` / `Arrange(RectDip)`（`f64` DIP、非負・有限値検査）との統合。
- [x] **`UNIT-WEBVIEW-006` (`webview-paint-scene-snapshot`)**
  - `crates/webview/src/paint.rs`:
    - レイアウトツリーから `SceneSnapshot` 描画プリミティブ（`PushClip` / `PopClip`, `FillRect`, `StrokeRect`, `GlyphRun`, `DrawLine`）への変換。
    - ヘッドレス単体検証用の決定論的ソフトウェアラスタライザ / DisplayList スナップショット比較ハーネス。

---

### Phase 3: 軽量スクリプト / JavaScript エンジン VM & DOM / MVVM ブリッジ

- [x] **`UNIT-WEBVIEW-007` (`webview-script-vm`)**
  - `crates/webview/src/script.rs`:
    - 純粋 Rust 製の軽量 JavaScript サブセット（ECMAScript サブセット: 変数 `let`/`const`、数値/文字列/真偽値/オブジェクト/配列、関数・クロージャ、条件分岐・ループ）レキサ・パーサ・AST 評価・スタック VM。
    - 決定論的サンドボックス制約: 1 イベントあたりの最大実行命令ステップ数（`max_script_steps`）、最大コールスタック深さ、最大 VM ヒープメモリ量の強制（超過時は安全に中断し DOM トランザクションロールバック、無限ループによる UI スレッド凍結を防止）。
- [x] **`UNIT-WEBVIEW-008` (`webview-script-dom-mvvm-bridge`)**
  - サンドボックス化された DOM API（`document.getElementById`, `querySelector`, `textContent`, `setAttribute`, `classList`, `addEventListener`）と MVVM ホストブリッジ（`freemvvm.getProperty`, `freemvvm.setProperty`, `freemvvm.invokeCommand`）の接続。
  - スクリプトによる DOM 変更時の差分 dirty マークと再スタイル・再レイアウト・再描画連携。

---

### Phase 4: Core Runtime / C ABI / 4言語ラッパー / View 言語埋め込み統合

- [x] **`UNIT-WEBVIEW-009` (`core-runtime-webview-control`)**
  - `crates/core-runtime/src/webview.rs` への `WebView` Control 統合:
    - `WebViewInstance`、`Property<Utf8>` ソース文字列の OneWay Binding、読み書き/読み取り専用 `Property` 公開、`Command` コールバック公開
    - スクリプト実行・ポインタ click 配送・テキスト抽出・RGBA8 ソフトウェアラスタライズ
- [x] **`UNIT-WEBVIEW-010` (`c-abi-webview-surface`)**
  - `crates/c-abi/src/lib.rs` および `crates/c-abi/include/freemvvm/abi.h` への `ui_abi_1_*webview*` 11 関数群の実装と公開ヘッダ同期検査。
- [x] **`UNIT-WEBVIEW-011` ～ `014` (`wrappers-webview-{cpp,rust,csharp,python}`)**
  - C++ (`wrappers/cpp`)、Rust (`wrappers/rust`)、C# (`wrappers/csharp`)、Python 3 (`wrappers/python`) の各ラッパーへの `WebView` クラス追加と単体・統合テスト、および将来の View 言語コンパイラ向け宣言的埋め込みアダプタ契約の整備。
