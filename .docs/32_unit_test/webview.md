# 単体試験：WebViewエンジンプラグイン

- 文書版: 1.0
- 状態: 1.0全文承認済み（利用者、2026-10-24）。実装単位ごとに順次実行・検証する
- 更新日: 2026-10-24

対象単位は `DomArena` / `ResourceBudget`、`MarkdownParser`、`HtmlParser`、`CssParser` / `Cascade`、`WebLayoutEngine`、`SceneSnapshot` / `AxSnapshot` 出力、`ScriptVm`、および Core Runtime / C ABI / 言語ラッパー統合。

上位根拠: [単体試験方針](unit-test-strategy.md)、[実装unit台帳契約](implementation-test-inventory.md)、[共通fixture・実行記録](README.md)。

設計根拠: [ui/webview.md](../13_detail_design/ui/webview.md) 1.0「DOM ArenaとMarkdown / HTML / CSSパーサ / WebLayoutEngineとSceneSnapshot / AxSnapshot生成 / 軽量Script VMとDOM / MVVM Bridge / C ABI宣言案」、[08-webview-sandbox-parser-limits.md](../14_secure_design/08-webview-sandbox-parser-limits.md) 1.0「SD-08」。

## WebView コアエンジン・スクリプト・統合試験

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-webview-dom-001"></a>UT-WEBVIEW-DOM-001 | `DomArena` への要素・テキスト追加、属性設定、親子走査、および最大バイト長・ノード数・最大ネスト深さ・属性数上限超過を入力する | 制限内では決定論的な `NodeId` と木構造を保持。上限超過時は `ResourceLimitExceeded` を返し、既存公開DOMを変更しない | [RQ-17-001](../11_requirements/RQ-17_webview-plugin.md#rq-17-001)、[RQ-17-002](../11_requirements/RQ-17_webview-plugin.md#rq-17-002)、[RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009)<br/>[TC-WEBVIEW-01](../30_release_test/requirement-test-matrix.md#case-tc-webview-01) |
| <a id="ut-webview-md-001"></a>UT-WEBVIEW-MD-001 | CommonMark / GFM（見出し、段落、強調・取消線、リスト、表、コードブロック、引用、リンク）および深さ上限超過Markdownを入力する | 期待通りのセマンティックDOMへ変換され、上限超過時は安全にエラーを返す | [RQ-17-003](../11_requirements/RQ-17_webview-plugin.md#rq-17-003)、[RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009)<br/>[TC-WEBVIEW-01](../30_release_test/requirement-test-matrix.md#case-tc-webview-01) |
| <a id="ut-webview-html-001"></a>UT-WEBVIEW-HTML-001 | HTML文書（ネスト要素、void要素、暗黙閉じタグ、文字実体参照、`<style>` / `<script>` 抽出、未知タグ）を解析する | 正規化されたDOMとスタイル/スクリプト抽出リストを生成し、未知タグでも異常終了しない | [RQ-17-004](../11_requirements/RQ-17_webview-plugin.md#rq-17-004)、[RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009)<br/>[TC-WEBVIEW-01](../30_release_test/requirement-test-matrix.md#case-tc-webview-01) |
| <a id="ut-webview-css-001"></a>UT-WEBVIEW-CSS-001 | UA既定スタイル、`<style>`、外部CSS、インラインstyleを与え、セレクタ詳細度（id/class/tag/combinator/pseudo）とカスケード解決を実行する | 各ノードの `ComputedStyle` が詳細度・出現順通りに決定され、未対応宣言は無視されルール上限超過時は拒否される | [RQ-17-004](../11_requirements/RQ-17_webview-plugin.md#rq-17-004)、[RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009)<br/>[TC-WEBVIEW-01](../30_release_test/requirement-test-matrix.md#case-tc-webview-01) |
| <a id="ut-webview-layout-001"></a>UT-WEBVIEW-LAYOUT-001 | Block（margin collapse）、Inline（折り返し）、Flexbox、Tableを含むDOMに対し異なるビューポート幅で `Measure` / `Arrange` を実行する | 各ボックスの論理DIP座標と内部スクロール高が算出され、非有限/負の制約は拒否して旧配置を保つ | [RQ-17-005](../11_requirements/RQ-17_webview-plugin.md#rq-17-005)<br/>[TC-WEBVIEW-02](../30_release_test/requirement-test-matrix.md#case-tc-webview-02) |
| <a id="ut-webview-paint-001"></a>UT-WEBVIEW-PAINT-001 | レイアウト済みツリーから `SceneSnapshot`（クリップ、背景、枠線、`GlyphRun`）および `AxSnapshot`（見出し、リンク、ボタン、入力欄、表）を生成する | ビューポートクリップ付きの描画コマンド列とセマンティックAccessibilityノード群が決定論的に一致する | [RQ-17-001](../11_requirements/RQ-17_webview-plugin.md#rq-17-001)、[RQ-17-012](../11_requirements/RQ-17_webview-plugin.md#rq-17-012)<br/>[TC-WEBVIEW-02](../30_release_test/requirement-test-matrix.md#case-tc-webview-02) |
| <a id="ut-webview-script-001"></a>UT-WEBVIEW-SCRIPT-001 | 軽量Script VMで変数・関数・制御構文・DOM操作を実行し、さらに無限ループ（燃料超過）・深すぎる再帰・未許可URIを実行する | 正常スクリプトはDOMを更新し再レイアウトを誘発。燃料・スタック超過時は即座に中断して未コミットDOM変更をロールバックし、外部URIを遮断する | [RQ-17-008](../11_requirements/RQ-17_webview-plugin.md#rq-17-008)、[RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009)、[RQ-17-010](../11_requirements/RQ-17_webview-plugin.md#rq-17-010)<br/>[TC-WEBVIEW-03](../30_release_test/requirement-test-matrix.md#case-tc-webview-03) |
| <a id="ut-webview-mvvm-001"></a>UT-WEBVIEW-MVVM-001 | Core Runtime上で `WebView` と `Property<Utf8>`（OneWay/TwoWay）、`PropertyValidator`、`Command` を接続し、C ABIおよび各言語ラッパーから操作する | Property更新で表示が同期し、入力確定/Validator拒否、リンク/スクリプトからのCommand起動と再入抑止が4言語共通に動作する | [RQ-17-006](../11_requirements/RQ-17_webview-plugin.md#rq-17-006)、[RQ-17-007](../11_requirements/RQ-17_webview-plugin.md#rq-17-007)、[RQ-17-011](../11_requirements/RQ-17_webview-plugin.md#rq-17-011)<br/>[TC-WEBVIEW-04](../30_release_test/requirement-test-matrix.md#case-tc-webview-04) |

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-24 | 1.0全文 | 利用者 | 「OK 専用ブランチを作って開始してください」により承認 |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 1.0 | 2026-10-24 | WebViewエンジンプラグインの単体試験設計を作成 | Antigravity | 2026-10-24承認 |
