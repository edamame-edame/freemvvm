# DD-15 初期Control 作業計画

- 状態: 完了（2026-10-12、DD-CONTROL-01～12承認済み）
- 着手日: 2026-10-12
- 対象: Widget共通意味論、Window、StackPanel、Text、TextBox、Buttonの初期Control契約
- 上位出典: Accepted要件 §5～8、BD-04～13、BD-23/24、V5～V8、B1～B4、C1～C4、A21～A32
- 先行詳細設計: 承認済みDD-08～14。特にDD-09 Validation、DD-10 Command、DD-11 Tree、DD-12 Layout、DD-13 Input/Focus、DD-14 TextBox/IME
- 対応する詳細設計: `.docs/13_detail_design/15-controls-v1.md`

## 目的

最初の縦断シナリオ（Window → StackPanel → TextBox/Text/Button）に必要なControl責務と相互接続を定める。上位判断および承認済みDDの範囲を繰り返し開き直さず、各Controlの最小semantic properties、Command activation、Binding接続、Accessibility意味情報、作成・所属の前提を判断ID付きでレビューする。

## 小分けにした作業範囲

1. **共通Widget契約**: AccessibleName、enabled/focusable等の共通state、default値と親stateとの関係。
2. **Container契約**: Window content root、StackPanel children/orientation、DD-11/12のtree/layout接続。
3. **表示・編集Leaf**: Text表示値、TextBox確定Value・editing state・DD-14接続。
4. **Action Leaf**: Button表示label、Command参照、CanExecuteとactivation経路。
5. **縦断・Accessibility確認**: Window/StackPanel/TextBox/Text/Buttonを使う最小シナリオ、BD-13意味情報と後続DD境界。

## 作業手順

1. ADR-0001、Accepted要件、BD-04～13/23/24と該当するB/C/V/A判断を確認し、個別の状態と承認範囲を記録する。
2. DD-08～14の承認済み仕様を照合する。特に、Property/Binding、Command、Tree、Layout、Focus/Input、IMEの契約を変更せず接続点だけ扱う。
3. Widget共通属性と各Controlの責務を上記5範囲で切り分ける。名前、表示値、編集値、確定値、Command、focusability、enabledの重複/独立を明示する。
4. Window/StackPanelのchild cardinality・順序・orientation・LayoutをDD-11/12へ接続し、承認済み規則を再提案しない。
5. Text/TextBoxの値とBindingの方向・初期値・外部更新をDD-07～09/14へ接続し、IME内部・Unicode位置・undo/redoは再決定しない。
6. Button activationをA21～A24のCommand executeへ接続する。pointer/Accessibility actionとkeyboard shortcut binding・Focus traversalを区別し、後者は `.docs/20_todo/keyboard-actions-value-memo.md` の範囲に残す。
7. BD-13のrole/name/state/value/action意味情報をControl別に整理する。OS API写像と公開ABI/API symbolの完全一覧は後続へ送る。
8. `.docs/13_detail_design/15-controls-v1.md` にDD-CONTROL判断、Control表、縦断状態例、正常系・失敗系検証、未決事項を保存する。
9. `.docs/20_todo/detail-design-phases.md` と `.docs/01_memory/detail-design-planning-status.md` を更新し、リンク・承認状態・`git diff --check` を確認する。

## 承認境界と停止条件

着手時は計画と未承認レビュー案を作成し、利用者承認前にDD-15を完了扱いにせずDD-16へ進まない境界とした。2026-10-12にDD-CONTROL-01～12の承認を得て完了記録を反映した。BD-11/12/13や承認済みDD-08～14は変更していない。公開ABI symbol、OS Accessibility API写像、Renderer具体描画、shortcut/key mapは確定していない。DD-16への着手指示は別途必要。

## 対象外

- Tree add/removeの所有・失敗原子性とLayout measure/arrange（DD-11/12で承認済み）
- Property、Binding、Validation、Commandの共通意味論（DD-07～10で承認済み）
- TextBox編集/IME内部状態（DD-14で承認済み）
- 特定key/modifierとControl操作・Commandのbinding、Tab traversal、Undo/Redo shortcut（候補メモを参照）
- Renderer API、Visual template、OS window backend、IME candidate UI、Accessibility OS adapter実装（DD-16/17）
- ABI function/symbolの完全な公開一覧とwrapper固有型（DD-06、DD-18～21）

## 完了条件

- 5つの初期Control責務、common properties、各ControlとProperty/Binding/Command/Input/Tree/Layoutの接続が承認済み上位判断と整合する。
- 未決値、親子との継承関係、activation/close/eventの境界を明示し、曖昧な事項を実装都合だけで固定しない。
- 縦断シナリオと各Controlの主要な失敗・disabled/detached状態を検証可能にする。
- 利用者は2026-10-12にDD-CONTROL-01～12を記載どおり承認した。承認記録を詳細設計とフェーズ状況へ反映済み。§7の未決事項は未決のまま後続へ送る。DD-16の着手は別途指示を待つ。
