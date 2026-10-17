# DD-14 TextBoxとIME 作業計画

- 状態: 完了（DD-TEXT-01～12利用者承認済み、未決事項を後続へ記録、2026-10-12）
- 着手日: 2026-10-12
- 対象: TextBox編集状態、caret/selection、IME semantic session、DD-09入力Validator接続
- 上位出典: `.docs/12_design/BD-09_ime-focus-transition-upper-design.md`、`BD-10_ime-platform-boundary-upper-design.md`、`BD-23_text-shaping-upper-design.md`、`BD-24_ime-edit-session-upper-design.md`、V5～V8、A25～A28、A30～A32
- 先行詳細設計: 承認済みDD-09 Validation、DD-13 Input/Focus、DD-02 lifetime、DD-03 dispatcher、DD-04 callback lifecycle、DD-07 Property、DD-08 Binding
- 対応する詳細設計: `.docs/13_detail_design/14-textbox-ime.md`

## 目的

TextBoxの確定値Propertyと編集中の文字列を一貫して扱い、通常編集・IME preedit・commit・explicit cancel・FocusLostをCore編集状態へ反映する詳細レビュー案を作る。DD-09のAccept/Incomplete/Reject、DD-13のFocus遷移、BD-09/10/24の承認済みIME契約を接続し、文字列・caret/selection・composition rangeの整合と失敗時の保持を判断ID付きで整理する。

特定のkey/modifierと編集操作・Focus移動・Commandを結び付ける契約（例: Tab/Shift+Tab、Ctrl+Z）は含めない。Undo/Redoの編集履歴も含めず、候補メモおよび後続設計へ送る。caret/selectionを保持・更新する契約と、キー割当は分けて扱う。

## 作業手順

1. ADR-0001、Accepted要件、A25～A28/A30～A32、V5～V8、BD-09/10/23/24、承認済みDD-09/13およびDD-02～08の関連節を読み、個別の承認状態と未決事項を確認する。
2. TextBoxの確定値、編集中の全文、selection/caret、active composition/session、input state/reasonの概念状態と所有者を区別する。
3. 編集範囲・候補全文・post-edit selectionを準備し、DD-09のValidator結果に応じて編集値、確定値、Bindingを更新する原子性を提案する。TextBox確定Property自身のValidator拒否時は未決事項として扱う。
4. BD-24のStart/Update/Commit/Explicit Cancel/FocusLost/stale eventをCore状態遷移へ接続する。session IDの具体表現、composition snapshot対象、重複・異常eventの処理を識別する。
5. UTF-8 byte offset、Unicode scalar、grapheme boundary等の選択位置表現候補を比較し、A25のABI byte offsetとCore内部caret単位を混同しない。
6. FocusLost後の再Focus、detach/破棄、外部Property更新、TextBox自身のProperty検証、IME commit candidate退避失敗、callback通知の接続を検討する。
7. OS candidate UI、composition decoration、text shaping、Platform API/event orderingのDD-16境界を守り、undo/redo・shortcut/action binding・key mappingを対象外にする。
8. `.docs/13_detail_design/14-textbox-ime.md` にDD-TEXT判断、状態遷移図、編集例、正常系・失敗系検証ケース、未決事項を記録する。
9. `.docs/20_todo/detail-design-phases.md` と `.docs/01_memory/detail-design-planning-status.md` に着手状態を反映し、リンクと `git diff --check` を確認する。

## 承認境界と停止条件

DD-14の着手指示は計画と未承認レビュー案の作成を許可した。DD-TEXT-01～12の利用者承認を受けて判断結果を同文書へ反映し、本計画を完了にした。DD-15への着手は別途指示を待つ。承認済みのBD-09/10/24、DD-09、DD-13は変更しない。ABI symbol、OS固有IME API、rendering/candidate UI実装、Control公開APIの完全一覧は本フェーズで確定していない。

## 対象外

- Undo/Redo履歴とそのkey binding、一般的なshortcut/action binding、Tab/Shift+TabのFocus traversal（`.docs/20_todo/keyboard-actions-value-memo.md` を参照）
- 特定キーからcaret移動・selection変更を起動する割当、Control横断のCommand binding
- OS固有IME API、IME candidate UIの選択・ページ操作・配置、composition decoration描画
- Text shaping/rasterization、font discovery、Visual text hit testing、line breakingの実装（BD-23およびDD-16/15）
- TextBoxの公開ABI symbol/API一覧、各言語wrapperの具体signature（DD-15、DD-18～21）
- 複数TextBox/複数Window間IME context共有、非同期/server validation

## 完了条件

- 編集値と確定値、selection/caret、IME session/composition、input state/reasonの責務が承認済み上位判断に結び付く。
- ordinary edit、Accept/Incomplete/Reject、IME commit/cancel/FocusLost/stale event、外部更新、detach/破棄、失敗時保持を検証可能にする。
- Unicode位置単位とProperty Validator拒否等、上位判断から決まらない事項を未決として可視化する。
- 利用者承認後に判断記録・計画・フェーズ状況を同じ作業サイクルで更新する。
