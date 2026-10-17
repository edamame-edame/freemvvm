# 要求-テストケース対応表 作業計画

- 状態: レビュー案作成済み（未承認）
- 着手日: 2026-10-12
- 対象: Accepted要件仕様の項目単位traceabilityと、test caseの未対応・重複・不要候補を発見する逆引き
- 保存先: `.docs/30_release_test/requirement-test-matrix.md`
- 参照元: `.docs/11_requirements/reqirements-specification.md` と各RQ章、対応するBD判断・DD、`.docs/30_release_test/release-test.md`、`.docs/31_version_test/version-test.md`
- Requirements Test Manifestとの関係: 要求→case対応表は本書、共通test rule/contractは独立manifestで管理する。2026-10-14の利用者指示により、旧DD-23検証案はmanifestへ再分類した。

## 目的

各要求項目に対して、満たす証拠を示すtest case IDを明示し、要求側から未対応要求を、test側から要求に結び付かない孤立・重複候補を発見できる対応表を作る。初期版の検証計画と、将来段階・未決仕様・プロセス要求を区別し、未設計をpass扱いにしない。

## 作業手順

1. 要件仕様§3～13の規範的な機能要求、platform/distribution/NFR、開発順、未決事項、文書管理方針を項目単位で列挙し、安定IDを振る。
2. 要求source-of-truthと承認済みBD/DDを照合し、初期release、後続stage、将来候補、承認済み意味論、未設計事項、プロセス確認を識別する。
3. case IDを定義し、各caseの対象境界、主な判定、要求リンク、実装証跡の提示方法、必要な共有/重複理由を記す。具体case未設計の場合はplaceholderとcoverage gapを明示する。
4. 要求→caseとcase→要求の双方向表を作り、各行に実装証跡欄を設ける。coverage status、判定根拠、unmapped要求、unlinked case、同一oracleを重ねている重複候補を一覧化する。
5. 複数要求を一つのcaseで確認する場合、同じsetup/操作/結果を共有できる理由と、各要求固有のassertionが失われないことを示す。
6. Collection等の未計画詳細設計、performance threshold、future control、target未検証などを未解決として報告し、所管文書/次アクションを付ける。
7. 独立文書 `.docs/30_release_test/requirement-test-matrix.md` に初期mapping案を保存した。全要求を実装済み・検証済みとは記さず、初期計画の状態と文書承認の境界を明記した。

## 対象外と停止条件

- 個別のRelease/Version test gateやunit-test実装方式を本対応表だけで変更すること。
- Accepted要求、BD/DDの意味・承認状態を対応表だけで変更すること。
- 実際に実装または実行されていないcaseを完了・passと記すこと。
- CI/test framework、runner/job、数値performance threshold、未定義APIの期待値を確定すること。
- 要求にない挙動をcoverageのために追加し、製品要求として扱うこと。

## レビューで確認する点

1. 規範的な要求項目がすべて安定IDとcaseまたは明示的な非実行理由を持つ。
2. すべてのtest caseが一つ以上の要求または承認済み設計判断に結び付く。
3. 共通caseの再利用には理由があり、個別要求のassertionが消えていない。
4. 未カバー、未設計、将来対象、process check、重複候補が同じ状態へ混ざらない。
5. この文書は独立してreviewでき、共通test rule/contractはRequirements Test Manifestを参照する。

## 承認記録

未承認。対応表の範囲・status・case粒度を利用者確認後に更新する。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-10-12 | DD-23とは独立した文書の作成 | 利用者 | 作成指示。対応表全体の承認は未了 |
| 2026-10-12 | 正式RQ IDへの移行 | 利用者 | 要求仕様再編の指示 |
| 2026-10-14 | test rule/contractとの責務分離 | 利用者 | 旧DD-23をRequirements Test Manifestへ再分類する指示。対応表全体の承認は未了 |

## 改訂履歴

| 版 | 日付 | 変更内容 | 作成者 |
|---|---|---|---|
| 0.1 | 2026-10-12 | 独立した対応表の作業計画と初期案を作成 | Codex |
| 0.2 | 2026-10-12 | 要求仕様再編に伴い正式RQの参照へ更新。旧196行を195要求・個別case88件へ移行。現在の計画は `requirements-restructuring.md` を参照 | Codex |
