# DD-13 InputとFocus 作業計画

- 状態: 完了（DD-INPUT-01～10利用者承認済み、2026-10-12）
- 着手日: 2026-10-12
- 対象: Platform Input AdapterからCoreへの正規化keyboard/text/pointer入力、Runtime active Window/Focus、Logical event routing
- 上位出典: .docs/12_design/BD-07_input-event-boundary-upper-design.md、BD-08_focus-upper-design.md、BD-09_ime-focus-transition-upper-design.md、BD-10_ime-platform-boundary-upper-design.md、Runtime R2～R4、ABI A25～A28
- 先行詳細設計: DD-03 dispatcher、DD-04 callback lifecycle、DD-09 Validation、DD-11 UI tree、DD-12 Layout
- 対応する詳細設計: .docs/13_detail_design/13-input-focus.md

## 目的

Platformごとのraw inputと共通Control動作の境界を保ち、初期版のkeyboard入力配送とFocus管理の詳細レビュー案を作る。Runtimeが管理するactive Window/focus、focus eligibilityと遷移、keyboard/text入力先、detach/deactivation、入力とIMEの接点を判断ID付きで整理する。特定のkey/modifierとFocus traversalやUI機能の対応付けは対象外とする。

## 作業手順

1. ADR-0001、Accepted要件/design overview、R2～R4、A25～A28、BD-07～10、DD-03/04/09/11/12の本文と個別状態を確認する。
2. raw inputの正規化とUI-thread処理、active Window/focusのsource of truth、Platform native focus同期の責務を分ける。
3. Focus取得条件、遷移commitと通知順、Window deactivate/reactivate、Control detach/disable時の処理を提案する。
4. keyboard key eventとtext/IME commitを区別し、active focusへの配送、Focus対象がない場合を提案する。特定のkey/modifierにUI動作を割り当てない。
5. event targetからLogical ancestorへどのようにrouteするか（direct/bubble/tunnel）、handled/default actionと公開subscriptionの境界を比較して提案する。
6. key down/up、text commit、Focus遷移、IME境界を整合させる。Focus traversalとshortcut/action bindingの検討は `.docs/20_todo/keyboard-actions-value-memo.md` に分離し、DD-13で詳細化しない。
7. FocusLost/FocusGained、DD-09のTextBox input validation、BD-09/10の未確定IME保持、DD-04のcallback再入規則との整合を確認する。
8. Visual hit-testの算出、IME session internals、Control固有key editing、公開event ABI等は後続DDへ割り当てる。
9. .docs/13_detail_design/13-input-focus.md にDD-INPUT判断、Mermaid状態遷移図、event routingモデル、検証ケースと未決事項を記録する。
10. .docs/20_todo/detail-design-phases.md と .docs/01_memory/detail-design-planning-status.md を更新し、参照先と git diff --check を確認する。

## 承認境界と停止条件

DD-13の着手指示は計画および未承認レビュー案の作成を許可した。利用者承認後、DD-INPUT-01～10を同文書へ承認結果として記録し、本計画を完了にする。DD-14への着手は別途指示を待つ。ABI symbol、Control固有編集動作、PlatformのIME実装、pointer hit-test詳細は本フェーズで確定しない。

## 対象外

- TextBox編集、caret移動、selection、Undo/Redo、IME composition内部状態の詳細（DD-14）
- Control別focusable/enabled/visibility Propertyの完全一覧と公開Control API（DD-15）
- pointer hit-test target算出、capture、gesture、render/bounds変換、Platform event-loop接続細則（DD-16）。DD-13は得られたtargetのCore event routingを扱う。
- Focus/Keyboard/Pointerの公開callbackまたはC ABI symbol一覧
- accessibility focus写像（DD-17）
- multi-seat入力とRuntime内に複数active focusを持つ拡張

## 完了条件

- Input normalize、Focus状態、keyboard/text配送、detach/deactivationの契約が上位判断に結び付く。
- 正常系、active focusなし、無効なFocus要求、detach、Window非active化、IME preedit、wrong-threadを検証できる。
- DD-14～17と未決事項を分離し、利用者承認後に判断結果・計画・フェーズ状況を同日に反映する。
