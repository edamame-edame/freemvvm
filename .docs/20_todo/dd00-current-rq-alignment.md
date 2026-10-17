# D00 / DD-00 現行RQ・BD・DD対応の整合計画

- 文書版: 0.3
- 状態: 完了（DD-00文書版0.2を利用者承認済み、2026-10-15）
- 作成日: 2026-10-15
- 対象: `.docs/13_detail_design/00-decision-map.md` の出典状態、上位判断索引、要求根拠・DDフェーズ対応
- 親計画: [要求・設計責務分離計画](requirements-design-separation.md)
- 調査台帳: [全BD/DD調査台帳 D00](requirements-design-separation-inventory.md)

## 目的

DD-00の2026-09-27承認範囲（上位出典・判断対応と未決事項の整理）を維持しつつ、現行の要求仕様・BD・DDの構成と状態に対応させる。承認済み文書の改訂となるため、差分は同じDD-00内に未承認の改訂案として示し、利用者の承認後に同文書の現在表・履歴へ反映する。

## 作業手順

1. 要求仕様索引とRQ-16、ADR-0001、DD-00が参照する基本設計文書、BD-36、DD-00～22の現在の承認範囲を確認する。
2. DD-00の出典表・BD索引・BD/DD対応表・初期縦断範囲・未決事項にある旧版・廃止DD-23・下流参照を特定する。
3. 各BD/DD群の上位要求根拠を出典元から照合し、章全体への一括参照で未対応を隠さず、要求根拠と作業割当を別列・別表で示す。
4. 要求test文書は詳細設計の下流成果物としてDD-00から参照せず、テスト共通契約は承認済みRequirements Test Manifest、個別caseは要求test対応表の担当として記述する。
5. 承認済みの設計意味・判断IDは変更せず、古い割当や不正確な状態表記だけを修正案としてDD-00に提示する。未承認の要求・BD・DDは承認済みと表示しない。
6. DD-00内の承認履歴・改訂履歴を追記し、元の2026-09-27承認範囲と今回の改訂案の承認境界を分ける。
7. 相対リンク、RQ/BD/DD対応、参照方向、承認状態、`git diff --check`を確認する。

## 対象外

- RQ、BD、DDの個別製品契約・判断内容の改訂や再承認。
- 未決のBD/DD/API項目の決定、新しい要求やtest caseの採用。
- DD-00を理由としたDD-23の復活。検証ルールは承認済みRequirements Test Manifest、case catalogは要求test対応表に従う。
- 下流の要求test成果物やrelease/version規約をDD-00から参照・管理すること。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-10-15 | D00の現行RQ/BD/DD整合見直し | 利用者 | 「次に行きましょう」に基づく次作業。具体的なDD-00改訂案の承認は未了 |
| 2026-10-15 | DD-00文書版0.2 / D00-RQ-01、D00-BD-01、D00-REF-01、D00-TEST-01、D00-GAP-01、D00-STATUS-01 | 利用者 | 改訂案を承認。本文・履歴を更新し、作業割当を本計画へ移管 |

## 改訂履歴

| 版 | 日付 | 変更内容 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 0.1 | 2026-10-15 | DD-00の現行出典・要求根拠・判断/フェーズ対応を整合する作業計画を作成 | Codex | 利用者の次作業指示 |
| 0.2 | 2026-10-15 | DD-00末尾に現行整合の改訂案を追加。既存本文は未変更 | Codex | 利用者レビュー待ち。改訂案の承認は未了 |
| 0.3 | 2026-10-15 | 承認済み案をDD-00へ反映し、DD割当・後続作業候補の管理先を本計画に移管 | Codex | 利用者によるDD-00 v0.2承認 |

## DDフェーズ割当・後続作業の移管記録

次表は旧DD-00にあった作業割当を計画情報として保持する。各行は現在の承認状態と要求を上書きせず、フェーズ番号は実装方式の承認を意味しない。実際の進行順は[詳細設計フェーズ計画](detail-design-phases.md)と各DD文書の承認状態を確認する。

| 上位判断群 | 計画上の主な詳細設計先 | 保持する後続作業・境界 |
|---|---|---|
| R1～R4、A1、A4 | DD-01～04 | token非再利用、object操作/release、shutdown、callback破棄を各上位契約の範囲で具体化 |
| A2～A3、BD-27～28 | DD-05、DD-06、DD-22 | error catalog/ABI表現、versioning詳細 |
| A5～A8、BD-01 | DD-05～07、DD-09、DD-18～21 | F64/DECIMAL生成・read/write・Validator ABI、decimal encoding、operator/converterは詳細化待ち。A5旧宣言は初期3値型段階の記録 |
| A9～A12、P1～P4、BD-02～03 | DD-02～04、DD-07 | 通知作業枠、yield/保留、診断、非収束停止、静的警告の具体化 |
| A13～A20、B1～B4、D1～D4、V1～V4 | DD-04、DD-07～09 | Binding graph/validation意味論を維持し、未決の表示順位・Validator差替え等を分離 |
| A21～A24、C1～C4、BD-25～26 | DD-03、DD-07、DD-10 | 同期ValidatorとAsyncCommand進捗・終端・operation IDの境界を維持 |
| A25～A32、BD-04～13、BD-23～24 | DD-02、DD-09、DD-11～17 | detached object、tree、layout、input/focus、IME、Control、renderer、accessibility |
| L1～L4、BD-14～22 | DD-18～22 | 4言語API/packageとtarget方針。承認済み上位契約内で具体化 |
| BD-29～32 | DD-22 | artifact trust・署名・manifestを具体化。key encoding/envelope、registry/fetch/cache運用は残課題 |
| BD-33～35 | DD-02～03、DD-12～17、DD-21～22 | stable ID、Collection batch、Selection、Grid virtualization。API・通知順・性能閾値は詳細化待ち |
| BD-36-01/02/04 | DD-02、DD-03、DD-06、DD-16、language wrappers | 個別詳細設計文書の出典欄で参照状態を確認。BD-36-03は未確認のため実装前提として割り当てない |

独立追跡が必要な作業候補:

- BD-15のPython lint/type check、MCP schema、temporary cache。
- BD-33～35のCollectionModel、SelectionModel、change batch、Grid virtualizationの詳細APIと振る舞い。
- BD-29～32のartifact provenance、signature trust、manifest、distribution接続。
- BD-23/24のfont/shaping corpus、IME platform API、edit/selection細則。

これらの作業候補は本計画と詳細設計フェーズ計画で調整する。DD-00本文には下流の担当番号や進捗を再掲しない。
