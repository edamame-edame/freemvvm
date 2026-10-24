# SD-08: WebView入力境界・パーサ上限・スクリプトサンドボックス

- 文書版: 1.0
- 状態: 設計承認済み・正本適用済み（2026-10-24）
- 文書種別: secure-design
- 根拠RQ: [RQ-17-001](../11_requirements/RQ-17_webview-plugin.md#rq-17-001)、[RQ-17-002](../11_requirements/RQ-17_webview-plugin.md#rq-17-002)、[RQ-17-008](../11_requirements/RQ-17_webview-plugin.md#rq-17-008)、[RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009)、[RQ-17-010](../11_requirements/RQ-17_webview-plugin.md#rq-17-010)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)
- 基本設計: [UIクラス：WebViewエンジンプラグイン](../12_design/ui/webview-plugin.md)

<a id="sd-08"></a>

本書はWebViewエンジンプラグインにおける非信頼入力（Markdown、HTML、CSS、Script、URI）に対する信頼境界、決定論的リソース上限、およびサンドボックス隔離の実現方式を定めるレビュー案である。

## 実現判断

| 設計ID | 実現手段・責務 | 要求根拠 |
| --- | --- | --- |
| <a id="sd-wv-01"></a>SD-WV-01 | ABIまたはProperty経由で渡されるMarkdown/HTML/CSS/Script文字列はすべてuntrusted inputとして扱い、UTF-8妥当性検査と入力バイト上限（初期既定: 単一文書1 MiB）を通過した後にのみパーサへ渡す。 | [RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006) |
| <a id="sd-wv-02"></a>SD-WV-02 | DOM Arenaは1文書あたりの最大ノード数（初期既定: 16,384ノード）、最大木深さ（初期既定: 128段）、要素あたり最大属性数（初期既定: 64個）を強制する。超過時は解析を中断し、直前の有効な公開DOMを維持する（fail-closed）。 | [RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009) |
| <a id="sd-wv-03"></a>SD-WV-03 | CSSパーサはルール数上限（初期既定: 4,096ルール）、セレクタ結合子長上限（初期既定: 16段）、宣言数上限を強制し、指数的バックトラックを起こさない線形・有界セレクタ照合のみを実装する。 | [RQ-17-004](../11_requirements/RQ-17_webview-plugin.md#rq-17-004)、[RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009) |
| <a id="sd-wv-04"></a>SD-WV-04 | 軽量スクリプトVMはOSシステムコール、ファイルI/O、ネットワークソケット、任意メモリポインタ、動的コード生成（`eval` / `Function`コンストラクタ）を持たない純粋バイトコードインタプリタとする。 | [RQ-17-008](../11_requirements/RQ-17_webview-plugin.md#rq-17-008)、[RQ-17-010](../11_requirements/RQ-17_webview-plugin.md#rq-17-010) |
| <a id="sd-wv-05"></a>SD-WV-05 | スクリプト実行ごとに決定論的な燃料カウンタ（初期既定: 100,000命令ステップ）、最大コールスタック深さ（初期既定: 64フレーム）、VMヒープ上限（初期既定: 4 MiB）を強制する。上限超過時はスクリプト実行を打ち切り、当該イベント内の未コミットDOM変更を破棄してエラー診断を記録する。 | [RQ-17-008](../11_requirements/RQ-17_webview-plugin.md#rq-17-008)、[RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009) |
| <a id="sd-wv-06"></a>SD-WV-06 | 外部URI解決は既定ですべて遮断（offline-by-default）とし、`http:`、`https:`、`file:`、`javascript:`、`data:`等の暗黙フェッチやスクリプト実行スキームを拒否する。ホストが明示登録した名前付きリソースまたはCommand起動のみを許可する。 | [RQ-17-007](../11_requirements/RQ-17_webview-plugin.md#rq-17-007)、[RQ-17-010](../11_requirements/RQ-17_webview-plugin.md#rq-17-010) |
| <a id="sd-wv-07"></a>SD-WV-07 | スクリプトからアクセス可能なViewModel PropertyおよびCommandは、ホストが当該WebViewインスタンスへ明示的にバインド（whitelist公開）した対象に限定し、Runtime内の任意のhandleトークン探索を禁止する。 | [RQ-17-006](../11_requirements/RQ-17_webview-plugin.md#rq-17-006)、[RQ-17-008](../11_requirements/RQ-17_webview-plugin.md#rq-17-008) |

## リソース予算の既定値と構成境界

| 制限項目 | 初期既定値 | 超過時の動作 |
| --- | --- | --- |
| 文書ソース最大バイト長 | 1 MiB (1,048,576 bytes) | 解析拒否・既存DOM維持・エラー通知 |
| DOM最大ノード数 | 16,384 nodes | 候補DOM破棄・既存DOM維持・エラー通知 |
| DOM最大ネスト深さ | 128 levels | 候補DOM破棄・既存DOM維持・エラー通知 |
| CSS最大ルール数 / セレクタ深さ | 4,096 rules / 16 combinators | 超過ルールまたは文書更新の拒否 |
| Script 1実行あたり最大命令ステップ数 | 100,000 steps | 実行中断・未コミット変更rollback・エラー通知 |
| Script最大コールスタック深さ | 64 frames | 実行中断・未コミット変更rollback・エラー通知 |
| Script VM最大ヒープメモリ | 4 MiB (4,194,304 bytes) | 実行中断・未コミット変更rollback・エラー通知 |

## 承認履歴

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| — | 0.1 / SD-WV-01～07 | — | レビュー待ち（未承認案） |
| 2026-10-24 | 1.0 / SD-WV-01～07 | 利用者 | 「OK 専用ブランチを作って開始してください」により承認 |

## 改訂履歴

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-24 | WebViewエンジンプラグインのセキュア設計初版案を作成 | Antigravity |
| 1.0 | 2026-10-24 | 利用者承認を反映し正本化 | Antigravity |
