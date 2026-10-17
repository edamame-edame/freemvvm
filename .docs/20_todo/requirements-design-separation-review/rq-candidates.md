# 要求移管・補足・未決条件の文案

- 文書版: 0.2
- 状態: 全案反映承認済み・正本適用済み（2026-10-17）。具体化未了は適用結果を参照
- 作成日: 2026-10-17
- 入口: [一括レビュー索引](index.md)

各SEP-RQ番号はレビュー内の仮候補IDで、正式RQ IDではない。引用枠はRQへ収録する本文案であり、根拠リンク・設計手段・検証手順は引用枠の外に置く。補足案も元のBD/DD承認を要求承認へ読み替えず、要求側で承認する。選択要の案は選択肢が残る。既存RQの意味維持・統合は要求変更と別に記録する。

対象段階は初期共通基盤を既定とする。Collection/Selectionは次段階、marker/JIT/base-call追加・Grid/TableView・free-threaded正式support等は未採用候補を維持する。未決保証は合格条件にしない。


<a id="candidate-001"></a>
## SEP-RQ-001 任意threadの参照取得・破棄

- 対応する現行要求: [RQ-05-002](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-003](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)
- 由来: [D01](../../13_detail_design/01-handle-registry.md)、[S02](../../12_design/runtime-contract.md)
- 分類・採否: 補足案。最終解放をどのqueueへ渡すかはDD。

> 所有参照を取得・破棄する操作を任意threadから利用できること。UI資源の最終解放が操作の戻りより遅れる場合は、その操作の成功と解放完了を区別できること。

要求testへの影響: [TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../../30_release_test/requirement-test-matrix.md#case-tc-thread-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-002"></a>
## SEP-RQ-002 破棄済み参照への重複破棄

- 対応する現行要求: [RQ-05-002](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-003](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)
- 由来: [D01](../../13_detail_design/01-handle-registry.md)、[S02](../../12_design/runtime-contract.md)
- 分類・採否: 補足案。token表や非再利用の64-bit方式を要求本文へ入れない。

> 破棄済みの同じ所有参照を再度破棄しようとした場合は失敗し、他の有効な所有参照と対象状態に影響しないこと。

要求testへの影響: [TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-003"></a>
## SEP-RQ-003 安全な参照を発行できない場合

- 対応する現行要求: [RQ-04-005](../../11_requirements/RQ-04_abi-boundary.md#rq-04-005)
- 由来: [D01](../../13_detail_design/01-handle-registry.md)、[S02](../../12_design/runtime-contract.md)
- 分類・採否: 補足案。発行器の幅・枯渇codeはDD。

> 安全に識別できる所有参照を新たに発行できない場合は取得・生成を失敗させ、既存の参照を別対象へ誤って解決しないこと。

要求testへの影響: [TC-CORE-OBJ-01](../../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-004"></a>
## SEP-RQ-004 参照の種類・所属の拒否結果

- 対応する現行要求: [RQ-04-006](../../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../../11_requirements/RQ-04_abi-boundary.md#rq-04-007)
- 由来: [D01](../../13_detail_design/01-handle-registry.md)、[S03](../../12_design/abi-spec.md)
- 分類・採否: 補足案。個々の数値statusは既存DD-06に保持する。

> 無効な参照、期待する対象種類の不一致、異なるRuntime所属の組合せを呼出し側が失敗理由として区別でき、拒否によって対象状態を変更しないこと。

要求testへの影響: [TC-CORE-OBJ-01](../../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-005"></a>
## SEP-RQ-005 破棄失敗時の参照保全

- 対応する現行要求: [RQ-05-003](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)
- 由来: [D02](../../13_detail_design/02-runtime-lifetime.md)、[D18](../../13_detail_design/18-wrapper-cpp.md)、[D19](../../13_detail_design/19-wrapper-csharp.md)、[D20](../../13_detail_design/20-wrapper-rust.md)、[D21](../../13_detail_design/21-wrapper-python.md)
- 分類・採否: 補足案。finalizer enqueue失敗を利用者結果に置換。回復不能なprocess終了は別候補で除外する。

> 所有参照の破棄を受け付けられなかった場合はその参照の所有を維持し、後から解放を再試行できること。言語側の自動後始末経路でも、解放に失敗した参照を管理不能なまま失わないこと。

要求testへの影響: [TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-WRAP-CPP-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-RS-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-rs-01)、[TC-WRAP-PY-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-006"></a>
## SEP-RQ-006 Runtime停止後の利用境界

- 対応する現行要求: [RQ-02-013](../../11_requirements/RQ-02_core-features.md#rq-02-013)、[RQ-05-003](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)
- 由来: [D02](../../13_detail_design/02-runtime-lifetime.md)、[S02](../../12_design/runtime-contract.md)、[S03](../../12_design/abi-spec.md)
- 分類・採否: 補足案。利用不可とcleanup用参照の残存を分け、RQ-05-003の「無効」をUI利用不可として明確化する。

> Runtime終了後はUI機能を利用できず、終了済みであることを識別できること。残存する所有参照は解放でき、不変なエラー情報はその参照の利用期間内に読み取れること。

要求testへの影響: [TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-007"></a>
## SEP-RQ-007 Runtime終了の再要求

- 対応する現行要求: [RQ-02-013](../../11_requirements/RQ-02_core-features.md#rq-02-013)
- 由来: [D02](../../13_detail_design/02-runtime-lifetime.md)、[B36](../../12_design/BD-36_runtime-shutdown-dispatch-upper-design.md)、[D03](../../13_detail_design/03-dispatcher.md)
- 分類・採否: 補足案。Closed state guardはDD。

> Runtime終了が完了した後に再度終了を要求しても成功し、利用者データの後始末や資源解放を重複して実行しないこと。

要求testへの影響: [TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-008"></a>
## SEP-RQ-008 callback/event中の終了要求

- 対応する現行要求: [RQ-06-002](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)、[RQ-05-009](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)
- 由来: [B36](../../12_design/BD-36_runtime-shutdown-dispatch-upper-design.md)、[D03](../../13_detail_design/03-dispatcher.md)、[D06](../../13_detail_design/06-abi-surface.md)、[D18](../../13_detail_design/18-wrapper-cpp.md)、[D19](../../13_detail_design/19-wrapper-csharp.md)、[D20](../../13_detail_design/20-wrapper-rust.md)、[D21](../../13_detail_design/21-wrapper-python.md)
- 分類・採否: BD-36-01/02/04の利用者確認済み方針をRQの承認として明示し直す案。

> callbackまたはUI event処理中にもRuntime終了を要求できること。現在の処理を完了させた後、未開始の通常作業より優先して終了処理を実行すること。

要求testへの影響: [TC-THREAD-01](../../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-CALLBACK-01](../../30_release_test/requirement-test-matrix.md#case-tc-callback-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-009"></a>
## SEP-RQ-009 終了要求の受付と完了

- 対応する現行要求: [RQ-06-002](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)
- 由来: [B36](../../12_design/BD-36_runtime-shutdown-dispatch-upper-design.md)、[D03](../../13_detail_design/03-dispatcher.md)、[D06](../../13_detail_design/06-abi-surface.md)、[D18](../../13_detail_design/18-wrapper-cpp.md)、[D19](../../13_detail_design/19-wrapper-csharp.md)、[D20](../../13_detail_design/20-wrapper-rust.md)、[D21](../../13_detail_design/21-wrapper-python.md)
- 分類・採否: 補足案。UI_PENDING数値・language result型はDD。

> 非同期の終了要求は受付後に戻り、終了完了を待たないこと。同期指定でもcallback/event中は受付後に戻り、workerからの同期指定はUI threadでの終了完了後に戻ること。受付済みと完了済みを区別できること。

要求testへの影響: [TC-THREAD-01](../../30_release_test/requirement-test-matrix.md#case-tc-thread-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-010"></a>
## SEP-RQ-010 停止したUI loopへの同期要求

- 対応する現行要求: [RQ-06-002](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)
- 由来: [B36](../../12_design/BD-36_runtime-shutdown-dispatch-upper-design.md)、[D03](../../13_detail_design/03-dispatcher.md)、[D06](../../13_detail_design/06-abi-surface.md)、[D16](../../13_detail_design/16-platform-rendering.md)、[D21](../../13_detail_design/21-wrapper-python.md)
- 分類・採否: 選択要。推奨はhost停止通知による未完了結果。代替は利用者設定timeoutを併用。任意の秒数・進行中cleanupの強制中断は今回定めない。

> UI処理を実行できないことが確定した場合は、同期終了要求を無期限に待ち続けず、未完了と待機終了理由を呼出し側が識別できること。待機終了を資源解放完了として扱わないこと。

要求testへの影響: [TC-THREAD-01](../../30_release_test/requirement-test-matrix.md#case-tc-thread-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-011"></a>
## SEP-RQ-011 ABI F32要求の未対応範囲

- 対応する現行要求: [RQ-04-002](../../11_requirements/RQ-04_abi-boundary.md#rq-04-002)
- 由来: [B01](../../12_design/BD-01_value-types-upper-design.md)、[S03](../../12_design/abi-spec.md)、[D07](../../13_detail_design/07-property-notification.md)
- 分類・採否: 差異解決要。推奨は現行F32要求を保持し、F32利用APIを未対応として計画化。代替はF32の対象段階を要求承認で変更。型宣言があるだけで充足扱いにしない。

> ABIで単精度と倍精度の浮動小数値を受け渡せること。初期Propertyの型集合とABI全体の対応範囲を区別すること。

要求testへの影響: [TC-ABI-REP-01](../../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-012"></a>
## SEP-RQ-012 初期Propertyの値型範囲

- 対応する現行要求: [RQ-02-002](../../11_requirements/RQ-02_core-features.md#rq-02-002)
- 由来: [B01](../../12_design/BD-01_value-types-upper-design.md)、[D07](../../13_detail_design/07-property-notification.md)
- 分類・採否: 補足案。OBJECT予約ID、bit配置、wire型はDD。文字列と数値の内蔵変換は将来候補のまま。

> 初期Propertyで真偽値、符号付き64-bit整数、有効なUTF-8文字列、倍精度浮動小数値、規定範囲の十進値を利用できること。初期範囲にnullable値・任意言語object・カスタム構造体値を含めず、それらをゼロや空文字列へ読み替えないこと。

要求testへの影響: [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-013"></a>
## SEP-RQ-013 F64の同値と保持結果

- 対応する現行要求: [RQ-02-002](../../11_requirements/RQ-02_core-features.md#rq-02-002)
- 由来: [B01](../../12_design/BD-01_value-types-upper-design.md)、[D07](../../13_detail_design/07-property-notification.md)
- 分類・採否: 補足案。全NaN同値は現行承認済みBDの意味を継承する要求案。有限値限定は利用者検証へ委ねる。

> 倍精度値はNaNと無限大を受け入れること。同符号の無限大、正負のゼロ、NaN同士はそれぞれ同値とし、同値設定では既存の格納表現を変えず変更通知を出さないこと。

要求testへの影響: [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-ABI-REP-01](../../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-014"></a>
## SEP-RQ-014 十進値の範囲と同値

- 対応する現行要求: [RQ-02-002](../../11_requirements/RQ-02_core-features.md#rq-02-002)
- 由来: [B01](../../12_design/BD-01_value-types-upper-design.md)、[D07](../../13_detail_design/07-property-notification.md)
- 分類・採否: 補足案。数値の提供範囲はRQ、8-byteのbit配置と比較作業域はDD。

> 十進値は符号・0～31のscale・0～2^58−1の係数で表せる範囲を正確に扱い、範囲外を黙って丸めないこと。1.0と1.00、正負のゼロは数値として同値とし、同値設定では既存の格納表現と通知状態を維持すること。

要求testへの影響: [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-ABI-REP-01](../../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-015"></a>
## SEP-RQ-015 UTF-8文字列の受渡し結果

- 対応する現行要求: [RQ-04-003](../../11_requirements/RQ-04_abi-boundary.md#rq-04-003)
- 由来: [B01](../../12_design/BD-01_value-types-upper-design.md)、[D05](../../13_detail_design/05-boundary-values-errors.md)、[S03](../../12_design/abi-spec.md)
- 分類・採否: 補足案。pointerのnull/length組合せとencode/decode方式はDD。

> 埋め込みNULを含む有効なUTF-8文字列を指定長のまま扱えること。不正UTF-8を拒否し、暗黙のUnicode正規化や別文字への置換をしないこと。

要求testへの影響: [TC-ABI-REP-01](../../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-OWN-01](../../30_release_test/requirement-test-matrix.md#case-tc-own-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-016"></a>
## SEP-RQ-016 buffer不足時の結果

- 対応する現行要求: [RQ-04-003](../../11_requirements/RQ-04_abi-boundary.md#rq-04-003)、[RQ-05-004](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-004)
- 由来: [D05](../../13_detail_design/05-boundary-values-errors.md)、[S03](../../12_design/abi-spec.md)
- 分類・採否: 補足案。caller buffer・出力値の具体形はDD。

> 出力bufferの容量が不足する場合は必要容量を知ることができ、bufferへ一部だけを書き込まないこと。容量照会と次の読取りの間の同一snapshot保証があるかをAPIごとに判断できること。

要求testへの影響: [TC-OWN-01](../../30_release_test/requirement-test-matrix.md#case-tc-own-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-017"></a>
## SEP-RQ-017 error詳細を生成できない失敗

- 対応する現行要求: [RQ-04-006](../../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../../11_requirements/RQ-04_abi-boundary.md#rq-04-007)
- 由来: [B27](../../12_design/BD-27_error-object-upper-design.md)、[D05](../../13_detail_design/05-boundary-values-errors.md)、[D06](../../13_detail_design/06-abi-surface.md)
- 分類・採否: 補足案。messageを機械識別に使わない既存RQ-15-011の内容をRQ-04へ統合する。

> 失敗詳細を生成できない資源不足時にも元の失敗分類を保持すること。error snapshotは保持中に別操作の結果で書き換わらず、未知のstatus/domain/codeも元の数値として取得できること。

要求testへの影響: [TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-018"></a>
## SEP-RQ-018 利用者定義エラーの自動採番

- 対応する現行要求: [RQ-04-007](../../11_requirements/RQ-04_abi-boundary.md#rq-04-007)
- 由来: [D06](../../13_detail_design/06-abi-surface.md)、[D18](../../13_detail_design/18-wrapper-cpp.md)、[D19](../../13_detail_design/19-wrapper-csharp.md)、[D20](../../13_detail_design/20-wrapper-rust.md)、[D21](../../13_detail_design/21-wrapper-python.md)
- 分類・採否: 補足案。ABI statusへユーザー値を埋め込まない。namespace範囲・生成manifestはDD。

> 利用者が失敗理由に名前付きの定義と自動採番を利用できること。公開した理由の識別値と意味を再採番・再利用せず、共通定義を使う場合は言語間で一致すること。

要求testへの影響: [TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-019"></a>
## SEP-RQ-019 機能照会と互換性

- 対応する現行要求: [RQ-04-009](../../11_requirements/RQ-04_abi-boundary.md#rq-04-009)、[RQ-04-010](../../11_requirements/RQ-04_abi-boundary.md#rq-04-010)
- 由来: [B28](../../12_design/BD-28_abi-release-version-upper-design.md)、[D06](../../13_detail_design/06-abi-surface.md)
- 分類・採否: 補足案。major prefixやsymbol探索はDD。

> 提供されない機能を利用前に識別でき、未知の機能識別子は未対応として扱えること。公開済み機能識別子とstatusの意味を再利用せず、wrapperが必要とするRuntime互換条件を判断できること。

要求testへの影響: [TC-ABI-COMP-01](../../30_release_test/requirement-test-matrix.md#case-tc-abi-comp-01)、[TC-ABI-COMP-02](../../30_release_test/requirement-test-matrix.md#case-tc-abi-comp-02)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-020"></a>
## SEP-RQ-020 購読の初期通知・順序

- 対応する現行要求: [RQ-07-006](../../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-05-006](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)
- 由来: [S04](../../12_design/mvvm-semantics.md)、[D04](../../13_detail_design/04-callback-lifecycle.md)、[D07](../../13_detail_design/07-property-notification.md)
- 分類・採否: 補足案。subscriber snapshotとgenerationはDD。解除済み対象は既存RQ-05-007に従う。

> Property購読時は初期変更通知を送らず、現在値を読取り操作で得られること。同じ配信対象の購読者は登録順に通知され、配信中に追加した購読はその配信回に割り込まないこと。

要求testへの影響: [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](../../30_release_test/requirement-test-matrix.md#case-tc-callback-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-021"></a>
## SEP-RQ-021 同値設定時の検証

- 対応する現行要求: [RQ-02-002](../../11_requirements/RQ-02_core-features.md#rq-02-002)
- 由来: [S04](../../12_design/mvvm-semantics.md)、[D07](../../13_detail_design/07-property-notification.md)、[D09](../../13_detail_design/09-validation-input.md)
- 分類・採否: 補足案。候補比較とvalidator呼出し順はDD。

> 現在値と同値を設定する場合も登録済みの値検証を適用すること。受理された同値設定は現在値の格納表現を置き換えず、値変更通知を生じさせないこと。

要求testへの影響: [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-022"></a>
## SEP-RQ-022 長期保留の診断設定

- 対応する現行要求: [RQ-07-008](../../11_requirements/RQ-07_binding.md#rq-07-008)
- 由来: [B03](../../12_design/BD-03_runtime-notification-upper-design.md)、[D08](../../13_detail_design/08-binding-graph.md)
- 分類・採否: 補足案。DD-BIND-11の承認済み意味を要求へ分離。単調時計・slice計数はDD。同期callbackを強制中断する機能は含めない。

> 長期保留の診断閾値を利用者が設定・無効化でき、既定値を5秒とすること。長期保留の検出だけを理由に更新連鎖を停止・破棄せず、同じ連鎖の同じ保留状態を重複診断し続けないこと。

要求testへの影響: [TC-BIND-01](../../30_release_test/requirement-test-matrix.md#case-tc-bind-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-023"></a>
## SEP-RQ-023 非収束診断と回復

- 対応する現行要求: [RQ-07-008](../../11_requirements/RQ-07_binding.md#rq-07-008)
- 由来: [B03](../../12_design/BD-03_runtime-notification-upper-design.md)、[D08](../../13_detail_design/08-binding-graph.md)
- 分類・採否: 補足案。hash/snapshot照合・履歴上限・再開chain生成はDD。診断の一般製品化範囲はRQ-09-035の将来候補と区別する。

> 非収束と判断できた更新連鎖のみを停止し、独立した連鎖と既に確定した値を維持すること。長期保留と非収束停止を区別して観測でき、保留更新の破棄と停止連鎖の再開を明示的に選べること。

要求testへの影響: [TC-BIND-01](../../30_release_test/requirement-test-matrix.md#case-tc-bind-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-024"></a>
## SEP-RQ-024 Validator登録条件

- 対応する現行要求: [RQ-02-002](../../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-05-006](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)
- 由来: [S03](../../12_design/abi-spec.md)、[D09](../../13_detail_design/09-validation-input.md)
- 分類・採否: 補足案。closure保持、typed callback signatureはDD。

> 一つのPropertyに登録できる同期Validatorは最大一つとし、登録だけでは現在値を再検証しないこと。重複登録失敗で既存登録を失わないこと。

要求testへの影響: [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](../../30_release_test/requirement-test-matrix.md#case-tc-callback-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-025"></a>
## SEP-RQ-025 判定callback中の禁止操作

- 対応する現行要求: [RQ-05-009](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)
- 由来: [S03](../../12_design/abi-spec.md)、[D04](../../13_detail_design/04-callback-lifecycle.md)、[D09](../../13_detail_design/09-validation-input.md)
- 分類・採否: 補足案。destroy/startなど他のcallback制約は種別表で個別化し、一律禁止へ広げない。

> ValidatorとTextBox入力判定callbackでは同じRuntimeのUI状態変更と自己登録解除を許さず、禁止操作を状態変更前に拒否すること。入力拒否とcallback自体の異常を区別できること。

要求testへの影響: [TC-CALLBACK-01](../../30_release_test/requirement-test-matrix.md#case-tc-callback-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-026"></a>
## SEP-RQ-026 TextBox入力三値の結果

- 対応する現行要求: [RQ-09-005](../../11_requirements/RQ-09_ui-scope.md#rq-09-005)
- 由来: [S04](../../12_design/mvvm-semantics.md)、[D09](../../13_detail_design/09-validation-input.md)、[D14](../../13_detail_design/14-textbox-ime.md)
- 分類・採否: 補足案。key/paste/drop/ATなど利用可能な入力経路に共通適用し、個々の入力機能の採用を追加しない。

> Acceptは候補を確定して伝播し、Incompleteは編集文字列と選択位置に保持して確定値へ伝播しないこと。通常Rejectは従前の編集値・選択を維持し理由を示すこと。IME commitのRejectは候補を未受理編集値として保持すること。

要求testへの影響: [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-VERT-03](../../30_release_test/requirement-test-matrix.md#case-tc-vert-03)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-027"></a>
## SEP-RQ-027 TextBox自身のProperty検証拒否

- 対応する現行要求: [RQ-09-005](../../11_requirements/RQ-09_ui-scope.md#rq-09-005)
- 由来: [D09](../../13_detail_design/09-validation-input.md)、[D14](../../13_detail_design/14-textbox-ime.md)
- 分類・採否: 選択要。推奨Bは編集候補保持＋確定値旧値＋専用拒否理由。代替Aは編集前へ戻す、Cは新状態追加。既存未決DD-TEXT-09を確定扱いにしない。

> 入力判定でAcceptされた候補をTextBoxの確定値検証が拒否した場合、確定値とViewModel値を維持し、拒否された候補と理由を利用者が確認・修正できること。

要求testへの影響: [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-028"></a>
## SEP-RQ-028 IME cancel後の入力理由

- 対応する現行要求: [RQ-09-005](../../11_requirements/RQ-09_ui-scope.md#rq-09-005)
- 由来: [B24](../../12_design/BD-24_ime-edit-session-upper-design.md)、[D14](../../13_detail_design/14-textbox-ime.md)
- 分類・採否: 選択要。推奨は開始snapshotの状態/理由も復元。代替はcancel専用の入力結果を追加。既承認は文字列・位置復元まで。

> IMEの明示cancelではcomposition開始直前の編集文字列・caret・selectionと入力状態・理由へ戻り、確定値検証とBinding伝播を起こさないこと。

要求testへの影響: [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-029"></a>
## SEP-RQ-029 IME commit準備失敗

- 対応する現行要求: [RQ-09-005](../../11_requirements/RQ-09_ui-scope.md#rq-09-005)
- 由来: [D14](../../13_detail_design/14-textbox-ime.md)、[D16](../../13_detail_design/16-platform-rendering.md)
- 分類・採否: 選択要。Core事前退避とAdapter再送保証を比較し、OS別に成立を確認する。無限のOOM下でも全入力保持という実現不能な無条件保証は追加しない。

> IME確定候補の処理準備が失敗した場合、候補を回復可能に保持できたかと再試行方法を利用者が判断できること。保持できなかった候補を処理成功として扱わないこと。

要求testへの影響: [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-030"></a>
## SEP-RQ-030 外部更新と編集中の値

- 対応する現行要求: [RQ-09-005](../../11_requirements/RQ-09_ui-scope.md#rq-09-005)
- 由来: [S03](../../12_design/abi-spec.md)、[D14](../../13_detail_design/14-textbox-ime.md)
- 分類・採否: 補足案。session identity・Adapter終了順はDD。

> 外部から異なる確定値が設定された場合は編集値をその値へ更新し、入力状態・理由を初期化すること。旧IME sessionの遅延イベントでその値を再び上書きしないこと。

要求testへの影響: [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-BIND-01](../../30_release_test/requirement-test-matrix.md#case-tc-bind-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-031"></a>
## SEP-RQ-031 Command実行受付と結果

- 対応する現行要求: [RQ-06-004](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)
- 由来: [S03](../../12_design/abi-spec.md)、[S04](../../12_design/mvvm-semantics.md)、[D10](../../13_detail_design/10-async-command.md)
- 分類・採否: 補足案。operation record確保やstart部分起動時のcleanupはDD/wrapper。

> Commandの実行受付成功と業務処理の成功・失敗を区別できること。受付後の開始処理の失敗はその実行の失敗として観測でき、受付前の失敗では実行を開始しないこと。

要求testへの影響: [TC-CMD-01](../../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-032"></a>
## SEP-RQ-032 取消・close・handle破棄の差

- 対応する現行要求: [RQ-06-004](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)、[RQ-05-002](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)
- 由来: [D10](../../13_detail_design/10-async-command.md)、[B26](../../12_design/BD-26_async-command-progress-upper-design.md)
- 分類・採否: 補足案。state/flag/IDでの処理はDD。screen ownerを新たにAPI化しない。

> 同じ実行への重複取消と終端済みの同じ実行への取消は追加効果を起こさず、古い実行への誤適用を防ぐこと。Commandの利用終了は新規実行を止め取消を要求し、worker終了をUI threadで待たないこと。所有参照の破棄だけでは利用終了を要求しないこと。

要求testへの影響: [TC-CMD-01](../../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-033"></a>
## SEP-RQ-033 Detached subtreeの利用

- 対応する現行要求: [RQ-09-003](../../11_requirements/RQ-09_ui-scope.md#rq-09-003)、[RQ-05-003](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)
- 由来: [B04](../../12_design/BD-04_ui-tree-lifetime-upper-design.md)、[D11](../../13_detail_design/11-ui-tree-lifetime.md)
- 分類・採否: 補足案。APIごとのDetached制約は別表へ記し、Runtime停止後の再所属は許さない。

> 親から切り離しても独立した所有参照を持つ対象とその子構造をCore上で利用できること。切離しだけで通常のProperty・Binding・Subscriptionを終了させず、描画・入力対象からは外すこと。

要求testへの影響: [TC-TREE-01](../../30_release_test/requirement-test-matrix.md#case-tc-tree-01)、[TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-034"></a>
## SEP-RQ-034 Tree変更の拒否結果

- 対応する現行要求: [RQ-09-003](../../11_requirements/RQ-09_ui-scope.md#rq-09-003)
- 由来: [B04](../../12_design/BD-04_ui-tree-lifetime-upper-design.md)、[D11](../../13_detail_design/11-ui-tree-lifetime.md)
- 分類・採否: 補足案。初期atomic reparent非提供の範囲を明示し、remove/addの方式はDD。peer再生成の失敗結果は別途未決とする。

> 複数親、親子循環、異なるRuntime所属を作る親子変更を拒否すること。変更失敗で既存の親子・値・登録を部分変更せず、Detached対象の再所属失敗時はDetached状態を維持すること。

要求testへの影響: [TC-TREE-01](../../30_release_test/requirement-test-matrix.md#case-tc-tree-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-035"></a>
## SEP-RQ-035 StackPanel公開設定と既定

- 対応する現行要求: [RQ-09-007](../../11_requirements/RQ-09_ui-scope.md#rq-09-007)
- 由来: [B06](../../12_design/BD-06_layout-upper-design.md)、[D12](../../13_detail_design/12-layout.md)、[D15](../../13_detail_design/15-controls-v1.md)
- 分類・採否: 範囲確認要。方向/Alignの提供はRQへ補足。Vertical/Stretch等の初期値は設定省略時の結果を保証するかを別に確認し、保証するならRQへ追加。計算式はDDへ。

> StackPanelの配置方向を縦・横から選び、子の配置位置と伸長を設定できること。

要求testへの影響: [TC-LAYOUT-01](../../30_release_test/requirement-test-matrix.md#case-tc-layout-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-036"></a>
## SEP-RQ-036 有効性とFocusの共通結果

- 対応する現行要求: [RQ-09-009](../../11_requirements/RQ-09_ui-scope.md#rq-09-009)、[RQ-10-008](../../11_requirements/RQ-10_accessibility.md#rq-10-008)
- 由来: [B08](../../12_design/BD-08_focus-upper-design.md)、[D13](../../13_detail_design/13-input-focus.md)、[D15](../../13_detail_design/15-controls-v1.md)
- 分類・採否: 範囲確認要。ancestor enabledとFocusable既定の外部保証はRQ。Focus復帰候補・同一要求no-opはDD。

> Control自身またはLogical祖先が無効ならInput・Focus・Button操作を受け付けず、子自身の設定値を変更しないこと。Focus対象を失った場合は無効な対象を残さず、別Controlを暗黙に選ばないこと。

要求testへの影響: [TC-FOCUS-01](../../30_release_test/requirement-test-matrix.md#case-tc-focus-01)、[TC-ACC-07](../../30_release_test/requirement-test-matrix.md#case-tc-acc-07)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-037"></a>
## SEP-RQ-037 初期機能の除外とButton未接続

- 対応する現行要求: [RQ-09-006](../../11_requirements/RQ-09_ui-scope.md#rq-09-006)、[RQ-09-010](../../11_requirements/RQ-09_ui-scope.md#rq-09-010)
- 由来: [D13](../../13_detail_design/13-input-focus.md)、[D15](../../13_detail_design/15-controls-v1.md)
- 分類・採否: 選択要。key割当除外は利用者指示を維持。CommandなしButtonは表示・無操作を推奨、生成拒否が代替。visibility/直接Click非提供は既存要求への影響を確認する独立範囲論点。

> 特定キーによるFocus移動・shortcut/action bindingは初期必須機能に含めないこと。ButtonへCommandが設定されていない場合は表示を許し、起動操作は実行しないこと。

要求testへの影響: [TC-INPUT-01](../../30_release_test/requirement-test-matrix.md#case-tc-input-01)、[TC-CMD-01](../../30_release_test/requirement-test-matrix.md#case-tc-cmd-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-038"></a>
## SEP-RQ-038 Focus通知と入力route

- 対応する現行要求: [RQ-09-009](../../11_requirements/RQ-09_ui-scope.md#rq-09-009)、[RQ-09-010](../../11_requirements/RQ-09_ui-scope.md#rq-09-010)
- 由来: [B07](../../12_design/BD-07_input-event-boundary-upper-design.md)、[B08](../../12_design/BD-08_focus-upper-design.md)、[D13](../../13_detail_design/13-input-focus.md)
- 分類・採否: 補足案。bubble/tunnelは未決の配送方式として比較し、初期bubble推奨を製品要求の確定へ読み替えない。

> Focus変更後の状態を観測でき、旧対象のFocusLostを新対象のFocusGainedより先に扱うこと。入力対象がないkeyboard/textイベントを後の別対象へ遅延配送しないこと。

要求testへの影響: [TC-FOCUS-01](../../30_release_test/requirement-test-matrix.md#case-tc-focus-01)、[TC-INPUT-01](../../30_release_test/requirement-test-matrix.md#case-tc-input-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-039"></a>
## SEP-RQ-039 Platform資源喪失と終了

- 対応する現行要求: [RQ-11-009](../../11_requirements/RQ-11_platform.md#rq-11-009)
- 由来: [B12](../../12_design/BD-12_rendering-platform-upper-design.md)、[D16](../../13_detail_design/16-platform-rendering.md)
- 分類・採否: 補足案。close accept/veto/defer、再表示、fallbackの採否は未決の選択肢を保持する。

> 表示資源の喪失でCoreの確定済みUI状態を失わず、失敗した描画を成功として扱わないこと。OSのWindow close要求だけでRuntime全体を暗黙終了しないこと。

要求testへの影響: [TC-PLAT-01](../../30_release_test/requirement-test-matrix.md#case-tc-plat-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-040"></a>
## SEP-RQ-040 Accessibility identityと操作境界

- 対応する現行要求: [RQ-10-006](../../11_requirements/RQ-10_accessibility.md#rq-10-006)、[RQ-10-007](../../11_requirements/RQ-10_accessibility.md#rq-10-007)、[RQ-10-009](../../11_requirements/RQ-10_accessibility.md#rq-10-009)
- 由来: [B13](../../12_design/BD-13_accessibility-upper-design.md)、[D17](../../13_detail_design/17-accessibility.md)
- 分類・採否: 補足案。OS proxyによるCore延命なし、snapshot、provider thread marshalはDD。

> 同じControlの生存中はAccessibility上の対象を安定して識別でき、終了済み対象へのquery/actionを新対象へ誤適用しないこと。参照するだけでkeyboard focusを変更せず、操作は対応する通常Control処理と同じ結果となること。

要求testへの影響: [TC-ACC-PLAT-01](../../30_release_test/requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-ACC-05](../../30_release_test/requirement-test-matrix.md#case-tc-acc-05)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-041"></a>
## SEP-RQ-041 Accessibility nameの空欄

- 対応する現行要求: [RQ-10-003](../../11_requirements/RQ-10_accessibility.md#rq-10-003)、[RQ-09-011](../../11_requirements/RQ-09_ui-scope.md#rq-09-011)
- 由来: [B13](../../12_design/BD-13_accessibility-upper-design.md)、[D15](../../13_detail_design/15-controls-v1.md)、[D17](../../13_detail_design/17-accessibility.md)
- 分類・採否: 範囲確認要。既存RQ-10-003は自動命名を未定義としているため、非提供の確定は今回要求承認を必要とする。

> Accessibility nameを表示文字列から独立して設定でき、未設定または空欄の場合は表示値等から自動合成しないこと。

要求testへの影響: [TC-ACC-02](../../30_release_test/requirement-test-matrix.md#case-tc-acc-02)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-042"></a>
## SEP-RQ-042 C++の例外無効buildとhandler

- 対応する現行要求: [RQ-03-001](../../11_requirements/RQ-03_language-support.md#rq-03-001)
- 由来: [D18](../../13_detail_design/18-wrapper-cpp.md)
- 分類・採否: 補足案。std::exception_ptr、noexcept、intrusive registryはDD。

> 例外無効buildでも共通機能を利用でき、失敗を結果として扱えること。例外有効buildではcallback例外の利用者handlerを設定でき、そのhandlerによってABI境界やcallback固有の失敗結果を壊さないこと。

要求testへの影響: [TC-WRAP-CPP-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-043"></a>
## SEP-RQ-043 C#の明示破棄と競合

- 対応する現行要求: [RQ-03-002](../../11_requirements/RQ-03_language-support.md#rq-03-002)
- 由来: [D19](../../13_detail_design/19-wrapper-csharp.md)
- 分類・採否: 補足案。SafeHandle最終採否、lease、GCHandle rootはDD。C++の例外無効build要件をC#へ継承しない。

> 明示破棄を繰り返しても二重解放せず、native呼出しと破棄が競合しても呼出し中の対象を失わないこと。callbackの例外処理を利用者が設定でき、元の失敗を識別できること。

要求testへの影響: [TC-WRAP-CS-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-cs-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-044"></a>
## SEP-RQ-044 Rust safe APIとpanic限界

- 対応する現行要求: [RQ-03-003](../../11_requirements/RQ-03_language-support.md#rq-03-003)
- 由来: [D20](../../13_detail_design/20-wrapper-rust.md)
- 分類・採否: 補足案。Send/Sync markerとtrait bound、Drop ticket、unsafe実装はDD。

> 通常のsafe API利用で借用参照を解放したり、期限外に利用したりせず共通機能を扱えること。捕捉可能なpanicをABIへ伝播させず、abortや二重panicで終了するprocessのcleanup完遂は保証対象に含めないこと。

要求testへの影響: [TC-WRAP-RS-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-rs-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-045"></a>
## SEP-RQ-045 Python明示終了とowner回収

- 対応する現行要求: [RQ-03-004](../../11_requirements/RQ-03_language-support.md#rq-03-004)
- 由来: [D21](../../13_detail_design/21-wrapper-python.md)
- 分類・採否: 補足案。全__del__の実行保証を要求しない。root registryがownerを強保持する循環を作らない設計はDD。

> 明示終了とscope終了による解放を利用できること。管理ownerが回収された場合もnative資源の後始末を管理経路へ引き渡し、owner自身の回収が資源を管理不能にしないこと。

要求testへの影響: [TC-WRAP-PY-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-046"></a>
## SEP-RQ-046 Python通常終了・例外と保証外

- 対応する現行要求: [RQ-03-004](../../11_requirements/RQ-03_language-support.md#rq-03-004)、[RQ-02-013](../../11_requirements/RQ-02_core-features.md#rq-02-013)
- 由来: [D21](../../13_detail_design/21-wrapper-python.md)
- 分類・採否: 境界選択要。管理外threadの検知はhost登録/entry wrapper契約を前提にする案。移植可能な検知元が未確定の経路は設計gapとし、atexit単独やbuiltin上書きで保証済みとしない。

> 明示終了、管理対象のmain/host処理終了、未捕捉main例外、SystemExit、host UI loop終了、wrapper所有worker例外の各経路で終了を要求でき、管理対象UI threadだけが残って通常終了を妨げないこと。callback例外は種別固有の失敗として処理すること。os._exit、fatal error、強制killなどprocessが即時終了する経路ではin-process cleanup完遂を保証しないこと。

要求testへの影響: [TC-WRAP-PY-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-047"></a>
## SEP-RQ-047 GIL非依存とsupport区分

- 対応する現行要求: [RQ-06-003](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)
- 由来: [D21](../../13_detail_design/21-wrapper-python.md)
- 分類・採否: 既存RQの意味維持。全APIの任意thread化やPython free-threaded版の正式support追加は含めない。

> worker変更を許す共有状態は明示的に同期され、変更を許さない状態は読取専用または値として利用できること。GILの存在を正しさの前提にしないこと。

要求testへの影響: [TC-THREAD-01](../../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-WRAP-PY-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-048"></a>
## SEP-RQ-048 診断・解析の能力と限界

- 対応する現行要求: [RQ-15-007](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-007)、[RQ-15-013](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-013)
- 由来: [B03](../../12_design/BD-03_runtime-notification-upper-design.md)、[B15](../../12_design/BD-15_python-lint-mcp-upper-design.md)、[D04](../../13_detail_design/04-callback-lifecycle.md)、[D21](../../13_detail_design/21-wrapper-python.md)
- 分類・採否: 補足案。callback marker、base chain-up、禁止違反検知の拡張は別の未採用候補。DD-24を完了・必須機能にしない。

> 静的に解決できる同一ControlのPropertyアクセスを診断できること。解析不能な動的箇所を安全確認済みと表示せず、同じ入力・設定に同じ診断を返し、古い解析結果を最新として返さないこと。

要求testへの影響: [TC-TRACE-01](../../30_release_test/requirement-test-matrix.md#case-tc-trace-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-049"></a>
## SEP-RQ-049 検証状態と正式support

- 対応する現行要求: [RQ-11-001](../../11_requirements/RQ-11_platform.md#rq-11-001)、[RQ-11-002](../../11_requirements/RQ-11_platform.md#rq-11-002)、[RQ-12-010](../../11_requirements/RQ-12_build-distribution.md#rq-12-010)
- 由来: [B12](../../12_design/BD-12_rendering-platform-upper-design.md)、[B16](../../12_design/BD-16_build-ci-target-matrix-upper-design.md)、[D22](../../13_detail_design/22-build-distribution.md)
- 分類・採否: 補足案。正式matrix・最低OS/TFM/MSRV/Python版は個別採否待ちを保持。

> build成功、Runtime動作確認、hardware profile確認の範囲を区別して公表すること。未検証profileや他architectureへ実機supportを拡大表示しないこと。

要求testへの影響: [TC-TARGET-ARMV7-01](../../30_release_test/requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-A64-01](../../30_release_test/requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-BOARD-01](../../30_release_test/requirement-test-matrix.md#case-tc-target-board-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-050"></a>
## SEP-RQ-050 artifact選択・identity・取得

- 対応する現行要求: [RQ-12-001](../../11_requirements/RQ-12_build-distribution.md#rq-12-001)、[RQ-12-003](../../11_requirements/RQ-12_build-distribution.md#rq-12-003)、[RQ-12-005](../../11_requirements/RQ-12_build-distribution.md#rq-12-005)
- 由来: [B17](../../12_design/BD-17_python-package-distribution-upper-design.md)、[B20](../../12_design/BD-20_rust-cargo-package-upper-design.md)、[B29](../../12_design/BD-29_artifact-provenance-upper-design.md)、[B30](../../12_design/BD-30_artifact-distribution-upper-design.md)、[D22](../../13_detail_design/22-build-distribution.md)
- 分類・採否: 補足案。catalog/cache/key/schemaやfetch commandはDD/SD。bit-for-bit再現build保証を追加しない。

> 利用するtarget/profile/ABI/featureに適合するnative artifactを選択し、不適合を利用前に拒否すること。同じnative binaryは言語package間で同じidentityを保つこと。通常導入時に既存の固定版を暗黙更新せず、取得・更新を明示でき、検証済みartifactをoffline利用できること。

要求testへの影響: [TC-ARTIFACT-01](../../30_release_test/requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-RS-01](../../30_release_test/requirement-test-matrix.md#case-tc-pkg-rs-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-051"></a>
## SEP-RQ-051 artifactの信頼と検証失敗

- 対応する現行要求: [RQ-15-016](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-016)
- 由来: [B29](../../12_design/BD-29_artifact-provenance-upper-design.md)、[B31](../../12_design/BD-31_artifact-signing-trust-upper-design.md)、[D22](../../13_detail_design/22-build-distribution.md)
- 分類・採否: RQ-12へ分割移管案。期限切れmetadataの利用可否と緊急失効の運用条件は別選択要。暗号方式はSD。

> 署名、内容digest、適合条件と既存の信頼根拠を検証してから公式artifactを利用すること。取得先で初めて得た鍵を自動信頼せず、未知・失効・検証不能なら拒否すること。認証済み鍵更新と明示trust bundleによるoffline検証を提供すること。

要求testへの影響: [TC-ARTIFACT-01](../../30_release_test/requirement-test-matrix.md#case-tc-artifact-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-052"></a>
## SEP-RQ-052 manifestの互換・曖昧性

- 対応する現行要求: [RQ-15-016](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-016)
- 由来: [B32](../../12_design/BD-32_artifact-manifest-schema-upper-design.md)、[D22](../../13_detail_design/22-build-distribution.md)
- 分類・採否: RQ-12へ分割移管案。field名、canonical encoding、parser順序と上限値はDD/SD。

> 未知schema major、未知必須項目、重複・曖昧で一意に解釈できないmanifestを拒否すること。未知optional項目は署名検証後に無視できるが、target・ABI・信頼判定に必要な情報をoptionalとして扱わないこと。

要求testへの影響: [TC-ARTIFACT-01](../../30_release_test/requirement-test-matrix.md#case-tc-artifact-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-053"></a>
## SEP-RQ-053 license/noticeの継承

- 対応する現行要求: [RQ-15-014](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-014)
- 由来: [B21](../../12_design/BD-21_license-upper-design.md)、[D22](../../13_detail_design/22-build-distribution.md)
- 分類・採否: 意味維持でRQ-12へ移管案。商標/CLA/docs/assets/SBOMの未決条件を今回採用しない。

> first-party source、header、wrapper、source package、binary SDKへ既承認のApache-2.0方針を適用し、再配布で必要なlicense/noticeと変更表示を保持すること。third-partyのlicense条件を独立して維持すること。

要求testへの影響: [TC-LICENSE-01](../../30_release_test/requirement-test-matrix.md#case-tc-license-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-054"></a>
## SEP-RQ-054 support lifecycleとEOL記録

- 対応する現行要求: [RQ-15-015](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-015)
- 由来: [B22](../../12_design/BD-22_support-lifecycle-upper-design.md)、[D22](../../13_detail_design/22-build-distribution.md)
- 分類・採否: 意味維持＋例外/継続アクセス補足案。SLA/LTS追加や保存年限を勝手に決めない。

> ABI majorとtarget profileごとに保守状態を公表すること。最新安定majorを通常保守し、直前majorは新安定major後24か月のsecurity/重大修正に限定すること。profile非推奨は原則12か月前に通知し、security上の短縮時は理由と影響を示すこと。EOL後も最終公開物・support表・告知を参照できること。

要求testへの影響: [TC-SUPPORT-01](../../30_release_test/requirement-test-matrix.md#case-tc-support-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-055"></a>
## SEP-RQ-055 Collection stable identity

- 対応する現行要求: [RQ-09-019](../../11_requirements/RQ-09_ui-scope.md#rq-09-019)
- 由来: [B33](../../12_design/BD-33_collection-grid-upper-design.md)
- 分類・採否: 補足案。Gridのcolumn IDはGrid採用時の候補に留め、TableViewを今回必須にしない。

> 次段階Collectionは順序位置から独立した項目IDを保持し、削除済みIDを別項目へ再利用しないこと。Controlが編集対象を識別する際も位置変化で別項目へ取り違えないこと。

要求testへの影響: [TC-COL-01](../../30_release_test/requirement-test-matrix.md#case-tc-col-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-056"></a>
## SEP-RQ-056 Selection順序と拒否

- 対応する現行要求: [RQ-09-020](../../11_requirements/RQ-09_ui-scope.md#rq-09-020)
- 由来: [B34](../../12_design/BD-34_selection-model-upper-design.md)
- 分類・採否: 補足案。後継の自動選択とmode縮退/navigationは未決候補を保持する。

> 次段階Selectionの公開順は現在のCollection順に従い、並べ替えで選択ID集合を変えないこと。Singleを破る複数選択の更新は全体を拒否し、項目削除でselected/currentからその項目を除くこと。

要求testへの影響: [TC-SEL-01](../../30_release_test/requirement-test-matrix.md#case-tc-sel-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-057"></a>
## SEP-RQ-057 Collection batchの観測結果

- 対応する現行要求: [RQ-09-019](../../11_requirements/RQ-09_ui-scope.md#rq-09-019)
- 由来: [B35](../../12_design/BD-35_collection-change-batch-upper-design.md)
- 分類・採否: 補足案。prepare/commit/reconcileはBD/DD。一般transactionへの拡張をしない。

> 無効なchange batchはCollection内容・順序・revision・Selection・公開通知を変更しないこと。有効なbatchは一つの更新として公開し、中間の不整合を外部callbackから観測させないこと。

要求testへの影響: [TC-COL-01](../../30_release_test/requirement-test-matrix.md#case-tc-col-01)、[TC-SEL-01](../../30_release_test/requirement-test-matrix.md#case-tc-sel-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-058"></a>
## SEP-RQ-058 text coverageと編集位置

- 対応する現行要求: [RQ-15-005](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-005)、[RQ-09-005](../../11_requirements/RQ-09_ui-scope.md#rq-09-005)
- 由来: [B23](../../12_design/BD-23_text-shaping-upper-design.md)、[D14](../../13_detail_design/14-textbox-ime.md)
- 分類・採否: 意味維持でRQ-09へ移管案。grapheme単位操作/RTL/他script/Unicode版は別採否待ち。内部byte/scalar/grapheme位置単位はDD。

> 初期版でUTF-8のLatin・日本語UI文字列を表示・編集でき、対応profileの文字・font依存条件を判断できること。

要求testへの影響: [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-PLAT-01](../../30_release_test/requirement-test-matrix.md#case-tc-plat-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-059"></a>
## SEP-RQ-059 性能測定と閾値

- 対応する現行要求: [RQ-13-001](../../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-001)、[RQ-13-011](../../11_requirements/RQ-13_nonfunctional-measurement.md#rq-13-011)
- 由来: [B12](../../12_design/BD-12_rendering-platform-upper-design.md)、[B16](../../12_design/BD-16_build-ci-target-matrix-upper-design.md)、[D22](../../13_detail_design/22-build-distribution.md)
- 分類・採否: 既存要求維持。代表workloadはtest計画、未承認数値をpass閾値へ追加しない。

> 対象profile、機能負荷、測定区間、revisionと各処理層の測定結果を記録できること。

要求testへの影響: [TC-NFR-01](../../30_release_test/requirement-test-matrix.md#case-tc-nfr-01)、[TC-NFR-06](../../30_release_test/requirement-test-matrix.md#case-tc-nfr-06)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-060"></a>
## SEP-RQ-060 文書承認とID移行

- 対応する現行要求: [RQ-16-003](../../11_requirements/RQ-16_document-governance.md#rq-16-003)、[RQ-16-004](../../11_requirements/RQ-16_document-governance.md#rq-16-004)
- 由来: [B00](../../12_design/BD-00_basic-design-resumption.md)、[S01](../../12_design/design-overview.md)、[D00](../../13_detail_design/00-decision-map.md)、[D01](../../13_detail_design/01-handle-registry.md)、[D02](../../13_detail_design/02-runtime-lifetime.md)、[D23](../../30_release_test/requirements-test-manifest.md)
- 分類・採否: 既存要求維持。RQ受入確認欄はrequirements-testへ移し、RQ08を再利用しない。

> 要求の変更承認、設計の変更承認、case設計、実装・実行状態を別々に追跡し、旧IDと新しい意味単位の対応を確認できること。

要求testへの影響: [TC-TRACE-01](../../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-DOC-01](../../30_release_test/requirement-test-matrix.md#case-tc-doc-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-061"></a>
## SEP-RQ-061 通知callbackの例外後の継続

- 対応する現行要求: [RQ-05-009](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-04-015](../../11_requirements/RQ-04_abi-boundary.md#rq-04-015)、[RQ-04-016](../../11_requirements/RQ-04_abi-boundary.md#rq-04-016)、[RQ-04-017](../../11_requirements/RQ-04_abi-boundary.md#rq-04-017)
- 由来: [D04](../../13_detail_design/04-callback-lifecycle.md)、[D18](../../13_detail_design/18-wrapper-cpp.md)、[D19](../../13_detail_design/19-wrapper-csharp.md)、[D20](../../13_detail_design/20-wrapper-rust.md)、[D21](../../13_detail_design/21-wrapper-python.md)
- 分類・採否: 補足案。abort/fatalは各言語の保証外条件に従う。trampoline/handler呼出し順はDD。

> 通知callbackで捕捉可能な言語例外が起きた場合、失敗を診断し、他の有効な通知対象の配信を継続すること。戻り値を持つcallbackでは各種別の失敗結果へ変換し、通常の入力拒否とcallback異常を区別できること。

要求testへの影響: [TC-CALLBACK-01](../../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-062"></a>
## SEP-RQ-062 単一操作の準備失敗

- 対応する現行要求: [RQ-04-006](../../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-07-009](../../11_requirements/RQ-07_binding.md#rq-07-009)
- 由来: [D05](../../13_detail_design/05-boundary-values-errors.md)、[S03](../../12_design/abi-spec.md)
- 分類・採否: 補足案。個別APIの成功境界に適用し、Command受付後の業務失敗とCollection一batchはそれぞれの契約を使う。

> 入力検証、必要な内容の保持、必須の結果資源の準備に失敗した操作は、その操作に属するCore状態・通知を部分更新しないこと。先に成功した独立操作と利用者callbackの副作用を自動的に取り消さないこと。

要求testへの影響: [TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

<a id="candidate-063"></a>
## SEP-RQ-063 同じBindingの重複接続

- 対応する現行要求: [RQ-07-007](../../11_requirements/RQ-07_binding.md#rq-07-007)
- 由来: [D08](../../13_detail_design/08-binding-graph.md)、[S04](../../12_design/mvvm-semantics.md)
- 分類・採否: 補足案。複数入力元拒否と重複の理由を区別するerror codeはDD-06既承認catalogに保持する。

> 同じsourceから同じtargetへの重複Binding接続を拒否し、失敗前の値・接続・通知を維持すること。

要求testへの影響: [TC-BIND-01](../../30_release_test/requirement-test-matrix.md#case-tc-bind-01)の対応assertionを承認後に追加・改訂する案。候補が未決の間は期待結果を確定しない。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
|---|---|---|---|
| 2026-10-17 | SEP-02 / 全要求候補 | — | 一括作成指示に基づくレビュー案。採用・正本適用は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 状態 |
|---|---|---|---|
| 0.1 | 2026-10-17 | 現行文書から責務分離・改訂文案・影響を作成 | レビュー待ち |

## 2026-10-17 適用結果

正本は [適用結果](../requirements-design-separation-application-result.md) と [正式要求ID移行](../../11_requirements/requirements-separation-id-migration.md) に記録した。以下の比較・引用・選択肢は作成時点の案として保持する。採用した選択は推奨案、未提示の実装詳細は後続課題として扱った。

### 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 全レビュー案の反映 | 利用者 | 「すべて反映して下さい。」により採用・正本適用を承認 |

### 改訂履歴

| 版 | 日付 | 内容 |
| --- | --- | --- |
| 0.2 | 2026-10-17 | 承認と正本適用結果を追記。作成時点の比較・引用を保持 |
