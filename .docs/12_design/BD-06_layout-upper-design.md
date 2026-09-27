# MVVM UIライブラリ Layoutの責務境界に関する上位判断案

- 状態: BD-06承認済み（2026-09-26）
- 対応: 要求仕様第12節「UI tree・Layout・Input・IME・Control」のうち、Layout計算のCore／Platform責務境界
- 根拠: Accepted設計骨格第3～6節、要求仕様第4・6節、承認済みBD-04・BD-05。DD-00は未承認

## BD-06 承認された判断

**Measure/ArrangeによるControlの配置計算はRust CoreがVisual Tree上で行い、Platform Adapterはrootの表示可能領域など環境入力を提供し、Coreが確定したboundsを描画・入力系へ渡す。Controlごとのレイアウト意味論を各Platform Adapterで別々に実装しない。** これにより、同じControl treeの配置結果を対象OS間で共通化する。

| 責務 | 提案する上位契約 |
|---|---|
| Core | Visual Treeを使ってmeasureとarrangeを行い、Control／Visual要素の最終配置boundsを確定する。Controlのlayout意味論を一か所に置く。 |
| Platform Adapter | Windowの利用可能なroot領域、scale等の環境情報をCoreへ渡す。Coreの最終boundsを対象OSの描画面へ写像する。 |
| 描画・Input | RendererとInputはCoreの配置結果を共通のboundsとして参照する。各backendで独自にControl配置を再計算しない。 |
| 未確定の単位系 | boundsの単位、scale／DPI変換、pixel rounding、座標原点と変換行列は別途決める。 |

### Sizer／Locationerのカスタマイズ

- Layoutには、必要サイズを決める`Sizer`と、親の領域内で最終位置・サイズを割り当てる`Locationer`の役割を設ける。
- 各要素または親コンテナは、`Align`等のレイアウト設定を指定できる。横・縦方向の配置やStretch等の選択肢、既定値、親子どちらの指定を優先するかは詳細設計で定義する。
- 推奨範囲は、初期版で組み込みSizer／Locationerとその設定値をカスタマイズ可能にすること。利用者が独自の測定・配置アルゴリズムを実装・登録する拡張APIは、4言語wrapper、決定性、UI thread上の処理時間、ABI callbackの負担を比較して別途判断する。

## 選択理由と範囲

- 設計骨格はLayoutをCoreの責務として置き、Measure/Arrangeを後続の詳細化対象としている。要求仕様もLayoutをCore必須機能に含める。
- Layoutをbackendごとに実装すると、同じ画面の配置・入力hit-test結果がOSやrendererによって異なる。Coreに置けば、renderer選定前にも共通のレイアウト意味論を設計できる。
- 本判断はLayout algorithm、Control固有のdesired size、margin/padding/alignment、overflow、virtualization、layout invalidationの頻度・優先度を決めない。
- Sizer／Locationerの役割と`Align`等の設定可能性は上位方針として含めるが、設定値の一覧や利用者独自アルゴリズムの拡張APIは決めない。
- 表示領域やscaleはPlatformから入るが、その単位変換規則は別判断とする。native controlを採るか独自描画にするか、対象OSの優先順位もここでは決めない。

## 承認記録

2026-09-26に利用者承認。Measure/ArrangeによるLayout計算をCoreのVisual Tree側に置き、Platformはroot表示領域等を提供し、確定boundsをOS側へ写像する。Sizerは必要サイズ、Locationerは配置を担当し、`Align`等を設定可能にする。独自アルゴリズムの公開拡張、単位系、rounding、個別Controlの測定規則は後続設計とする。DD-00の承認やDD-01以降へ進む判断ではない。
