# MVVM UIライブラリ SelectionModel意味論に関する上位判断案

- 状態: BD-34承認済み（2026-09-27）
- 対応: 選択件数モード、current itemとSelection・keyboard focusの関係、削除時の状態
- 根拠: BD-08、BD-11、BD-33、要求仕様第4～6節。DD-00は未承認

## 既存の承認済み契約

- BD-08はRuntime内でactive keyboard focusを最大一つのLogical Controlとして管理する。List/Grid内のcurrent itemはこのControl focusとは別概念である。
- BD-11は初期Controlの責務を決めたが、ListView、ComboBox、TableViewは後続段階であり、selection modeやkeyboard navigationは未決。
- BD-33はSelectionModelを独立Core objectとし、stable item IDを参照すること、Collectionからitemが削除されたらそのIDをSelectionから外すことを承認した。
- selectionの単一／複数、range selection、focus、keyboard navigation、accessibility semanticsはBD-33で未決として残した。

## BD-34 承認済み判断

1. **SelectionModelは`Single`と`Multiple`の選択件数modeを持つ。** Singleは選択集合を0件または1件、Multipleは0件以上とする。選択しないControlにはSelectionModelを要求しない。具体Controlの既定modeは個別Control設計で決める。
2. **`current item`は選択集合およびRuntime keyboard focusと独立して0件または1件を持てる。** current itemはcollection内の現在位置・navigation cursorを表し、必ず選択されるとは限らない。keyboard focusは引き続きControl上のBD-08状態であり、SelectionModelがOS/UI focusを所有しない。
3. **選択・current itemはstable item IDで参照する。** collection indexは表示・順序のための一時値とする。複数選択の外部公開順はcollectionの現在順に従い、並べ替え後も選択ID集合自体は変えない。
4. **Collectionから選択中またはcurrent itemのIDが削除された場合、そのIDを両stateから除く。** Collection change適用後に整合済みSelection stateを一度通知する。後継itemをcurrentに選ぶかはControl固有keyboard/navigation規則で定め、この上位判断では自動選択しない。
5. **Single modeの不変条件を破る複数IDのprogrammatic更新は全体を拒否し、既存selectionを維持する。** UI操作で別itemを選ぶと、単一selectionはそのitemへ置き換わる。MultipleからSingleへmode変更するAPIと既存複数選択の縮退規則は後続設計に送る。
6. **SelectionModelはcollection identityに結び付ける。** 別CollectionModelへ差し替えるとき古いitem IDを新collectionへ自動流用しない。selection/current itemのreset・restore方法は明示的な後続API判断とする。

## 理由とトレードオフ

- current itemを選択と分けると、矢印キーで現在位置を移動してからSpace/Enterで選択するControlや、選択を維持したままfocusを移す動作を表現できる。
- Runtimeのkeyboard focusとitem selectionを分ければ、Control間のfocus遷移で選択状態が意図せず失われない。
- stable IDを使うと、並べ替え後にindexが変わっても選択対象を維持できる。削除された対象を暗黙に別itemへ移さず、Controlごとのnavigation規則に判断を残せる。
- Collection順で選択項目を公開すると表示順が安定する一方、選択した順序を必要とする利用用途は別のAPIを要する。
- Single modeの無効なprogrammatic updateを原子的に拒否するため、wrapper間で暗黙の切り詰め方が異なることを防げる。

## この判断では決めないこと

- List/ComboBox/Table/Treeごとの既定mode、selection gesture、range selection、toggle/extend modifier。
- keyboard navigation、Tab順、focus移動、current itemとselectionを同期するControl option。
- Collection reset/replace時のID保全ポリシー、選択のserialize/restore、async sourceとの競合。
- event/property通知の具体順序、batch ABI、選択件数の上限。
- accessibility role/state mapping、選択項目の読み上げ・virtualized item公開。
- mode変更APIとMultiple→Single移行時の選択縮退規則。

## 承認記録

- 利用者承認（2026-09-27）。上記1～6を上位判断として採用する。
- `Single`/`Multiple` cardinalityをSelectionModel modeで表し、Control既定値は個別設計に送る。
- current itemをselected IDsおよびBD-08 keyboard focusから独立させる。
- IDsはstableで、削除されたitem IDをselection/currentから外し、自動で別itemを選ばない。
- Single modeに反するprogrammatic updateは既存状態を変えずに拒否する。
- Control固有のgesture/navigation/accessibilityおよびreset semanticsは後続判断とする。

Controlごとの既定mode、gesture/navigation、reset/restore API、accessibility細則は後続とする。DD-00は未承認のため、DD-01以降へ進まない。
