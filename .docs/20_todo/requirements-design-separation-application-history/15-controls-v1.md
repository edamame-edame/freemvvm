# 適用前記録 / 15-controls-v1.md

- 記録日: 2026-10-17
- 区分: 2026-10-17改訂前の履歴。現行仕様として使用しない
- 元文書: [15-controls-v1.md](../../13_detail_design/15-controls-v1.md)

---

# DD-15 初期Control

- 状態: 承認済み（2026-10-12、DD-CONTROL-01～12）
- 作成日: 2026-10-12
- 対象: Widget共通意味論、Window、StackPanel、Text、TextBox、Button
- 上位出典: Accepted要件 §5～8、BD-04～13/23/24、V5～V8、B1～B4、C1～C4、A21～A32
- 先行詳細設計: 承認済みDD-08～14
- 作業計画: `.docs/20_todo/dd15-initial-controls.md`

本書は最初の縦断シナリオに必要なControl契約を定める。利用者は2026-10-12にDD-CONTROL-01～12を記載どおり承認した。これは公開API/ABIや実装済みControlを意味しない。§7に列挙した事項は未決のまま後続へ送る。承認済みTree、Layout、Property、Binding、Command、Validation、Focus、TextBox/IME意味論を再定義しない。OSへのAccessibility写像・描画・key bindingの既定は後続または別候補とする。

## 1. 責務境界

- BD-11のとおり`Widget`は共通抽象、`Container`は子を持ちうる分類、`Window`/`StackPanel`はContainer、`TextBox`/`Text`/`Button`は葉Widgetとする。
- Logical Treeはアプリが組み立てる意味上の所属と所有を持つ。Visual Tree/Layoutは描画・配置・hit-test情報を担い、重複所有しない（BD-04～06、DD-11/12）。
- PropertyとBindingはCoreの共通Property graphへ接続する。ButtonのCommandはA21～A24/DD-10のCommand objectを起動する。Control固有状態を別のBinding semanticsにしない。
- CoreがControlとAccessibilityの意味上のsource of truthを持ち、共通RendererとPlatform Adapterが表示面・入力・IME・Accessibility OS接続を行う（BD-12/13）。
- 特定key/modifierをControl操作またはCommandへ割り当てない。Buttonへのkeyboard shortcut、TextBox caret key mapping、Tab traversal、Ctrl+Z等は`.docs/20_todo/keyboard-actions-value-memo.md`の候補範囲。

## 2. 上位判断と承認済みDDの継承

| 範囲 | 継承する決定 |
|---|---|
| Widget/Container/Tree | Logical parentは一つ、所有参照とDetached/reparentの規則はDD-11に従う。Windowは一つのcontent root、StackPanelは順序付きchildren。 |
| Layout | StackPanelのVertical/Horizontal、Vertical既定、Align/constraints/Measure/ArrangeはDD-12に従う。spacing/margin/padding等をここで追加しない。 |
| TextBox | `Value`は確定値Property。editing text、selection/caret、IME session、preedit/cancel/commitはDD-14に従う。 |
| Text/Binding | Textは表示専用。初期縦断ではViewModel text sourceからTextBox.ValueへTwoWay、Text.ContentへOneWayで結ぶ（B1～B4、DD-08/09）。 |
| Button/Command | activationからCommand.executeへ接続し、can-executeと実行中二重起動拒否をDD-10/A21～A24に従って扱う。 |
| Focus/Input | active focus、enabled/focusable eligibility、Input routeはDD-13に従う。Controlごとのfocusable既定値はここで確認する。 |
| Accessibility | 各初期Controlにrole/name/state/value/action（該当時）を持たせ、Logical Treeを基礎にする（BD-13）。OS API写像はDD-17。 |

## 3. 判断案

| ID | 承認された判断 | 根拠・境界 |
|---|---|---|
| DD-CONTROL-01 | 初期版は具象`Window`、`StackPanel`、`TextBox`、`Text`、`Button`を生成可能とし、`Widget`/`Container`は抽象分類で直接生成しない案。各Controlは同一Runtimeに属するCore objectとして作る。 | BD-11、DD-01/02/11。言語wrapperのclass/trait/API名はDD-18～21へ送る。 |
| DD-CONTROL-02 | 初期Control共通の意味情報として`AccessibleName`（UTF-8、空欄可、既定空）を持たせ、表示文字列やWindow titleと独立させる。表示Text/Labelからのfallbackとlabel関係は加えない。 | BD-13は明示的name設定/取得、空欄およびfallback未決を認める。OS-specific name propertyへの写像はDD-17。 |
| DD-CONTROL-03 | Widgetはown `IsEnabled` BOOL（既定true）を持ち、effective enabledは自分と全Logical ancestorのenabledの論理積とする案。親を無効化しても子のown値は変更しない。effective値はInput/Focus、Button execute可否、Accessibility stateで共有する。 | BD-08/13、A21、DD-10/13。親disabled時に子のProperty状態を変更せず操作だけ抑止する意味を確認する。 |
| DD-CONTROL-04 | `Focusable`はWidgetごとに扱い、初期既定をTextBox/Button=true、Text/StackPanel/Window=falseとする。Window active/focus状態は子Controlのkeyboard focusと区別する。明示FocusableをContainerに付けた場合の実挙動は後続scope設計へ送る。 | BD-08/13、DD-INPUT-02/03/08。既定値がControl役割に合うか確認する。 |
| DD-CONTROL-05 | 初期Controlには共通`IsVisible`/Hidden/Collapsed APIを追加せず、画面から外す操作はLogical Treeからのremoveで行う案。visibilityによるMeasure/Focus/Accessibility eligibilityを暗黙に作らない。 | 初期要件にvisibility機能は列挙されず、BD-06/08/13は詳細を後続へ送る。将来必要ならVisible/Hidden/Collapsedの意味を別判断する。 |
| DD-CONTROL-06 | Windowは一つのLogical content rootを受け持ち、rootの追加/除去と寿命はDD-11のTree操作に従う。Windowの開閉・surface生成はControlのLogical ownershipとは分け、Core意味論・Platform操作境界をDD-16で接続する。 | BD-04/11/12、DD-TREE-02/03/08。titlebar、native decoration、close callback、Window titleのPlatform表示は確定しない。 |
| DD-CONTROL-07 | StackPanelはDD-12の順序付きchildrenとorientation/AlignをそのままControl意味論とし、独自のlayout Propertyや追加spacingを作らない。追加/削除/並び順変更はDD-11のUI-thread原子操作へ接続する。 | BD-06/11、DD-TREE-02/03、DD-LAYOUT-04/05。公開collection facade/child API名は決めない。 |
| DD-CONTROL-08 | Textは`Content` UTF-8 Propertyを表示値として持ち、既定は空文字列、OneWay Bindingのtargetにできる読み取り専用Controlとする。Accessibility valueは同じ表示値を参照し、`AccessibleName`は別設定とする。 | BD-11/13、A6、B1～B4。wrap/ellipsis/selection/copy/action、fallback name、line-break policyはここで決めない。 |
| DD-CONTROL-09 | TextBoxはA30/DD-14の確定`Value` PropertyをTwoWay Binding可能とし、editing text、selection/caret、input state/reasonはDD-14の状態getter/subscriptionに接続する。Control levelに別のTextBox-only Binding semanticsを追加しない。 | A25～A32、DD-09/14。編集API、公開property handle一覧、symbolはABI/wrapper設計へ送る。 |
| DD-CONTROL-10 | Buttonは表示用UTF-8 `Text`/labelと`AccessibleName`を独立させ、optional Command associationからactivationを処理する案。Command associationはgeneric Value Propertyでなくtyped Command referenceとし、同Runtime Commandのみを設定可能にする案。 | BD-11/13、A21～A24、DD-10。label property名、Command associationの設定/binding面は確認対象。 |
| DD-CONTROL-11 | Buttonのeffective executable stateを`IsEnabled && assigned Command exists && Command.can_execute`とする案。disabled状態のactivationはCommand.executeへ進まず、実行中の二度押しもDD-10の実行可否規則に従う。CommandなしのButtonを表示・無操作にするか、生成/設定を拒否するかは選択肢として確認する。 | A21～A24、DD-CMD-02/07、BD-13。Command由来のProperty notificationを同期flushしない。 |
| DD-CONTROL-12 | Pointer activationとAccessibility invokeは一つのsemantic Button activation経路を通し、同じCommand.execute eligibilityを適用する。Buttonの直接Click callback、OS別button semantics、keyboard key bindingは初期契約に追加しない案。 | BD-07/11/13、DD-13、DD-10。Pointer hit-test/input gestureはDD-16、AT操作のOS bridgeはDD-17。 |

## 4. Control契約の一覧案

| Control | Core meaning / value | Property/Command接続 | Focus / Accessibility |
|---|---|---|---|
| Window | top-level container、単一content root、active window state | Tree owner。root layout areaはDD-12/16から供給 | window role、AccessibleName、active/focus state。window surface lifecycleはDD-16 |
| StackPanel | ordered child container、orientation既定Vertical | child collectionをDD-11へ接続、LayoutはDD-12 | group/container role、children order、name/state。focusable既定は確認案 |
| Text | static text、UTF-8 Content | OneWay target | static text role、name、display value、actionなし |
| TextBox | editable text、確定ValueとDD-14 editing state | TwoWay target/source、DD-09 Validator | editable text role、name/value/state/focus/enabled/validation |
| Button | action leaf、display label | optional typed Command association、CanExecute連動 | button role、name/state/focus/enabled。invokeはsemantic activationへ |

## 5. 縦断シナリオ案

1. RuntimeにWindowを作り、StackPanelを一つのcontent rootとして所属させる。
2. StackPanelにTextBox、Text、Buttonを順序付きで追加する。
3. ViewModelのUTF-8 `Name`をTextBoxの確定`Value`へTwoWay、`Message`をTextの`Content`へOneWayで接続する。
4. TextBoxで入力candidateをDD-09/14経路に通し、Acceptのみ確定Property/Bindingへ伝播する。IME preeditは未確定編集状態に留まる。
5. Button activationでassigned Commandがありかつeffective enabledならDD-10 executeを一度依頼し、実行中はeffective executable=falseとなる。
6. Window/StackPanel/leaf controlsからAccessibleName、role相当、state/valueを共通Core意味情報として得る。具体OS bridgeはテスト対象にしない。

このシナリオはcontrol semanticsのレビュー例であり、具体コンストラクタ名、ABI symbol、UI source-code DSL、keyboard shortcutを定めない。

## 6. 検証ケース案

1. 5具象Controlが同Runtime上で作られ、Widget/Container分類、Window single-root、StackPanel orderがDD-11制約と一致する。
2. StackPanel orientation/layout/AlignがDD-12に従い、未承認spacing/margin propertyを暗黙に作らない。
3. Text.ContentはUTF-8値として読め、OneWay updateで表示値/Accessibility valueが更新され、TextBox editing stateへ結合しない。
4. TextBox.Value TwoWayとediting/preedit状態を区別し、DD-09/14のAccept/Incomplete/Reject、IME focus loss、外部値更新に従う。
5. Buttonなし/Command設定/Command条件false/Command実行中/parent disabledを試し、execute回数とeffective enabled/accessibility stateが合う。
6. AccessibleNameは表示文字列から独立し、未設定を空欄として読み取れる。name fallbackは起きない。
7. disabled ControlはFocus要求・Button activation・accessibility stateで同じeffective enabled値を観測する。ancestor disabled時のpropagation案も確認する。
8. detached subtreeはDD-11に従いWindow/Layout/Input/Accessibility対象から外れ、再所属時にControl Core値・Property/Bindingが保たれる。
9. Command associationの変更、Button破棄、Window close、Command owner close/shutdownでA21～A24/DD-10のlifetime契約を壊さない。
10. Pointer activationとAccessibility invokeが共通semantic routeを通る。keyboard shortcutを本書が暗黙に付与していない。

## 7. 未決事項と後続設計

- Common `IsEnabled`のancestor effective semantics、Control別Focusable既定値、Focusable Containerの操作: 利用者確認対象。本判断でTab traversalは決めない。
- visibility propertyが初期Controlに必要か、その表示/measure/focus/accessibility意味: 要求追加または別判断まで保留。
- Window title/close event/native decoration、Window surface open/closeとRuntime/Command screen ownerの接続: DD-16およびlifetime/wrapper設計。
- Button label property名とText/AccessibleNameのfallback、Command associationのAPI/Binder surface、Command strong-referenceとclose owner。特に複数Button共有時に一Button破棄がCommandをcloseしないことを確認。
- Text overflow/wrapping/selection、TextBox public selection/caret API、button visual states/style/theme: Renderer/Control follow-up。
- OS Accessibility mapping・event payload・AT実機検証: DD-17。C ABI symbolsと4言語の具体型: DD-18～21。
- Tab/Shift+Tab、Ctrl+Z、その他key/action bindingsとUndo/Redo: `.docs/20_todo/keyboard-actions-value-memo.md`。本承認案に含めない。

## 8. 上位判断へ戻す境界

Widget分類・各Controlの基本責務、Accessibility意味情報、独自描画方針、初期Window/StackPanel構造はBD-11/12/13の範囲を変更しない。Property/Binding/Command/Input/IME/Tree/Layoutの承認済み意味論と衝突する提案はDD-15内だけで確定せず、該当判断の上位設計へ戻す。

## 9. 承認記録

- 2026-10-12、利用者はDD-CONTROL-01～12を記載どおり承認した。
- §7の未決事項は本承認により確定せず、後続設計の課題として維持する。特にWindow surface/close、Button label名とCommand association/lifetime、Text表示規則、Accessibility OS写像とABI symbolsを後続へ送る。
- Tab/Shift+Tab、Ctrl+Z等のkey/action bindingは承認対象に含めず、`.docs/20_todo/keyboard-actions-value-memo.md` の候補範囲に残す。
- DD-16への着手は本承認に含まれず、別途指示を待つ。
