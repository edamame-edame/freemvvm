# 詳細設計 / クラス責務・関連・所有・メソッドの補足

- 更新日: 2026-10-17
- スコープ: DD19本文のクラス補足とBD1.0承認状態
- 計画・結果: [クラス設計補足](../20_todo/dd-class-design-supplement.md)
- 入口: [詳細設計](../13_detail_design/README.md)

利用者の「OK、承認します。DDはクラス設計をもう少し補足してください。」を、新BD11本文と入口の各1.0全文の承認、およびDDクラス補足の作成指示として扱った。BD承認を未作成の補足案へ拡張していない。

DD19本文それぞれに1.1改訂案の補足節を追加。計118行の型責務・保持データ・主要メソッド・所有/寿命、19個の縦向きclass図、呼出し/失敗境界とRQリンクを記載した。CoreはRustの合成/trait、4言語facadeとC ABI境界を区別する。文書は追加分冊していない。

Controlの共通state合成、Model/Builder/PreparedStateとimmutable segment/snapshot、handle公開参照とValidationPin、登録closureとInvocationGuard、Commandとoperation/cancel tokenを分けた。wrapperはlease/ticket/root/anchorの既存方式、Platform/build/解析器は各内部責務を型へ対応付けた。新内部型名とメソッド分担は改訂案であり、公開API新設の根拠にしない。

既承認1.0は `archive/approved-1.0-before-class-supplement/` に原文保存（Git除外）。各本文から補足節・版/状態・1.1履歴を除くと1.0と一致することを検査した。1.0の承認は維持、各1.1全文は確認待ち。次の承認指示では文書版全体の履歴へ結果を追記する。

静的検査は32文書・1448リンク/anchor・222表、class図の端点/brace/fenceにエラー0件。変更対象外295ファイルはhash一致。製品build/test、Mermaid描画、commit/pushは未実施。

## 図の選択的補強

利用者は「全部の文書に必要とは言っていません。設計の特定に必要な個所に補強してください。」と範囲を訂正した。図の種類を全19文書へ一律に揃えない。型と所有は既存クラス図、状態遷移・処理順は既存図を使い、責務の接続先または保持データを読み取りにくい箇所を選ぶ。

今回の対象は6本文。controls、input-focus、platform/abstractionへ機能委譲と依存境界のコンポーネント図、collections、commands、text-editingへ保持先・更新条件のデータフロー図を追加した。他の13本文には図を追加していない。Mermaidは縦向きとし、図ごとに矢印の意味を説明する。1.1改訂案内の補強であり、1.0承認範囲を変更しない。

追加部分を除くと6本文の作業前1.1と一致し、1.0本文保持検査も成功。追加図6個の構造、32文書の1454リンク/anchorと222表は静的検査エラー0件。今回の変更対象外328ファイルのhash一致。Mermaid描画は未確認。

## 1.1承認後の状態

2026-10-17の「OK,テスト文書作りましょうか」により、DD19本文と入口の1.1全文（クラス補足・選択的追加図を含む）を承認済みへ更新した。上記の確認待ちは作成時点の過去状態。テスト文書作成を開始したが、作成した0.1テスト案やtest実装/実行の承認は別管理とする。引継ぎは[test文書作成メモ](test-document-authoring.md)。
