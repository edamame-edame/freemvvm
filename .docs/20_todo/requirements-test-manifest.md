# requirements-test-manifest 作業計画

- 文書版: 0.3
- 状態: 完了（Requirements Test Manifest 0.2利用者承認済み、2026-10-14）
- 作成日: 2026-10-14
- 対象: テスト共通ルール・契約、要求traceability、case/evidence状態、coverage監査
- 保存先: `.docs/30_release_test/requirements-test-manifest.md`
- 引継ぎ元: 旧DD-23「縦断検証設計」案（再分類後に削除）と `.docs/20_todo/dd23-verification.md` の作業記録
- 関連文書: `.docs/30_release_test/requirement-test-matrix.md`、`.docs/30_release_test/release-test.md`、`.docs/31_version_test/version-test.md`、`.docs/32_unit_test/unit-test-strategy.md`

## 目的

横断的な検証レベル・case/evidence管理・要求coverage監査の共通規則を、詳細設計から分離してテスト契約文書にまとめる。要求テスト対応表のRQ-case catalog、Release/Version testの固有gate、unit-testの実装単位方針を複製せず、各文書が相互に矛盾なく使うための共通語彙と記録条件を定める。

## 作業手順

1. RQ-16の文書ロール、RQ章の承認状態、要求テスト対応表の役割と承認境界、Release/Version test規約、unit-test方針を再確認する。
2. DD-23案を分類し、テスト契約へ移す内容、要求test対応表へ委ねる内容、Release/Version testへ委ねる内容、DD/BDに残すべき製品設計内容があればその境界を記録する。
3. `.docs/30_release_test/requirements-test-manifest.md` に目的・適用範囲、用語、traceability規則、case/evidenceの状態契約、coverage audit、変更時再確認、各テスト文書との責務境界を文書化する。
4. RQ-case catalogや縦断caseの詳細、release/versionの個別gate、CI/framework/jobや数値閾値を重複・先取りしない。既存承認範囲は変えず、case設計・実装・実行結果を未承認/未実施のまま表示する。
5. DD-23を詳細設計フェーズから再分類し、DDフェーズ計画・進捗メモ・この作業計画の状態を整合させる。設計判断が残らない場合に限り詳細設計の完了項目へ数えない。
6. Markdown参照、承認状態、traceabilityの向き、重複記載、改訂/承認履歴、差分を確認してレビュー案を提示する。

## 2026-10-14 実施記録

- `.docs/30_release_test/requirements-test-manifest.md` 文書版0.1を作成。要求testのcoverage、case/evidence状態、traceability、coverage監査に絞り、個別case一覧とRelease/Version gateは各担当文書へ分けた。
- 旧DD-23を詳細設計対象外として再分類し、DDフェーズ表から外した。Requirements test matrix、unit-test strategy、DD-00/22、BD-00等に残るDD-23の現行参照を再分類した。移行案内ファイルは利用者指示により削除した。
- 文書版0.2のmanifestと共通ルール案を利用者へ提示し、承認を受けた。個別case、他test成果物、test実装・実行の承認範囲は拡張していない。

## 2026-10-14 追記

- 利用者の指示に基づき、共通ルール案からルールごとのIDを除去する。
- 要求から値を決められないデータをtestで使う場合の代表使用値・理由・出典・traceabilityの記録条件を明記する。
- 検証境界を示すtest ID prefix規則を定義し、logical case IDと実行test IDを区別する。既存case IDはこの編集だけで一括改番しない。
- 文書版0.2として更新し、ルール本文・prefix体系を利用者レビューへ提示した。
- 利用者は2026-10-14に文書版0.2と共通ルールを承認した。承認履歴をmanifestに記録し、本計画を完了とした。

## 対象外

- RQ本文、BD/DD/SD、要求テスト対応表の要求→case割当、Release testまたはVersion testの既存規則をこのmanifestだけで変更すること。
- test framework、CI provider、runner/job、command、result formatのwire schema、retention、coverage率・performance threshold等の実装運用値を決めること。
- 未承認caseや未実施テストを承認済み・合格として扱うこと。

## レビュー確認点

1. manifestがルール/契約を定め、要求テスト対応表がRQ-case対応を管理する関係が明確。
2. Release test・Version test・unit-testの既存責務と矛盾・重複しない。
3. RQの承認状態、論理case設計状態、実装/実行状態、証跡の有無を区別できる。
4. 参照は根拠となる上流仕様へ向き、要求から下流成果物へのリンクを作らない。
5. DD-23は詳細設計として承認・完了扱いにせず、DD計画に再分類を明記する。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-10-14 | テスト共通ルール・契約文書への再分類と作成 | 利用者 | 作成指示。文書内容の承認は未了 |
| 2026-10-14 | ルールID省略、未規定データの代表値記録、test ID prefix規則 | 利用者 | 追記指示。文書版0.2の承認は未了 |
| 2026-10-14 | Requirements Test Manifest 文書版0.2 | 利用者 | 共通ルール案を含む版0.2を承認 |

## 改訂履歴

| 版 | 日付 | 変更内容 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 0.1 | 2026-10-14 | DD-23案をテスト契約文書へ再編する計画を作成 | Codex | 利用者の作成指示。個別ルールは未承認 |
| 0.2 | 2026-10-14 | ルールID省略、未規定データの代表値traceability、test ID prefix体系の追記 | Codex | 利用者の追記指示。文書内容は未承認 |
| 0.3 | 2026-10-14 | manifest 0.2の利用者承認を記録し、作業計画を完了 | Codex | 2026-10-14承認を記録 |
