# Keyboard Action Binding 付加価値候補メモ

- 状態: アイデア記録（詳細設計・実装の着手指示ではない）
- 記録日: 2026-10-12
- 関連: DD-13 InputとFocus、DD-14 TextBoxとIME、DD-15 初期Control

## 検討したい価値

キーバインドとUI機能を結び付け、共通のkeyboard操作を利用者へ提供することは、付加価値候補として検討する価値がある。個々のControlがkeyを独自解釈するだけでなく、必要に応じてFocus移動やUI actionへ対応付ける仕組みを考える。

例:

- Tab / Shift+TabなどによるFocus traversal
- Ctrl+ZなどによるUndo action
- 将来追加するUI command/actionへのkey binding

これらは例示であり、既定のキー割当、必須機能、対象Control、実装時期を決めたものではない。

## 将来の検討論点

- 共通のkey binding機構をCoreが提供するか、各wrapper/Control/アプリへ委ねるか。
- actionの有効範囲をfocused Control、Container、Window、Runtime、アプリのどこに置くか。
- 同じkeyがTextBox編集、IME、Focus traversal、Commandなど複数用途に当たる場合の優先順位と伝播。
- OS/keyboard layout/修飾キー差、利用者による割当変更、既定値の提供方針。
- Accessibility、競合するshortcut、key repeat、押下とtext commitの区別との整合。
- Undo/Redoの対象範囲、編集履歴、Commandとの関係をどの別設計で扱うか。

## 扱い

本メモは機能要求や承認済み仕様ではない。DD-13ではraw keyboard/text eventの正規化・配送とFocus状態を扱い、key/modifierをFocus traversalまたはUI機能へ結び付ける契約は含めない。将来この候補に着手する場合は、対象ユースケース、初期対象範囲、上位設計への反映要否を先に計画してレビューする。
