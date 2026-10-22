# 適用前記録 / 12-layout.md

- 記録日: 2026-10-17
- 区分: 2026-10-17改訂前の履歴。現行仕様として使用しない
- 元文書: [12-layout.md](../../13_detail_design/12-layout.md)

---

# DD-12 Layout

- 状態: 内容承認済み（2026-10-12）
- 作成日: 2026-10-11
- 対象: Core Measure/Arrange、Sizer/Locationer、StackPanel最小動作、Layout invalidation
- 上位出典: .docs/12_design/BD-04_ui-tree-lifetime-upper-design.md、BD-05_logical-visual-tree-upper-design.md、BD-06_layout-upper-design.md、BD-11_initial-control-contract-upper-design.md
- 先行詳細設計: DD-02、DD-03、DD-07、DD-08、DD-09、DD-11
- 作業計画: .docs/20_todo/dd12-layout.md

本書のDD-LAYOUT-01～08は2026-10-12に利用者承認済みである。第7節の未決事項は本承認の対象外として残す。Measure/ArrangeはRust CoreがVisual Tree上で実行し、Platformから与えられるroot表示領域等を入力としてboundsを確定する。公開ABIの完全なC宣言や配布用headerを定義しない。

## 1. 範囲と前提

- BD-04/05/11およびDD-11に従い、Logical Treeの所属・寿命とVisual Treeの配置構造を区別する。LayoutはVisual Tree上の表示要素を測定・配置する。
- Visual TreeはLogical Widgetの所有権を重複して持たない。各Visual要素のownerとLogical要素との対応はDD-11/Control/template設計に従う。
- BD-06に従い、Layout意味論はCoreに一元化し、Platformはroot利用可能領域とscale等の環境情報を提供し、Coreの確定boundsを描画・入力系へ渡す。
- DD-02/03に従い、Layout状態の変更はUI thread上で行い、workerからのUI状態操作を許さない。
- Boundsの単位、scale/DPI変換、座標原点・変換行列、pixel roundingはBD-06の未決事項であり、本書では数値表現を定めない。

## 2. 承認された判断

| ID | 判断 | 根拠・境界 | 状態 |
|---|---|---|---|
| DD-LAYOUT-01 | LayoutはVisual Treeの各要素に対し、SizerによるMeasureとLocationerによるArrangeの二段階で行う。Measureは親から利用可能サイズ制約を受けてDesiredSizeを返し、Arrangeは親が割り当てた最終矩形を子へ配分し、各要素のFinalBoundsを確定する。DesiredSizeとFinalBoundsを別値として保持する。 | BD-06。計算主体をCoreに置き、Platformで再計算しない。 | 承認済み（2026-10-12） |
| DD-LAYOUT-02 | Layout関連操作と測定結果のcommitはUI thread専用とする。各要素のサイズ制約は横・縦それぞれ最小値と最大値を持つ。寸法は非負でNaNを許さず、最大値には「上限なし」を表す意味値を許す。上下限が逆転する制約は無効入力として拒否し、既存制約・DesiredSize・FinalBoundsを変更しない。ABI上の符号化は未決のままにする。 | R2、A2、BD-06。無限値のABI表現や具体statusはABI/単位設計へ送る。 | 承認済み（2026-10-12） |
| DD-LAYOUT-03 | Measureは子から親へDesiredSizeを集約する。Arrangeはrootから子へfinal rectangleを割り当てる。最終boundsは親座標系の矩形とし、絶対座標変換は別レイヤで合成する。Measureは利用可能制約の変化時に再実行し、Arrangeは最終割当矩形の変化時に再実行する。 | BD-06とMeasure/Arrangeの役割。座標系の数値単位・変換行列は対象外。 | 承認済み（2026-10-12） |
| DD-LAYOUT-04 | StackPanelはLogical childrenの順序を保ち、初期版のorientationはVertical/Horizontalを提供し既定値をVerticalとする。Verticalではmain axis=height、cross axis=width、Horizontalではmain axis=width、cross axis=heightとする。Measureではmain axisの利用可能上限を子へ「上限なし」として渡し、cross axisの利用可能制約を渡す。DesiredSizeはmain axisで子の合計、cross axisで子の最大値を基礎に制約内へ収める。 | BD-11の順序付きchildren、BD-06の組み込みSizer/Locationer。spacing/margin/paddingは初期動作に追加せずDD-15/後続へ送る。 | 承認済み（2026-10-12） |
| DD-LAYOUT-05 | StackPanelのArrangeではmain axis上にchildrenをDesiredSize順で連続配置し、cross axisは各childのAlignに従う。初期Align候補はStart、Center、End、Stretchとする。明示child AlignはContainer既定Alignに優先し、未指定時はcross-axis Stretch、main-axis Start相当とする。Stretchは割り当てたcross-axis領域へ広げるが、childのmaximum制約を超えない。 | BD-06はAlign等の設定可能性と優先順位を詳細化へ送っていた。本判断で候補値と初期既定を定める。 | 承認済み（2026-10-12） |
| DD-LAYOUT-06 | 初期Sizer/LocationerはCore組み込み実装とし、利用者独自アルゴリズムをABI callbackとして登録する拡張点はDD-12で追加しない。初期StackPanel以外のControl固有DesiredSizeは各Control設計が定める。 | BD-06の推奨を採用する。拡張APIは別の基本設計判断を要する。 | 承認済み（2026-10-12） |
| DD-LAYOUT-07 | Layout invalidationはVisual Tree/親子順の変更、root利用可能領域変更、測定結果に影響するProperty変更でdirty化する。Measure-dirtyは祖先へ伝播し、Arrange-dirtyは必要な配置部分へ伝播する。重複dirty通知は一処理区間でcoalesceし、Layout結果を次に消費する前に必要範囲を再計算する。 | DD-03/07/11、BD-06。layout schedulingとrender frameの正確な統合はDD-16へ送る。 | 承認済み（2026-10-12） |
| DD-LAYOUT-08 | Measure/Arrangeは決定的かつ冪等とし、入力制約、Visual構造、配置Propertyが同一なら同じDesiredSize/FinalBoundsを返す。制約の不正、破棄済みVisual、Runtime shutdownでは既存の公開結果を部分更新せず失敗する。独自callbackを呼ばないためlayout pass中の利用者コード再入は発生させない。 | DD-02/03、BD-06。浮動小数点の許容誤差・rounding・エラー診断codeは未決のまま残す。 | 承認済み（2026-10-12） |

## 3. Measure / Arrange の処理

### 3.1 Measure

Measureは、親が子にどれだけの領域を使えるかを寸法制約として渡し、各子のDesiredSizeを得る処理である。

- 親は自身のSizerに沿ってchildrenを測定し、集約結果を自身のDesiredSizeとして返す。
- StackPanelは順序を保って各childをMeasureする。Verticalでは高さを、Horizontalでは幅を「上限なし」として測定させる。
- 子のDesiredSizeは寸法制約を満たし、上限なしの軸でも値は有限かつ非負とする。
- 初期版ではMeasure中にProperty通知やBindingを発生させず、Measure結果をViewModelへ自動反映しない。

### 3.2 Arrange

Arrangeは親が受け取ったFinalRectの内側へchildrenの位置とサイズを割り当てる処理である。

- 親のLocationerが各childの割当矩形を決める。childのLocationerは自身の矩形と子への配分を決める。
- StackPanelはmain axisでDesiredSizeを順に使い、cross axisはAlign・最小/最大制約からサイズとoffsetを決める。
- Main axisでchildrenのDesiredSize合計が割当領域より小さくても、未使用領域を自動配分しない。余白・spacing・均等割当は初期仕様に含めない。
- FinalBoundsはLayout完了後に一貫して参照できる。Platform/RendererがControl固有配置を再計算しない。

## 4. 初期StackPanelの計算例（単位は抽象）

Vertical StackPanelに、順序通り A=(desired 80,20)、B=(desired 60,30) の二子があり、Panelへ(available width 100, height 100)が与えられる例を示す。dimensionsは抽象単位でありpixelを意味しない。

- Measure: AとBにcross width 100、main height上限なしを渡す。
- StackPanel DesiredSize: width=max(80,60)=80、height=20+30=50。
- Arrange: PanelのFinalRectが(100,100)なら、既定cross Align=StretchでAは幅100・高さ20、Bは幅100・高さ30となる。
- Vertical main axisの残り50は未使用であり、子へ比例配分しない。
- child Align=Centerなら、そのchildのDesiredSize幅を保ち、Panel内で中央に置く。Stretchでもchild maximum widthが70なら幅70とする。

Horizontalでは横幅合計・最大高さで対称に扱う。実寸の単位、fractional値のpixel mapping、roundingはBD-06の未決事項に留める。

## 5. Layout invalidation

| 変更 | Measure dirty | Arrange dirty |
|---|---|---|
| Visual child追加/削除/順序変更 | 変更されたContainerと祖先 | 変更されたContainerと関係する子孫 |
| root利用可能領域変更 | 制約依存のroot subtree | root subtree |
| child DesiredSizeに影響するProperty | 当該要素と祖先 | 当該要素を含む影響範囲 |
| Align/最終割当だけに影響するProperty | 原則なし | 当該要素と配置を再計算する祖先 |
| Platform peerの描画状態だけの変更 | なし | Layout外。DD-16で扱う |

各変更は処理区間で重複をまとめる。Measure-dirtyの要素は祖先にdirtyがあればその祖先から再Measureし、Arrangeは最新DesiredSizeとroot領域から再実行する。dirty伝播の内部データ構造、priority、時間sliceはここで固定しない。Layout結果を消費する前に未処理dirtyを反映する保証と、その消費点はDD-16と整合させる。

## 6. 状態フロー

入力変更/root領域変更/Visual構造変更
→ Measure dirty を記録
→ Visual Tree上でMeasure（childrenからparent）
→ DesiredSizeをcommit
→ Arrange dirty を記録
→ Visual Tree上でArrange（rootからchildren）
→ FinalBoundsをcommit
→ Renderer/Input側が共通boundsを参照

Measure/Arrange中に失敗した場合は今回の結果を部分commitせず、以前のcomplete layout snapshotを保持する。dirty状態は失敗診断または後続retryが可能なように残す。再試行方針と公開診断は後続設計で扱う。

## 7. 未決事項と後続設計

- Layout寸法の単位、scale/DPI変換、原点、座標変換行列、pixel rounding: BD-06が後続判断へ送る。DD-LAYOUT-01～08の承認では数値表現を決めていない。
- NaN、infinity、overflow、許容誤差、比較閾値のABI表現・error status: 単位/値型/error catalog設計。DD-LAYOUT-02の制約意味論承認は表現を確定しない。
- Text/TextBox/Button等のControl別Sizer、font metrics、content padding、minimum size: DD-15とText/Renderer設計。
- Margin、padding、spacing、均等Stretch、最大制約を越えるDesiredSize、overflow/clipping: DD-15/後続Layout判断。
- Window/rootへavailable sizeを渡す具体Platform hookとscale連携、layout invalidationから描画frameまでの接続: DD-16。
- Hit-testがbounds、clip、transformを使う細則: DD-13/16。
- 頻繁なProperty更新時のlayout coalescing budget/priority、large tree incremental algorithm: 性能検証後の後続設計。
- 独自Sizer/Locationer登録拡張API: 上位設計で再判断する場合のみ対象にする。

## 8. 上位判断へ戻す境界

承認済みDD-LAYOUT-01～08はBD-06のMeasure/ArrangeをCore Visual Treeへ置く責務と、Platformがroot環境値を供給する分離を維持する。bounds単位、scale/rounding、Renderer API、Control固有測定が既承認契約と両立しない場合はDD-12だけで上書きせず、BDまたは対応するABI/Platform/Control設計へ戻す。

## 9. 承認記録

利用者は2026-10-12にDD-LAYOUT-01～08を内容承認した。承認範囲はCoreでのMeasure/Arrange、寸法制約の意味論、StackPanelのVertical/Horizontal測定・配置、Align候補と既定値、組み込みSizer/Locationer、Layout invalidation、決定性と失敗時の結果保持である。第7節の単位/scale/rounding、ABI表現、Control別Sizer、Renderer接続、overflow/clipping等は本承認の対象外として後続設計に残す。
