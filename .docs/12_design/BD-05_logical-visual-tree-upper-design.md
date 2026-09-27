# MVVM UIライブラリ Logical Tree／Visual Treeの役割に関する上位判断案

- 状態: BD-05承認済み（2026-09-26）
- 対応: 要求仕様第12節「UI tree・Layout・Input・IME・Control」のうち、Logical TreeとVisual Treeの役割分担
- 根拠: Acceptedの設計骨格第3～6節、common-ui-library-research.md「画面基盤」、承認済みBD-04。DD-00は未承認

## BD-05 承認された判断

**初期版からLogical TreeとVisual Treeを別の概念として扱う。Logical Treeはアプリが組み立てるControlの意味上の包含関係、Visual Treeは実際に配置・描画・hit-testする表示構造とする。Control template等が導入された後も、アプリの論理的な構成をtemplate内部の表示部品で変えないためである。** 初期Controlでは両者が一対一に見える構成を許す。

| 観点 | 提案する上位契約 |
|---|---|
| Logical Tree | アプリが構築するControlの包含関係を表す。初期縦断シナリオのWindow、StackPanel、TextBox、Text、Buttonはこの関係で組み立てる。BD-04の親子所有・寿命規則はLogical Treeに適用する。 |
| Visual Tree | Layoutと描画の対象となる実表示構造を表す。Hit-testもVisual Tree上の位置・形状を使って候補を得る。LayoutのMeasure/Arrange規則と各入力種別のdispatchは別判断にする。 |
| template内部要素 | Control template等が作る内部表示要素はVisual Treeに含められるが、アプリのLogical Tree上の子として露出させない。これにより表示構造をControl内部で変更しても、アプリの宣言した親子関係を変えない。 |
| 対応関係 | Logical elementがVisual Tree上で一つ以上の表示要素に対応する場合、対応がない場合を許す。Visual要素から関連するLogical Controlをたどれる対応関係をCore内で保持する。対応表現や有効・無効状態による包含の細則は後続設計で定める。 |
| 初期版 | Template機能がない基本Controlでは、Logical TreeとVisual Treeが同じ並びになる単純な実装で開始できる。将来のtemplate対応で二つの概念を保ったまま内部Visual要素を追加する。 |

### BD-04との整合

BD-04で承認したCore内部強参照によるtree所有は、アプリが構築するLogical Treeの親子所属に適用する。Visual Treeは表示構造として別に扱い、Logical Treeの子を二重に所有する参照グラフにはしない。template由来のVisual要素の所有者と破棄順はtemplate／UI treeの詳細設計で定め、BD-04の「循環する内部参照を作らない」規則を満たす。

## この判断では決めないこと

- Logical TreeをたどるDataContext、resource、name scope、event routing、accessibilityの継承規則。
- Visual要素の型、Visual Treeを更新する時点、templateの交換・失敗時の原子性。
- Measure/Arrange、z-order、clip、hit-test精度、render invalidation。
- Pointer/keyboard/IME/focusの経路、detach時のfocus cleanup。
- Accessibility treeがLogical Tree、Visual Treeのどちらを基準にするか。

## 選択理由と限界

- 設計骨格はLogical TreeとVisual TreeをCore責務として明示し、調査資料もtemplateが内部要素を増やすため両treeの用途を明確にするよう推奨している。
- 一つのtreeだけにtemplate内部の見た目を押し込むと、Controlの内部構造変更がアプリ向けの子階層や親子寿命へ影響しやすい。別概念なら、アプリの構造とrendererが必要とする構造を切り分けられる。
- 二つのtreeを持つこと自体でDataContext継承やevent routingの規則までは定まらない。また、Visual要素をCore所有の別objectにする具体方式は詳細設計が必要。

## 承認記録

2026-09-26に利用者承認。Logical TreeとVisual Treeを初期版から別概念とし、Logical TreeをアプリのControl包含とBD-04の所有寿命、Visual Treeをlayout・描画・hit-test対象として定めた。template内部要素はLogical Treeへ露出させない。DataContext、event routing、Accessibility、Layout詳細は後続判断とする。DD-00の承認やDD-01以降へ進む判断ではない。
