# DD-17 Accessibility 作業計画

- 状態: 完了（DD-ACC-01～10承認済み、2026-10-12）
- 着手日: 2026-10-12
- 対象: Core Accessibility snapshot/tree、初期ControlのOS provider mapping、変更通知、supported actionのCore経路接続
- 上位出典: Accepted要件 §7、BD-05/08/11/12/13、承認済みDD-13～16、DD-15で承認済みの初期Control境界
- 対応する詳細設計: `.docs/13_detail_design/17-accessibility.md`

## 目的

BD-13が定めるPlatform中立なAccessibility意味情報とLogical Treeを、OS Accessibility provider/APIへ写像する契約を定める。初期Controlのrole/name/state/value/focus/action、OSからのquery/action、Core state変更の通知、provider nodeの寿命を扱う。独自描画のVisual内部要素を別Widgetとして露出せず、既存のFocus/Input/Command/TextBox契約へ接続する。

## 小分けにした作業範囲

1. **Treeとnode identity**: Logical WidgetからOS accessibility node/providerへの対応、順序、追加/削除/reparent、peer寿命。
2. **共通snapshot**: Window/StackPanel/Text/TextBox/Buttonのrole/name/state/value/focus/supported actionをCore snapshotとして表す境界。
3. **OS mapping**: WindowsとLinux x64の初期mapping候補、OS API差をAdapter内に閉じる方式、unsupported capabilityの表現。
4. **Actionsとnotifications**: AT invoke/focus/edit requestを既存Core Input/Command/TextBoxへ戻す契約、Core commit後のOS notification。
5. **Threadingとfailure**: OS provider callbackのthread、snapshot read、UI thread dispatch、stale node/provider、shutdown/detach時の失敗動作。
6. **検証条件と未決事項**: Windows → Linux x64のAccessibleName/Tree/action検証、AT製品・適合宣言等の未決境界。

## 作業手順

1. ADR、Accepted要件、BD-05/08/11/12/13とDD-13～16の状態・承認IDを確認する。
2. BD-13の承認済み意味情報/Logical Tree方針、DD-15の承認Control、§7に残る未決意味、DD-13 Focus/InputとDD-16 peer/thread境界を整理する。
3. Core snapshot/treeとOS provider mappingの分担を判断ID付きで提案する。OS API候補を具体名で記す場合は公式一次資料の適用範囲を確認する。
4. Window/StackPanel/Text/TextBox/Buttonのrole/name/state/value/action接続表を作り、空name・focus/enabled/validation・TextBox edit valueなど未決依存を明示する。
5. accessibility actionは既存のCore意味経路へ接続する。Tab traversal、Ctrl+Z、他のkey/modifier shortcutを追加しない。
6. provider query/action、Core state change通知、node create/remove/reparent、Runtime shutdownのsequence/state diagramと正常/失敗検証を作成する。
7. `.docs/13_detail_design/17-accessibility.md` を未承認レビュー案として保存し、計画・フェーズ・状況メモ、参照リンク、`git diff --check`を確認する。

## 承認境界と停止条件

DD-17着手時は計画と未承認レビュー案を作成した。OS API候補やmapping案は利用者承認まで確定仕様にしない。BD-13/DD-15の未決事項を本フェーズで暗黙に決着させず、上位判断が必要な事項は選択肢と推奨を示して確認する。DD-ACC-01～10の承認後にDD-17を完了扱いとし、DD-18は別途着手指示を受けて開始する（2026-10-12に指示済み）。ABI provider interfaceの完全な関数宣言や言語wrapper APIは対象外。

## 対象外

- Keyboard shortcut/Tab traversal/Undo-Redo/action binding（DD-13と候補メモの境界）
- Accessibility OS provider ABI、C ABI symbol一覧、4言語wrapper固有型（DD-06/18～21）
- DD-15で未決のAccessibleName fallback/label association/role-state全列挙/selection-caret細部を上位承認なしに追加確定すること
- Hidden/offscreen/visibility mapping、virtualized collection、custom composite semantics（初期Control外またはDD-15未決）
- 特定screen reader/switch access製品の完全適合、正式な認証・適合宣言・サポート保証（上位要求/検証判断）
- Renderer pixelsからの意味推測、native Controlを意味source-of-truthにする構成（BD-11/12/13と不整合）

## 完了条件

- Logical TreeからOS Accessibility tree/provider nodeへの対応、順序、identity、寿命と変更通知が記述される。
- 5初期Controlのrole/name/state/value/focus/action mappingと、未決のため確定しない値が追跡可能である。
- Provider queryはCore UI stateを直接変更せず、AT actionsは既存Core Input/Focus/Command/TextBox semanticsへ接続する。
- OS callback thread、UI thread dispatch、snapshot coherence、stale node、failure/shutdown動作が記述される。
- BD-13のWindows→Linux検証順と未決のAT製品/適合範囲が分離される。
- 利用者承認後に判断結果を本書・作業計画・フェーズ状況へ反映し、未決境界を維持する。

## 完了・承認記録

利用者は2026-10-12にDD-ACC-01～10を記載どおり承認した。判断を `.docs/13_detail_design/17-accessibility.md` と本計画へ反映し、DD-17を完了とした。§8に記載したDD-15由来のControl意味論、OS固有profileとbus依存、AT実機・適合範囲は未決のまま維持する。DD-18の着手は利用者から別途指示を受けた。
