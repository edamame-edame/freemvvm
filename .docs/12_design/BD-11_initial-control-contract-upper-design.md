# MVVM UIライブラリ 初期Control契約に関する上位判断案

- 状態: BD-11承認済み（2026-09-26）
- 対応: 要求仕様第5・6節の最初の縦断シナリオに含むControlの責務境界
- 根拠: 承認済みA21～A32、BD-04～BD-10、要求仕様第5～7節。DD-00は未承認

## BD-11 承認された判断

**共通抽象を`Widget`とし、その下に子Widgetを持ちうる分類として抽象`Container`を設ける。WindowとStackPanelはContainer、TextBox・Text・Buttonは初期版の葉Widgetとする。アプリが組み立てる所有・包含関係はLogical Treeで表し、配置・描画・hit-testは対応するVisual TreeとCore Layoutに委ねる。各型の共通役割だけをこの判断で定め、個別Property一覧、子数、見た目の既定値、イベントroutingや描画方式は後続に残す。**

| Control | 承認された初期責務 | 既存判断との接続 |
|---|---|---|
| Widget | UI要素の共通抽象。tree所属と共通のCore object寿命を持つ。直接生成できるか、共通Propertyを何にするかはここでは決めない。 | BD-04の親子所有・Detached寿命、BD-05のLogical/Visual分離 |
| Container | 子Widgetを持ちうる抽象分類。Logical Treeの子関係とBD-04の所有参照を持つカテゴリとして扱う。具体的な子数、順序、追加・削除APIを一律には規定しない。 | BD-04、BD-05 |
| Window | Runtime内のtop-level UI領域であり、初期案ではContainerの一種。アプリのLogical Tree rootとして一つのcontent rootを受け持ち、active WindowとFocusはRuntime管理へ同期する。 | BD-04、BD-05、BD-08 |
| StackPanel | Containerの一種。順序付きLogical childを保持し、子をVisual Treeへ対応付ける。Sizer／Locationerと設定可能なAlignにより並べて配置する。 | BD-04～BD-06 |
| Text | 葉Widget。文字列を表示する読み取り用Controlで、OneWay Bindingの表示先になれる。TextBoxの編集・IME責務は持たない。 | BD-05、BD-06 |
| TextBox | 葉Widget。編集文字列とselection/caret、入力状態をCore側で持つ編集Control。commit候補をA25～A28で検証し、IME session連携はBD-09／BD-10に従う。 | A25～A28、BD-07～BD-10 |
| Button | 葉Widget。ユーザー操作でCommandを起動するControl。CanExecute等の既承認Command契約に従い、実行中の同一Command再入をA21～A24に従って扱う。 | A21～A24、BD-07～BD-08 |

最初の縦断シナリオは、`Widget`を基底に`Window : Container`、`StackPanel : Container`、`TextBox / Text / Button : Widget`というカテゴリで表す。実際のLogical Treeは`Window → StackPanel → (TextBox, Text, Button)`であり、Windowのcontent rootは一つ、StackPanelのchildrenは順序付きとする案を推奨する。初期版では単純なVisual構成がLogical Treeと一対一に対応してよい。子所有参照はBD-04に従い、Widget間にLogical Tree以外の重複所有関係を作らない。

## 理由

- 要求仕様第5節が示す縦断シナリオを、そのまま初期Controlの最小責務セットとして扱える。
- `Widget`と`Container`を分ければ、子を持つ能力を各Container実装に散らさず分類でき、後からScrollViewerやListView等を加える際の共通基盤になる。
- Controlの意味をCoreで揃えると、C++／C#／Rust／Python wrapperから同じBinding、Command、Focus、入力検証の契約を使える。
- Tree所有、Layout、Input、Focus、IMEはすでに上位判断がある。ここではその接点を明示し、Controlごとに重複する別規則を作らない。

## この判断では決めないこと

- 各Controlの全Property、型、既定値、公開API名や初期化順。
- `Container`をRustの基底struct、trait、ABI上の別handle型のどれで表すか。上位契約では`Widget`との分類関係を定め、実装・公開形は詳細設計で決める。
- Windowのcontent rootを常に一つに制限するか、将来複数子を許すか。初期縦断では一つを推奨するが、公開APIの制約はここでは確定しない。
- TextBoxの複数行、選択編集コマンド、Undo/Redo、明示IME cancel。
- Buttonのpointer/keyboard activation詳細、event routing、視覚状態やstyle。
- StackPanelの方向・spacing等の既定値、overflow、子追加APIの原子性。
- Controlのtheme/template、native controlか独自描画か、pixel・DPI単位。
- Accessibility role/state/actionと各Platform APIへの写像。要求仕様第7節の必須性は維持し、別の上位判断で最小範囲を決める。

## 承認記録

2026-09-26に利用者承認。共通抽象を`Element`ではなく`Widget`とし、子Widgetを持ちうる抽象分類`Container`を設ける。WindowとStackPanelはContainer、TextBox・Text・Buttonは初期版の葉Widgetとする。Logical TreeとVisual Treeの役割、各Controlの責務は上記の通り。最初の縦断シナリオではWindowのcontent rootを一つ、StackPanelのchildrenを順序付きとする。Containerの公開API、一般の子数制約、ControlごとのProperty、描画・Accessibilityの詳細は後続判断とする。DD-00の承認やDD-01以降へ進む判断ではない。
