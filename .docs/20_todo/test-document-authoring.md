# テスト文書 / 詳細設計から実行可能な試験設計への具体化

- 作成日: 2026-10-17
- 状態: 初回試験設計作成・静的検査完了。追加9文書の0.1全文承認済み（利用者、2026-10-17）
- 指示根拠: 「OK,テスト文書作りましょうか」。DD1.1改訂案の承認とテスト文書作成の開始指示として扱う。

## 作業範囲

要求testの既存280RQ・92論理caseと共通manifestは維持し、重複する要求カタログを作らない。承認済みDDをもとに、unitの前提・操作・観測・期待値・後始末・異常注入を具体化する。独立検証可能なunit境界は候補として記し、未存在source path/commandやCI実装を確定しない。実装と同じrevisionで実行する契約を引き継ぐ。

既存テスト規約を入口に、単体試験を6文書（Core/runtime、events、collections、UI、wrappers、platform/build）へまとめる。実環境依存の項目はunitで証明できる範囲とplatform/package試験を区別する。別途、release用の縦断scenarioとversion用の回帰選択・互換確認文書を作成する。将来段階は現行必須と区別する。

追加文書は0.1レビュー案・未実装・未実行。今回のDD承認を未作成のテスト文書へ拡張しない。既存設計との不整合はgapへ記録し、期待値をtest都合で確定しない。設計IDは導入せず、テスト識別子は既存規約のprefixを用い、設計根拠は文書版・節で示す。

## to-do

- [x] 作業指示・Git規則・既存unit/release/version規約・manifest・要求対応表を確認する。
- [x] DD19本文と索引の1.1全文承認を記録し、既存履歴を維持する。
- [x] RQ/BD/DD/SD根拠と未決事項を確認し、対象単位・fixture・試験項目を具体化する。
- [x] 単体試験の入口と6分野の詳細文書を作成する。
- [x] release縦断scenarioとversion回帰文書を作成し、既存規約から案内する。
- [x] testcase識別子・RQ/TC/設計リンク・表・fence・変更範囲を静的検査する。
- [x] 結果と引継ぎを作業メモへ保存する。

## 完了条件と対象外

各caseに判定可能な入力/操作とoracleがあり、共通fixtureを含め前提と後始末を辿れること。要求test、unit、integration/system、platform/packageの証明範囲を混同しないこと。試験実装、実行、workflow実装、release判定、commit/pushは今回の対象外。

## 作成結果と検査

作成後、利用者の「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」により既存9文書の0.1全文を承認済みへ更新した。以下の未承認記述は作成時点の状態を残す記録である。[環境整備計画](development-environment-bootstrap.md)へ進む。試験実装・実行状態は未完了のまま保持する。

入口は[単体試験設計](../32_unit_test/README.md)、[release縦断](../30_release_test/system-scenarios.md)、[version回帰](../31_version_test/regression-plan.md)。単体6分野70項目、release9scenario、version6項目を作成した。新文書は計9本の0.1未承認案、全test未実装・未実行。280RQ・92TCの既存対応表を変更せず、全要求の詳細化完了ともしていない。

追加予定test ID85件の重複なし。BD/DDとtest案内を含む1983リンク/anchor・277表、fenceを静的検査してエラー0件。DD20文書の1.1承認記録、19本文の1.0原文保持、変更対象外312ファイルのhash一致を確認した。git diff --check成功。製品build/test、CI実装/実行、Mermaid描画は未実施。

Command進捗channel、TextBox通知順、Focus図の無効要求矢印の3差異を[接続gap](../32_unit_test/README.md)へ記録した。未定義APIをtest都合で確定せず、対応unit実装着手前にDDの整合改訂へ戻す。以後は文書レビュー→差異解消→実装基盤/UNIT境界確定→同一revisionの実装と対応test作成・実行へ進む。これら後続作業は今回完了扱いにしない。
