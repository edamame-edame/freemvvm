# 要求テストの観測・oracle

- 文書版: 1.1
- 状態: 観測設計承認済み（2026-10-17）。未実行
- 共通契約: [Requirements Test Manifest](requirements-test-manifest.md)

期待結果はリンク先RQを正本とする。代表使用値は要求で決まらず実施に必要な場合に限って個別test計画に用途・選定理由・適用範囲とともに記載する。API/profile未設計のcaseはblockedとして詳細化する。

<a id="assert-rq-01-001"></a>
## RQ-01-001

- 根拠: [RQ-01-001](../11_requirements/RQ-01_purpose-architecture.md#rq-01-001)
- case: [TC-ARCH-01](requirement-test-matrix.md#case-tc-arch-01)、[TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)、[TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)、[TC-VERT-04](requirement-test-matrix.md#case-tc-vert-04)、[TC-VERT-05](requirement-test-matrix.md#case-tc-vert-05)、[TC-VERT-06](requirement-test-matrix.md#case-tc-vert-06)、[TC-VERT-07](requirement-test-matrix.md#case-tc-vert-07)、[TC-VERT-08](requirement-test-matrix.md#case-tc-vert-08)、[TC-WRAP-CPP-01](requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-WRAP-RS-01](requirement-test-matrix.md#case-tc-wrap-rs-01)
- 前提・操作・観測: 4言語それぞれで同一の初期縦断シナリオを実行し、TextBoxからの値更新、Textへの値反映、ButtonによるCommand実行と実行可否、UI応答が同じ期待結果になることを確認する。
- 期待結果: C++、C#、Rust、Python 3の各利用者が、同じMVVMの基本機能とUIシナリオを使えること。利用可能なプラットフォーム上で共通のUIライブラリとして動作し、言語ごとのAPI表現が異なっても、Property、Binding、CommandおよびUIの利用結果に意味上の差を生じさせないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-01-002"></a>
## RQ-01-002

- 根拠: [RQ-01-002](../11_requirements/RQ-01_purpose-architecture.md#rq-01-002)
- case: [TC-ABI-COMP-01](requirement-test-matrix.md#case-tc-abi-comp-01)、[TC-ABI-COMP-02](requirement-test-matrix.md#case-tc-abi-comp-02)、[TC-ABI-REP-01](requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-ARCH-01](requirement-test-matrix.md#case-tc-arch-01)、[TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 4言語のAPIが同じABIランタイムへ接続し、同じCore状態を読み書きしていることを構成と縦断シナリオで確認する。言語間で同じ操作を行ったとき、値・通知・Command状態の結果が一致することを確認する。
- 期待結果: 共通UIランタイムはRustで実装する。言語ごとの利用層はversioned C ABIを介して共通ランタイムを利用し、各言語のAPI型・例外・オブジェクト表現をABI境界へ持ち出さないこと。利用者から見た機能状態の正本は共通ランタイムにあり、言語層ごとに独立した状態や通知規則を持たないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-001"></a>
## RQ-02-001

- 根拠: [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)
- case: [TC-CORE-OBJ-01](requirement-test-matrix.md#case-tc-core-obj-01)
- 前提・操作・観測: 作成・参照・所有者解放・明示破棄の各経路で、対象が有効な間は同じ対象を参照でき、寿命終了後は無効な参照として検出されることを確認する。
- 期待結果: UI要素やMVVM機能を、利用者が一貫して参照・操作できるCore上の対象として扱えること。対象の所有・共有・破棄に関する契約が定義され、利用中の対象が意図せず消失せず、利用終了後に保持され続けないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-002"></a>
## RQ-02-002

- 根拠: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)
- case: [TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)
- 前提・操作・観測: 値の設定・読出し・変更通知を確認し、型不一致の更新が既存値を壊さず拒否されること、および等値更新で変更通知が発生しないことを確認する。
- 期待結果: ViewModelとUIが値を共有し、値の変更を相手側から観測できること。宣言された値種別と整合しない更新は受け入れず、値が実際に変わらない更新で不要な変更通知を生じさせないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-003"></a>
## RQ-02-003

- 根拠: [RQ-02-003](../11_requirements/RQ-02_core-features.md#rq-02-003)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)
- 前提・操作・観測: 各方向で値が期待先へ伝播すること、無効な接続が既存値を部分変更しないこと、検証失敗が識別可能であること、解除後に変更が伝播しないことを確認する。
- 期待結果: OneTime、OneWay、TwoWayの更新方向を利用できること。初期値の同期、変更の伝播、検証失敗、循環または競合する接続、解除後の伝播停止について一貫した結果を提供すること。複数Propertyをまとめる一般transactionや自動rollbackは要求しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-004"></a>
## RQ-02-004

- 根拠: [RQ-02-004](../11_requirements/RQ-02_core-features.md#rq-02-004)
- case: [TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)
- 前提・操作・観測: 実行可否に応じて実行の可否が変わり、成功・失敗・キャンセルを区別して観測できること、完了結果がUI側へ反映されることを確認する。
- 期待結果: UI操作から処理を実行でき、実行可否と実行中・完了・失敗・キャンセルの状態を利用者へ示せること。非同期処理の完了や結果がUIへ反映され、同一処理の再入規則が予測可能であること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-005"></a>
## RQ-02-005

- 根拠: [RQ-02-005](../11_requirements/RQ-02_core-features.md#rq-02-005)
- case: [TC-COL-01](requirement-test-matrix.md#case-tc-col-01)
- 前提・操作・観測: 有効な変更後に項目と順序が期待どおりになり、並べ替え後も同一項目を識別できることを確認する。無効な一括変更時は集合状態が部分適用されないことを確認する。
- 期待結果: 次段階で、項目の追加・削除・並べ替え等の集合状態を保持し、変更を利用側が把握できること。項目の識別は順序位置の変化だけで別項目に切り替わらないこと。対象範囲は[次段階のCollectionModel要求](../11_requirements/RQ-09_ui-scope.md#rq-09-019)に従う。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-006"></a>
## RQ-02-006

- 根拠: [RQ-02-006](../11_requirements/RQ-02_core-features.md#rq-02-006)
- case: [TC-SEL-01](requirement-test-matrix.md#case-tc-sel-01)
- 前提・操作・観測: 単一・複数選択の各状態、選択されていない現在項目、項目削除後の選択・現在状態を確認し、フォーカス移動だけで選択状態が暗黙に変化しないことを確認する。
- 期待結果: 次段階で、項目集合に対する選択状態と現在項目を扱えること。選択状態・現在項目・キーボードフォーカスの意味を混同せず、対象項目の削除後に存在しない項目が選択中または現在項目として残らないこと。対象範囲は[次段階のSelectionModel要求](../11_requirements/RQ-09_ui-scope.md#rq-09-020)に従う。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-007"></a>
## RQ-02-007

- 根拠: [RQ-02-007](../11_requirements/RQ-02_core-features.md#rq-02-007)
- case: [TC-TREE-01](requirement-test-matrix.md#case-tc-tree-01)
- 前提・操作・観測: Widgetを追加・削除した後、表示順・入力対象・意味上の階層が定義された結果となることを確認する。内部描画要素がアプリ構成上の独立Widgetとして誤って観測されないことを確認する。
- 期待結果: Widgetの視覚的な親子構造を構成し、構造に沿った描画・入力対象の決定を可能にすること。アプリが組み立てる意味上の階層と描画都合の内部要素を区別し、内部表現だけで意味上の親子関係を不意に変えないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-008"></a>
## RQ-02-008

- 根拠: [RQ-02-008](../11_requirements/RQ-02_core-features.md#rq-02-008)
- case: [TC-LAYOUT-01](requirement-test-matrix.md#case-tc-layout-01)
- 前提・操作・観測: 初期Controlを含む画面で、サイズ制約や親領域の変更後に配置結果が更新され、重複・欠落なく表示されることを確認する。
- 期待結果: 利用可能な領域とWidgetの配置規則に基づいて、各Widgetの必要サイズと表示位置を決定できること。親子関係に応じてレイアウトを再計算し、結果をPlatform表示領域へ反映できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-009"></a>
## RQ-02-009

- 根拠: [RQ-02-009](../11_requirements/RQ-02_core-features.md#rq-02-009)
- case: [TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)
- 前提・操作・観測: 入力イベントが適切なControlへ届き、TextBox値の編集・確定・拒否の結果が定義どおり観測できることを確認する。
- 期待結果: Platformから届く入力を共通UI機能で扱い、対象Controlの操作へ結び付けられること。少なくとも初期共通基盤ではキーボード入力とTextBox編集入力を利用でき、入力検証の結果を利用者へ保持・伝達できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-010"></a>
## RQ-02-010

- 根拠: [RQ-02-010](../11_requirements/RQ-02_core-features.md#rq-02-010)
- case: [TC-FOCUS-01](requirement-test-matrix.md#case-tc-focus-01)
- 前提・操作・観測: Focus取得・喪失時の状態がCoreとPlatformで一致し、キーボード・文字入力が現在の対象へ届けられることを確認する。
- 期待結果: 現在キーボード・文字入力を受けるControlを識別し、フォーカス変更をControlとPlatform間で同期できること。Focus状態はSelection状態と区別されること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-011"></a>
## RQ-02-011

- 根拠: [RQ-02-011](../11_requirements/RQ-02_core-features.md#rq-02-011)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)
- 前提・操作・観測: 変換中は未確定文字列と編集位置が維持され、確定後は検証結果に応じた値が表示・伝播されることを確認する。Focus移動で未確定文字列が黙って失われないことを確認する。
- 期待結果: IMEの未確定入力と確定入力を区別し、変換中の編集状態を保持した上で、確定候補を入力検証へ渡せること。IMEを備えるPlatformで日本語等の変換入力を利用できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-012"></a>
## RQ-02-012

- 根拠: [RQ-02-012](../11_requirements/RQ-02_core-features.md#rq-02-012)
- case: [TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)
- 前提・操作・観測: 正常・失敗・不正引数・終了済み対象等の結果を呼出し側が区別でき、ABI境界を越えて言語固有例外やpanicが伝播しないことを確認する。
- 期待結果: Core/API操作と非同期処理の失敗を呼出し側が識別し、原因に応じて処理できる形で伝えること。言語固有の例外やpanicを共通公開境界の外へ漏らさないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-013"></a>
## RQ-02-013

- 根拠: [RQ-02-013](../11_requirements/RQ-02_core-features.md#rq-02-013)
- case: [TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)、[TC-VERT-08](requirement-test-matrix.md#case-tc-vert-08)
- 前提・操作・観測: 通常破棄、親子からの切離し、callback解除、Runtime終了の各経路で、参照可能性と資源解放が規定どおりであることを確認する。
- 期待結果: Core対象・callback・Platform資源の利用期間と解放契機を明確にし、所有者が終了した対象や不要になった内部資源を保持し続けないこと。終了処理がUI資源の解放を行えること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-014"></a>
## RQ-02-014

- 根拠: [RQ-02-014](../11_requirements/RQ-02_core-features.md#rq-02-014)
- case: [TC-THREAD-01](requirement-test-matrix.md#case-tc-thread-01)
- 前提・操作・観測: UI threadからの操作が成功し、workerからの操作が所定の引渡し経路を通ること、直接操作の拒否または結果が既存状態を破壊しないことを確認する。
- 期待結果: UI状態の読み書きとPlatform操作を安全なUI実行スレッドで行えること。workerから要求されたUI更新は安全に引き渡され、thread間でデータ競合を起こさないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-015"></a>
## RQ-02-015

- 根拠: [RQ-02-015](../11_requirements/RQ-02_core-features.md#rq-02-015)
- case: [TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)、[TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)
- 前提・操作・観測: Runtime終了後にUI操作・error読取り・残存参照解放を別々に行い、それぞれの可否を観測する。
- 期待結果: Runtime終了後はUI機能を利用できず、終了済みであることを識別できること。残存する所有参照は解放でき、不変なエラー情報はその参照の利用期間内に読み取れること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-016"></a>
## RQ-02-016

- 根拠: [RQ-02-016](../11_requirements/RQ-02_core-features.md#rq-02-016)
- case: [TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)
- 前提・操作・観測: 終了完了後に再終了し、destroy/資源解放回数が増えず成功結果を保つことを確認する。
- 期待結果: Runtime終了が完了した後に再度終了を要求しても成功し、利用者データの後始末や資源解放を重複して実行しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-017"></a>
## RQ-02-017

- 根拠: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)
- case: [TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)
- 前提・操作・観測: 対応RQの観測結果を個別assertionとして現行caseに照合する。初期Propertyで真偽値、符号付き64-bit整数、有効なUTF-8文字列、倍精度浮動小数値、規定範囲の十進値を利用できること。初期範囲にnullable値・任意言語object・カスタム構造体値を含めず、それらをゼロや空文字列へ読み替えないこと。
- 期待結果: 初期Propertyで真偽値、符号付き64-bit整数、有効なUTF-8文字列、倍精度浮動小数値、規定範囲の十進値を利用できること。初期範囲にnullable値・任意言語object・カスタム構造体値を含めず、それらをゼロや空文字列へ読み替えないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-018"></a>
## RQ-02-018

- 根拠: [RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018)
- case: [TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)、[TC-ABI-REP-01](requirement-test-matrix.md#case-tc-abi-rep-01)
- 前提・操作・観測: 正負ゼロとNaNの異なるbit表現を同値再設定し、stored representationと通知回数を観測する。
- 期待結果: 倍精度値はNaNと無限大を受け入れること。同符号の無限大、正負のゼロ、NaN同士はそれぞれ同値とし、同値設定では既存の格納表現を変えず変更通知を出さないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-019"></a>
## RQ-02-019

- 根拠: [RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019)
- case: [TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)、[TC-ABI-REP-01](requirement-test-matrix.md#case-tc-abi-rep-01)
- 前提・操作・観測: 1.0/1.00、正負ゼロ、最大係数、scale31、範囲外候補で同値/保持/拒否を観測する。
- 期待結果: 十進値は符号・0～31のscale・0～2^58−1の係数で表せる範囲を正確に扱い、範囲外を黙って丸めないこと。1.0と1.00、正負のゼロは数値として同値とし、同値設定では既存の格納表現と通知状態を維持すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-020"></a>
## RQ-02-020

- 根拠: [RQ-02-020](../11_requirements/RQ-02_core-features.md#rq-02-020)
- case: [TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)
- 前提・操作・観測: 同値candidateを拒否するvalidatorを登録し、検証が起動することと値/通知の非変更を確認する。
- 期待結果: 現在値と同値を設定する場合も登録済みの値検証を適用すること。受理された同値設定は現在値の格納表現を置き換えず、値変更通知を生じさせないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-02-021"></a>
## RQ-02-021

- 根拠: [RQ-02-021](../11_requirements/RQ-02_core-features.md#rq-02-021)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)
- 前提・操作・観測: Validator重複登録を拒否し、現在値の再評価が起こらず旧Validatorが有効なことを確認する。
- 期待結果: 一つのPropertyに登録できる同期Validatorは最大一つとし、登録だけでは現在値を再検証しないこと。重複登録失敗で既存登録を失わないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-001"></a>
## RQ-03-001

- 根拠: [RQ-03-001](../11_requirements/RQ-03_language-support.md#rq-03-001)
- case: [TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)、[TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)、[TC-VERT-04](requirement-test-matrix.md#case-tc-vert-04)、[TC-VERT-05](requirement-test-matrix.md#case-tc-vert-05)、[TC-VERT-06](requirement-test-matrix.md#case-tc-vert-06)、[TC-VERT-07](requirement-test-matrix.md#case-tc-vert-07)、[TC-VERT-08](requirement-test-matrix.md#case-tc-vert-08)、[TC-WRAP-CPP-01](requirement-test-matrix.md#case-tc-wrap-cpp-01)
- 前提・操作・観測: C++の利用コードで縦断シナリオを実行し、Property・Binding・Command・UIの結果が共通の期待値になることを確認する。
- 期待結果: C++アプリケーションから、共通Coreが提供する初期共通基盤のUI機能とMVVM機能を利用できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-002"></a>
## RQ-03-002

- 根拠: [RQ-03-002](../11_requirements/RQ-03_language-support.md#rq-03-002)
- case: [TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)、[TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)、[TC-VERT-04](requirement-test-matrix.md#case-tc-vert-04)、[TC-VERT-05](requirement-test-matrix.md#case-tc-vert-05)、[TC-VERT-06](requirement-test-matrix.md#case-tc-vert-06)、[TC-VERT-07](requirement-test-matrix.md#case-tc-vert-07)、[TC-VERT-08](requirement-test-matrix.md#case-tc-vert-08)、[TC-WRAP-CS-01](requirement-test-matrix.md#case-tc-wrap-cs-01)
- 前提・操作・観測: C#の利用コードで縦断シナリオを実行し、Property・Binding・Command・UIの結果が共通の期待値になることを確認する。
- 期待結果: C#アプリケーションから、共通Coreが提供する初期共通基盤のUI機能とMVVM機能を利用できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-003"></a>
## RQ-03-003

- 根拠: [RQ-03-003](../11_requirements/RQ-03_language-support.md#rq-03-003)
- case: [TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)、[TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)、[TC-VERT-04](requirement-test-matrix.md#case-tc-vert-04)、[TC-VERT-05](requirement-test-matrix.md#case-tc-vert-05)、[TC-VERT-06](requirement-test-matrix.md#case-tc-vert-06)、[TC-VERT-07](requirement-test-matrix.md#case-tc-vert-07)、[TC-VERT-08](requirement-test-matrix.md#case-tc-vert-08)、[TC-WRAP-RS-01](requirement-test-matrix.md#case-tc-wrap-rs-01)
- 前提・操作・観測: Rustの利用コードで縦断シナリオを実行し、Property・Binding・Command・UIの結果が共通の期待値になることを確認する。
- 期待結果: Rustアプリケーションから、共通Coreが提供する初期共通基盤のUI機能とMVVM機能を利用できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-004"></a>
## RQ-03-004

- 根拠: [RQ-03-004](../11_requirements/RQ-03_language-support.md#rq-03-004)
- case: [TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)、[TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)、[TC-VERT-04](requirement-test-matrix.md#case-tc-vert-04)、[TC-VERT-05](requirement-test-matrix.md#case-tc-vert-05)、[TC-VERT-06](requirement-test-matrix.md#case-tc-vert-06)、[TC-VERT-07](requirement-test-matrix.md#case-tc-vert-07)、[TC-VERT-08](requirement-test-matrix.md#case-tc-vert-08)、[TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)
- 前提・操作・観測: Python 3の利用コードで縦断シナリオを実行し、Property・Binding・Command・UIの結果が共通の期待値になることを確認する。
- 期待結果: Python 3アプリケーションから、共通Coreが提供する初期共通基盤のUI機能とMVVM機能を利用できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-005"></a>
## RQ-03-005

- 根拠: [RQ-03-005](../11_requirements/RQ-03_language-support.md#rq-03-005)
- case: [TC-WRAP-CPP-01](requirement-test-matrix.md#case-tc-wrap-cpp-01)
- 前提・操作・観測: 例外有効/無効buildで同じfailureを扱い、handler設定・handler例外・callback固有結果を分けて確認する。
- 期待結果: 例外無効buildでも共通機能を利用でき、失敗を結果として扱えること。例外有効buildではcallback例外の利用者handlerを設定でき、そのhandlerによってABI境界やcallback固有の失敗結果を壊さないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-006"></a>
## RQ-03-006

- 根拠: [RQ-03-006](../11_requirements/RQ-03_language-support.md#rq-03-006)
- case: [TC-WRAP-CS-01](requirement-test-matrix.md#case-tc-wrap-cs-01)
- 前提・操作・観測: native call中Disposeとcallback再入Disposeを競合させ、lease中利用と最終解放一回性を確認する。
- 期待結果: 明示破棄を繰り返しても二重解放せず、native呼出しと破棄が競合しても呼出し中の対象を失わないこと。callbackの例外処理を利用者が設定でき、元の失敗を識別できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-007"></a>
## RQ-03-007

- 根拠: [RQ-03-007](../11_requirements/RQ-03_language-support.md#rq-03-007)
- case: [TC-WRAP-RS-01](requirement-test-matrix.md#case-tc-wrap-rs-01)
- 前提・操作・観測: borrowの期限/Send制約をcompile-fail型検査で、捕捉panicを実行検査で分ける。abortは別processで保証外境界を確認する。
- 期待結果: 通常のsafe API利用で借用参照を解放したり、期限外に利用したりせず共通機能を扱えること。捕捉可能なpanicをABIへ伝播させず、abortや二重panicで終了するprocessのcleanup完遂は保証対象に含めないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-008"></a>
## RQ-03-008

- 根拠: [RQ-03-008](../11_requirements/RQ-03_language-support.md#rq-03-008)
- case: [TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)
- 前提・操作・観測: 明示close、with例外終了、owner回収でtokenの後始末を確認し、finalizerの時刻を要求oracleにしない。
- 期待結果: 明示終了とscope終了による解放を利用できること。管理ownerが回収された場合もnative資源の後始末を管理経路へ引き渡し、owner自身の回収が資源を管理不能にしないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-009"></a>
## RQ-03-009

- 根拠: [RQ-03-009](../11_requirements/RQ-03_language-support.md#rq-03-009)
- case: [TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)
- 前提・操作・観測: main例外/SystemExit/host loop終了/worker例外を独立processで起こし、管理対象UIが残らないかとcleanup証拠を確認する。
- 期待結果: 明示終了、管理対象のmain/host処理終了、未捕捉main例外、SystemExit、host UI loop終了、wrapper所有worker例外の各経路で終了を要求でき、[通常終了の協調条件](../11_requirements/RQ-03_language-support.md#rq-03-015)を満たす管理対象UI threadだけが残って通常終了を妨げないこと。外部hostを使う利用者はmain/hostの管理対象を登録し、その処理終了・loop停止を通知すること。callback例外は種別固有の失敗として扱うこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 部分/詳細待ち / profile・interface・parameter具体化。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-010"></a>
## RQ-03-010

- 根拠: [RQ-03-010](../11_requirements/RQ-03_language-support.md#rq-03-010)
- case: [TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)
- 前提・操作・観測: main例外/SystemExit/host loop終了/worker例外を独立processで起こし、管理対象UIが残らないかとcleanup証拠を確認する。
- 期待結果: os._exit、fatal error、強制killなどprocessが即時終了する経路ではin-process cleanup完遂を保証しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 部分/詳細待ち / profile・interface・parameter具体化。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-011"></a>
## RQ-03-011

- 根拠: [RQ-03-011](../11_requirements/RQ-03_language-support.md#rq-03-011)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)
- 前提・操作・観測: 静的解決例と動的未解決例、同入力再解析、更新競合時のstale diagnosticを確認する。
- 期待結果: 静的に解決できる同一ControlのPropertyアクセスを診断できること。解析不能な動的箇所を安全確認済みと表示せず、同じ入力・設定に同じ診断を返し、古い解析結果を最新として返さないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-03-012"></a>
## RQ-03-012

- 根拠: [RQ-03-012](../11_requirements/RQ-03_language-support.md#rq-03-012)
- case: [TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: Pythonのlint/type checkと、MCP/CLI/CIの各入口で同じ入力・設定から同じ診断を利用できること。未解決の動的箇所を安全確認済みと表示しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-001"></a>
## RQ-04-001

- 根拠: [RQ-04-001](../11_requirements/RQ-04_abi-boundary.md#rq-04-001)
- case: [TC-ABI-REP-01](requirement-test-matrix.md#case-tc-abi-rep-01)
- 前提・操作・観測: ABI宣言と4言語の対応型を照合し、値の境界を含む往復試験で桁幅・符号の解釈が一致することを確認する。
- 期待結果: ABIで扱う整数は幅が定まった整数型で表現できること。呼出側の処理系に依存する幅の整数を共通契約として要求しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-002"></a>
## RQ-04-002

- 根拠: [RQ-04-002](../11_requirements/RQ-04_abi-boundary.md#rq-04-002)
- case: [TC-ABI-REP-01](requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)
- 前提・操作・観測: F32の実際の受渡しAPIが未定義なら未対応として記録し、C型宣言だけの検査を要求適合passとしない。
- 期待結果: ABIで単精度と倍精度の浮動小数値を受け渡せること。初期Propertyの型集合とABI全体の対応範囲を区別すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 設計gap / F32利用API未設計。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-003"></a>
## RQ-04-003

- 根拠: [RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003)
- case: [TC-ABI-REP-01](requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-OWN-01](requirement-test-matrix.md#case-tc-own-01)
- 前提・操作・観測: ASCII・多byte文字・空文字列を各言語から渡し、指定長の範囲で同じ文字列が取得できることを確認する。
- 期待結果: ABIで文字列をUTF-8のbyte列と明示的な長さで受け渡せること。文字列終端の暗黙規則だけに依存しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-004"></a>
## RQ-04-004

- 根拠: [RQ-04-004](../11_requirements/RQ-04_abi-boundary.md#rq-04-004)
- case: [TC-ABI-REP-01](requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-CORE-OBJ-01](requirement-test-matrix.md#case-tc-core-obj-01)
- 前提・操作・観測: 公開宣言に内部objectの構造や実装pointerが現れず、4言語から同じhandle契約を利用できることを確認する。
- 期待結果: ABI利用者がCore内部の具体的なobject表現に依存せず、opaque handleで対象を識別できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-005"></a>
## RQ-04-005

- 根拠: [RQ-04-005](../11_requirements/RQ-04_abi-boundary.md#rq-04-005)
- case: [TC-CORE-OBJ-01](requirement-test-matrix.md#case-tc-core-obj-01)、[TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)
- 前提・操作・観測: 公開契約で保持・解放対象が識別され、保持・解放を伴う境界試験で早期解放や保持漏れが発生しないことを確認する。
- 期待結果: handleの参照をABI利用者が明示的に保持・解放できること。個別関数で所有参照と借用参照の役割を区別できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-006"></a>
## RQ-04-006

- 根拠: [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)
- case: [TC-ABI-REP-01](requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 成功と失敗を含む関数呼出しでstatusが返り、4言語wrapperが同じstatusを解釈できることを確認する。
- 期待結果: ABI関数の実行結果を、言語に依存しないstatus値で呼出側へ返せること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-007"></a>
## RQ-04-007

- 根拠: [RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)
- case: [TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 失敗時にerror objectを取得・解放でき、呼出し元言語で失敗として処理できることを確認する。詳細項目やcode catalogは本要求では固定しない。
- 期待結果: statusだけでは伝えられない失敗情報を、言語固有例外を境界越しに渡さず扱えるerror objectを提供すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-008"></a>
## RQ-04-008

- 根拠: [RQ-04-008](../11_requirements/RQ-04_abi-boundary.md#rq-04-008)
- case: [TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)、[TC-OWN-01](requirement-test-matrix.md#case-tc-own-01)
- 前提・操作・観測: 4言語からcallbackを登録して実行・解除し、userdataの有効期間とdestroyの呼出し責任が契約どおりになることを確認する。
- 期待結果: callback登録で関数、利用者状態を表すuserdata、userdata解放用destroy処理を組み合わせて渡せること。言語固有objectやdelegateそのものをABIで保持させないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-009"></a>
## RQ-04-009

- 根拠: [RQ-04-009](../11_requirements/RQ-04_abi-boundary.md#rq-04-009)
- case: [TC-ABI-COMP-01](requirement-test-matrix.md#case-tc-abi-comp-01)、[TC-ABI-COMP-02](requirement-test-matrix.md#case-tc-abi-comp-02)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: ABI versionの問合せ結果が各言語で取得でき、runtimeが示すversionを利用側が識別できることを確認する。
- 期待結果: 利用中のABI versionを呼出側が問い合わせられること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-010"></a>
## RQ-04-010

- 根拠: [RQ-04-010](../11_requirements/RQ-04_abi-boundary.md#rq-04-010)
- case: [TC-ABI-COMP-02](requirement-test-matrix.md#case-tc-abi-comp-02)
- 前提・操作・観測: 対応機能と未対応機能を照会し、結果が4言語で同じ意味に解釈されることを確認する。
- 期待結果: 利用側がABI機能の対応有無を照会できること。未対応機能を利用側が判別できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-011"></a>
## RQ-04-011

- 根拠: [RQ-04-011](../11_requirements/RQ-04_abi-boundary.md#rq-04-011)
- case: [TC-ABI-EXPOSE-01](requirement-test-matrix.md#case-tc-abi-expose-01)
- 前提・操作・観測: 公開ヘッダ、exported symbol宣言、各言語のFFI宣言にRust固有型や借用参照が現れないことを確認する。
- 期待結果: Rust型、trait、generic、borrowを公開ABIの引数・戻り値・公開構造体に含めない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-012"></a>
## RQ-04-012

- 根拠: [RQ-04-012](../11_requirements/RQ-04_abi-boundary.md#rq-04-012)
- case: [TC-ABI-EXPOSE-01](requirement-test-matrix.md#case-tc-abi-expose-01)
- 前提・操作・観測: 公開ヘッダと各言語のFFI宣言がC ABIで扱える表現だけで構成され、C++ class・STL・RTTI・exception型をABI契約に含まないことを確認する。
- 期待結果: C++ class、STL型、RTTI情報、C++ exceptionを前提とする型表現を公開ABIに含めない。exceptionの実行時伝播禁止はRQ-04-016で扱う。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-013"></a>
## RQ-04-013

- 根拠: [RQ-04-013](../11_requirements/RQ-04_abi-boundary.md#rq-04-013)
- case: [TC-ABI-EXPOSE-01](requirement-test-matrix.md#case-tc-abi-expose-01)
- 前提・操作・観測: 公開ABIと他言語のFFI宣言がPython object表現を参照しないことを確認する。
- 期待結果: Python objectを公開ABIの引数・戻り値・公開構造体に含めない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-014"></a>
## RQ-04-014

- 根拠: [RQ-04-014](../11_requirements/RQ-04_abi-boundary.md#rq-04-014)
- case: [TC-ABI-EXPOSE-01](requirement-test-matrix.md#case-tc-abi-expose-01)
- 前提・操作・観測: 公開ABIと他言語のFFI宣言がCLR object・delegate表現を参照しないことを確認する。
- 期待結果: C# objectまたはdelegateを公開ABIの引数・戻り値・公開構造体に含めない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-015"></a>
## RQ-04-015

- 根拠: [RQ-04-015](../11_requirements/RQ-04_abi-boundary.md#rq-04-015)
- case: [TC-ABI-EXPOSE-01](requirement-test-matrix.md#case-tc-abi-expose-01)、[TC-WRAP-RS-01](requirement-test-matrix.md#case-tc-wrap-rs-01)
- 前提・操作・観測: panicが発生した場合もABI呼出し元へunwindが伝播せず、境界で定めた失敗経路として観測されることを確認する。
- 期待結果: Rust panicをABI境界の外へ伝播させない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-016"></a>
## RQ-04-016

- 根拠: [RQ-04-016](../11_requirements/RQ-04_abi-boundary.md#rq-04-016)
- case: [TC-ABI-EXPOSE-01](requirement-test-matrix.md#case-tc-abi-expose-01)、[TC-WRAP-CPP-01](requirement-test-matrix.md#case-tc-wrap-cpp-01)
- 前提・操作・観測: C++ wrapper内で例外が発生した場合もABI呼出し元へ伝播せず、境界で定めた失敗経路として観測されることを確認する。
- 期待結果: C++ exceptionをABI境界の外へ伝播させない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-017"></a>
## RQ-04-017

- 根拠: [RQ-04-017](../11_requirements/RQ-04_abi-boundary.md#rq-04-017)
- case: [TC-ABI-EXPOSE-01](requirement-test-matrix.md#case-tc-abi-expose-01)、[TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)
- 前提・操作・観測: Python callback等で例外が発生した場合もC ABI呼出しへ伝播せず、境界で定めた失敗経路として観測されることを確認する。
- 期待結果: Python exceptionをABI境界の外へ伝播させない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-018"></a>
## RQ-04-018

- 根拠: [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)
- case: [TC-CORE-OBJ-01](requirement-test-matrix.md#case-tc-core-obj-01)
- 前提・操作・観測: 発行不可状態を内部fixtureで作り、外部取得失敗と既存参照の誤解決不在を分けて確認する。
- 期待結果: 安全に識別できる所有参照を新たに発行できない場合は取得・生成を失敗させ、既存の参照を別対象へ誤って解決しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-019"></a>
## RQ-04-019

- 根拠: [RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)
- case: [TC-CORE-OBJ-01](requirement-test-matrix.md#case-tc-core-obj-01)、[TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)
- 前提・操作・観測: 対応RQの観測結果を個別assertionとして現行caseに照合する。無効な参照、期待する対象種類の不一致、異なるRuntime所属の組合せを呼出し側が失敗理由として区別でき、拒否によって対象状態を変更しないこと。
- 期待結果: 無効な参照、期待する対象種類の不一致、異なるRuntime所属の組合せを呼出し側が失敗理由として区別でき、拒否によって対象状態を変更しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-020"></a>
## RQ-04-020

- 根拠: [RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020)
- case: [TC-ABI-REP-01](requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-OWN-01](requirement-test-matrix.md#case-tc-own-01)
- 前提・操作・観測: 空、多byte、埋込みNUL、不正UTF-8を入出力し、指定長・拒否・無変換を観測する。
- 期待結果: 埋め込みNULを含む有効なUTF-8文字列を指定長のまま扱えること。不正UTF-8を拒否し、暗黙のUnicode正規化や別文字への置換をしないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-021"></a>
## RQ-04-021

- 根拠: [RQ-04-021](../11_requirements/RQ-04_abi-boundary.md#rq-04-021)
- case: [TC-OWN-01](requirement-test-matrix.md#case-tc-own-01)
- 前提・操作・観測: 必要容量より1byte少ないbufferへsentinelを置いて取得し、必要容量とbuffer全体の非変更を確認する。
- 期待結果: 出力bufferの容量が不足する場合は必要容量を知ることができ、bufferへ一部だけを書き込まないこと。容量照会と次の読取りの間の同一snapshot保証があるかをAPIごとに判断できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-022"></a>
## RQ-04-022

- 根拠: [RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)
- case: [TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 主処理の失敗後にerror確保も失敗させ、元statusを保持すること、取得済みsnapshotが次操作で変わらないことを確認する。
- 期待結果: 失敗詳細を生成できない資源不足時にも元の失敗分類を保持すること。error snapshotは保持中に別操作の結果で書き換わらず、未知のstatus/domain/codeも元の数値として取得できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-023"></a>
## RQ-04-023

- 根拠: [RQ-04-023](../11_requirements/RQ-04_abi-boundary.md#rq-04-023)
- case: [TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 名前付きerror定義へ新項目を加え、既存公開codeの不変性と4言語catalog整合を確認する。
- 期待結果: 利用者が失敗理由に名前付きの定義と自動採番を利用できること。公開した理由の識別値と意味を再採番・再利用せず、共通定義を使う場合は言語間で一致すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-024"></a>
## RQ-04-024

- 根拠: [RQ-04-024](../11_requirements/RQ-04_abi-boundary.md#rq-04-024)
- case: [TC-ABI-COMP-01](requirement-test-matrix.md#case-tc-abi-comp-01)、[TC-ABI-COMP-02](requirement-test-matrix.md#case-tc-abi-comp-02)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 対応RQの観測結果を個別assertionとして現行caseに照合する。提供されない機能を利用前に識別でき、未知の機能識別子は未対応として扱えること。公開済み機能識別子とstatusの意味を再利用せず、wrapperが必要とするRuntime互換条件を判断できること。
- 期待結果: 提供されない機能を利用前に識別でき、未知の機能識別子は未対応として扱えること。公開済み機能識別子とstatusの意味を再利用せず、wrapperが必要とするRuntime互換条件を判断できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-025"></a>
## RQ-04-025

- 根拠: [RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)
- case: [TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)、[TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)
- 前提・操作・観測: input/copy/output準備失敗を個別注入し、対象操作の状態/通知非変更と先行独立操作の保持を確認する。
- 期待結果: 入力検証、必要な内容の保持、必須の結果資源の準備に失敗した操作は、その操作に属するCore状態・通知を部分更新しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-026"></a>
## RQ-04-026

- 根拠: [RQ-04-026](../11_requirements/RQ-04_abi-boundary.md#rq-04-026)
- case: [TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)、[TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)
- 前提・操作・観測: input/copy/output準備失敗を個別注入し、対象操作の状態/通知非変更と先行独立操作の保持を確認する。
- 期待結果: 先に成功した独立操作と利用者callbackの副作用を自動的に取り消さないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-027"></a>
## RQ-04-027

- 根拠: [RQ-04-027](../11_requirements/RQ-04_abi-boundary.md#rq-04-027)
- case: [TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 失敗をstatus/domain/codeで機械識別でき、messageは説明用として利用できること。causeとdiagnosticを区別し、保持した失敗snapshotの利用期間を判断できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-028"></a>
## RQ-04-028

- 根拠: [RQ-04-028](../11_requirements/RQ-04_abi-boundary.md#rq-04-028)
- case: [TC-ABI-COMP-01](requirement-test-matrix.md#case-tc-abi-comp-01)、[TC-ABI-COMP-02](requirement-test-matrix.md#case-tc-abi-comp-02)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 安定版RuntimeはMAJOR.MINOR.PATCHで識別し、majorはABI互換系列を示すこと。同majorのminorは後方互換追加、patchは互換修正とすること。wrapperが必要とするminimum互換条件を公表すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-001"></a>
## RQ-05-001

- 根拠: [RQ-05-001](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-001)
- case: [TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)、[TC-OWN-01](requirement-test-matrix.md#case-tc-own-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 利用者が、APIに渡す参照またはAPIから受け取る参照について、借用参照か所有参照か、および破棄責任を判断できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-002"></a>
## RQ-05-002

- 根拠: [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)
- case: [TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)、[TC-OWN-01](requirement-test-matrix.md#case-tc-own-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 同じ対象の所有参照を複数取得し、それぞれ独立して破棄できること。ある参照の破棄で他の有効な所有参照を無効にしないこと。Subscription、Binding、Validatorなどの登録は、その登録の所有参照を破棄しただけでは解除されず、当該機能の解除条件またはRuntime終了まで登録の外部動作を維持すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-003"></a>
## RQ-05-003

- 根拠: [RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)
- case: [TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)、[TC-VERT-08](requirement-test-matrix.md#case-tc-vert-08)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 所有参照は明示的な破棄まで保持できること。Runtime終了後はUI利用を終了し、残る所有参照の解放と不変エラー情報の読取りを引き続き行えること。利用終了にthreadまたは順序の条件がある場合、利用者がその条件を判断できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-004"></a>
## RQ-05-004

- 根拠: [RQ-05-004](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-004)
- case: [TC-OWN-01](requirement-test-matrix.md#case-tc-own-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 利用者が、入力文字列・配列、出力buffer、callbackへ渡される値の所有者と利用可能な期間を判断できること。非同期処理に渡した入力は、その処理が必要とする期間、正しい内容で利用できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-005"></a>
## RQ-05-005

- 根拠: [RQ-05-005](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-005)
- case: [TC-OWN-01](requirement-test-matrix.md#case-tc-own-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 利用者が入力pointer、出力buffer、callback引数pointerのそれぞれについて、利用可能な期間を判断でき、その期間内に値を利用できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-006"></a>
## RQ-05-006

- 根拠: [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)
- case: [TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: callbackの登録が成功した場合、利用者が当該APIの登録終了条件を判断でき、その条件が成立するまでcallbackが呼出可能であること。登録に関連する所有参照の破棄だけでは登録が解除されないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-007"></a>
## RQ-05-007

- 根拠: [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)
- case: [TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: callback解除が完了した後は新しいcallback呼出しが始まらないこと。解除時点ですでに実行中のcallbackがある場合、そのcallbackが終了するまではcallbackに必要な利用者データを利用可能に保つこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-008"></a>
## RQ-05-008

- 根拠: [RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)
- case: [TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: callback登録が成功した場合、登録が無効となりuserdataを使うcallbackが終了した後に、利用者が指定した後始末処理を一度だけ実行すること。登録に失敗した場合は、利用者がuserdataの後始末責任を保持すること。callbackが一度も実行されない場合にも、登録成功後の後始末結果を利用者が判断できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-009"></a>
## RQ-05-009

- 根拠: [RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)
- case: [TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 利用者がcallbackの種類ごとに実行可能なAPI操作と、実行できない操作の結果を判断できること。callback中に発生した通知は、実行中callbackが終了する前に別のcallback呼出しとして割り込まないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-010"></a>
## RQ-05-010

- 根拠: [RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010)
- case: [TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](requirement-test-matrix.md#case-tc-thread-01)
- 前提・操作・観測: 同じ対象の二つの所有参照を作り、一方をworkerで破棄して他方の利用と遅延UI解放を観測する。
- 期待結果: 所有参照を取得・破棄する操作を任意threadから利用できること。UI資源の最終解放が操作の戻りより遅れる場合は、その操作の成功と解放完了を区別できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-011"></a>
## RQ-05-011

- 根拠: [RQ-05-011](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-011)
- case: [TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)
- 前提・操作・観測: 破棄済みの同じ参照を再破棄し、別参照の有効性と対象値が変わらないことを観測する。
- 期待結果: 破棄済みの同じ所有参照を再度破棄しようとした場合は失敗し、他の有効な所有参照と対象状態に影響しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-012"></a>
## RQ-05-012

- 根拠: [RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)
- case: [TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)、[TC-WRAP-CPP-01](requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-RS-01](requirement-test-matrix.md#case-tc-wrap-rs-01)、[TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)
- 前提・操作・観測: UI finalizer受付失敗を注入し、破棄失敗後も同じ所有を再試行できるかを確認する。自動回収ではticket移管をunit側で別検証する。
- 期待結果: 所有参照の破棄を受け付けられなかった場合はその参照の所有を維持し、後から解放を再試行できること。言語側の自動後始末経路でも、解放に失敗した参照を管理不能なまま失わないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-013"></a>
## RQ-05-013

- 根拠: [RQ-05-013](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-013)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)
- 前提・操作・観測: 判定callback内からUI変更/自己解除を試み、状態の非変更と入力拒否/異常の分類を観測する。終了要求は受付後returnし、callback終了までcleanupされないことを別assertionで確認する。
- 期待結果: ValidatorとTextBox入力判定callbackでは同じRuntimeのUI状態変更と自己登録解除を許さず、禁止操作を状態変更前に拒否すること。入力拒否とcallback自体の異常を区別できること。 Runtime終了要求の受付はこの禁止の対象外とし、現在callback終了後の終了要求規則に従うこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-014"></a>
## RQ-05-014

- 根拠: [RQ-05-014](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-014)
- case: [TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)、[TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)
- 前提・操作・観測: 一購読callbackで捕捉例外を起こし、他購読へ継続することと種別別の異常結果を確認する。
- 期待結果: 通知callbackで捕捉可能な言語例外が起きた場合、失敗を診断し、他の有効な通知対象の配信を継続すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-05-015"></a>
## RQ-05-015

- 根拠: [RQ-05-015](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-015)
- case: [TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)、[TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)
- 前提・操作・観測: 一購読callbackで捕捉例外を起こし、他購読へ継続することと種別別の異常結果を確認する。
- 期待結果: 戻り値を持つcallbackでは各種別の失敗結果を得られ、通常の入力拒否とcallback異常を区別できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-001"></a>
## RQ-06-001

- 根拠: [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)
- case: [TC-THREAD-01](requirement-test-matrix.md#case-tc-thread-01)
- 前提・操作・観測: UI thread上の操作は反映され、workerからの直接操作はthread違反となり値・通知を変えないことを確認する。明示的に任意threadを許す操作とUI操作が区別されることも確認する。
- 期待結果: Runtime作成時に結び付いたUI thread上でUI状態の作成・読取・変更・購読操作を行うこと。UI操作を別threadから直接実行した場合は明確なthread違反として扱い、失敗操作でUI状態を変更しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-002"></a>
## RQ-06-002

- 根拠: [RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)
- case: [TC-THREAD-01](requirement-test-matrix.md#case-tc-thread-01)
- 前提・操作・観測: workerからの要求がUI threadで一度だけ実行され、要求受付が処理完了を待たないことを確認する。受付失敗または停止時に未実行処理の所有者と破棄結果を識別できることを確認する。
- 期待結果: workerがUI更新を要求できる引渡し手段を提供すること。要求の受付とUI上での実行完了を区別し、引渡し先でUI操作を直列に処理すること。Runtime停止後など実行できない要求を成功扱いにしないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-003"></a>
## RQ-06-003

- 根拠: [RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)
- case: [TC-THREAD-01](requirement-test-matrix.md#case-tc-thread-01)、[TC-WRAP-CPP-01](requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-WRAP-RS-01](requirement-test-matrix.md#case-tc-wrap-rs-01)
- 前提・操作・観測: GILの暗黙直列性を使わない競合条件でclose/root変更を試験し、UI専用状態へworkerが直接書けないことを確認する。
- 期待結果: 複数threadから変更可能な共有状態は、同時変更を安全に扱える同期を備えること。workerから変更させない状態はread-onlyとして公開するか、値のsnapshot/copyで渡すこと。GIL等、特定言語処理系の暗黙の直列化だけを排他制御の根拠にしない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-004"></a>
## RQ-06-004

- 根拠: [RQ-06-004](../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)
- case: [TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 成功・失敗・キャンセルの各経路と、キャンセル要求後に成功または失敗した場合を区別して観測する。実行中の重複起動規則、古い実行の通知、画面終了後の通知、Runtime停止時の処理を確認する。
- 期待結果: 非同期Commandの実行中状態と、成功・失敗・キャンセルの終端結果を区別すること。実行はUI threadを長時間占有せず、キャンセル要求とworkerの実際の終了結果を区別し、古い実行から遅れて届いた通知が現在の実行状態を上書きしないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-005"></a>
## RQ-06-005

- 根拠: [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)
- case: [TC-THREAD-01](requirement-test-matrix.md#case-tc-thread-01)、[TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)
- 前提・操作・観測: callbackから終了要求を出し、現在callback完了と未開始通常workとの前後関係を記録する。
- 期待結果: callbackまたはUI event処理中にもRuntime終了を要求できること。現在の処理を完了させた後、未開始の通常作業より優先して終了処理を実行すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-006"></a>
## RQ-06-006

- 根拠: [RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006)
- case: [TC-THREAD-01](requirement-test-matrix.md#case-tc-thread-01)
- 前提・操作・観測: UI通常context、UI callback、workerの各同期/非同期呼出しで受付/完了と戻る時点を観測する。
- 期待結果: 非同期の終了要求は受付後に戻り、終了完了を待たないこと。同期指定でもcallback/event中は受付後に戻り、workerからの同期指定はUI threadでの終了完了後に戻ること。受付済みと完了済みを区別できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-007"></a>
## RQ-06-007

- 根拠: [RQ-06-007](../11_requirements/RQ-06_threading-dispatch.md#rq-06-007)
- case: [TC-THREAD-01](requirement-test-matrix.md#case-tc-thread-01)
- 前提・操作・観測: host停止通知を同期要求と競合させ、未完了理由と待機終了を観測する。timeout選択が未決なら期待値を固定しない。
- 期待結果: UI処理を実行できないことが確定した場合は、同期終了要求を無期限に待ち続けず、未完了と待機終了理由を呼出し側が識別できること。待機終了を資源解放完了として扱わないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-008"></a>
## RQ-06-008

- 根拠: [RQ-06-008](../11_requirements/RQ-06_threading-dispatch.md#rq-06-008)
- case: [TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 受付前のrecord確保失敗と、受付後のstart失敗を分け、operation有無・execute結果・終端結果を観測する。
- 期待結果: Commandの実行受付成功と業務処理の成功・失敗を区別できること。受付後の開始処理の失敗はその実行の失敗として観測でき、受付前の失敗では実行を開始しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-009"></a>
## RQ-06-009

- 根拠: [RQ-06-009](../11_requirements/RQ-06_threading-dispatch.md#rq-06-009)
- case: [TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 同ID再cancel、終端後cancel、古いID、close後完了、handle releaseのみを分けて実行する。
- 期待結果: 同じ実行への重複取消と終端済みの同じ実行への取消は追加効果を起こさず、古い実行へ誤適用しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-010"></a>
## RQ-06-010

- 根拠: [RQ-06-010](../11_requirements/RQ-06_threading-dispatch.md#rq-06-010)
- case: [TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 同ID再cancel、終端後cancel、古いID、close後完了、handle releaseのみを分けて実行する。
- 期待結果: Commandの利用終了は新規実行を止め取消を要求し、worker終了をUI threadで待たないこと。所有参照の破棄だけでは利用終了を要求しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-06-011"></a>
## RQ-06-011

- 根拠: [RQ-06-011](../11_requirements/RQ-06_threading-dispatch.md#rq-06-011)
- case: [TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 中間進捗は最新値への集約を許し、終端結果は進捗と独立して一度だけ反映すること。過去実行および終端後の通知で現在状態を変更しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-001"></a>
## RQ-07-001

- 根拠: [RQ-07-001](../11_requirements/RQ-07_binding.md#rq-07-001)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)
- 前提・操作・観測: 作成時に値が同期し、source・targetの後続変更が相手へ伝わらず、変更通知callbackがBindingを介して開始されないことを確認する。
- 期待結果: 接続作成時にsourceの現在値をtargetへ一度だけ反映し、その後のどちら側の変更も相手へ伝播しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-002"></a>
## RQ-07-002

- 根拠: [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-VERT-04](requirement-test-matrix.md#case-tc-vert-04)
- 前提・操作・観測: 初期値とsource変更がtargetに反映され、targetの直接変更がsourceを変えず、その後のsource変更がtargetを更新することを確認する。
- 期待結果: 作成時にsourceからtargetへ同期し、以後sourceの変更をtargetへ伝えること。targetの直接変更はsourceへ返さず、次のsource変更まではtarget側で観測できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-003"></a>
## RQ-07-003

- 根拠: [RQ-07-003](../11_requirements/RQ-07_binding.md#rq-07-003)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)
- 前提・操作・観測: 初期同期のsource優先と両方向の変更伝播を確認し、両側の直接変更が同一処理中に起きたとき最後の成功値へ収束し、無限往復しないことを確認する。
- 期待結果: 作成時はsourceからtargetへ同期し、以後どちら側の直接変更も相手へ伝えること。同じ外側処理中に両方が直接変更された場合は、最後に成功した直接変更の値を両側で観測すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-004"></a>
## RQ-07-004

- 根拠: [RQ-07-004](../11_requirements/RQ-07_binding.md#rq-07-004)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)
- 前提・操作・観測: 3 modeの方向表を4言語から同じ入力列で検証し、初期同期・source変更・target変更の伝播先が一致することを確認する。
- 期待結果: 各Binding modeのsource/target方向と、直接変更が逆向きへ戻るか否かを明示すること。方向の意味をwrapperごとに変えないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-005"></a>
## RQ-07-005

- 根拠: [RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: setter直後のsource getterと、callback終了前後のtarget getterを記録し、4言語で同じ時点に同じ値を観測することを確認する。
- 期待結果: setterが成功したProperty自身のgetterは新値を直ちに返すこと。Binding先への伝播は外側のUI処理が戻った後に行い、同一callback中の相手側readでは旧値の場合があることを一貫して扱うこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-006"></a>
## RQ-07-006

- 根拠: [RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 一つの処理区間で同じsourceを複数回変更し、targetが最終値になってから外部購読者へ通知されること、target通知は後続配信回となることを確認する。
- 期待結果: 同じ処理区間の複数source更新は最終値を伝播すること。該当する配信回ではBinding伝播を外部Property通知より先に完了し、Bindingによる変更の通知を再帰させないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-007"></a>
## RQ-07-007

- 根拠: [RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)
- 前提・操作・観測: 循環・重複・競合する入力元を持つ接続作成が失敗し、失敗前のProperty値と既存Bindingの伝播結果が維持されることを確認する。
- 期待結果: 初期契約のBindingグラフで自己接続・有向循環、または一つのtargetへ複数のOneWay入力元を作る接続を受け入れないこと。拒否する接続の登録により既存値や接続状態を変えないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-008"></a>
## RQ-07-008

- 根拠: [RQ-07-008](../11_requirements/RQ-07_binding.md#rq-07-008)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: callbackからPropertyを更新し、callbackが再帰呼出しされず後続配信で値が反映されることを確認する。反復更新が作業枠を越える場合にUI処理へ制御を返すことも確認する。
- 期待結果: 通知処理から生じた更新を同じ通知処理へ再帰的に割り込ませず、後続の配信で反映すること。反復更新があってもUIへ制御を返せ、非収束の診断・停止対象を識別できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-009"></a>
## RQ-07-009

- 根拠: [RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)
- case: [TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)、[TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 失敗したsetter後も対象Propertyの確定値が維持されること、Aのsetter成功後にBのsetterが失敗してもAは新値、Bは旧値であることを確認する。
- 期待結果: 失敗したProperty更新は当該Propertyの確定値を変えず、成功したProperty更新で確定した値は、後続する別Property更新の失敗を理由に取り消さないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-010"></a>
## RQ-07-010

- 根拠: [RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)
- case: [TC-BIND-INIT-REJECT-01](requirement-test-matrix.md#case-tc-bind-init-reject-01)、[TC-BIND-PROPAGATION-REJECT-01](requirement-test-matrix.md#case-tc-bind-propagation-reject-01)、[TC-ERROR-01](requirement-test-matrix.md#case-tc-error-01)、[TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 初期同期拒否時に作成失敗が返り、target値・接続状態が変化しないことを確認する。作成後の拒否時はsourceの確定値、targetの従前値、Bindingエラーを確認し、次の有効な伝播後に値が同期し現在エラーが消えることを確認する。
- 期待結果: Binding作成時の初期同期でtargetが値を拒否した場合、作成失敗を呼出側へ返し、targetの確定値と接続状態を変更しないこと。作成後の伝播でtargetが値を拒否した場合、先行するsource setterの成功を取り消さず、sourceの確定値とtargetの従前の確定値を保ち、拒否内容をBinding固有の機械可読なエラーとして観測可能にすること。次の有効な伝播で値を同期し、現在のBindingエラーを解消すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-011"></a>
## RQ-07-011

- 根拠: [RQ-07-011](../11_requirements/RQ-07_binding.md#rq-07-011)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)
- 前提・操作・観測: Binding handleのrelease後もsourceの変更がtargetへ伝播することを確認する。明示解除後は将来の変更と保留中の変更が伝播せず、解除時の各Property値が維持され、Binding固有のerror状態が終了することを確認する。
- 期待結果: Bindingの所有参照の破棄だけでは接続を解除せず、明示解除またはRuntime終了まで伝播を維持すること。解除後は将来および保留中の変更をその接続経由で伝播せず、各Propertyの確定値を維持すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-012"></a>
## RQ-07-012

- 根拠: [RQ-07-012](../11_requirements/RQ-07_binding.md#rq-07-012)
- case: [TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)
- 前提・操作・観測: 購読追加/解除を配信callbackから行い、当該回の対象・順序と後続回の対象を区別して観測する。
- 期待結果: Property購読時は初期変更通知を送らず、現在値を読取り操作で得られること。同じ配信対象の購読者は登録順に通知され、配信中に追加した購読はその配信回に割り込まないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-013"></a>
## RQ-07-013

- 根拠: [RQ-07-013](../11_requirements/RQ-07_binding.md#rq-07-013)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 単調時計fixtureで閾値前後・無効化・同chain再観測を確認し、診断だけで更新停止しないことを観測する。
- 期待結果: 長期保留の診断閾値を利用者が設定・無効化でき、既定値を5秒とすること。長期保留の検出だけを理由に更新連鎖を停止・破棄せず、同じ連鎖の同じ保留状態を重複診断し続けないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-014"></a>
## RQ-07-014

- 根拠: [RQ-07-014](../11_requirements/RQ-07_binding.md#rq-07-014)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 独立した二chainの一方を反復させ、停止対象・確定値・長期保留との差・破棄/再開結果を観測する。
- 期待結果: 非収束と判断できた更新連鎖のみを停止し、独立した連鎖と既に確定した値を維持すること。長期保留と非収束停止を区別して観測でき、保留更新の破棄と停止連鎖の再開を明示的に選べること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-015"></a>
## RQ-07-015

- 根拠: [RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)
- 前提・操作・観測: 同じsource-target接続の再登録を拒否し、既存graph/値/通知を比較する。
- 期待結果: 同じsourceから同じtargetへの重複Binding接続を拒否し、失敗前の値・接続・通知を維持すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-07-016"></a>
## RQ-07-016

- 根拠: [RQ-07-016](../11_requirements/RQ-07_binding.md#rq-07-016)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 初期Property/View入力検証は短時間の同期結果を扱い、Core管理の非同期検証・pending状態は提供しないこと。長時間・remote検証と古い結果の識別はアプリ側の非同期処理・状態で扱えること。複数Propertyの一括再評価・共通表示の追加は含めないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-001"></a>
## RQ-09-001

- 根拠: [RQ-09-001](../11_requirements/RQ-09_ui-scope.md#rq-09-001)
- case: [TC-TARGET-A64-01](requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-ARMV7-01](requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TARGET-LATER-01](requirement-test-matrix.md#case-tc-target-later-01)、[TC-TARGET-LNX-01](requirement-test-matrix.md#case-tc-target-lnx-01)、[TC-TARGET-WIN-01](requirement-test-matrix.md#case-tc-target-win-01)、[TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)
- 前提・操作・観測: Windowに初期シナリオのルートを設定し、表示内容がそのルートに従うことを確認する。
- 期待結果: WindowはアプリのUIルートとなり、初期シナリオの内容を表示する一つのcontent rootを持つ。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-002"></a>
## RQ-09-002

- 根拠: [RQ-09-002](../11_requirements/RQ-09_ui-scope.md#rq-09-002)
- case: [TC-TREE-01](requirement-test-matrix.md#case-tc-tree-01)、[TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 初期Controlが共通Widgetとして扱われ、共通操作・状態の読み取りができることを確認する。
- 期待結果: WidgetはUI要素の共通概念であり、初期Controlの共通状態と操作対象を表せる。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-003"></a>
## RQ-09-003

- 根拠: [RQ-09-003](../11_requirements/RQ-09_ui-scope.md#rq-09-003)
- case: [TC-TREE-01](requirement-test-matrix.md#case-tc-tree-01)、[TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 初期シナリオでWindowとStackPanelの親子関係を確認する。子数や追加・削除APIの共通制約は本要求で定めない。
- 期待結果: Containerは子Widgetを持ちうるWidgetの分類である。初期共通基盤ではWindowとStackPanelが該当する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-004"></a>
## RQ-09-004

- 根拠: [RQ-09-004](../11_requirements/RQ-09_ui-scope.md#rq-09-004)
- case: [TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)、[TC-VERT-04](requirement-test-matrix.md#case-tc-vert-04)
- 前提・操作・観測: ViewModel側の値を変えるとTextの表示値が更新され、Textから編集できないことを確認する。
- 期待結果: Textは読み取り専用の文字列表示Controlで、OneWay Bindingの表示先となる。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-005"></a>
## RQ-09-005

- 根拠: [RQ-09-005](../11_requirements/RQ-09_ui-scope.md#rq-09-005)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)、[TC-VERT-06](requirement-test-matrix.md#case-tc-vert-06)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)
- 前提・操作・観測: 入力・選択・caretの結果を読み取れること、および承認されたcommit・validation条件で成功値または失敗状態が観測できることを確認する。
- 期待結果: TextBoxは文字列入力Controlであり、入力文字列と選択・caret状態を保持できる。TwoWay Bindingとcommit時の検証を扱う。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-006"></a>
## RQ-09-006

- 根拠: [RQ-09-006](../11_requirements/RQ-09_ui-scope.md#rq-09-006)
- case: [TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-05](requirement-test-matrix.md#case-tc-vert-05)、[TC-VERT-06](requirement-test-matrix.md#case-tc-vert-06)
- 前提・操作・観測: 実行可否の両状態で操作し、実行回数と実行中の再操作結果が条件を満たすことを確認する。
- 期待結果: ButtonはCommandを起動する。CanExecuteがfalseなら起動せず、同一Commandの実行中に再入させない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-007"></a>
## RQ-09-007

- 根拠: [RQ-09-007](../11_requirements/RQ-09_ui-scope.md#rq-09-007)
- case: [TC-LAYOUT-01](requirement-test-matrix.md#case-tc-layout-01)、[TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)
- 前提・操作・観測: 初期シナリオの子順と対応するVisual配置を確認する。特定の方向、spacing、overflow規則は別途承認なしに受入条件へ加えない。
- 期待結果: StackPanelは順序付きの子を配置し、配置方向と子の配置位置・伸長を設定できること。設定省略時の結果は本章の既定値要求に従うこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-008"></a>
## RQ-09-008

- 根拠: [RQ-09-008](../11_requirements/RQ-09_ui-scope.md#rq-09-008)
- case: [TC-LAYOUT-01](requirement-test-matrix.md#case-tc-layout-01)
- 前提・操作・観測: 入力領域を変えたとき各Widgetの配置結果が再計算され、Coreの配置結果とPlatformのroot領域が整合することを確認する。寸法既定値や公開APIはここで固定しない。
- 期待結果: 利用可能な表示領域と各Widgetの配置条件に従って表示位置・大きさを決定でき、親領域や配置条件の変化後も表示・入力対象の位置が一致すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-009"></a>
## RQ-09-009

- 根拠: [RQ-09-009](../11_requirements/RQ-09_ui-scope.md#rq-09-009)
- case: [TC-FOCUS-01](requirement-test-matrix.md#case-tc-focus-01)
- 前提・操作・観測: フォーカス対象がない状態とある状態を確認し、設定・解除後に現在対象が一意に観測できることを確認する。
- 期待結果: active Windowと、0または1個のactive focus対象を識別できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-010"></a>
## RQ-09-010

- 根拠: [RQ-09-010](../11_requirements/RQ-09_ui-scope.md#rq-09-010)
- case: [TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)
- 前提・操作・観測: TextBoxがactive focusのとき入力文字が届き、非active対象へ誤配送されないことを確認する。特定のキー割当やfocus navigationは要求しない。
- 期待結果: 対象Platformの入力を共通UI機能で利用でき、keyboard・文字入力が現在のactive focus対象へ届くこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-011"></a>
## RQ-09-011

- 根拠: [RQ-09-011](../11_requirements/RQ-09_ui-scope.md#rq-09-011)
- case: [TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)
- 前提・操作・観測: 設定したnameをControlから取得し、PlatformのAccessibility公開を介して対応対象のnameとして確認できることを確認する。自動命名規則は固定しない。
- 期待結果: 初期ControlのAccessibility nameを設定・取得でき、対応するUI要素の意味情報として利用できること。空欄の扱いは本章およびRQ-10のname要求に従うこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-012"></a>
## RQ-09-012

- 根拠: [RQ-09-012](../11_requirements/RQ-09_ui-scope.md#rq-09-012)
- case: [TC-FEATURE-NEXT-01](requirement-test-matrix.md#case-tc-feature-next-01)
- 前提・操作・観測: stage計画に対象として記録され、具体化・承認された必須caseの結果が確認できる。
- 期待結果: CheckBoxを次段階の対象に含める。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-013"></a>
## RQ-09-013

- 根拠: [RQ-09-013](../11_requirements/RQ-09_ui-scope.md#rq-09-013)
- case: [TC-FEATURE-NEXT-02](requirement-test-matrix.md#case-tc-feature-next-02)
- 前提・操作・観測: 同上。
- 期待結果: ToggleButtonを次段階の対象に含める。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-014"></a>
## RQ-09-014

- 根拠: [RQ-09-014](../11_requirements/RQ-09_ui-scope.md#rq-09-014)
- case: [TC-FEATURE-NEXT-03](requirement-test-matrix.md#case-tc-feature-next-03)
- 前提・操作・観測: 同上。
- 期待結果: ScrollViewerを次段階の対象に含める。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-015"></a>
## RQ-09-015

- 根拠: [RQ-09-015](../11_requirements/RQ-09_ui-scope.md#rq-09-015)
- case: [TC-FEATURE-NEXT-04](requirement-test-matrix.md#case-tc-feature-next-04)
- 前提・操作・観測: 同上。
- 期待結果: Imageを次段階の対象に含める。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-016"></a>
## RQ-09-016

- 根拠: [RQ-09-016](../11_requirements/RQ-09_ui-scope.md#rq-09-016)
- case: [TC-FEATURE-NEXT-05](requirement-test-matrix.md#case-tc-feature-next-05)
- 前提・操作・観測: 同上。
- 期待結果: Borderを次段階の対象に含める。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-017"></a>
## RQ-09-017

- 根拠: [RQ-09-017](../11_requirements/RQ-09_ui-scope.md#rq-09-017)
- case: [TC-FEATURE-NEXT-06](requirement-test-matrix.md#case-tc-feature-next-06)
- 前提・操作・観測: 同上。
- 期待結果: Separatorを次段階の対象に含める。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-018"></a>
## RQ-09-018

- 根拠: [RQ-09-018](../11_requirements/RQ-09_ui-scope.md#rq-09-018)
- case: [TC-FEATURE-NEXT-07](requirement-test-matrix.md#case-tc-feature-next-07)
- 前提・操作・観測: 同上。
- 期待結果: ProgressBarを次段階の対象に含める。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-019"></a>
## RQ-09-019

- 根拠: [RQ-09-019](../11_requirements/RQ-09_ui-scope.md#rq-09-019)
- case: [TC-COL-01](requirement-test-matrix.md#case-tc-col-01)、[TC-NFR-07](requirement-test-matrix.md#case-tc-nfr-07)
- 前提・操作・観測: Model整合性のbatch成功・拒否と、独立入力のcommit・validationを別々に実行し、部分適用がなく、独立入力を不必要に待たせないことを確認する。
- 期待結果: 次段階で、UI要素の所属や単一値の種類と独立した項目集合を扱え、順序から独立した項目識別、構造化された変更内容、可視範囲の一貫した内容を利用できること。集合の一括更新は全件成功または全件拒否とし、独立した入力の値確定・業務検証を別入力待ちにしないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-020"></a>
## RQ-09-020

- 根拠: [RQ-09-020](../11_requirements/RQ-09_ui-scope.md#rq-09-020)
- case: [TC-SEL-01](requirement-test-matrix.md#case-tc-sel-01)
- 前提・操作・観測: cardinalityごとの選択、current itemとfocusの独立性、選択中・current item削除後の状態を確認する。
- 期待結果: 次段階で、項目の安定した識別に基づく単一・複数選択を利用できること。現在項目は選択状態およびkeyboard focusから独立し、削除項目を選択中・現在項目として残さないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-021"></a>
## RQ-09-021

- 根拠: [RQ-09-021](../11_requirements/RQ-09_ui-scope.md#rq-09-021)
- case: [TC-FEATURE-NEXT-08](requirement-test-matrix.md#case-tc-feature-next-08)
- 前提・操作・観測: ListViewで項目表示と選択状態を確認し、安定IDと共通Modelの結果に矛盾がないことを確認する。個別navigationは具体化時に定める。
- 期待結果: ListViewを次段階の対象に含め、共通CollectionModel・SelectionModelの契約を利用する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-022"></a>
## RQ-09-022

- 根拠: [RQ-09-022](../11_requirements/RQ-09_ui-scope.md#rq-09-022)
- case: [TC-FEATURE-NEXT-09](requirement-test-matrix.md#case-tc-feature-next-09)、[TC-NFR-07](requirement-test-matrix.md#case-tc-nfr-07)
- 前提・操作・観測: 大きな項目集合で表示対象範囲を移動し、可視範囲とsnapshotが一致し、画面外項目が論理データから失われないことを確認する。
- 期待結果: 仮想化Listを次段階の対象に含め、表示範囲とCollectionModelのvisible range snapshotを整合させる。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-023"></a>
## RQ-09-023

- 根拠: [RQ-09-023](../11_requirements/RQ-09_ui-scope.md#rq-09-023)
- case: [TC-FEATURE-NEXT-10](requirement-test-matrix.md#case-tc-feature-next-10)
- 前提・操作・観測: 承認されたStyle条件を複数Controlに適用し、指定された表示属性が一致することを確認する。優先順位やselector文法は具体化時に定める。
- 期待結果: Styleを次段階の対象に含め、Controlの表示規則を再利用可能にする。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-024"></a>
## RQ-09-024

- 根拠: [RQ-09-024](../11_requirements/RQ-09_ui-scope.md#rq-09-024)
- case: [TC-FEATURE-NEXT-11](requirement-test-matrix.md#case-tc-feature-next-11)
- 前提・操作・観測: 承認されたThemeを適用し、対象Controlの指定属性がThemeの値に反映されることを確認する。切替時期や継承規則は具体化時に定める。
- 期待結果: Themeを次段階の対象に含め、複数Controlで共有する外観設定を切替可能にする。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-025"></a>
## RQ-09-025

- 根拠: [RQ-09-025](../11_requirements/RQ-09_ui-scope.md#rq-09-025)
- case: [TC-FEATURE-NEXT-12](requirement-test-matrix.md#case-tc-feature-next-12)
- 前提・操作・観測: 承認されたResource参照が指定値を返し、参照対象が未定義の場合の結果が具体化した条件に一致することを確認する。検索範囲は具体化時に定める。
- 期待結果: Resourceを次段階の対象に含め、UI構成から再利用可能な値を参照できるようにする。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-026"></a>
## RQ-09-026

- 根拠: [RQ-09-026](../11_requirements/RQ-09_ui-scope.md#rq-09-026)
- case: [TC-FEATURE-NEXT-13](requirement-test-matrix.md#case-tc-feature-next-13)
- 前提・操作・観測: Template差替え後もControlの論理上の親子関係と公開意味情報が保たれ、表示構成が指定どおり変わることを確認する。
- 期待結果: Templateを次段階の対象に含め、Controlの内部表示構成を差し替え可能にする。内部visual部品をLogical Tree上の独立Controlとして扱うかは既承認のTree境界に従う。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-027"></a>
## RQ-09-027

- 根拠: [RQ-09-027](../11_requirements/RQ-09_ui-scope.md#rq-09-027)
- case: [TC-ROADMAP-01](requirement-test-matrix.md#case-tc-roadmap-01)
- 前提・操作・観測: 未採用候補が必須機能・必須case・完了条件へ混入していないことを確認する。
- 期待結果: ComboBox
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 将来候補・採用時再評価。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-028"></a>
## RQ-09-028

- 根拠: [RQ-09-028](../11_requirements/RQ-09_ui-scope.md#rq-09-028)
- case: [TC-ROADMAP-01](requirement-test-matrix.md#case-tc-roadmap-01)
- 前提・操作・観測: 同上。
- 期待結果: TreeView
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 将来候補・採用時再評価。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-029"></a>
## RQ-09-029

- 根拠: [RQ-09-029](../11_requirements/RQ-09_ui-scope.md#rq-09-029)
- case: [TC-ROADMAP-01](requirement-test-matrix.md#case-tc-roadmap-01)
- 前提・操作・観測: 同上。
- 期待結果: TableView
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 将来候補・採用時再評価。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-030"></a>
## RQ-09-030

- 根拠: [RQ-09-030](../11_requirements/RQ-09_ui-scope.md#rq-09-030)
- case: [TC-ROADMAP-01](requirement-test-matrix.md#case-tc-roadmap-01)
- 前提・操作・観測: 同上。
- 期待結果: Dialog
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 将来候補・採用時再評価。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-031"></a>
## RQ-09-031

- 根拠: [RQ-09-031](../11_requirements/RQ-09_ui-scope.md#rq-09-031)
- case: [TC-ROADMAP-01](requirement-test-matrix.md#case-tc-roadmap-01)
- 前提・操作・観測: 同上。
- 期待結果: Popup
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 将来候補・採用時再評価。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-032"></a>
## RQ-09-032

- 根拠: [RQ-09-032](../11_requirements/RQ-09_ui-scope.md#rq-09-032)
- case: [TC-ROADMAP-01](requirement-test-matrix.md#case-tc-roadmap-01)
- 前提・操作・観測: 同上。
- 期待結果: Menu
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 将来候補・採用時再評価。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-033"></a>
## RQ-09-033

- 根拠: [RQ-09-033](../11_requirements/RQ-09_ui-scope.md#rq-09-033)
- case: [TC-ROADMAP-01](requirement-test-matrix.md#case-tc-roadmap-01)
- 前提・操作・観測: 同上。
- 期待結果: ToolBar
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 将来候補・採用時再評価。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-034"></a>
## RQ-09-034

- 根拠: [RQ-09-034](../11_requirements/RQ-09_ui-scope.md#rq-09-034)
- case: [TC-ROADMAP-01](requirement-test-matrix.md#case-tc-roadmap-01)
- 前提・操作・観測: 同上。
- 期待結果: TabView
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 将来候補・採用時再評価。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-035"></a>
## RQ-09-035

- 根拠: [RQ-09-035](../11_requirements/RQ-09_ui-scope.md#rq-09-035)
- case: [TC-ROADMAP-01](requirement-test-matrix.md#case-tc-roadmap-01)
- 前提・操作・観測: 同上。
- 期待結果: binding診断
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 将来候補・採用時再評価。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-036"></a>
## RQ-09-036

- 根拠: [RQ-09-036](../11_requirements/RQ-09_ui-scope.md#rq-09-036)
- case: [TC-ROADMAP-01](requirement-test-matrix.md#case-tc-roadmap-01)
- 前提・操作・観測: 同上。
- 期待結果: UIテスト支援
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 将来候補・採用時再評価。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-037"></a>
## RQ-09-037

- 根拠: [RQ-09-037](../11_requirements/RQ-09_ui-scope.md#rq-09-037)
- case: [TC-COL-01](requirement-test-matrix.md#case-tc-col-01)、[TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-FEATURE-NEXT-08](requirement-test-matrix.md#case-tc-feature-next-08)、[TC-FEATURE-NEXT-09](requirement-test-matrix.md#case-tc-feature-next-09)、[TC-FEATURE-NEXT-13](requirement-test-matrix.md#case-tc-feature-next-13)、[TC-SEL-01](requirement-test-matrix.md#case-tc-sel-01)
- 前提・操作・観測: 共通契約が個別Controlの設計開始前に定義されていることを確認する。Model batchの整合性検証と独立入力validationの検証が分離されていることも確認する。
- 期待結果: List、ComboBox、Tree、Tableに共通するModel、Selection、Item template、Collection差分通知、virtualizationの契約を、個別Controlより先に設計する。Collection change batchの全件適用／全件拒否はModel整合性を対象とし、独立入力欄の有効なcommitを別入力のvalidation待ちにしない。Gridを初期Controlへ追加する判断は本要求に含めない。Controlごとのmode、gesture、navigationは、各Controlの採用範囲を承認した後に具体化する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-038"></a>
## RQ-09-038

- 根拠: [RQ-09-038](../11_requirements/RQ-09_ui-scope.md#rq-09-038)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)
- 前提・操作・観測: 同じ編集候補をAccept/Incomplete/通常Reject/IME Rejectへ振り分け、編集全文・選択・確定値・理由を別観測する。
- 期待結果: Acceptは候補を確定して伝播し、Incompleteは編集文字列と選択位置に保持して確定値へ伝播しないこと。通常Rejectは従前の編集値・選択を維持し理由を示すこと。IME commitのRejectは候補を未受理編集値として保持すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-039"></a>
## RQ-09-039

- 根拠: [RQ-09-039](../11_requirements/RQ-09_ui-scope.md#rq-09-039)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)
- 前提・操作・観測: 入力Accept後にTextBox自身の確定値Validatorを拒否させる。案Bを採用し、candidate保持・旧確定値・拒否reasonを観測する。
- 期待結果: 入力判定でAcceptされた候補をTextBoxの確定値検証が拒否した場合、確定値とViewModel値を維持し、拒否された候補と理由を利用者が確認・修正できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-040"></a>
## RQ-09-040

- 根拠: [RQ-09-040](../11_requirements/RQ-09_ui-scope.md#rq-09-040)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: composition開始前に理由を持つ状態を作り、Update後Cancelする。開始時state/reasonの復元をoracleとする。
- 期待結果: IMEの明示cancelではcomposition開始直前の編集文字列・caret・selectionと入力状態・理由へ戻り、確定値検証とBinding伝播を起こさないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-041"></a>
## RQ-09-041

- 根拠: [RQ-09-041](../11_requirements/RQ-09_ui-scope.md#rq-09-041)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)
- 前提・操作・観測: IME Commit候補copy/退避に失敗を注入し、候補保持場所と再送可能性をCore/Adapter別に確認する。
- 期待結果: IME確定候補の処理準備が失敗した場合、候補を回復可能に保持できたかと再試行方法を利用者が判断できること。保持できなかった候補を処理成功として扱わないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 部分/詳細待ち / profile・interface・parameter具体化。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-042"></a>
## RQ-09-042

- 根拠: [RQ-09-042](../11_requirements/RQ-09_ui-scope.md#rq-09-042)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)
- 前提・操作・観測: preedit中に外部確定値を変更し、旧sessionの遅延Commitで新値が上書きされないことを確認する。
- 期待結果: 外部から異なる確定値が設定された場合は編集値をその値へ更新し、入力状態・理由を初期化すること。旧IME sessionの遅延イベントでその値を再び上書きしないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-043"></a>
## RQ-09-043

- 根拠: [RQ-09-043](../11_requirements/RQ-09_ui-scope.md#rq-09-043)
- case: [TC-TREE-01](requirement-test-matrix.md#case-tc-tree-01)、[TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)
- 前提・操作・観測: subtreeをdetachし、独立参照からProperty更新/Binding継続と描画/Input非対象を確認する。
- 期待結果: 親から切り離しても独立した所有参照を持つ対象とその子構造をCore上で利用できること。切離しだけで通常のProperty・Binding・Subscriptionを終了させず、描画・入力対象からは外すこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-044"></a>
## RQ-09-044

- 根拠: [RQ-09-044](../11_requirements/RQ-09_ui-scope.md#rq-09-044)
- case: [TC-TREE-01](requirement-test-matrix.md#case-tc-tree-01)
- 前提・操作・観測: 複数親/循環/別Runtimeを拒否し、再add失敗時のDetached構造・値・登録が維持されることを確認する。
- 期待結果: 複数親、親子循環、異なるRuntime所属を作る親子変更を拒否すること。変更失敗で既存の親子・値・登録を部分変更せず、Detached対象の再所属失敗時はDetached状態を維持すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-045"></a>
## RQ-09-045

- 根拠: [RQ-09-045](../11_requirements/RQ-09_ui-scope.md#rq-09-045)
- case: [TC-LAYOUT-01](requirement-test-matrix.md#case-tc-layout-01)
- 前提・操作・観測: 明示した各direction/Alignを確認し、省略時既定は承認済み既定値RQを別assertionとして確認する。
- 期待結果: StackPanelの配置方向を縦・横から選び、子の配置位置と伸長を設定できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-046"></a>
## RQ-09-046

- 根拠: [RQ-09-046](../11_requirements/RQ-09_ui-scope.md#rq-09-046)
- case: [TC-FOCUS-01](requirement-test-matrix.md#case-tc-focus-01)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)
- 前提・操作・観測: 親disabledで子own値を保持しつつ操作/Focusを拒否し、detachで代替Focusが暗黙選択されないことを確認する。
- 期待結果: Control自身またはLogical祖先が無効ならInput・Focus・Button操作を受け付けず、子自身の設定値を変更しないこと。Focus対象を失った場合は無効な対象を残さず、別Controlを暗黙に選ばないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-047"></a>
## RQ-09-047

- 根拠: [RQ-09-047](../11_requirements/RQ-09_ui-scope.md#rq-09-047)
- case: [TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)
- 前提・操作・観測: key割当の将来候補を必須caseにしない。CommandなしButtonは表示可能でactivationがno-opとなることを確認する。
- 期待結果: 特定キーによるFocus移動・shortcut/action bindingは初期必須機能に含めないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-048"></a>
## RQ-09-048

- 根拠: [RQ-09-048](../11_requirements/RQ-09_ui-scope.md#rq-09-048)
- case: [TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)
- 前提・操作・観測: key割当の将来候補を必須caseにしない。CommandなしButtonは表示可能でactivationがno-opとなることを確認する。
- 期待結果: Commandが設定されていないButtonの表示を許し、起動操作で処理を実行しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-049"></a>
## RQ-09-049

- 根拠: [RQ-09-049](../11_requirements/RQ-09_ui-scope.md#rq-09-049)
- case: [TC-FOCUS-01](requirement-test-matrix.md#case-tc-focus-01)、[TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)
- 前提・操作・観測: Focus移動時のstate/Lost/Gained順と、targetなし入力後に別targetへ誤配送しないことを確認する。
- 期待結果: Focus変更後の状態を観測でき、旧対象のFocusLostを新対象のFocusGainedより先に扱うこと。入力対象がないkeyboard/textイベントを後の別対象へ遅延配送しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-050"></a>
## RQ-09-050

- 根拠: [RQ-09-050](../11_requirements/RQ-09_ui-scope.md#rq-09-050)
- case: [TC-COL-01](requirement-test-matrix.md#case-tc-col-01)
- 前提・操作・観測: insert/move/remove後のitem IDと編集対象を比較し、位置の変化で別itemへ誤適用しないことを確認する。
- 期待結果: 次段階Collectionは順序位置から独立した項目IDを保持し、削除済みIDを別項目へ再利用しないこと。Controlが編集対象を識別する際も位置変化で別項目へ取り違えないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-051"></a>
## RQ-09-051

- 根拠: [RQ-09-051](../11_requirements/RQ-09_ui-scope.md#rq-09-051)
- case: [TC-SEL-01](requirement-test-matrix.md#case-tc-sel-01)
- 前提・操作・観測: 並べ替え後の選択集合/公開順、Singleへ複数ID設定拒否、削除後selected/currentを確認する。
- 期待結果: 次段階Selectionの公開順は現在のCollection順に従い、並べ替えで選択ID集合を変えないこと。Singleを破る複数選択の更新は全体を拒否し、項目削除でselected/currentからその項目を除くこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-052"></a>
## RQ-09-052

- 根拠: [RQ-09-052](../11_requirements/RQ-09_ui-scope.md#rq-09-052)
- case: [TC-COL-01](requirement-test-matrix.md#case-tc-col-01)、[TC-SEL-01](requirement-test-matrix.md#case-tc-sel-01)
- 前提・操作・観測: batch途中に不正operationを入れ、内容・revision・Selection・通知不変と、成功時の整合snapshotを確認する。
- 期待結果: 無効なchange batchはCollection内容・順序・revision・Selection・公開通知を変更しないこと。有効なbatchは一つの更新として公開し、中間の不整合を外部callbackから観測させないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 後続stage / 専用DD・詳細case待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-053"></a>
## RQ-09-053

- 根拠: [RQ-09-053](../11_requirements/RQ-09_ui-scope.md#rq-09-053)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)
- 前提・操作・観測: 対応RQの観測結果を個別assertionとして現行caseに照合する。初期版でUTF-8のLatin・日本語UI文字列を表示・編集でき、対応profileの文字・font依存条件を判断できること。
- 期待結果: 初期版でUTF-8のLatin・日本語UI文字列を表示・編集でき、対応profileの文字・font依存条件を判断できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-054"></a>
## RQ-09-054

- 根拠: [RQ-09-054](../11_requirements/RQ-09_ui-scope.md#rq-09-054)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 初期版の保証にChinese、Korean、RTL、emojiの表示・編集を含めないこと。対応を追加する場合は対象文字・profileを明示すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-055"></a>
## RQ-09-055

- 根拠: [RQ-09-055](../11_requirements/RQ-09_ui-scope.md#rq-09-055)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: Focus離脱では未確定文字列・caret・selectionを保持し、確定や明示取消として扱わないこと。OS候補一覧・学習状態・変換内部状態の復元は保証しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-056"></a>
## RQ-09-056

- 根拠: [RQ-09-056](../11_requirements/RQ-09_ui-scope.md#rq-09-056)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: commitとFocusLostが競合した場合はPlatformでの受信順に従い、終了済み入力sessionの遅延イベントを適用しないこと。Undo/Redoの追加保証は含めないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-057"></a>
## RQ-09-057

- 根拠: [RQ-09-057](../11_requirements/RQ-09_ui-scope.md#rq-09-057)
- case: [TC-LAYOUT-01](requirement-test-matrix.md#case-tc-layout-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: StackPanelの省略時方向はVertical、Alignの省略値はStretchとすること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-058"></a>
## RQ-09-058

- 根拠: [RQ-09-058](../11_requirements/RQ-09_ui-scope.md#rq-09-058)
- case: [TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)、[TC-FOCUS-01](requirement-test-matrix.md#case-tc-focus-01)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 初期Control自身のenabled設定の省略値はtrueとすること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-059"></a>
## RQ-09-059

- 根拠: [RQ-09-059](../11_requirements/RQ-09_ui-scope.md#rq-09-059)
- case: [TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)、[TC-FOCUS-01](requirement-test-matrix.md#case-tc-focus-01)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 初期focusable値はTextBoxとButtonがtrue、Window・StackPanel・Textがfalseとすること。Window/StackPanelへの明示focusable=trueはRQ-09-070に従い拒否すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-060"></a>
## RQ-09-060

- 根拠: [RQ-09-060](../11_requirements/RQ-09_ui-scope.md#rq-09-060)
- case: [TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 初期共通ControlにVisible/Hidden/Collapsedの切替機能を必須提供しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-09-061"></a>
## RQ-09-061

- 根拠: [RQ-09-061](../11_requirements/RQ-09_ui-scope.md#rq-09-061)
- case: [TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: 初期Buttonの必須起動機能はCommandとし、直接Click callbackの提供を必須としないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-001"></a>
## RQ-10-001

- 根拠: [RQ-10-001](../11_requirements/RQ-10_accessibility.md#rq-10-001)
- case: [TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 初期Controlを構成し、Core側で意味情報を読み取り、OSのAccessibility機構から対応する対象が利用可能であることを確認する。
- 期待結果: Window、StackPanel、Text、TextBox、Buttonについて、Coreで定義する意味情報をPlatform経由で利用可能にする。Accessibilityは初期基盤に含める。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-002"></a>
## RQ-10-002

- 根拠: [RQ-10-002](../11_requirements/RQ-10_accessibility.md#rq-10-002)
- case: [TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: Core上のControl種別とOSから取得したroleを比較し、TextBoxが編集可能な文字列Control、Buttonが起動可能なControlなど表の意味と合うことを確認する。
- 期待結果: 各Controlのroleは上表の意味を表す。OS側のroleが異なる語彙を使う場合も、利用者に同じControl種別を示す。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-003"></a>
## RQ-10-003

- 根拠: [RQ-10-003](../11_requirements/RQ-10_accessibility.md#rq-10-003)
- case: [TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: nameを設定・取得し、設定値との一致を確認する。未設定時に特定のfallback文字列を要求しない。
- 期待結果: Controlは設定されたAccessibility nameを保持し、対応する対象の意味情報から同じnameを取得できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-004"></a>
## RQ-10-004

- 根拠: [RQ-10-004](../11_requirements/RQ-10_accessibility.md#rq-10-004)
- case: [TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 状態を変更した前後でCoreの状態とAccessibility情報を比較し、変更が反映されることを確認する。未定義の細分化状態は要求しない。
- 期待結果: Controlの意味上のstateは上表に示す範囲で取得できる。TextBoxではvalidation状態を含める。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-005"></a>
## RQ-10-005

- 根拠: [RQ-10-005](../11_requirements/RQ-10_accessibility.md#rq-10-005)
- case: [TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: TextおよびTextBoxの値を変更し、Coreの現在値とAccessibility取得値が一致することを確認する。値のないControlに値を捏造しない。
- 期待結果: valueが意味を持つControlでは上表の値を公開する。Textは表示文字列、TextBoxは編集中の文字列を表す。ButtonやContainerに値を要求しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-006"></a>
## RQ-10-006

- 根拠: [RQ-10-006](../11_requirements/RQ-10_accessibility.md#rq-10-006)
- case: [TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: focusを移動・解除し、Coreの現在対象とOS Accessibility情報のfocus対象を比較する。
- 期待結果: Accessibility上のfocus状態はCoreが管理するactive focus対象と一致する。focus対象でないControlをfocus中として公開しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-007"></a>
## RQ-10-007

- 根拠: [RQ-10-007](../11_requirements/RQ-10_accessibility.md#rq-10-007)
- case: [TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)、[TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 支援技術から公開actionを実行し、通常の入力・Command経路と同じ結果になることを確認する。未対応actionを公開しない。
- 期待結果: Accessibility actionはControlが実際にサポートする操作に対応する。TextBoxの編集操作とButtonの起動は通常のControl処理経路を通る。特定キーとactionの対応表は要求しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-008"></a>
## RQ-10-008

- 根拠: [RQ-10-008](../11_requirements/RQ-10_accessibility.md#rq-10-008)
- case: [TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: enabled状態を切り替え、取得状態が一致すること、disabled時に操作要求がControl動作やCommandを起こさないことを確認する。
- 期待結果: enabled / disabled状態を公開し、disabled Controlへの操作要求が実行されないことを反映する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-009"></a>
## RQ-10-009

- 根拠: [RQ-10-009](../11_requirements/RQ-10_accessibility.md#rq-10-009)
- case: [TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: OSのAccessibility機構で各初期Controlを検査し、取得情報・actionが上表およびCore snapshotと意味的に一致することを確認する。
- 期待結果: PlatformはCoreのrole、name、state、value、focus、supported actionの意味を、対象OSのAccessibility機構へ意味を保って写像する。個別OS API型・属性名は本要求では固定しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-010"></a>
## RQ-10-010

- 根拠: [RQ-10-010](../11_requirements/RQ-10_accessibility.md#rq-10-010)
- case: [TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-DOC-01](requirement-test-matrix.md#case-tc-doc-01)
- 前提・操作・観測: 参照した指針と採用した意味を区別でき、要求していないWeb固有動作や完全適合の主張が追加されていないことを確認する。
- 期待結果: W3C ARIA Authoring Practices Guideは参考資料として利用する。Web固有の慣習をdesktop・組み込みLinuxへ適用する範囲は本プロジェクトで判断し、完全なARIA適合を主張しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-011"></a>
## RQ-10-011

- 根拠: [RQ-10-011](../11_requirements/RQ-10_accessibility.md#rq-10-011)
- case: [TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: AT queryだけのFocus不変更、stale node action拒否、通常Control actionとの結果一致を確認する。
- 期待結果: 同じControlの生存中はAccessibility上の対象を安定して識別でき、終了済み対象へのquery/actionを新対象へ誤適用しないこと。参照するだけでkeyboard focusを変更せず、操作は対応する通常Control処理と同じ結果となること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-012"></a>
## RQ-10-012

- 根拠: [RQ-10-012](../11_requirements/RQ-10_accessibility.md#rq-10-012)
- case: [TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 表示文字列/nameを別値にし、空nameの公開結果を確認する。自動fallbackが起きないことを確認する。
- 期待結果: Accessibility nameを表示文字列から独立して設定でき、未設定または空欄の場合は表示値等から自動合成しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-10-013"></a>
## RQ-10-013

- 根拠: [RQ-10-013](../11_requirements/RQ-10_accessibility.md#rq-10-013)
- case: [TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: アプリが構成するLogical階層に従って初期Accessibilityの親子を公開し、template内のVisual部品は既定で独立対象として公開しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-001"></a>
## RQ-11-001

- 根拠: [RQ-11-001](../11_requirements/RQ-11_platform.md#rq-11-001)
- case: [TC-TARGET-ARMV7-01](requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-SUPPORT-01](requirement-test-matrix.md#case-tc-support-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 対象releaseのsupport matrixにLinux ARMv7のprofileとbuild/runtime/hardware検証状態が明記され、buildだけの結果を実機support済みとして扱っていないことを確認する。
- 期待結果: 組み込みLinuxを製品の正式対象に含め、Linux ARMv7向けtargetを用意する。正式support範囲はABI majorとtarget profileの組で示す。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-002"></a>
## RQ-11-002

- 根拠: [RQ-11-002](../11_requirements/RQ-11_platform.md#rq-11-002)
- case: [TC-TARGET-A64-01](requirement-test-matrix.md#case-tc-target-a64-01)、[TC-SUPPORT-01](requirement-test-matrix.md#case-tc-support-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: RQ-11-001と同じ区分でAArch64のprofileと各検証状態が個別に記録されることを確認する。
- 期待結果: 組み込みLinuxを製品の正式対象に含め、Linux AArch64向けtargetを用意する。正式support範囲はABI majorとtarget profileの組で示す。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-003"></a>
## RQ-11-003

- 根拠: [RQ-11-003](../11_requirements/RQ-11_platform.md#rq-11-003)
- case: [TC-TARGET-LNX-01](requirement-test-matrix.md#case-tc-target-lnx-01)、[TC-TARGET-A64-01](requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-ARMV7-01](requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TARGET-LATER-01](requirement-test-matrix.md#case-tc-target-later-01)、[TC-TARGET-WIN-01](requirement-test-matrix.md#case-tc-target-win-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: Linux x64 native build/test結果と、そこから生成したARM向けbuild/link結果が別profileの証跡として記録されることを確認する。
- 期待結果: Linux x64を初期開発・CI targetに含め、native build/testとARM cross-buildのhostとして利用する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-004"></a>
## RQ-11-004

- 根拠: [RQ-11-004](../11_requirements/RQ-11_platform.md#rq-11-004)
- case: [TC-TARGET-WIN-01](requirement-test-matrix.md#case-tc-target-win-01)、[TC-TARGET-A64-01](requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-ARMV7-01](requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TARGET-LATER-01](requirement-test-matrix.md#case-tc-target-later-01)、[TC-TARGET-LNX-01](requirement-test-matrix.md#case-tc-target-lnx-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: Windows x64 native buildと定められたsmoke確認の結果がtarget profileに紐付いて記録されることを確認する。
- 期待結果: Windows x64を初期開発・CI targetに含め、native buildと初期UI縦断確認を行えるようにする。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-005"></a>
## RQ-11-005

- 根拠: [RQ-11-005](../11_requirements/RQ-11_platform.md#rq-11-005)
- case: [TC-TARGET-LATER-01](requirement-test-matrix.md#case-tc-target-later-01)
- 前提・操作・観測: 候補が現行releaseの必須support対象として誤記されていないことを確認する。採用時は対象profileと検証範囲を要求変更で確定する。
- 期待結果: Windows ARM64を後続target候補として保持する。初期CI matrixへの追加や正式supportは本要求だけでは確定しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-006"></a>
## RQ-11-006

- 根拠: [RQ-11-006](../11_requirements/RQ-11_platform.md#rq-11-006)
- case: [TC-TARGET-LATER-01](requirement-test-matrix.md#case-tc-target-later-01)
- 前提・操作・観測: RQ-11-005と同様に、候補と正式support対象が区別されていることを確認する。
- 期待結果: macOS x64およびARM64を後続target候補として保持する。初期CI matrixへの追加や正式supportは本要求だけでは確定しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-007"></a>
## RQ-11-007

- 根拠: [RQ-11-007](../11_requirements/RQ-11_platform.md#rq-11-007)
- case: [TC-TARGET-LATER-01](requirement-test-matrix.md#case-tc-target-later-01)
- 前提・操作・観測: Linuxのtarget tripleやsupport表示をbare metal/MCUへ流用していないことを確認する。候補採用時には必要なruntime・OS前提を別途要求として定める。
- 期待結果: bare metalおよびMCUは通常のLinux targetと同一視せず、独立したplatform profile候補として扱う。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-008"></a>
## RQ-11-008

- 根拠: [RQ-11-008](../11_requirements/RQ-11_platform.md#rq-11-008)
- case: [TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 依存構造を確認し、OS固有描画依存がRenderer/backend側に位置すること、およびbackendを交換してもCoreのControl意味論を変更しない境界であることを示す。
- 期待結果: Controlの状態と意味は共通Coreを正本とし、特定OSの描画APIやnative control状態を正本としないこと。描画backendを交換してもCoreのControl意味を保てる構成とすること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-009"></a>
## RQ-11-009

- 根拠: [RQ-11-009](../11_requirements/RQ-11_platform.md#rq-11-009)
- case: [TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: Coreから特定OSのwindowing・device APIへの直接依存がないことと、Platform境界からCoreへ共通Runtime操作を渡せる構成を確認する。
- 期待結果: Window/surface、OS event loop、device接続等のplatform backendをRust Coreから分離する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-010"></a>
## RQ-11-010

- 根拠: [RQ-11-010](../11_requirements/RQ-11_platform.md#rq-11-010)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)
- 前提・操作・観測: 依存構造とtext処理境界を確認し、Coreの共通契約がOS固有text engine型を公開・保持しないことを示す。
- 期待結果: text engineをRust Coreから分離する。Coreの文字列・Text意味論を特定OSのtext engine型に依存させない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-011"></a>
## RQ-11-011

- 根拠: [RQ-11-011](../11_requirements/RQ-11_platform.md#rq-11-011)
- case: [TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)
- 前提・操作・観測: 依存構造を確認し、codec実装が差し替え可能な境界の外側にあり、Core共通契約へcodec固有型が漏れないことを示す。
- 期待結果: image codecをRust Coreから分離する。Coreの共通UI機能が特定codec実装に直接依存しない構成とする。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-012"></a>
## RQ-11-012

- 根拠: [RQ-11-012](../11_requirements/RQ-11_platform.md#rq-11-012)
- case: [TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)、[TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)
- 前提・操作・観測: OS/device固有event型がCoreの共通処理へ直接流入しない依存構造を確認する。backend固有情報が必要な箇所は境界内で扱われることを示す。
- 期待結果: OS/device由来の入力backendをRust Coreから分離する。入力は共通処理へ渡す境界を持つ。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-013"></a>
## RQ-11-013

- 根拠: [RQ-11-013](../11_requirements/RQ-11_platform.md#rq-11-013)
- case: [TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)
- 前提・操作・観測: Core意味情報の正本がOS固有APIに依存せず、OS接続がbackend境界に置かれる依存構造を確認する。
- 期待結果: Accessibility backendをRust Coreから分離する。Core意味情報をOS固有Accessibility機構へ接続する責務をbackendに置く。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-014"></a>
## RQ-11-014

- 根拠: [RQ-11-014](../11_requirements/RQ-11_platform.md#rq-11-014)
- case: [TC-ARM-BACKEND-01](requirement-test-matrix.md#case-tc-arm-backend-01)
- 前提・操作・観測: 対象profileで利用可否・検証結果を記録し、次候補へ進む際に未実施理由または結果が残っていることを確認する。
- 期待結果: DRM/KMS + EGL/OpenGL ESを最初に検証する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-015"></a>
## RQ-11-015

- 根拠: [RQ-11-015](../11_requirements/RQ-11_platform.md#rq-11-015)
- case: [TC-ARM-BACKEND-02](requirement-test-matrix.md#case-tc-arm-backend-02)
- 前提・操作・観測: 順序と対象profileに沿った採否・検証結果を記録する。
- 期待結果: Waylandを第2候補として検証する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-016"></a>
## RQ-11-016

- 根拠: [RQ-11-016](../11_requirements/RQ-11_platform.md#rq-11-016)
- case: [TC-ARM-BACKEND-03](requirement-test-matrix.md#case-tc-arm-backend-03)
- 前提・操作・観測: 順序と対象profileに沿った採否・検証結果を記録する。
- 期待結果: X11を第3候補として検証する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-017"></a>
## RQ-11-017

- 根拠: [RQ-11-017](../11_requirements/RQ-11_platform.md#rq-11-017)
- case: [TC-ARM-BACKEND-04](requirement-test-matrix.md#case-tc-arm-backend-04)
- 前提・操作・観測: 順序と対象profileに沿った採否・検証結果を記録する。
- 期待結果: framebufferを第4候補として検証する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-018"></a>
## RQ-11-018

- 根拠: [RQ-11-018](../11_requirements/RQ-11_platform.md#rq-11-018)
- case: [TC-ARM-BACKEND-05](requirement-test-matrix.md#case-tc-arm-backend-05)
- 前提・操作・観測: 順序と対象profileに沿った採否・検証結果を記録する。
- 期待結果: software rendererを第5候補として検証する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-021"></a>
## RQ-11-021

- 根拠: [RQ-11-021](../11_requirements/RQ-11_platform.md#rq-11-021)
- case: [TC-TARGET-ARMV7-01](requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-A64-01](requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)
- 前提・操作・観測: ARMv7/AArch64 cross-build結果と実機profile結果を別集計し、他profileへhardware合格が転記されないことを確認する。
- 期待結果: build成功、Runtime動作確認、hardware profile確認の範囲を区別して公表すること。未検証profileや他architectureへ実機supportを拡大表示しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-020"></a>
## RQ-11-020

- 根拠: [RQ-11-020](../11_requirements/RQ-11_platform.md#rq-11-020)
- case: [TC-SUPPORT-01](requirement-test-matrix.md#case-tc-support-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)
- 前提・操作・観測: surface/device failureとOS close requestを注入し、Core値保持とRuntime状態を別観測する。
- 期待結果: 表示資源の喪失でCoreの確定済みUI状態を失わず、失敗した描画を成功として扱わないこと。OSのWindow close要求だけでRuntime全体を暗黙終了しないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-11-019"></a>
## RQ-11-019

- 根拠: [RQ-11-019](../11_requirements/RQ-11_platform.md#rq-11-019)
- case: [TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)
- 前提・操作・観測: 依存関係とデータ境界を点検し、列挙した実機依存機能の処理がPlatform側に隔離され、Coreへ渡る値が共通契約で扱えることを確認する。profileごとの差異は対象profileの検証記録に残す。
- 期待結果: GPU、input device、IME、display rotation、power management、window systemなどの実機依存要素はPlatform Adapterに閉じ込める。Coreの共通意味論や共通状態が特定実機・driverの表現を正本として持たないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-001"></a>
## RQ-12-001

- 根拠: [RQ-12-001](../11_requirements/RQ-12_build-distribution.md#rq-12-001)
- case: [TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)、[TC-TARGET-A64-01](requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-ARMV7-01](requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TARGET-LATER-01](requirement-test-matrix.md#case-tc-target-later-01)、[TC-TARGET-LNX-01](requirement-test-matrix.md#case-tc-target-lnx-01)、[TC-TARGET-WIN-01](requirement-test-matrix.md#case-tc-target-win-01)
- 前提・操作・観測: 各対象targetのartifact metadataと実際のbinary architectureを照合する。consumerから取得・linkできることを確認し、Build/link成功とtarget上のRuntime成功を別々に記録する。
- 期待結果: target tripleごとにRust Coreを含むnative C ABI runtime artifactを生成する。cross-compile artifactは生成対象のtargetを明示し、別targetのartifactを流用・誤表示しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-002"></a>
## RQ-12-002

- 根拠: [RQ-12-002](../11_requirements/RQ-12_build-distribution.md#rq-12-002)
- case: [TC-TARGET-A64-01](requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-ARMV7-01](requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TARGET-LATER-01](requirement-test-matrix.md#case-tc-target-later-01)、[TC-TARGET-LNX-01](requirement-test-matrix.md#case-tc-target-lnx-01)、[TC-TARGET-WIN-01](requirement-test-matrix.md#case-tc-target-win-01)
- 前提・操作・観測: 同一OS内のarchitecture違いとOS違いを区別してartifactを取得・linkできること、およびサポート表示が検証済みprofileの範囲と一致することを確認する。
- 期待結果: OSおよびCPU architectureごとに対応するnative libraryを提供する。ABI majorとtarget profileごとの正式support範囲を識別可能にする。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-003"></a>
## RQ-12-003

- 根拠: [RQ-12-003](../11_requirements/RQ-12_build-distribution.md#rq-12-003)
- case: [TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)
- 前提・操作・観測: 各言語consumerが同じtargetの共通runtime identityを利用し、同じABI majorとCore状態へ接続することをpackage設定・実行結果で確認する。
- 期待結果: 同一target向けのC ABI native runtimeをC++、C#、Rust、Python wrapperから共有する。各wrapperが独自にCoreを複製したり、異なるABI runtimeを暗黙に使ったりしない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-004"></a>
## RQ-12-004

- 根拠: [RQ-12-004](../11_requirements/RQ-12_build-distribution.md#rq-12-004)
- case: [TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-LICENSE-01](requirement-test-matrix.md#case-tc-license-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)
- 前提・操作・観測: install/export後の独立consumer projectがconfig packageを検索し、公開wrapperをcompile/linkして同梱対象のnative runtimeを解決できることを確認する。repository内build pathへの依存を要求しない。
- 期待結果: C++の標準integrationをCMake install/export packageとする。public C++ wrapperと対応targetの共通C ABI native libraryを含み、downstream projectは`find_package(... CONFIG)`で利用する。初期targetはWindows x64とLinux x64。vcpkg/Conan registryは本要求で指定しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-005"></a>
## RQ-12-005

- 根拠: [RQ-12-005](../11_requirements/RQ-12_build-distribution.md#rq-12-005)
- case: [TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-LICENSE-01](requirement-test-matrix.md#case-tc-license-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01)
- 前提・操作・観測: 両crateが期待する境界で配布され、safe API consumerが共通runtimeを利用できることを確認する。target/profile/ABIの不一致は検出され、適合しないartifactの利用前に明示的に失敗することを確認する。
- 期待結果: Rust配布をraw FFIの`-sys` crateとsafe API crateに分け、Rust Coreはwrapper crateへ含めず共通C ABI runtimeを別artifactとして使う。別環境生成artifactの取得・取込経路をfeatureで有効にし、Cargo `--target`でtarget triple、named artifact profileでboard/toolchain差を選択する。featureをtarget指定の代替にしない。registry、fetch command、cache、signature運用は本要求で指定しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-006"></a>
## RQ-12-006

- 根拠: [RQ-12-006](../11_requirements/RQ-12_build-distribution.md#rq-12-006)
- case: [TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-LICENSE-01](requirement-test-matrix.md#case-tc-license-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)
- 前提・操作・観測: NuGet packageの構成が指定RIDに対応し、各RID consumerが対応native libraryを解決・loadできることを確認する。未採用ARM profileを配布済みsupportとして表示しない。
- 期待結果: C# managed wrapperとRID別native C ABI libraryを同一NuGet packageの`runtimes/{rid}/native/`に含める。初期RIDは`win-x64`と`linux-x64`。組み込みARM RID packageはprofile互換性確認後に判断する。feed/package IDの実配置と、RQ-03-014に定めるTFM基準を区別する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-007"></a>
## RQ-12-007

- 根拠: [RQ-12-007](../11_requirements/RQ-12_build-distribution.md#rq-12-007)
- case: [TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: pure-Python wrapperが配布され、対象環境で追加のextension compileなしに共通C ABI runtimeをloadして利用開始できることを確認する。
- 期待結果: 初期Python wrapperはPython標準ライブラリ`ctypes`でC ABIを呼び出し、compiled extensionを必須としない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-008"></a>
## RQ-12-008

- 根拠: [RQ-12-008](../11_requirements/RQ-12_build-distribution.md#rq-12-008)
- case: [TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-LICENSE-01](requirement-test-matrix.md#case-tc-license-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)、[TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)
- 前提・操作・観測: wrapper distributionとnative artifactの内容・責務が分離され、対象profileに合うruntimeを組み合わせてloadできることを確認する。未確認ARM profileが互換wheelとして誤表示されないことを確認する。
- 期待結果: Python wrapper distributionとtarget別native runtime distributionを分ける。初期native wheel対象はWindows x64とLinux x64。組み込みLinux ARMはprofile互換性が確認されるまでmanylinux/musllinux wheelと表示しない。package名、index、exact wheel tagは本要求で指定しない。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-009"></a>
## RQ-12-009

- 根拠: [RQ-12-009](../11_requirements/RQ-12_build-distribution.md#rq-12-009)
- case: [TC-TARGET-A64-01](requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-ARMV7-01](requirement-test-matrix.md#case-tc-target-armv7-01)
- 前提・操作・観測: CI設定またはbuild provenanceから各設定の版・選択値を特定でき、targetごとのcompile/link結果に対応付けられることを確認する。具体的な版番号は対象profile確定時に管理する。
- 期待結果: CIで使用するYocto SDK、sysroot、linker、C library設定を固定し、target build条件を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-010"></a>
## RQ-12-010

- 根拠: [RQ-12-010](../11_requirements/RQ-12_build-distribution.md#rq-12-010)
- case: [TC-TARGET-A64-01](requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-ARMV7-01](requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TARGET-LATER-01](requirement-test-matrix.md#case-tc-target-later-01)、[TC-TARGET-LNX-01](requirement-test-matrix.md#case-tc-target-lnx-01)、[TC-TARGET-WIN-01](requirement-test-matrix.md#case-tc-target-win-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: targetの追加順と各stageの完了記録を確認する。ARMではbuild/link結果のみをRuntime verifiedと表示せず、実機確認は対象board/profileに限定して記録する。
- 期待結果: 初期CI targetをWindows x64、Linux x64、組み込みLinux ARMv7/AArch64の順で追加する。Linux x64 hostからARM向けをcross-compileする。ARM cross-build/link gateと代表実機上のRuntime/hardware profile gateを区別する。exact compiler/sysrootはbuild profileに記録し、初期profileと代表boardはRQ-11-024に従う。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-011"></a>
## RQ-12-011

- 根拠: [RQ-12-011](../11_requirements/RQ-12_build-distribution.md#rq-12-011)
- case: [TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: target/profile/ABI不一致、同identity異digest、cache miss/offline/明示更新を確認する。
- 期待結果: target/profile/ABI/featureに適合するnative artifactを選べ、不適合を利用前に拒否すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-012"></a>
## RQ-12-012

- 根拠: [RQ-12-012](../11_requirements/RQ-12_build-distribution.md#rq-12-012)
- case: [TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: target/profile/ABI不一致、同identity異digest、cache miss/offline/明示更新を確認する。
- 期待結果: 同じnative binaryは言語package間で同じidentityを保つこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-013"></a>
## RQ-12-013

- 根拠: [RQ-12-013](../11_requirements/RQ-12_build-distribution.md#rq-12-013)
- case: [TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: target/profile/ABI不一致、同identity異digest、cache miss/offline/明示更新を確認する。
- 期待結果: 通常導入で固定版を暗黙更新せず、取得・更新を明示でき、[期限・失効条件](../11_requirements/RQ-12_build-distribution.md#rq-12-023)を満たす検証済みartifactをoffline利用できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-014"></a>
## RQ-12-014

- 根拠: [RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)
- case: [TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 改ざんpayload/不正署名/未知鍵/失効鍵/独立trust bundleを検証し、拒否時にlink/loadへ渡らないことを確認する。
- 期待結果: 署名、内容digest、適合条件と既存の信頼根拠を検証してから公式artifactを利用すること。取得先で初めて得た鍵を自動信頼せず、未知・失効・検証不能なら拒否すること。認証済み鍵更新と明示trust bundleによるoffline検証を提供すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 部分/詳細待ち / profile・interface・parameter具体化。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-015"></a>
## RQ-12-015

- 根拠: [RQ-12-015](../11_requirements/RQ-12_build-distribution.md#rq-12-015)
- case: [TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: duplicate key/unknown major/critical field/optional fieldを分けて検査する。parser内部上限はSD設計根拠でunit検証する。
- 期待結果: 未知schema major、未知必須項目、重複・曖昧で一意に解釈できないmanifestを拒否すること。未知optional項目は署名検証後に無視できるが、target・ABI・信頼判定に必要な情報をoptionalとして扱わないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: 部分/詳細待ち / profile・interface・parameter具体化。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-016"></a>
## RQ-12-016

- 根拠: [RQ-12-016](../11_requirements/RQ-12_build-distribution.md#rq-12-016)
- case: [TC-LICENSE-01](requirement-test-matrix.md#case-tc-license-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: source/binaryと言語packageのinventoryをlicense/noticeの収録物へ照合する。
- 期待結果: first-party source、header、wrapper、source package、binary SDKへ既承認のApache-2.0方針を適用し、再配布で必要なlicense/noticeと変更表示を保持すること。third-partyのlicense条件を独立して維持すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-017"></a>
## RQ-12-017

- 根拠: [RQ-12-017](../11_requirements/RQ-12_build-distribution.md#rq-12-017)
- case: [TC-SUPPORT-01](requirement-test-matrix.md#case-tc-support-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: major公開日・限定保守期限・profile告知・security短縮・EOL公開記録を運用証拠で照合する。
- 期待結果: 最新安定ABI majorを通常保守し、直前majorは新安定major後24か月、security/重大障害修正に限って保守すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-018"></a>
## RQ-12-018

- 根拠: [RQ-12-018](../11_requirements/RQ-12_build-distribution.md#rq-12-018)
- case: [TC-SUPPORT-01](requirement-test-matrix.md#case-tc-support-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: major公開日・限定保守期限・profile告知・security短縮・EOL公開記録を運用証拠で照合する。
- 期待結果: ABI majorとtarget profileごとに保守状態を公表すること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-019"></a>
## RQ-12-019

- 根拠: [RQ-12-019](../11_requirements/RQ-12_build-distribution.md#rq-12-019)
- case: [TC-SUPPORT-01](requirement-test-matrix.md#case-tc-support-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: major公開日・限定保守期限・profile告知・security短縮・EOL公開記録を運用証拠で照合する。
- 期待結果: profile非推奨は原則12か月前に通知し、security上の短縮時は理由と影響を示すこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-020"></a>
## RQ-12-020

- 根拠: [RQ-12-020](../11_requirements/RQ-12_build-distribution.md#rq-12-020)
- case: [TC-SUPPORT-01](requirement-test-matrix.md#case-tc-support-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: major公開日・限定保守期限・profile告知・security短縮・EOL公開記録を運用証拠で照合する。
- 期待結果: EOL後も最終公開物・support表・告知を参照できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-12-021"></a>
## RQ-12-021

- 根拠: [RQ-12-021](../11_requirements/RQ-12_build-distribution.md#rq-12-021)
- case: [TC-LICENSE-01](requirement-test-matrix.md#case-tc-license-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 要求の提供範囲・対象stageと観測可能な結果を、対応caseの観測記録へ分けて保存する。
- 期待結果: first-party source、改変source、改変binary、app binaryの再配布を許可すること。third-partyの個別license条件は独立して維持すること。repository公開、CLA/DCO、docs/assetsのlicense、SBOM運用の追加を含めないこと。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: case/assertion割当済・実装/実行未証明。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-001"></a>
## RQ-13-001

- 根拠: [RQ-13-001](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-001)
- case: [TC-NFR-01](requirement-test-matrix.md#case-tc-nfr-01)
- 前提・操作・観測: 対象boardで値が得られ、起動区間の定義と実測結果が一緒に追跡できることを確認する。閾値未承認の値を合否保証に使わない。
- 期待結果: 起動時間を実測し、どの起動区間を測ったか、時間単位、board/profile、機能負荷と結果を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-002"></a>
## RQ-13-002

- 根拠: [RQ-13-002](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-002)
- case: [TC-NFR-02](requirement-test-matrix.md#case-tc-nfr-02)
- 前提・操作・観測: 対象artifactとsize定義を特定でき、得られた値を同じ対象条件の別結果と比較できることを確認する。
- 期待結果: バイナリサイズを実測し、計測対象artifact群、sizeの定義、target/profileと結果を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-003"></a>
## RQ-13-003

- 根拠: [RQ-13-003](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-003)
- case: [TC-NFR-03](requirement-test-matrix.md#case-tc-nfr-03)
- 前提・操作・観測: 測定値の対象範囲と取得条件を識別でき、異なる定義の値を同じ指標として混ぜていないことを確認する。
- 期待結果: メモリ使用量を実測し、対象process/範囲、測定方法、実行条件と結果を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-004"></a>
## RQ-13-004

- 根拠: [RQ-13-004](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-004)
- case: [TC-NFR-04](requirement-test-matrix.md#case-tc-nfr-04)
- 前提・操作・観測: 対象boardでFPSの結果が得られ、比較する結果ごとの描画負荷条件を追跡できることを確認する。
- 期待結果: 描画FPSを実測し、描画対象の機能・負荷条件、測定区間と結果を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-005"></a>
## RQ-13-005

- 根拠: [RQ-13-005](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-005)
- case: [TC-NFR-05](requirement-test-matrix.md#case-tc-nfr-05)
- 前提・操作・観測: 計測値がlayout処理に対応し、他層の処理を含む場合は含む範囲を識別できることを確認する。
- 期待結果: layout処理時間を実測し、対象layout処理の範囲、入力規模・条件と結果を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-006"></a>
## RQ-13-006

- 根拠: [RQ-13-006](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-006)
- case: [TC-NFR-06](requirement-test-matrix.md#case-tc-nfr-06)
- 前提・操作・観測: 測定対象がProperty更新であることと、比較する更新条件を識別できることを確認する。
- 期待結果: Property更新コストを実測し、対象Property・更新条件と観測値を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-007"></a>
## RQ-13-007

- 根拠: [RQ-13-007](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-007)
- case: [TC-NFR-07](requirement-test-matrix.md#case-tc-nfr-07)
- 前提・操作・観測: Collection差分処理に対応する値と、その処理対象の条件が区別して記録されることを確認する。
- 期待結果: Collection差分処理コストを実測し、対象Collection・差分条件と観測値を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-008"></a>
## RQ-13-008

- 根拠: [RQ-13-008](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-008)
- case: [TC-NFR-08](requirement-test-matrix.md#case-tc-nfr-08)
- 前提・操作・観測: callback/FFI境界の測定結果を他の処理時間から識別でき、測定条件が結果に紐付くことを確認する。
- 期待結果: callback / FFI crossingコストを実測し、呼出し境界、方向、対象callback条件と観測値を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-009"></a>
## RQ-13-009

- 根拠: [RQ-13-009](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-009)
- case: [TC-NFR-09](requirement-test-matrix.md#case-tc-nfr-09)
- 前提・操作・観測: 最大値の対象区間・負荷・集計定義を追跡でき、単一値だけが条件不明で報告されていないことを確認する。
- 期待結果: UI threadの最大処理時間を実測し、対象負荷、観測範囲、最大値の集計定義と結果を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-010"></a>
## RQ-13-010

- 根拠: [RQ-13-010](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-010)
- case: [TC-NFR-10](requirement-test-matrix.md#case-tc-nfr-10)
- 前提・操作・観測: 実機と実行条件に結び付いた安定性結果が記録され、cross-build成功のみで実機安定性を示した扱いにしていないことを確認する。
- 期待結果: 選定した実機profileで安定性を実測し、実行条件、観測期間、安定性の判定方法および異常の有無・内容を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-13-011"></a>
## RQ-13-011

- 根拠: [RQ-13-011](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-011)
- case: [TC-NFR-01](requirement-test-matrix.md#case-tc-nfr-01)、[TC-NFR-02](requirement-test-matrix.md#case-tc-nfr-02)、[TC-NFR-03](requirement-test-matrix.md#case-tc-nfr-03)、[TC-NFR-04](requirement-test-matrix.md#case-tc-nfr-04)、[TC-NFR-05](requirement-test-matrix.md#case-tc-nfr-05)、[TC-NFR-06](requirement-test-matrix.md#case-tc-nfr-06)、[TC-NFR-07](requirement-test-matrix.md#case-tc-nfr-07)、[TC-NFR-08](requirement-test-matrix.md#case-tc-nfr-08)、[TC-NFR-09](requirement-test-matrix.md#case-tc-nfr-09)、[TC-NFR-10](requirement-test-matrix.md#case-tc-nfr-10)
- 前提・操作・観測: 対応RQの観測結果を個別assertionとして現行caseに照合する。対象profile、機能負荷、測定区間、revisionと各処理層の測定結果を記録できること。
- 期待結果: 同一機能を用い、描画、layout、text、画像、GPU同期、FFI境界の各処理層を区別してprofileする。実装言語だけによる性能評価にしない。 対象profile、機能負荷、測定区間、revisionと各処理層の測定結果を記録できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-001"></a>
## RQ-14-001

- 根拠: [RQ-14-001](../11_requirements/RQ-14_delivery-sequence.md#rq-14-001)
- case: [TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)
- 前提・操作・観測: 承認済みABI仕様と、その承認記録が存在する。
- 期待結果: 実装開始前にC ABI仕様を確定する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-002"></a>
## RQ-14-002

- 根拠: [RQ-14-002](../11_requirements/RQ-14_delivery-sequence.md#rq-14-002)
- case: [TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-CALLBACK-01](requirement-test-matrix.md#case-tc-callback-01)、[TC-OWN-01](requirement-test-matrix.md#case-tc-own-01)、[TC-THREAD-01](requirement-test-matrix.md#case-tc-thread-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 各契約を含む承認済み仕様と承認記録が存在する。
- 期待結果: ABI versioning、error、ownership、thread規約を実装開始前に確定する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-003"></a>
## RQ-14-003

- 根拠: [RQ-14-003](../11_requirements/RQ-14_delivery-sequence.md#rq-14-003)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)、[TC-CORE-OBJ-01](requirement-test-matrix.md#case-tc-core-obj-01)、[TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-PROP-01](requirement-test-matrix.md#case-tc-prop-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 対象機能の実装revisionと、そのrevisionを検証した結果が記録される。
- 期待結果: RustでObject、Property、Binding、Commandを実装する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-004"></a>
## RQ-14-004

- 根拠: [RQ-14-004](../11_requirements/RQ-14_delivery-sequence.md#rq-14-004)
- case: [TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-WRAP-CPP-01](requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-WRAP-RS-01](requirement-test-matrix.md#case-tc-wrap-rs-01)
- 前提・操作・観測: 4言語それぞれのwrapper成果物と対応するビルド結果が確認できる。
- 期待結果: C++、C#、Rust、Python wrapperを作成する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-005"></a>
## RQ-14-005

- 根拠: [RQ-14-005](../11_requirements/RQ-14_delivery-sequence.md#rq-14-005)
- case: [TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)、[TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)、[TC-VERT-04](requirement-test-matrix.md#case-tc-vert-04)、[TC-VERT-05](requirement-test-matrix.md#case-tc-vert-05)、[TC-VERT-06](requirement-test-matrix.md#case-tc-vert-06)、[TC-VERT-07](requirement-test-matrix.md#case-tc-vert-07)、[TC-VERT-08](requirement-test-matrix.md#case-tc-vert-08)、[TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 言語ごとの対象revision、実行環境、結果が記録され、成功または未解決失敗が区別される。
- 期待結果: 4言語でMVVM smoke testを実行する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-006"></a>
## RQ-14-006

- 根拠: [RQ-14-006](../11_requirements/RQ-14_delivery-sequence.md#rq-14-006)
- case: [TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)、[TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)、[TC-VERT-04](requirement-test-matrix.md#case-tc-vert-04)、[TC-VERT-05](requirement-test-matrix.md#case-tc-vert-05)、[TC-VERT-06](requirement-test-matrix.md#case-tc-vert-06)、[TC-VERT-07](requirement-test-matrix.md#case-tc-vert-07)、[TC-VERT-08](requirement-test-matrix.md#case-tc-vert-08)
- 前提・操作・観測: 対象Controlの実装と各Controlの要求機能を確認した結果が記録される。
- 期待結果: Window、StackPanel、TextBox、Text、Buttonを追加する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-007"></a>
## RQ-14-007

- 根拠: [RQ-14-007](../11_requirements/RQ-14_delivery-sequence.md#rq-14-007)
- case: [TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-FOCUS-01](requirement-test-matrix.md#case-tc-focus-01)、[TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)、[TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 対象環境での入力確認結果と未対応条件が記録される。
- 期待結果: focus、keyboard、IMEの基礎機能を追加する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-008"></a>
## RQ-14-008

- 根拠: [RQ-14-008](../11_requirements/RQ-14_delivery-sequence.md#rq-14-008)
- case: [TC-ACC-01](requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](requirement-test-matrix.md#case-tc-acc-02)、[TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-ACC-03](requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 対象Controlのname/roleが利用者または検証ツールから確認できる結果が記録される。
- 期待結果: accessibility nameとroleを追加する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-009"></a>
## RQ-14-009

- 根拠: [RQ-14-009](../11_requirements/RQ-14_delivery-sequence.md#rq-14-009)
- case: [TC-COL-01](requirement-test-matrix.md#case-tc-col-01)、[TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-FEATURE-NEXT-13](requirement-test-matrix.md#case-tc-feature-next-13)、[TC-SEL-01](requirement-test-matrix.md#case-tc-sel-01)
- 前提・操作・観測: 各機能の実装revisionと対応する機能確認結果が記録される。
- 期待結果: Collection、Selection、Templateを追加する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-010"></a>
## RQ-14-010

- 根拠: [RQ-14-010](../11_requirements/RQ-14_delivery-sequence.md#rq-14-010)
- case: [TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 対象platformで描画が行われた結果と使用した実行環境が記録される。
- 期待結果: rendererとplatform adapterを実装する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-011"></a>
## RQ-14-011

- 根拠: [RQ-14-011](../11_requirements/RQ-14_delivery-sequence.md#rq-14-011)
- case: [TC-ABI-COMP-01](requirement-test-matrix.md#case-tc-abi-comp-01)、[TC-ABI-COMP-02](requirement-test-matrix.md#case-tc-abi-comp-02)、[TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)
- 前提・操作・観測: CI実行で互換性検査が起動し、検出対象の互換・非互換例の結果が確認できる。
- 期待結果: ABI compatibility testをCIに追加する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-012"></a>
## RQ-14-012

- 根拠: [RQ-14-012](../11_requirements/RQ-14_delivery-sequence.md#rq-14-012)
- case: [TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-NFR-01](requirement-test-matrix.md#case-tc-nfr-01)、[TC-NFR-02](requirement-test-matrix.md#case-tc-nfr-02)、[TC-NFR-03](requirement-test-matrix.md#case-tc-nfr-03)、[TC-NFR-04](requirement-test-matrix.md#case-tc-nfr-04)、[TC-NFR-05](requirement-test-matrix.md#case-tc-nfr-05)、[TC-NFR-06](requirement-test-matrix.md#case-tc-nfr-06)、[TC-NFR-07](requirement-test-matrix.md#case-tc-nfr-07)、[TC-NFR-08](requirement-test-matrix.md#case-tc-nfr-08)、[TC-NFR-09](requirement-test-matrix.md#case-tc-nfr-09)、[TC-NFR-10](requirement-test-matrix.md#case-tc-nfr-10)、[TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 測定対象、実機・構成、条件、結果、測定時のrevisionが記録される。
- 期待結果: 実機で性能を測定する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-14-013"></a>
## RQ-14-013

- 根拠: [RQ-14-013](../11_requirements/RQ-14_delivery-sequence.md#rq-14-013)
- case: [TC-DEV-01](requirement-test-matrix.md#case-tc-dev-01)、[TC-NFR-01](requirement-test-matrix.md#case-tc-nfr-01)、[TC-NFR-02](requirement-test-matrix.md#case-tc-nfr-02)、[TC-NFR-03](requirement-test-matrix.md#case-tc-nfr-03)、[TC-NFR-04](requirement-test-matrix.md#case-tc-nfr-04)、[TC-NFR-05](requirement-test-matrix.md#case-tc-nfr-05)、[TC-NFR-06](requirement-test-matrix.md#case-tc-nfr-06)、[TC-NFR-07](requirement-test-matrix.md#case-tc-nfr-07)、[TC-NFR-08](requirement-test-matrix.md#case-tc-nfr-08)、[TC-NFR-09](requirement-test-matrix.md#case-tc-nfr-09)、[TC-NFR-10](requirement-test-matrix.md#case-tc-nfr-10)
- 前提・操作・観測: 追加した場合は根拠となる測定・課題と再測定結果を記録し、追加しない場合は不要と判断した根拠を記録する。
- 期待結果: 性能測定等の根拠から必要と認めた箇所に限りbatch APIまたは最適化を追加する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-16-001"></a>
## RQ-16-001

- 根拠: [RQ-16-001](../11_requirements/RQ-16_document-governance.md#rq-16-001)
- case: [TC-DOC-01](requirement-test-matrix.md#case-tc-doc-01)
- 前提・操作・観測: 決定内容と適用範囲、決定状態が対象ADR本文から確認でき、草案とAcceptedを識別できる。
- 期待結果: ADRはアーキテクチャ上の決定、決定理由、適用範囲、状態を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-16-002"></a>
## RQ-16-002

- 根拠: [RQ-16-002](../11_requirements/RQ-16_document-governance.md#rq-16-002)
- case: [TC-DOC-01](requirement-test-matrix.md#case-tc-doc-01)
- 前提・操作・観測: 各調査結論の根拠と比較内容、未決事項を資料から追跡でき、調査結果だけで承認済み要求・設計判断を置き換えていない。
- 期待結果: 調査資料は外部根拠、比較対象、代替案、未決事項を記録する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-16-003"></a>
## RQ-16-003

- 根拠: [RQ-16-003](../11_requirements/RQ-16_document-governance.md#rq-16-003)
- case: [TC-DOC-01](requirement-test-matrix.md#case-tc-doc-01)、[TC-TRACE-01](requirement-test-matrix.md#case-tc-trace-01)
- 前提・操作・観測: 対応RQの観測結果を個別assertionとして現行caseに照合する。要求の変更承認、設計の変更承認、case設計、実装・実行状態を別々に追跡し、旧IDと新しい意味単位の対応を確認できること。
- 期待結果: 要求仕様書を承認済み要求の正本とする。要求変更は要求側で対象範囲と受入条件をレビュー・承認し、BD、API具体化を含むDD、SD、要求test、softwareおよび対応unit-testへの影響を追跡する。要求の承認は要求仕様書側で完結し、基本設計の承認を前提にしない。各文書には承認履歴表と改訂履歴表を設け、承認履歴に日付・対象版またはID・承認者・結論・根拠を、改訂履歴に版・日付・変更内容/理由・作成者・承認履歴との対応を残す。記録は追記し、過去履歴を上書きしない。 要求の変更承認、設計の変更承認、case設計、実装・実行状態を別々に追跡し、旧IDと新しい意味単位の対応を確認できること。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-16-004"></a>
## RQ-16-004

- 根拠: [RQ-16-004](../11_requirements/RQ-16_document-governance.md#rq-16-004)
- case: [TC-DOC-01](requirement-test-matrix.md#case-tc-doc-01)
- 前提・操作・観測: 不整合の記録から差異、変更対象、承認者、影響先を確認できる。要求またはADR/設計の正式変更前に下流文書だけで結論を上書きせず、解決前の設計・実装がAccepted ADRの制約を破っていない。
- 期待結果: 要求、ADR、設計に不整合がある場合は、それぞれの承認範囲と差異、影響を記録し、要求変更またはADR/設計変更として明示的に整合する。解決までは適用中のAccepted ADRの制約を維持し、影響する設計・実装を保留する。要求を下流設計に合わせて暗黙に変更しない。Accepted ADRの制約を変える場合は、そのADR変更を別途承認する。
- 後始末: 登録・接続・所有参照を明示解除/解放し、終了検証では残存参照の解放まで記録する。process監査は使用記録を保存する。
- 状態: process/測定計画割当・実行記録待ち。実test ID・command・SHA・target/profile・結果・reportは未登録。

<a id="assert-rq-04-029"></a>
## RQ-04-029

- 根拠: [RQ-04-029](../11_requirements/RQ-04_abi-boundary.md#rq-04-029)
- case: [TC-NUM-01](requirement-test-matrix.md#case-tc-num-01)
- 前提・操作・観測: 単精度の有限境界・符号付きzero・infinity・NaNを拡張し、倍精度の表現可能/中間値/overflowをExactとNearestEvenで縮小する。値、符号、拒否と出力初期値を観測する。
- 期待結果: 単精度・倍精度を明示変換できること。精度一致必須または最近接偶数丸めを指定でき、有限値の範囲外を拒否すること。NaN payloadの保存は保証せず、符号付きzeroとinfinityの意味を保つこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-02-022"></a>
## RQ-02-022

- 根拠: [RQ-02-022](../11_requirements/RQ-02_core-features.md#rq-02-022)
- case: [TC-NUM-01](requirement-test-matrix.md#case-tc-num-01)
- 前提・操作・観測: 次段階converterへASCII符号・数字・小数点、整数境界、scale31と係数境界、指数/空白/桁区切り/不正UTF8を与え、parse/format往復と表現不能の拒否を確認する。
- 期待結果: 次段階でUTF8と整数・十進値の明示parse/formatを提供すること。初期の文法はlocale非依存の符号・数字・十進点とし、表現不能を黙って丸めないこと。Bindingの暗黙型変換は追加しないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・次段階・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-062"></a>
## RQ-09-062

- 根拠: [RQ-09-062](../11_requirements/RQ-09_ui-scope.md#rq-09-062)
- case: [TC-COL-01](requirement-test-matrix.md#case-tc-col-01)
- 前提・操作・観測: 五値型の列を持つModelから可視snapshotを取得し、Modelの後続変更・削除と所有参照破棄後も同snapshotのID/revision/内容が変わらないことを読む。
- 期待結果: 次段階Collectionの列に既存の五値種別を指定できること。可視範囲と安定ID・revisionを持つ一貫したsnapshotを保持し、元Modelの後続変更に影響されず読めること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・次段階・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-063"></a>
## RQ-09-063

- 根拠: [RQ-09-063](../11_requirements/RQ-09_ui-scope.md#rq-09-063)
- case: [TC-COL-01](requirement-test-matrix.md#case-tc-col-01)
- 前提・操作・観測: CollectionとSelectionを同batchで変更し通知順と同revisionを記録する。通知callbackで同期batchを試み拒否・非変更を確認し、post後の更新を区別する。
- 期待結果: 同じ一括更新の集合変更通知を選択変更通知より先に観測し、両方から同じ確定revisionを読めること。通知中の再入一括更新は拒否し、後続処理へ依頼できること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・次段階・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-064"></a>
## RQ-09-064

- 根拠: [RQ-09-064](../11_requirements/RQ-09_ui-scope.md#rq-09-064)
- case: [TC-SEL-01](requirement-test-matrix.md#case-tc-sel-01)
- 前提・操作・観測: 複数選択からSingleへ変更し全拒否を確認する。current項目を削除して自動補充がないこと、別Collectionの選択状態と混同しないことを観測する。
- 期待結果: 複数選択を保ったままSingleへ縮退する変更は拒否すること。削除後currentを自動補充せず、対象集合を変更する場合は別の選択状態として明示できること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・次段階・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-06-012"></a>
## RQ-06-012

- 根拠: [RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012)
- case: [TC-THREAD-01](requirement-test-matrix.md#case-tc-thread-01)、[TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)
- 前提・操作・観測: 非同期・callback同期・worker同期から終了を要求する。受付後host停止、query、復旧retry、観測owner破棄を競合させ、未完了理由とClosedを区別する。
- 期待結果: 受付済み終了要求の未完了・完了・host実行不可と理由をthreadに依存せず観測できること。観測参照の破棄で終了要求を取り消さず、停止から同UI hostが復旧した場合は明示再試行できること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-03-013"></a>
## RQ-03-013

- 根拠: [RQ-03-013](../11_requirements/RQ-03_language-support.md#rq-03-013)
- case: [TC-WRAP-CS-01](requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01)
- 前提・操作・観測: 実行中callback/ticketを残す配置でloaderのprocess保持を確認する。collectible/hot unloadを対応済み表示しないことと、通常cleanup経路の維持を検査する。
- 期待結果: 初期版の通常利用ではnative libraryをprocess終了まで保持し、active callback/ticketがある状態のhot unloadとcollectible managed unloadを提供対象に含めないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-07-017"></a>
## RQ-07-017

- 根拠: [RQ-07-017](../11_requirements/RQ-07_binding.md#rq-07-017)
- case: [TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)
- 前提・操作・観測: 保留開始後に閾値を短縮/延長/無効化し、最初の保留時刻基準で診断が変わること、値・未配信作業が捨てられないことを観測する。
- 期待結果: 診断閾値の変更は既存の保留連鎖にも適用し、最初に保留した時点からの経過に対して評価すること。無効化で連鎖の値や保留作業を捨てないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-06-013"></a>
## RQ-06-013

- 根拠: [RQ-06-013](../11_requirements/RQ-06_threading-dispatch.md#rq-06-013)
- case: [TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01)
- 前提・操作・観測: 進捗0/1/未定を報告し、範囲外/NaNと古いoperationを拒否する。中間sampleをまとめても終端結果が一度だけ反映されることを確認する。
- 期待結果: Commandの単一channelの進捗を明示報告・読取りできること。0～1の正規化値と未定を区別し、中間sampleの全保存は保証しないこと。終端結果を進捗の集約で失わないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-065"></a>
## RQ-09-065

- 根拠: [RQ-09-065](../11_requirements/RQ-09_ui-scope.md#rq-09-065)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)
- 前提・操作・観測: 日本語を含む編集文字列へ範囲置換・caret/selection・resetを行う。UTF8途中境界の指定を拒否し、navigationを有効にするprofileではUnicode16.0の採用表へ照合する。
- 期待結果: 文字列範囲の置換、selection/caretの明示変更、確定値へのresetを利用でき、無効な文字境界を指定する更新を拒否すること。文字単位navigationを採用する場合はUnicode16.0のgrapheme境界に従うこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-066"></a>
## RQ-09-066

- 根拠: [RQ-09-066](../11_requirements/RQ-09_ui-scope.md#rq-09-066)
- case: [TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)、[TC-BIND-01](requirement-test-matrix.md#case-tc-bind-01)
- 前提・操作・観測: Accept/Incomplete/Preedit/IME commitと確定Property拒否を起こし、Binding・Property・編集通知の順とsnapshot整合、Preeditの確定通知不発生を観測する。
- 期待結果: 同じ編集確定のBinding伝播とProperty通知の後に、本文・selection・composition・入力状態/理由が一致した編集snapshotを通知すること。Preeditでは確定値通知を発生させないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-067"></a>
## RQ-09-067

- 根拠: [RQ-09-067](../11_requirements/RQ-09_ui-scope.md#rq-09-067)
- case: [TC-LAYOUT-01](requirement-test-matrix.md#case-tc-layout-01)
- 前提・操作・観測: scale変更、隣接edge、祖先clipとhit-testを観測し、NaN/負寸法/計算overflowで旧公開配置が維持されることを確認する。
- 期待結果: 論理寸法とdevice scaleを区別でき、無効な数値や表現不能な配置結果を拒否して以前の公開配置を保つこと。初期Controlの描画・hit-testは祖先のclient範囲で切り取ること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-068"></a>
## RQ-09-068

- 根拠: [RQ-09-068](../11_requirements/RQ-09_ui-scope.md#rq-09-068)
- case: [TC-INPUT-01](requirement-test-matrix.md#case-tc-input-01)
- 前提・操作・観測: targetから複数Logical祖先へ配送し、handledで後続/既定動作が止まることを観測する。pointer追加候補が初期必須へ混入していないことを監査する。
- 期待結果: 入力は対象からLogical祖先へ扱え、handled指定で以後の配送と既定動作を止められること。pointer move/down/up/wheelと単一pointer captureを追加候補として区別すること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-069"></a>
## RQ-09-069

- 根拠: [RQ-09-069](../11_requirements/RQ-09_ui-scope.md#rq-09-069)
- case: [TC-TREE-01](requirement-test-matrix.md#case-tc-tree-01)
- 前提・操作・観測: Detached subtreeの再所属でpeer準備を失敗させ、構造・値・登録・Focus不変と明示retry成功後の一括所属を観測する。
- 期待結果: 切離し対象の表示資源再作成に失敗してもDetached構造・確定値・登録を保持し、明示再試行できること。所属だけを成功として公開しないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-070"></a>
## RQ-09-070

- 根拠: [RQ-09-070](../11_requirements/RQ-09_ui-scope.md#rq-09-070)
- case: [TC-FOCUS-01](requirement-test-matrix.md#case-tc-focus-01)
- 前提・操作・観測: Window/StackPanelのfocusable=trueを試み拒否・旧値維持を確認し、Window activeと子focusが独立であることを観測する。
- 期待結果: 初期版ではWindow/StackPanelの明示focusable=trueへの変更を受け付けないこと。Windowのactive状態は子のkeyboard focusと区別すること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-071"></a>
## RQ-09-071

- 根拠: [RQ-09-071](../11_requirements/RQ-09_ui-scope.md#rq-09-071)
- case: [TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)
- 前提・操作・観測: Window open/close/reopenとAccept/Decline/Deferを順に起こし、未解決receipt破棄・Runtime終了時veto不可・Core値保持を観測する。
- 期待結果: Windowの表示開始・終了を明示でき、終了後もCore状態を保持して再表示できること。close要求をAccept/Decline/Deferで判断でき、既定Accept、未解決判断の破棄はDeclineとすること。Runtime終了は個別vetoの対象にしないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-11-022"></a>
## RQ-11-022

- 根拠: [RQ-11-022](../11_requirements/RQ-11_platform.md#rq-11-022)
- case: [TC-PLAT-01](requirement-test-matrix.md#case-tc-plat-01)
- 前提・操作・観測: profile公開条件を照合し、device lossからの明示復旧と失敗を観測する。software backendへ暗黙切替がないことを確認する。
- 期待結果: 初期profileの描画backendと依存条件を公表すること。device喪失後の復旧可否を識別でき、software rendererへの暗黙切替を行わないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-072"></a>
## RQ-09-072

- 根拠: [RQ-09-072](../11_requirements/RQ-09_ui-scope.md#rq-09-072)
- case: [TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02)
- 前提・操作・観測: 単一paragraphとclient clipを観測し、soft-wrap/ellipsis/copyを初期必須と表示しないこと、Button labelとAccessibility nameが独立であることを確認する。
- 期待結果: 初期Textは単一paragraphの表示、client clipを提供し、soft-wrap/ellipsis/選択copyの追加を必須に含めないこと。Buttonの表示labelとAccessibility nameは独立すること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-11-023"></a>
## RQ-11-023

- 根拠: [RQ-11-023](../11_requirements/RQ-11_platform.md#rq-11-023)
- case: [TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-IME-01](requirement-test-matrix.md#case-tc-ime-01)
- 前提・操作・観測: 各profileのfont/IME/provider能力表と実行環境を照合する。desktop bus欠落とDRM IME broker欠落を未対応として識別し、実機未実施を成功扱いにしない。
- 期待結果: 各profileのfont/文字対応、日本語IMEの導入条件、Accessibility bus/provider有無を識別できること。desktop LinuxのAT-SPI profileはbusを必要とし、DRM profileはIME brokerを明示すること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-10-014"></a>
## RQ-10-014

- 根拠: [RQ-10-014](../11_requirements/RQ-10_accessibility.md#rq-10-014)
- case: [TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01)
- 前提・操作・観測: OS同期編集actionの待機を仮想時計/開始barrierで制御し、既定2秒、設定変更、未開始取消と開始済み結果不明を区別する。Invokeの非同期受付とは別assertionにする。
- 期待結果: OS同期編集actionの待機制限を設定・識別でき、初期既定を2秒とすること。未開始の取消と開始済み結果不明を区別し、timeoutをrollback成功として表示しないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-03-014"></a>
## RQ-03-014

- 根拠: [RQ-03-014](../11_requirements/RQ-03_language-support.md#rq-03-014)
- case: [TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01)
- 前提・操作・観測: 最低版と固定release版で別々にbuildし、各packageのminimum metadataを検査する。C# library sourceの言語版とconsumer sourceを区別し、未検証処理系をsupportと表示しない。
- 期待結果: 初期利用基準をC++17/CMake3.28、Rust1.85 edition2024、net10.0、CPython3.13とすること。C# wrapperのライブラリソースはC#14を基準とし、consumerのソース言語版と区別すること。正式supportの処理系/profileを公表し、free-threaded版とPyPy/subinterpreterのsupportを通常版から推定しないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-03-015"></a>
## RQ-03-015

- 根拠: [RQ-03-015](../11_requirements/RQ-03_language-support.md#rq-03-015)
- case: [TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)
- 前提・操作・観測: 管理entryのreturn・一般例外・SystemExitと登録host停止を起こし、協調条件下のcleanupとroot drainを観測する。未登録host/停止済みhostは未完了として返し、永久停止callbackは隔離processで保証外境界を確認する。
- 期待結果: 管理entryの通常return・一般例外・SystemExitと管理host停止でcleanupを要求できること。hostはUI thread終了前に停止通知を接続し、callback/finalizer/handlerの利用者処理は有限に戻り、終了処理を同期待ちするmainとの循環待ちを作らないこと。未登録host・停止不能な利用者処理・即時process終了でcleanup完遂を保証しないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-03-016"></a>
## RQ-03-016

- 根拠: [RQ-03-016](../11_requirements/RQ-03_language-support.md#rq-03-016)
- case: [TC-WRAP-PY-01](requirement-test-matrix.md#case-tc-wrap-py-01)
- 前提・操作・観測: 基底__init__未到達・途中失敗とclose/context/native entry overrideを試み、資源取得の前後で管理不能tokenが残らないことを確認する。一般演算子にsuperを強制しないことを検査する。
- 期待結果: 管理ownerの基底初期化未実行を検出し、資源取得の前後で明確に失敗できること。close/context/native entryの保護対象をoverride不可とし、任意演算子に一律base callを要求しないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-03-017"></a>
## RQ-03-017

- 根拠: [RQ-03-017](../11_requirements/RQ-03_language-support.md#rq-03-017)
- case: [TC-ANALYZER-01](requirement-test-matrix.md#case-tc-analyzer-01)
- 前提・操作・観測: 4言語でsignature/禁止効果/async不一致・再帰警告・動的Unknownのfixtureを解析する。同入力のCLI/CI/MCP結果とgenerationを比較し、未解析と診断0件を区別する。
- 期待結果: callback種別の明示annotationを各言語で利用でき、型・禁止効果・同期/非同期不一致を診断できること。禁止を証明できない経路はUnknownとし、警告とerrorを区別すること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-12-022"></a>
## RQ-12-022

- 根拠: [RQ-12-022](../11_requirements/RQ-12_build-distribution.md#rq-12-022)
- case: [TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)
- 前提・操作・観測: manifest bytes/depth/member/string/signatureの各既定上限の境界と超過を与え、構成変更前後の拒否・policy identity・非公開状態を確認する。
- 期待結果: manifestの資源制限を構成でき、初期既定値はbytes1 MiB/depth32/member4096/string64 KiB/signature8とすること。制限超過を明示拒否し、許容される新manifestへの変更とpolicy変更を追跡できること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-12-023"></a>
## RQ-12-023

- 根拠: [RQ-12-023](../11_requirements/RQ-12_build-distribution.md#rq-12-023)
- case: [TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01)
- 前提・操作・観測: root365日/manifest30日/revocation7日の期限内外とsequence rollbackを仮想時計で起こす。offline bundleでも期限超過を拒否し、未取得失効を検知済みとしないことを確認する。
- 期待結果: 明示trust bundleでも期限切れ・rollback・検証不能を拒否すること。期間をroot365日/manifest30日/revocation7日とし、長期offlineは別profileの承認済み期間を用いること。取得していない最新失効を即時検知すると保証しないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-11-024"></a>
## RQ-11-024

- 根拠: [RQ-11-024](../11_requirements/RQ-11_platform.md#rq-11-024)
- case: [TC-TARGET-WIN-01](requirement-test-matrix.md#case-tc-target-win-01)、[TC-TARGET-LNX-01](requirement-test-matrix.md#case-tc-target-lnx-01)、[TC-TARGET-ARMV7-01](requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-A64-01](requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01)
- 前提・操作・観測: 4つの初期profileと代表boardのmetadataを照合する。ARMv7 build証拠とAArch64実機証拠を別記録とし、同一結果の転記や未実施support表示を拒否する。
- 期待結果: 初期profileをWindows11 x64、Debian12 x64、ARMv7/AArch64 cross-buildと代表RPi4 AArch64とすること。初期ARMv7はbuild検証表示に限定し、実機未確認を明示すること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-04-030"></a>
## RQ-04-030

- 根拠: [RQ-04-030](../11_requirements/RQ-04_abi-boundary.md#rq-04-030)
- case: [TC-ABI-COMP-01](requirement-test-matrix.md#case-tc-abi-comp-01)
- 前提・操作・観測: 1.0.0/ABI1とrc.Nのversion出力、独立wrapperのrequired Runtime/ABI/featuresを確認し、不適合組合せを利用前に拒否する。
- 期待結果: 初回stableをRuntime1.0.0/ABI1とし、pre-releaseをrc.Nで識別できること。wrapper versionが独立でも必要Runtime/ABI/featureを判断できること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-12-024"></a>
## RQ-12-024

- 根拠: [RQ-12-024](../11_requirements/RQ-12_build-distribution.md#rq-12-024)
- case: [TC-SUPPORT-01](requirement-test-matrix.md#case-tc-support-01)
- 前提・操作・観測: EOL済みprofileの最終artifact/metadata/告知の参照可能性、内部logの保存期限と権限を運用記録で監査する。実際に期限未到達ならpolicyと削除job検証を分ける。
- 期待結果: EOL後も最終artifact・metadata・support告知を参照可能にすること。内部test logはEOL+24か月、security監査logは2年という期間で運用責任を定めること。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-12-025"></a>
## RQ-12-025

- 根拠: [RQ-12-025](../11_requirements/RQ-12_build-distribution.md#rq-12-025)
- case: [TC-ARCH-01](requirement-test-matrix.md#case-tc-arch-01)
- 前提・操作・観測: 4wrapper packageのlink/load経路とdigestを比較し、独立Coreの静的同梱がないことをartifact inspectionで確認する。
- 期待結果: 初期配布は共通native dynamic runtimeを利用し、各言語へ独立Coreを静的同梱するvariantを提供対象に含めないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・初期共通基盤・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

<a id="assert-rq-09-073"></a>
## RQ-09-073

- 根拠: [RQ-09-073](../11_requirements/RQ-09_ui-scope.md#rq-09-073)
- case: [TC-FEATURE-NEXT-10](requirement-test-matrix.md#case-tc-feature-next-10)、[TC-FEATURE-NEXT-11](requirement-test-matrix.md#case-tc-feature-next-11)、[TC-FEATURE-NEXT-12](requirement-test-matrix.md#case-tc-feature-next-12)、[TC-FEATURE-NEXT-13](requirement-test-matrix.md#case-tc-feature-next-13)
- 前提・操作・観測: 次段階のStyle再利用・Theme切替・local/祖先/Runtime参照とTemplate差替えを起こし、欠落/型不一致/cycle時の失敗とLogical identity維持を観測する。
- 期待結果: 次段階でtyped表示設定の再利用、共有外観切替、local→Logical祖先→Runtime themeのResource参照、Logical identityを保つ内部表示差替えを提供すること。未定義/型不一致/cycleを明示失敗とし、任意script実行を必須に含めないこと。
- 後始末: 作成したowner・登録・host・stagingを各契約で解放し、fault fixture/仮想時計/設定を復元する。保証外の永久停止は隔離processの終了を管理する。
- 状態: 観測設計済・次段階・未実装/未実行。実test ID・command・SHA・profile・reportは未登録。

## 適用前の受入確認文の移管記録

旧RQ-15の文は当時の確認文を保持する。現行期待結果は上記assertionとRQを優先する。

| 旧RQ/移管元 | 当時の確認文 |
| --- | --- |
| [RQ-01-001](../11_requirements/RQ-01_purpose-architecture.md#rq-01-001) | 4言語それぞれで同一の初期縦断シナリオを実行し、TextBoxからの値更新、Textへの値反映、ButtonによるCommand実行と実行可否、UI応答が同じ期待結果になることを確認する。 |
| [RQ-01-002](../11_requirements/RQ-01_purpose-architecture.md#rq-01-002) | 4言語のAPIが同じABIランタイムへ接続し、同じCore状態を読み書きしていることを構成と縦断シナリオで確認する。言語間で同じ操作を行ったとき、値・通知・Command状態の結果が一致することを確認する。 |
| [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001) | 作成・参照・所有者解放・明示破棄の各経路で、対象が有効な間は同じ対象を参照でき、寿命終了後は無効な参照として検出されることを確認する。 |
| [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002) | 値の設定・読出し・変更通知を確認し、型不一致の更新が既存値を壊さず拒否されること、および等値更新で変更通知が発生しないことを確認する。 |
| [RQ-02-003](../11_requirements/RQ-02_core-features.md#rq-02-003) | 各方向で値が期待先へ伝播すること、無効な接続が既存値を部分変更しないこと、検証失敗が識別可能であること、解除後に変更が伝播しないことを確認する。 |
| [RQ-02-004](../11_requirements/RQ-02_core-features.md#rq-02-004) | 実行可否に応じて実行の可否が変わり、成功・失敗・キャンセルを区別して観測できること、完了結果がUI側へ反映されることを確認する。 |
| [RQ-02-005](../11_requirements/RQ-02_core-features.md#rq-02-005) | 有効な変更後に項目と順序が期待どおりになり、並べ替え後も同一項目を識別できることを確認する。無効な一括変更時は集合状態が部分適用されないことを確認する。 |
| [RQ-02-006](../11_requirements/RQ-02_core-features.md#rq-02-006) | 単一・複数選択の各状態、選択されていない現在項目、項目削除後の選択・現在状態を確認し、フォーカス移動だけで選択状態が暗黙に変化しないことを確認する。 |
| [RQ-02-007](../11_requirements/RQ-02_core-features.md#rq-02-007) | Widgetを追加・削除した後、表示順・入力対象・意味上の階層が定義された結果となることを確認する。内部描画要素がアプリ構成上の独立Widgetとして誤って観測されないことを確認する。 |
| [RQ-02-008](../11_requirements/RQ-02_core-features.md#rq-02-008) | 初期Controlを含む画面で、サイズ制約や親領域の変更後に配置結果が更新され、重複・欠落なく表示されることを確認する。 |
| [RQ-02-009](../11_requirements/RQ-02_core-features.md#rq-02-009) | 入力イベントが適切なControlへ届き、TextBox値の編集・確定・拒否の結果が定義どおり観測できることを確認する。 |
| [RQ-02-010](../11_requirements/RQ-02_core-features.md#rq-02-010) | Focus取得・喪失時の状態がCoreとPlatformで一致し、キーボード・文字入力が現在の対象へ届けられることを確認する。 |
| [RQ-02-011](../11_requirements/RQ-02_core-features.md#rq-02-011) | 変換中は未確定文字列と編集位置が維持され、確定後は検証結果に応じた値が表示・伝播されることを確認する。Focus移動で未確定文字列が黙って失われないことを確認する。 |
| [RQ-02-012](../11_requirements/RQ-02_core-features.md#rq-02-012) | 正常・失敗・不正引数・終了済み対象等の結果を呼出し側が区別でき、ABI境界を越えて言語固有例外やpanicが伝播しないことを確認する。 |
| [RQ-02-013](../11_requirements/RQ-02_core-features.md#rq-02-013) | 通常破棄、親子からの切離し、callback解除、Runtime終了の各経路で、参照可能性と資源解放が規定どおりであることを確認する。 |
| [RQ-02-014](../11_requirements/RQ-02_core-features.md#rq-02-014) | UI threadからの操作が成功し、workerからの操作が所定の引渡し経路を通ること、直接操作の拒否または結果が既存状態を破壊しないことを確認する。 |
| [RQ-03-001](../11_requirements/RQ-03_language-support.md#rq-03-001) | C++の利用コードで縦断シナリオを実行し、Property・Binding・Command・UIの結果が共通の期待値になることを確認する。 |
| [RQ-03-002](../11_requirements/RQ-03_language-support.md#rq-03-002) | C#の利用コードで縦断シナリオを実行し、Property・Binding・Command・UIの結果が共通の期待値になることを確認する。 |
| [RQ-03-003](../11_requirements/RQ-03_language-support.md#rq-03-003) | Rustの利用コードで縦断シナリオを実行し、Property・Binding・Command・UIの結果が共通の期待値になることを確認する。 |
| [RQ-03-004](../11_requirements/RQ-03_language-support.md#rq-03-004) | Python 3の利用コードで縦断シナリオを実行し、Property・Binding・Command・UIの結果が共通の期待値になることを確認する。 |
| [RQ-04-001](../11_requirements/RQ-04_abi-boundary.md#rq-04-001) | ABI宣言と4言語の対応型を照合し、値の境界を含む往復試験で桁幅・符号の解釈が一致することを確認する。 |
| [RQ-04-002](../11_requirements/RQ-04_abi-boundary.md#rq-04-002) | 宣言型の対応関係と代表値の往復結果を4言語で照合し、浮動小数値として一貫して扱えることを確認する。 |
| [RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003) | ASCII・多byte文字・空文字列を各言語から渡し、指定長の範囲で同じ文字列が取得できることを確認する。 |
| [RQ-04-004](../11_requirements/RQ-04_abi-boundary.md#rq-04-004) | 公開宣言に内部objectの構造や実装pointerが現れず、4言語から同じhandle契約を利用できることを確認する。 |
| [RQ-04-005](../11_requirements/RQ-04_abi-boundary.md#rq-04-005) | 公開契約で保持・解放対象が識別され、保持・解放を伴う境界試験で早期解放や保持漏れが発生しないことを確認する。 |
| [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006) | 成功と失敗を含む関数呼出しでstatusが返り、4言語wrapperが同じstatusを解釈できることを確認する。 |
| [RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007) | 失敗時にerror objectを取得・解放でき、呼出し元言語で失敗として処理できることを確認する。詳細項目やcode catalogは本要求では固定しない。 |
| [RQ-04-008](../11_requirements/RQ-04_abi-boundary.md#rq-04-008) | 4言語からcallbackを登録して実行・解除し、userdataの有効期間とdestroyの呼出し責任が契約どおりになることを確認する。 |
| [RQ-04-009](../11_requirements/RQ-04_abi-boundary.md#rq-04-009) | ABI versionの問合せ結果が各言語で取得でき、runtimeが示すversionを利用側が識別できることを確認する。 |
| [RQ-04-010](../11_requirements/RQ-04_abi-boundary.md#rq-04-010) | 対応機能と未対応機能を照会し、結果が4言語で同じ意味に解釈されることを確認する。 |
| [RQ-04-011](../11_requirements/RQ-04_abi-boundary.md#rq-04-011) | 公開ヘッダ、exported symbol宣言、各言語のFFI宣言にRust固有型や借用参照が現れないことを確認する。 |
| [RQ-04-012](../11_requirements/RQ-04_abi-boundary.md#rq-04-012) | 公開ヘッダと各言語のFFI宣言がC ABIで扱える表現だけで構成され、C++ class・STL・RTTI・exception型をABI契約に含まないことを確認する。 |
| [RQ-04-013](../11_requirements/RQ-04_abi-boundary.md#rq-04-013) | 公開ABIと他言語のFFI宣言がPython object表現を参照しないことを確認する。 |
| [RQ-04-014](../11_requirements/RQ-04_abi-boundary.md#rq-04-014) | 公開ABIと他言語のFFI宣言がCLR object・delegate表現を参照しないことを確認する。 |
| [RQ-04-015](../11_requirements/RQ-04_abi-boundary.md#rq-04-015) | panicが発生した場合もABI呼出し元へunwindが伝播せず、境界で定めた失敗経路として観測されることを確認する。 |
| [RQ-04-016](../11_requirements/RQ-04_abi-boundary.md#rq-04-016) | C++ wrapper内で例外が発生した場合もABI呼出し元へ伝播せず、境界で定めた失敗経路として観測されることを確認する。 |
| [RQ-04-017](../11_requirements/RQ-04_abi-boundary.md#rq-04-017) | Python callback等で例外が発生した場合もC ABI呼出しへ伝播せず、境界で定めた失敗経路として観測されることを確認する。 |
| [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001) | UI thread上の操作は反映され、workerからの直接操作はthread違反となり値・通知を変えないことを確認する。明示的に任意threadを許す操作とUI操作が区別されることも確認する。 |
| [RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002) | workerからの要求がUI threadで一度だけ実行され、要求受付が処理完了を待たないことを確認する。受付失敗または停止時に未実行処理の所有者と破棄結果を識別できることを確認する。 |
| [RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003) | 競合する更新で破損・不整合が起きず、read-onlyまたはcopyとして渡した値が所有側の後続変更により途中で書き換わらないことを確認する。PythonではGILが排他制御として働かない条件を前提にしても共有状態の契約が保たれることを確認する。 |
| [RQ-06-004](../11_requirements/RQ-06_threading-dispatch.md#rq-06-004) | 成功・失敗・キャンセルの各経路と、キャンセル要求後に成功または失敗した場合を区別して観測する。実行中の重複起動規則、古い実行の通知、画面終了後の通知、Runtime停止時の処理を確認する。 |
| [RQ-07-001](../11_requirements/RQ-07_binding.md#rq-07-001) | 作成時に値が同期し、source・targetの後続変更が相手へ伝わらず、変更通知callbackがBindingを介して開始されないことを確認する。 |
| [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002) | 初期値とsource変更がtargetに反映され、targetの直接変更がsourceを変えず、その後のsource変更がtargetを更新することを確認する。 |
| [RQ-07-003](../11_requirements/RQ-07_binding.md#rq-07-003) | 初期同期のsource優先と両方向の変更伝播を確認し、両側の直接変更が同一処理中に起きたとき最後の成功値へ収束し、無限往復しないことを確認する。 |
| [RQ-07-004](../11_requirements/RQ-07_binding.md#rq-07-004) | 3 modeの方向表を4言語から同じ入力列で検証し、初期同期・source変更・target変更の伝播先が一致することを確認する。 |
| [RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005) | setter直後のsource getterと、callback終了前後のtarget getterを記録し、4言語で同じ時点に同じ値を観測することを確認する。 |
| [RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006) | 一つの処理区間で同じsourceを複数回変更し、targetが最終値になってから外部購読者へ通知されること、target通知は後続配信回となることを確認する。 |
| [RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007) | 循環・重複・競合する入力元を持つ接続作成が失敗し、失敗前のProperty値と既存Bindingの伝播結果が維持されることを確認する。 |
| [RQ-07-008](../11_requirements/RQ-07_binding.md#rq-07-008) | callbackからPropertyを更新し、callbackが再帰呼出しされず後続配信で値が反映されることを確認する。反復更新が作業枠を越える場合にUI処理へ制御を返すことも確認する。 |
| [RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009) | 失敗したsetter後も対象Propertyの確定値が維持されること、Aのsetter成功後にBのsetterが失敗してもAは新値、Bは旧値であることを確認する。 |
| [RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010) | 初期同期拒否時に作成失敗が返り、target値・接続状態が変化しないことを確認する。作成後の拒否時はsourceの確定値、targetの従前値、Bindingエラーを確認し、次の有効な伝播後に値が同期し現在エラーが消えることを確認する。 |
| [RQ-07-011](../11_requirements/RQ-07_binding.md#rq-07-011) | Binding handleのrelease後もsourceの変更がtargetへ伝播することを確認する。明示解除後は将来の変更と保留中の変更が伝播せず、解除時の各Property値が維持され、Binding固有のerror状態が終了することを確認する。 |
| [RQ-09-001](../11_requirements/RQ-09_ui-scope.md#rq-09-001) | Windowに初期シナリオのルートを設定し、表示内容がそのルートに従うことを確認する。 |
| [RQ-09-002](../11_requirements/RQ-09_ui-scope.md#rq-09-002) | 初期Controlが共通Widgetとして扱われ、共通操作・状態の読み取りができることを確認する。 |
| [RQ-09-003](../11_requirements/RQ-09_ui-scope.md#rq-09-003) | 初期シナリオでWindowとStackPanelの親子関係を確認する。子数や追加・削除APIの共通制約は本要求で定めない。 |
| [RQ-09-004](../11_requirements/RQ-09_ui-scope.md#rq-09-004) | ViewModel側の値を変えるとTextの表示値が更新され、Textから編集できないことを確認する。 |
| [RQ-09-005](../11_requirements/RQ-09_ui-scope.md#rq-09-005) | 入力・選択・caretの結果を読み取れること、および承認されたcommit・validation条件で成功値または失敗状態が観測できることを確認する。 |
| [RQ-09-006](../11_requirements/RQ-09_ui-scope.md#rq-09-006) | 実行可否の両状態で操作し、実行回数と実行中の再操作結果が条件を満たすことを確認する。 |
| [RQ-09-007](../11_requirements/RQ-09_ui-scope.md#rq-09-007) | 初期シナリオの子順と対応するVisual配置を確認する。特定の方向、spacing、overflow規則は別途承認なしに受入条件へ加えない。 |
| [RQ-09-008](../11_requirements/RQ-09_ui-scope.md#rq-09-008) | 入力領域を変えたとき各Widgetの配置結果が再計算され、Coreの配置結果とPlatformのroot領域が整合することを確認する。寸法既定値や公開APIはここで固定しない。 |
| [RQ-09-009](../11_requirements/RQ-09_ui-scope.md#rq-09-009) | フォーカス対象がない状態とある状態を確認し、設定・解除後に現在対象が一意に観測できることを確認する。 |
| [RQ-09-010](../11_requirements/RQ-09_ui-scope.md#rq-09-010) | TextBoxがactive focusのとき入力文字が届き、非active対象へ誤配送されないことを確認する。特定のキー割当やfocus navigationは要求しない。 |
| [RQ-09-011](../11_requirements/RQ-09_ui-scope.md#rq-09-011) | 設定したnameをControlから取得し、PlatformのAccessibility公開を介して対応対象のnameとして確認できることを確認する。自動命名規則は固定しない。 |
| [RQ-09-012](../11_requirements/RQ-09_ui-scope.md#rq-09-012) | stage計画に対象として記録され、具体化・承認された必須caseの結果が確認できる。 |
| [RQ-09-013](../11_requirements/RQ-09_ui-scope.md#rq-09-013) | 同上。 |
| [RQ-09-014](../11_requirements/RQ-09_ui-scope.md#rq-09-014) | 同上。 |
| [RQ-09-015](../11_requirements/RQ-09_ui-scope.md#rq-09-015) | 同上。 |
| [RQ-09-016](../11_requirements/RQ-09_ui-scope.md#rq-09-016) | 同上。 |
| [RQ-09-017](../11_requirements/RQ-09_ui-scope.md#rq-09-017) | 同上。 |
| [RQ-09-018](../11_requirements/RQ-09_ui-scope.md#rq-09-018) | 同上。 |
| [RQ-09-019](../11_requirements/RQ-09_ui-scope.md#rq-09-019) | Model整合性のbatch成功・拒否と、独立入力のcommit・validationを別々に実行し、部分適用がなく、独立入力を不必要に待たせないことを確認する。 |
| [RQ-09-020](../11_requirements/RQ-09_ui-scope.md#rq-09-020) | cardinalityごとの選択、current itemとfocusの独立性、選択中・current item削除後の状態を確認する。 |
| [RQ-09-021](../11_requirements/RQ-09_ui-scope.md#rq-09-021) | ListViewで項目表示と選択状態を確認し、安定IDと共通Modelの結果に矛盾がないことを確認する。個別navigationは具体化時に定める。 |
| [RQ-09-022](../11_requirements/RQ-09_ui-scope.md#rq-09-022) | 大きな項目集合で表示対象範囲を移動し、可視範囲とsnapshotが一致し、画面外項目が論理データから失われないことを確認する。 |
| [RQ-09-023](../11_requirements/RQ-09_ui-scope.md#rq-09-023) | 承認されたStyle条件を複数Controlに適用し、指定された表示属性が一致することを確認する。優先順位やselector文法は具体化時に定める。 |
| [RQ-09-024](../11_requirements/RQ-09_ui-scope.md#rq-09-024) | 承認されたThemeを適用し、対象Controlの指定属性がThemeの値に反映されることを確認する。切替時期や継承規則は具体化時に定める。 |
| [RQ-09-025](../11_requirements/RQ-09_ui-scope.md#rq-09-025) | 承認されたResource参照が指定値を返し、参照対象が未定義の場合の結果が具体化した条件に一致することを確認する。検索範囲は具体化時に定める。 |
| [RQ-09-026](../11_requirements/RQ-09_ui-scope.md#rq-09-026) | Template差替え後もControlの論理上の親子関係と公開意味情報が保たれ、表示構成が指定どおり変わることを確認する。 |
| [RQ-09-027](../11_requirements/RQ-09_ui-scope.md#rq-09-027) | 未採用候補が必須機能・必須case・完了条件へ混入していないことを確認する。 |
| [RQ-09-028](../11_requirements/RQ-09_ui-scope.md#rq-09-028) | 同上。 |
| [RQ-09-029](../11_requirements/RQ-09_ui-scope.md#rq-09-029) | 同上。 |
| [RQ-09-030](../11_requirements/RQ-09_ui-scope.md#rq-09-030) | 同上。 |
| [RQ-09-031](../11_requirements/RQ-09_ui-scope.md#rq-09-031) | 同上。 |
| [RQ-09-032](../11_requirements/RQ-09_ui-scope.md#rq-09-032) | 同上。 |
| [RQ-09-033](../11_requirements/RQ-09_ui-scope.md#rq-09-033) | 同上。 |
| [RQ-09-034](../11_requirements/RQ-09_ui-scope.md#rq-09-034) | 同上。 |
| [RQ-09-035](../11_requirements/RQ-09_ui-scope.md#rq-09-035) | 同上。 |
| [RQ-09-036](../11_requirements/RQ-09_ui-scope.md#rq-09-036) | 同上。 |
| [RQ-09-037](../11_requirements/RQ-09_ui-scope.md#rq-09-037) | 共通契約が個別Controlの設計開始前に定義されていることを確認する。Model batchの整合性検証と独立入力validationの検証が分離されていることも確認する。 |
| [RQ-10-001](../11_requirements/RQ-10_accessibility.md#rq-10-001) | 初期Controlを構成し、Core側で意味情報を読み取り、OSのAccessibility機構から対応する対象が利用可能であることを確認する。 |
| [RQ-10-002](../11_requirements/RQ-10_accessibility.md#rq-10-002) | Core上のControl種別とOSから取得したroleを比較し、TextBoxが編集可能な文字列Control、Buttonが起動可能なControlなど表の意味と合うことを確認する。 |
| [RQ-10-003](../11_requirements/RQ-10_accessibility.md#rq-10-003) | nameを設定・取得し、設定値との一致を確認する。未設定時に特定のfallback文字列を要求しない。 |
| [RQ-10-004](../11_requirements/RQ-10_accessibility.md#rq-10-004) | 状態を変更した前後でCoreの状態とAccessibility情報を比較し、変更が反映されることを確認する。未定義の細分化状態は要求しない。 |
| [RQ-10-005](../11_requirements/RQ-10_accessibility.md#rq-10-005) | TextおよびTextBoxの値を変更し、Coreの現在値とAccessibility取得値が一致することを確認する。値のないControlに値を捏造しない。 |
| [RQ-10-006](../11_requirements/RQ-10_accessibility.md#rq-10-006) | focusを移動・解除し、Coreの現在対象とOS Accessibility情報のfocus対象を比較する。 |
| [RQ-10-007](../11_requirements/RQ-10_accessibility.md#rq-10-007) | 支援技術から公開actionを実行し、通常の入力・Command経路と同じ結果になることを確認する。未対応actionを公開しない。 |
| [RQ-10-008](../11_requirements/RQ-10_accessibility.md#rq-10-008) | enabled状態を切り替え、取得状態が一致すること、disabled時に操作要求がControl動作やCommandを起こさないことを確認する。 |
| [RQ-10-009](../11_requirements/RQ-10_accessibility.md#rq-10-009) | OSのAccessibility機構で各初期Controlを検査し、取得情報・actionが上表およびCore snapshotと意味的に一致することを確認する。 |
| [RQ-10-010](../11_requirements/RQ-10_accessibility.md#rq-10-010) | 参照した指針と採用した意味を区別でき、要求していないWeb固有動作や完全適合の主張が追加されていないことを確認する。 |
| [RQ-11-001](../11_requirements/RQ-11_platform.md#rq-11-001) | 対象releaseのsupport matrixにLinux ARMv7のprofileとbuild/runtime/hardware検証状態が明記され、buildだけの結果を実機support済みとして扱っていないことを確認する。 |
| [RQ-11-002](../11_requirements/RQ-11_platform.md#rq-11-002) | RQ-11-001と同じ区分でAArch64のprofileと各検証状態が個別に記録されることを確認する。 |
| [RQ-11-003](../11_requirements/RQ-11_platform.md#rq-11-003) | Linux x64 native build/test結果と、そこから生成したARM向けbuild/link結果が別profileの証跡として記録されることを確認する。 |
| [RQ-11-004](../11_requirements/RQ-11_platform.md#rq-11-004) | Windows x64 native buildと定められたsmoke確認の結果がtarget profileに紐付いて記録されることを確認する。 |
| [RQ-11-005](../11_requirements/RQ-11_platform.md#rq-11-005) | 候補が現行releaseの必須support対象として誤記されていないことを確認する。採用時は対象profileと検証範囲を要求変更で確定する。 |
| [RQ-11-006](../11_requirements/RQ-11_platform.md#rq-11-006) | RQ-11-005と同様に、候補と正式support対象が区別されていることを確認する。 |
| [RQ-11-007](../11_requirements/RQ-11_platform.md#rq-11-007) | Linuxのtarget tripleやsupport表示をbare metal/MCUへ流用していないことを確認する。候補採用時には必要なruntime・OS前提を別途要求として定める。 |
| [RQ-11-008](../11_requirements/RQ-11_platform.md#rq-11-008) | 依存構造を確認し、OS固有描画依存がRenderer/backend側に位置すること、およびbackendを交換してもCoreのControl意味論を変更しない境界であることを示す。 |
| [RQ-11-009](../11_requirements/RQ-11_platform.md#rq-11-009) | Coreから特定OSのwindowing・device APIへの直接依存がないことと、Platform境界からCoreへ共通Runtime操作を渡せる構成を確認する。 |
| [RQ-11-010](../11_requirements/RQ-11_platform.md#rq-11-010) | 依存構造とtext処理境界を確認し、Coreの共通契約がOS固有text engine型を公開・保持しないことを示す。 |
| [RQ-11-011](../11_requirements/RQ-11_platform.md#rq-11-011) | 依存構造を確認し、codec実装が差し替え可能な境界の外側にあり、Core共通契約へcodec固有型が漏れないことを示す。 |
| [RQ-11-012](../11_requirements/RQ-11_platform.md#rq-11-012) | OS/device固有event型がCoreの共通処理へ直接流入しない依存構造を確認する。backend固有情報が必要な箇所は境界内で扱われることを示す。 |
| [RQ-11-013](../11_requirements/RQ-11_platform.md#rq-11-013) | Core意味情報の正本がOS固有APIに依存せず、OS接続がbackend境界に置かれる依存構造を確認する。 |
| [RQ-11-014](../11_requirements/RQ-11_platform.md#rq-11-014) | 対象profileで利用可否・検証結果を記録し、次候補へ進む際に未実施理由または結果が残っていることを確認する。 |
| [RQ-11-015](../11_requirements/RQ-11_platform.md#rq-11-015) | 順序と対象profileに沿った採否・検証結果を記録する。 |
| [RQ-11-016](../11_requirements/RQ-11_platform.md#rq-11-016) | 順序と対象profileに沿った採否・検証結果を記録する。 |
| [RQ-11-017](../11_requirements/RQ-11_platform.md#rq-11-017) | 順序と対象profileに沿った採否・検証結果を記録する。 |
| [RQ-11-018](../11_requirements/RQ-11_platform.md#rq-11-018) | 順序と対象profileに沿った採否・検証結果を記録する。 |
| [RQ-12-001](../11_requirements/RQ-12_build-distribution.md#rq-12-001) | 各対象targetのartifact metadataと実際のbinary architectureを照合する。consumerから取得・linkできることを確認し、Build/link成功とtarget上のRuntime成功を別々に記録する。 |
| [RQ-12-002](../11_requirements/RQ-12_build-distribution.md#rq-12-002) | 同一OS内のarchitecture違いとOS違いを区別してartifactを取得・linkできること、およびサポート表示が検証済みprofileの範囲と一致することを確認する。 |
| [RQ-12-003](../11_requirements/RQ-12_build-distribution.md#rq-12-003) | 各言語consumerが同じtargetの共通runtime identityを利用し、同じABI majorとCore状態へ接続することをpackage設定・実行結果で確認する。 |
| [RQ-12-004](../11_requirements/RQ-12_build-distribution.md#rq-12-004) | install/export後の独立consumer projectがconfig packageを検索し、公開wrapperをcompile/linkして同梱対象のnative runtimeを解決できることを確認する。repository内build pathへの依存を要求しない。 |
| [RQ-12-005](../11_requirements/RQ-12_build-distribution.md#rq-12-005) | 両crateが期待する境界で配布され、safe API consumerが共通runtimeを利用できることを確認する。target/profile/ABIの不一致は検出され、適合しないartifactの利用前に明示的に失敗することを確認する。 |
| [RQ-12-006](../11_requirements/RQ-12_build-distribution.md#rq-12-006) | NuGet packageの構成が指定RIDに対応し、各RID consumerが対応native libraryを解決・loadできることを確認する。未採用ARM profileを配布済みsupportとして表示しない。 |
| [RQ-12-007](../11_requirements/RQ-12_build-distribution.md#rq-12-007) | pure-Python wrapperが配布され、対象環境で追加のextension compileなしに共通C ABI runtimeをloadして利用開始できることを確認する。 |
| [RQ-12-008](../11_requirements/RQ-12_build-distribution.md#rq-12-008) | wrapper distributionとnative artifactの内容・責務が分離され、対象profileに合うruntimeを組み合わせてloadできることを確認する。未確認ARM profileが互換wheelとして誤表示されないことを確認する。 |
| [RQ-12-009](../11_requirements/RQ-12_build-distribution.md#rq-12-009) | CI設定またはbuild provenanceから各設定の版・選択値を特定でき、targetごとのcompile/link結果に対応付けられることを確認する。具体的な版番号は対象profile確定時に管理する。 |
| [RQ-12-010](../11_requirements/RQ-12_build-distribution.md#rq-12-010) | targetの追加順と各stageの完了記録を確認する。ARMではbuild/link結果のみをRuntime verifiedと表示せず、実機確認は対象board/profileに限定して記録する。 |
| [RQ-13-001](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-001) | 対象boardで値が得られ、起動区間の定義と実測結果が一緒に追跡できることを確認する。閾値未承認の値を合否保証に使わない。 |
| [RQ-13-002](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-002) | 対象artifactとsize定義を特定でき、得られた値を同じ対象条件の別結果と比較できることを確認する。 |
| [RQ-13-003](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-003) | 測定値の対象範囲と取得条件を識別でき、異なる定義の値を同じ指標として混ぜていないことを確認する。 |
| [RQ-13-004](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-004) | 対象boardでFPSの結果が得られ、比較する結果ごとの描画負荷条件を追跡できることを確認する。 |
| [RQ-13-005](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-005) | 計測値がlayout処理に対応し、他層の処理を含む場合は含む範囲を識別できることを確認する。 |
| [RQ-13-006](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-006) | 測定対象がProperty更新であることと、比較する更新条件を識別できることを確認する。 |
| [RQ-13-007](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-007) | Collection差分処理に対応する値と、その処理対象の条件が区別して記録されることを確認する。 |
| [RQ-13-008](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-008) | callback/FFI境界の測定結果を他の処理時間から識別でき、測定条件が結果に紐付くことを確認する。 |
| [RQ-13-009](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-009) | 最大値の対象区間・負荷・集計定義を追跡でき、単一値だけが条件不明で報告されていないことを確認する。 |
| [RQ-13-010](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-010) | 実機と実行条件に結び付いた安定性結果が記録され、cross-build成功のみで実機安定性を示した扱いにしていないことを確認する。 |
| [RQ-13-011](../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-011) | 共通するboard/profileと機能条件を追跡でき、各層の測定結果が独立して特定できることを確認する。言語間の比較を示す場合、機能と条件の差を併記する。 |
| [RQ-14-001](../11_requirements/RQ-14_delivery-sequence.md#rq-14-001) | 承認済みABI仕様と、その承認記録が存在する。 |
| [RQ-14-002](../11_requirements/RQ-14_delivery-sequence.md#rq-14-002) | 各契約を含む承認済み仕様と承認記録が存在する。 |
| [RQ-14-003](../11_requirements/RQ-14_delivery-sequence.md#rq-14-003) | 対象機能の実装revisionと、そのrevisionを検証した結果が記録される。 |
| [RQ-14-004](../11_requirements/RQ-14_delivery-sequence.md#rq-14-004) | 4言語それぞれのwrapper成果物と対応するビルド結果が確認できる。 |
| [RQ-14-005](../11_requirements/RQ-14_delivery-sequence.md#rq-14-005) | 言語ごとの対象revision、実行環境、結果が記録され、成功または未解決失敗が区別される。 |
| [RQ-14-006](../11_requirements/RQ-14_delivery-sequence.md#rq-14-006) | 対象Controlの実装と各Controlの要求機能を確認した結果が記録される。 |
| [RQ-14-007](../11_requirements/RQ-14_delivery-sequence.md#rq-14-007) | 対象環境での入力確認結果と未対応条件が記録される。 |
| [RQ-14-008](../11_requirements/RQ-14_delivery-sequence.md#rq-14-008) | 対象Controlのname/roleが利用者または検証ツールから確認できる結果が記録される。 |
| [RQ-14-009](../11_requirements/RQ-14_delivery-sequence.md#rq-14-009) | 各機能の実装revisionと対応する機能確認結果が記録される。 |
| [RQ-14-010](../11_requirements/RQ-14_delivery-sequence.md#rq-14-010) | 対象platformで描画が行われた結果と使用した実行環境が記録される。 |
| [RQ-14-011](../11_requirements/RQ-14_delivery-sequence.md#rq-14-011) | CI実行で互換性検査が起動し、検出対象の互換・非互換例の結果が確認できる。 |
| [RQ-14-012](../11_requirements/RQ-14_delivery-sequence.md#rq-14-012) | 測定対象、実機・構成、条件、結果、測定時のrevisionが記録される。 |
| [RQ-14-013](../11_requirements/RQ-14_delivery-sequence.md#rq-14-013) | 追加した場合は根拠となる測定・課題と再測定結果を記録し、追加しない場合は不要と判断した根拠を記録する。 |
| [RQ-12-010](../11_requirements/RQ-12_build-distribution.md#rq-12-010)、[RQ-11-003](../11_requirements/RQ-11_platform.md#rq-11-003)、[RQ-11-004](../11_requirements/RQ-11_platform.md#rq-11-004) | 対応計画と検証記録がこの順序を示し、ARM成果物のbuild hostとtargetを区別している。 |
| [RQ-01-002](../11_requirements/RQ-01_purpose-architecture.md#rq-01-002)、[RQ-11-008](../11_requirements/RQ-11_platform.md#rq-11-008) | 描画・入力試験で共通Control状態がnative controlに依存せず維持されることを確認する。 |
| [RQ-11-008](../11_requirements/RQ-11_platform.md#rq-11-008)、[RQ-11-009](../11_requirements/RQ-11_platform.md#rq-11-009) | 責務分担と変更・検証記録から、CoreとPlatform描画backendの境界を確認できる。 |
| [RQ-09-002](../11_requirements/RQ-09_ui-scope.md#rq-09-002)、[RQ-09-003](../11_requirements/RQ-09_ui-scope.md#rq-09-003) | 初期Controlの分類がこの一覧と一致し、旧Element表記が残る箇所は移行上の別名として識別される。 |
| [RQ-09-005](../11_requirements/RQ-09_ui-scope.md#rq-09-005)、[RQ-09-053](../11_requirements/RQ-09_ui-scope.md#rq-09-053)、[RQ-09-054](../11_requirements/RQ-09_ui-scope.md#rq-09-054) | 各対象環境の能力記録とテスト結果で、UTF-8、Latin、日本語の表示・編集を確認できる。未確認環境は明示される。 |
| [RQ-09-005](../11_requirements/RQ-09_ui-scope.md#rq-09-005)、[RQ-09-038](../11_requirements/RQ-09_ui-scope.md#rq-09-038)、[RQ-09-040](../11_requirements/RQ-09_ui-scope.md#rq-09-040)、[RQ-09-042](../11_requirements/RQ-09_ui-scope.md#rq-09-042)、[RQ-09-055](../11_requirements/RQ-09_ui-scope.md#rq-09-055)、[RQ-09-056](../11_requirements/RQ-09_ui-scope.md#rq-09-056) | Focus離脱、明示cancel、commitとの競合、終了後の遅延eventを含む確認で、各結果がこの規則と一致する。 |
| [RQ-07-008](../11_requirements/RQ-07_binding.md#rq-07-008)、[RQ-07-013](../11_requirements/RQ-07_binding.md#rq-07-013)、[RQ-07-014](../11_requirements/RQ-07_binding.md#rq-07-014)、[RQ-03-011](../11_requirements/RQ-03_language-support.md#rq-03-011) | 静的に解決できる対象で警告を確認し、実行時の非収束連鎖が有限処理で停止して診断可能であることを確認する。 |
| [RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)、[RQ-09-005](../11_requirements/RQ-09_ui-scope.md#rq-09-005)、[RQ-02-021](../11_requirements/RQ-02_core-features.md#rq-02-021)、[RQ-05-013](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-013)、[RQ-07-016](../11_requirements/RQ-07_binding.md#rq-07-016) | 同期Validatorが同期結果を返し、remote/long-running処理がアプリ側状態とCommandで表現され、古い結果をアプリ側が識別できることを確認する。 |
| [RQ-06-004](../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)、[RQ-06-008](../11_requirements/RQ-06_threading-dispatch.md#rq-06-008)、[RQ-06-009](../11_requirements/RQ-06_threading-dispatch.md#rq-06-009)、[RQ-06-010](../11_requirements/RQ-06_threading-dispatch.md#rq-06-010)、[RQ-06-011](../11_requirements/RQ-06_threading-dispatch.md#rq-06-011) | 進捗の集約が許され、終端結果が一度だけ反映されること、古いoperationおよび終端後の通知が反映されないことを確認する。 |
| [RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009) | UI処理後にBindingと通知がまとめて処理され、複数setterの一部失敗時に全体rollbackを前提としないことを確認する。 |
| [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)、[RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)、[RQ-04-023](../11_requirements/RQ-04_abi-boundary.md#rq-04-023)、[RQ-04-027](../11_requirements/RQ-04_abi-boundary.md#rq-04-027) | 同一の失敗を機械判定情報と表示用messageで区別でき、snapshotの有効期間とcause/diagnosticの用途が確認できる。 |
| [RQ-04-009](../11_requirements/RQ-04_abi-boundary.md#rq-04-009)、[RQ-04-024](../11_requirements/RQ-04_abi-boundary.md#rq-04-024)、[RQ-04-028](../11_requirements/RQ-04_abi-boundary.md#rq-04-028) | release metadataとABI symbol majorが対応し、minor/patch変更が定めた互換範囲を守ることを確認する。 |
| [RQ-12-007](../11_requirements/RQ-12_build-distribution.md#rq-12-007)、[RQ-03-011](../11_requirements/RQ-03_language-support.md#rq-03-011)、[RQ-03-012](../11_requirements/RQ-03_language-support.md#rq-03-012) | extensionなしでABIを呼ぶPython wrapperと、同じ診断形式を用いた各実行入口、増分解析の再利用・更新結果を確認する。 |
| [RQ-12-004](../11_requirements/RQ-12_build-distribution.md#rq-12-004)、[RQ-12-005](../11_requirements/RQ-12_build-distribution.md#rq-12-005)、[RQ-12-006](../11_requirements/RQ-12_build-distribution.md#rq-12-006)、[RQ-12-008](../11_requirements/RQ-12_build-distribution.md#rq-12-008)、[RQ-12-016](../11_requirements/RQ-12_build-distribution.md#rq-12-016)、[RQ-12-021](../11_requirements/RQ-12_build-distribution.md#rq-12-021) | 各配布物に適用licenseと必要noticeが含まれ、third-party dependencyのlicense情報が上流条件を保つことを確認する。 |
| [RQ-11-001](../11_requirements/RQ-11_platform.md#rq-11-001)、[RQ-11-002](../11_requirements/RQ-11_platform.md#rq-11-002)、[RQ-11-020](../11_requirements/RQ-11_platform.md#rq-11-020)、[RQ-12-017](../11_requirements/RQ-12_build-distribution.md#rq-12-017)、[RQ-12-018](../11_requirements/RQ-12_build-distribution.md#rq-12-018)、[RQ-12-019](../11_requirements/RQ-12_build-distribution.md#rq-12-019)、[RQ-12-020](../11_requirements/RQ-12_build-distribution.md#rq-12-020) | 公開保守情報からmajor/profileごとの期間・範囲を確認し、buildとruntimeの検証状態、非推奨通知、EOLを識別できる。 |
| [RQ-12-004](../11_requirements/RQ-12_build-distribution.md#rq-12-004)、[RQ-12-005](../11_requirements/RQ-12_build-distribution.md#rq-12-005)、[RQ-12-006](../11_requirements/RQ-12_build-distribution.md#rq-12-006)、[RQ-12-008](../11_requirements/RQ-12_build-distribution.md#rq-12-008)、[RQ-12-011](../11_requirements/RQ-12_build-distribution.md#rq-12-011)、[RQ-12-012](../11_requirements/RQ-12_build-distribution.md#rq-12-012)、[RQ-12-013](../11_requirements/RQ-12_build-distribution.md#rq-12-013)、[RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-015](../11_requirements/RQ-12_build-distribution.md#rq-12-015) | 配布・検証記録で署名、独立trust root、rotation/revocation、offline検証、targetとdigestの照合を確認し、検証失敗時にartifactを受け入れないことを確認する。 |
| [RQ-12-010](../11_requirements/RQ-12_build-distribution.md#rq-12-010) | CI記録でOS/targetごとのjobと順序を確認し、ARM cross-build成功をruntime実機検証成功として誤記していないことを確認する。 |
| [RQ-10-001](../11_requirements/RQ-10_accessibility.md#rq-10-001)、[RQ-10-002](../11_requirements/RQ-10_accessibility.md#rq-10-002)、[RQ-10-003](../11_requirements/RQ-10_accessibility.md#rq-10-003)、[RQ-10-004](../11_requirements/RQ-10_accessibility.md#rq-10-004)、[RQ-10-005](../11_requirements/RQ-10_accessibility.md#rq-10-005)、[RQ-10-006](../11_requirements/RQ-10_accessibility.md#rq-10-006)、[RQ-10-007](../11_requirements/RQ-10_accessibility.md#rq-10-007)、[RQ-10-008](../11_requirements/RQ-10_accessibility.md#rq-10-008)、[RQ-10-009](../11_requirements/RQ-10_accessibility.md#rq-10-009)、[RQ-10-011](../11_requirements/RQ-10_accessibility.md#rq-10-011)、[RQ-10-012](../11_requirements/RQ-10_accessibility.md#rq-10-012)、[RQ-10-013](../11_requirements/RQ-10_accessibility.md#rq-10-013) | 対象Widgetの共通意味情報とOS側から取得した情報を照合し、初期treeにtemplate内Visual要素が既定で現れないことを確認する。 |
| [RQ-14-002](../11_requirements/RQ-14_delivery-sequence.md#rq-14-002) | 要求本文と履歴から採用範囲および未決範囲が特定でき、下流案を要求承認と取り違えていない。 |
| [RQ-14-003](../11_requirements/RQ-14_delivery-sequence.md#rq-14-003)、[RQ-14-005](../11_requirements/RQ-14_delivery-sequence.md#rq-14-005) | 要求本文に各保証の採否があり、未決の実装アルゴリズムと区別できる。 |
| [RQ-14-010](../11_requirements/RQ-14_delivery-sequence.md#rq-14-010)、[RQ-14-012](../11_requirements/RQ-14_delivery-sequence.md#rq-14-012) | 対象・保証・未対応範囲が要求本文で識別でき、backend選択などの方式決定と区別できる。 |
| [RQ-14-007](../11_requirements/RQ-14_delivery-sequence.md#rq-14-007)、[RQ-14-008](../11_requirements/RQ-14_delivery-sequence.md#rq-14-008) | 保証対象と対象外が要求本文で識別でき、OS API等の実現方式と区別できる。 |
| [RQ-16-001](../11_requirements/RQ-16_document-governance.md#rq-16-001) | 決定内容と適用範囲、決定状態が対象ADR本文から確認でき、草案とAcceptedを識別できる。 |
| [RQ-16-002](../11_requirements/RQ-16_document-governance.md#rq-16-002) | 各調査結論の根拠と比較内容、未決事項を資料から追跡でき、調査結果だけで承認済み要求・設計判断を置き換えていない。 |
| [RQ-16-003](../11_requirements/RQ-16_document-governance.md#rq-16-003) | 要求変更の記録から対象・要求受入条件・承認者・日付・結論・根拠と影響先を追跡できる。改訂版を各履歴表で区別でき、基本設計承認が要求承認条件に含まれていない。 |
| [RQ-16-004](../11_requirements/RQ-16_document-governance.md#rq-16-004) | 不整合の記録から差異、変更対象、承認者、影響先を確認できる。要求またはADR/設計の正式変更前に下流文書だけで結論を上書きせず、解決前の設計・実装がAccepted ADRの制約を破っていない。 |
| [RQ-11-019](../11_requirements/RQ-11_platform.md#rq-11-019) | 依存関係とデータ境界を点検し、列挙した実機依存機能の処理がPlatform側に隔離され、Coreへ渡る値が共通契約で扱えることを確認する。profileごとの差異は対象profileの検証記録に残す。 |

<a id="assert-rq-17-001"></a>
## RQ-17-001
- 根拠: [RQ-17-001](../11_requirements/RQ-17_webview-plugin.md#rq-17-001)
- case: [TC-WEBVIEW-01](requirement-test-matrix.md#case-tc-webview-01)、[TC-WEBVIEW-02](requirement-test-matrix.md#case-tc-webview-02)
- 期待結果: OS固有ブラウザ部品に依存せずCoreエンジン単体でDOM・スタイル・レイアウト・描画プリミティブを生成できること。

<a id="assert-rq-17-002"></a>
## RQ-17-002
- 根拠: [RQ-17-002](../11_requirements/RQ-17_webview-plugin.md#rq-17-002)
- case: [TC-WEBVIEW-01](requirement-test-matrix.md#case-tc-webview-01)
- 期待結果: `Cargo.toml` およびビルド依存にサードパーティ製外部crateを含まず、標準ライブラリのみでビルド・テストが完結すること。

<a id="assert-rq-17-003"></a>
## RQ-17-003
- 根拠: [RQ-17-003](../11_requirements/RQ-17_webview-plugin.md#rq-17-003)
- case: [TC-WEBVIEW-01](requirement-test-matrix.md#case-tc-webview-01)、[TC-WEBVIEW-02](requirement-test-matrix.md#case-tc-webview-02)
- 期待結果: CommonMark/GFMの見出し・段落・強調・リスト・表・コード・引用・リンクから正しいDOM構造と描画コマンドが生成されること。

<a id="assert-rq-17-004"></a>
## RQ-17-004
- 根拠: [RQ-17-004](../11_requirements/RQ-17_webview-plugin.md#rq-17-004)
- case: [TC-WEBVIEW-01](requirement-test-matrix.md#case-tc-webview-01)、[TC-WEBVIEW-02](requirement-test-matrix.md#case-tc-webview-02)
- 期待結果: HTML要素とCSS規則（詳細度・カスケード・ボックスモデル・Flex/Table）が解決され、未知要素や未対応プロパティで安全にフォールバックすること。

<a id="assert-rq-17-005"></a>
## RQ-17-005
- 根拠: [RQ-17-005](../11_requirements/RQ-17_webview-plugin.md#rq-17-005)
- case: [TC-WEBVIEW-02](requirement-test-matrix.md#case-tc-webview-02)、[TC-WEBVIEW-04](requirement-test-matrix.md#case-tc-webview-04)
- 期待結果: UI Tree上のWidgetとしてMeasure/ArrangeおよびDetached維持・再所属が機能し、ビューポート寸法に応じた折り返しとスクロール範囲が算出されること。

<a id="assert-rq-17-006"></a>
## RQ-17-006
- 根拠: [RQ-17-006](../11_requirements/RQ-17_webview-plugin.md#rq-17-006)
- case: [TC-WEBVIEW-04](requirement-test-matrix.md#case-tc-webview-04)
- 期待結果: UTF-8 PropertyおよびOneWay/TwoWay Bindingによって文書ソースやフォーム入力値が同期され、Validator拒否時に既存確定値を維持すること。

<a id="assert-rq-17-007"></a>
## RQ-17-007
- 根拠: [RQ-17-007](../11_requirements/RQ-17_webview-plugin.md#rq-17-007)
- case: [TC-WEBVIEW-04](requirement-test-matrix.md#case-tc-webview-04)
- 期待結果: リンク選択・ボタン操作・スクリプトアクションからCore Commandが起動され、CanExecute=falseや再入時に抑止されること。

<a id="assert-rq-17-008"></a>
## RQ-17-008
- 根拠: [RQ-17-008](../11_requirements/RQ-17_webview-plugin.md#rq-17-008)
- case: [TC-WEBVIEW-03](requirement-test-matrix.md#case-tc-webview-03)、[TC-WEBVIEW-04](requirement-test-matrix.md#case-tc-webview-04)
- 期待結果: 軽量スクリプトからDOM要素の参照・更新・イベント処理および許可されたProperty/Command連携が実行でき、変更がレイアウト・描画へ反映されること。

<a id="assert-rq-17-009"></a>
## RQ-17-009
- 根拠: [RQ-17-009](../11_requirements/RQ-17_webview-plugin.md#rq-17-009)
- case: [TC-WEBVIEW-01](requirement-test-matrix.md#case-tc-webview-01)、[TC-WEBVIEW-03](requirement-test-matrix.md#case-tc-webview-03)
- 期待結果: 入力バイト長・DOMノード数・ネスト深さ・CSSルール数・スクリプト命令ステップ数/スタック/メモリの上限超過時に、フリーズせず安全に中断し直前の有効状態を保つこと。

<a id="assert-rq-17-010"></a>
## RQ-17-010
- 根拠: [RQ-17-010](../11_requirements/RQ-17_webview-plugin.md#rq-17-010)
- case: [TC-WEBVIEW-03](requirement-test-matrix.md#case-tc-webview-03)
- 期待結果: 暗黙の外部ネットワーク通信やファイルシステムアクセスを行わず、未許可URIスキームを拒否すること。

<a id="assert-rq-17-011"></a>
## RQ-17-011
- 根拠: [RQ-17-011](../11_requirements/RQ-17_webview-plugin.md#rq-17-011)
- case: [TC-WEBVIEW-04](requirement-test-matrix.md#case-tc-webview-04)
- 期待結果: versioned C ABIおよび4言語ラッパー（C++/C#/Rust/Python 3）から共通の意味論でWebViewを生成・操作・破棄できること。

<a id="assert-rq-17-012"></a>
## RQ-17-012
- 根拠: [RQ-17-012](../11_requirements/RQ-17_webview-plugin.md#rq-17-012)
- case: [TC-WEBVIEW-02](requirement-test-matrix.md#case-tc-webview-02)
- 期待結果: WebView内部の見出し・リンク・ボタン・入力欄・テキスト・表がAxSnapshotの意味ノードとして出力されること。

## 承認履歴（今回）

| 日付 | 版・対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 1.0 / 受入確認のRQからの分離と要求別assertion | 利用者 | 全案反映指示により承認。実装・実行証跡のない項目は未実装・未実行を維持 |

## 改訂履歴（今回）

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 1.0 | 2026-10-17 | 受入確認のRQからの分離と要求別assertion | Codex |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 1.1 / 31要求の個別oracle追加と既存条件改訂の同期 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 1.1 | 2026-10-17 | 31要求の個別oracle追加と既存条件改訂の同期を正本へ適用し、旧判断との対応を保持 | Codex |
