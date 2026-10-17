# 設計 / 未設計事項の一括レビュー案作成状況

- 更新日: 2026-10-17
- 範囲: 後続課題と現行BD/DD/SDに残った未設計事項の案作成
- 状態: 全200判断承認済み・正本適用済み（2026-10-17）

## 入口と件数

[全件索引](../20_todo/remaining-design-review/index.md)、[承認台帳](../20_todo/remaining-design-review/approval-register.md)、[元課題対応表](../20_todo/remaining-design-review/scope-inventory.md)、[作業計画](../20_todo/remaining-design-batch-review.md)を参照する。

20分野169判断と、要求へ戻す31判断を25文書へ保存した。承認台帳は200件で、全件未承認・未適用。394箇所の照合には既解決事項・履歴・一般的な処理保留も含むため、394件すべてが未設計という意味ではない。

## 引継ぎ注意

各方式・数値・API・最低版・profileは推奨案で、確定仕様ではない。利用者可視の条件を変える判断は要求変更案を別に承認する。特にPython通常終了の協調条件、offline期限、入力/表示の初期範囲を方式承認だけで変更しない。

SDKのexact patch/hash、鍵の実体、namespace所有、実機証跡、実テストpath/reportは[実環境の採取値](../20_todo/remaining-design-review/external-inputs.md)へ分けた。実在・取得済み・試験済みと捏造しない。将来候補の具体案は採用済み機能への昇格を意味しない。

## 検査

25文書・90表・16 Mermaidブロック・2089リンクについて表/リンク/anchor/ID/台帳対応の静的検査はエラー0件。正本98ファイルは作業開始時のSHA-256と一致した。製品実装、テスト実行、Mermaid描画検証は行っていない。

## 適用後の状況

利用者の「全件反映して下さい。」で採用・適用を承認した。[正本適用状況](remaining-design-application-status.md)と[適用結果](../20_todo/remaining-design-application-result.md)を現在の入口とする。上記作成時の未承認・正本不変更の記述は適用前の履歴である。
