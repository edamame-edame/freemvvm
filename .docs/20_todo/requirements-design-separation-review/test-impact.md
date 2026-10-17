# SEP-03 / 要求test・単体検証・gateの改訂案

- 文書版: 0.2
- 状態: 全案反映承認済み・正本適用済み（2026-10-17）。具体化未了は適用結果を参照
- 作成日: 2026-10-17
- 入口: [一括レビュー索引](index.md)

[Requirements Test Manifest](../../30_release_test/requirements-test-manifest.md) v0.2の承認済みルールを適用する。要求の受入確認はtest成果物へ、内部構造の確認は対応unitへ分ける。以下はcase改訂案であり、実testを作成・実行した記録ではない。

## 現行caseと全要求の対応

[全RQ→test改訂対応](rq-test-map.md)は現行RQ全件を含み、[RQ ID移行](rq-id-map.md)と同じ集合を検査する。既存TC-*は論理case IDとして保持する。実test IDはUT-/REQT-/ABI-/WRAP-/INT-/SYS-/PLAT-/PKG-/REL-/VER-の検証境界を先頭に置く。1caseに複数RQがある場合は、assertionごとに元RQと採否状態を記録する。

## 仮要求候補ごとのcase変更案

同じ既存caseへ複数候補を追加する行は、別caseの大量増設を意味しない。期待値が別なら同じcase内の個別assertionかcase分割を選び、同じ前提・操作・期待・境界の重複だけを統合する。選択要の候補は要求採否待ち、Collection/Selectionは後続stage・DD gapとして記録する。

| 仮要求 | 既存case | 追加・改訂する操作と観測 | 適用条件 |
|---|---|---|---|
| [SEP-RQ-001 任意threadの参照取得・破棄](rq-candidates.md#candidate-001) | [TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../../30_release_test/requirement-test-matrix.md#case-tc-thread-01) | 同じ対象の二つの所有参照を作り、一方をworkerで破棄して他方の利用と遅延UI解放を観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-002 破棄済み参照への重複破棄](rq-candidates.md#candidate-002) | [TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01) | 破棄済みの同じ参照を再破棄し、別参照の有効性と対象値が変わらないことを観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-003 安全な参照を発行できない場合](rq-candidates.md#candidate-003) | [TC-CORE-OBJ-01](../../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01) | 発行不可状態を内部fixtureで作り、外部取得失敗と既存参照の誤解決不在を分けて確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-004 参照の種類・所属の拒否結果](rq-candidates.md#candidate-004) | [TC-CORE-OBJ-01](../../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01) | 対応RQの観測結果を個別assertionとして現行caseに照合する。無効な参照、期待する対象種類の不一致、異なるRuntime所属の組合せを呼出し側が失敗理由として区別でき、拒否によって対象状態を変更しないこと。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-005 破棄失敗時の参照保全](rq-candidates.md#candidate-005) | [TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-WRAP-CPP-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-RS-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-rs-01)、[TC-WRAP-PY-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01) | UI finalizer受付失敗を注入し、破棄失敗後も同じ所有を再試行できるかを確認する。自動回収ではticket移管をunit側で別検証する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-006 Runtime停止後の利用境界](rq-candidates.md#candidate-006) | [TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01) | Runtime終了後にUI操作・error読取り・残存参照解放を別々に行い、それぞれの可否を観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-007 Runtime終了の再要求](rq-candidates.md#candidate-007) | [TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01) | 終了完了後に再終了し、destroy/資源解放回数が増えず成功結果を保つことを確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-008 callback/event中の終了要求](rq-candidates.md#candidate-008) | [TC-THREAD-01](../../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-CALLBACK-01](../../30_release_test/requirement-test-matrix.md#case-tc-callback-01) | callbackから終了要求を出し、現在callback完了と未開始通常workとの前後関係を記録する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-009 終了要求の受付と完了](rq-candidates.md#candidate-009) | [TC-THREAD-01](../../30_release_test/requirement-test-matrix.md#case-tc-thread-01) | UI通常context、UI callback、workerの各同期/非同期呼出しで受付/完了と戻る時点を観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-010 停止したUI loopへの同期要求](rq-candidates.md#candidate-010) | [TC-THREAD-01](../../30_release_test/requirement-test-matrix.md#case-tc-thread-01) | host停止通知を同期要求と競合させ、未完了理由と待機終了を観測する。timeout選択が未決なら期待値を固定しない。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-011 ABI F32要求の未対応範囲](rq-candidates.md#candidate-011) | [TC-ABI-REP-01](../../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01) | F32の実際の受渡しAPIが未定義なら未対応として記録し、C型宣言だけの検査を要求適合passとしない。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-012 初期Propertyの値型範囲](rq-candidates.md#candidate-012) | [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01) | 対応RQの観測結果を個別assertionとして現行caseに照合する。初期Propertyで真偽値、符号付き64-bit整数、有効なUTF-8文字列、倍精度浮動小数値、規定範囲の十進値を利用できること。初期範囲にnullable値・任意言語object・カスタム構造体値を含めず、それらをゼロや空文字列へ読み替えないこと。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-013 F64の同値と保持結果](rq-candidates.md#candidate-013) | [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-ABI-REP-01](../../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01) | 正負ゼロとNaNの異なるbit表現を同値再設定し、stored representationと通知回数を観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-014 十進値の範囲と同値](rq-candidates.md#candidate-014) | [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-ABI-REP-01](../../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01) | 1.0/1.00、正負ゼロ、最大係数、scale31、範囲外候補で同値/保持/拒否を観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-015 UTF-8文字列の受渡し結果](rq-candidates.md#candidate-015) | [TC-ABI-REP-01](../../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-OWN-01](../../30_release_test/requirement-test-matrix.md#case-tc-own-01) | 空、多byte、埋込みNUL、不正UTF-8を入出力し、指定長・拒否・無変換を観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-016 buffer不足時の結果](rq-candidates.md#candidate-016) | [TC-OWN-01](../../30_release_test/requirement-test-matrix.md#case-tc-own-01) | 必要容量より1byte少ないbufferへsentinelを置いて取得し、必要容量とbuffer全体の非変更を確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-017 error詳細を生成できない失敗](rq-candidates.md#candidate-017) | [TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01) | 主処理の失敗後にerror確保も失敗させ、元statusを保持すること、取得済みsnapshotが次操作で変わらないことを確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-018 利用者定義エラーの自動採番](rq-candidates.md#candidate-018) | [TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01) | 名前付きerror定義へ新項目を加え、既存公開codeの不変性と4言語catalog整合を確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-019 機能照会と互換性](rq-candidates.md#candidate-019) | [TC-ABI-COMP-01](../../30_release_test/requirement-test-matrix.md#case-tc-abi-comp-01)、[TC-ABI-COMP-02](../../30_release_test/requirement-test-matrix.md#case-tc-abi-comp-02) | 対応RQの観測結果を個別assertionとして現行caseに照合する。提供されない機能を利用前に識別でき、未知の機能識別子は未対応として扱えること。公開済み機能識別子とstatusの意味を再利用せず、wrapperが必要とするRuntime互換条件を判断できること。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-020 購読の初期通知・順序](rq-candidates.md#candidate-020) | [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](../../30_release_test/requirement-test-matrix.md#case-tc-callback-01) | 購読追加/解除を配信callbackから行い、当該回の対象・順序と後続回の対象を区別して観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-021 同値設定時の検証](rq-candidates.md#candidate-021) | [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01) | 同値candidateを拒否するvalidatorを登録し、検証が起動することと値/通知の非変更を確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-022 長期保留の診断設定](rq-candidates.md#candidate-022) | [TC-BIND-01](../../30_release_test/requirement-test-matrix.md#case-tc-bind-01) | 単調時計fixtureで閾値前後・無効化・同chain再観測を確認し、診断だけで更新停止しないことを観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-023 非収束診断と回復](rq-candidates.md#candidate-023) | [TC-BIND-01](../../30_release_test/requirement-test-matrix.md#case-tc-bind-01) | 独立した二chainの一方を反復させ、停止対象・確定値・長期保留との差・破棄/再開結果を観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-024 Validator登録条件](rq-candidates.md#candidate-024) | [TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](../../30_release_test/requirement-test-matrix.md#case-tc-callback-01) | Validator重複登録を拒否し、現在値の再評価が起こらず旧Validatorが有効なことを確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-025 判定callback中の禁止操作](rq-candidates.md#candidate-025) | [TC-CALLBACK-01](../../30_release_test/requirement-test-matrix.md#case-tc-callback-01) | 判定callback内からUI変更/自己解除を試み、状態の非変更と入力拒否/異常の分類を観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-026 TextBox入力三値の結果](rq-candidates.md#candidate-026) | [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-VERT-03](../../30_release_test/requirement-test-matrix.md#case-tc-vert-03) | 同じ編集候補をAccept/Incomplete/通常Reject/IME Rejectへ振り分け、編集全文・選択・確定値・理由を別観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-027 TextBox自身のProperty検証拒否](rq-candidates.md#candidate-027) | [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01) | 入力Accept後にTextBox自身の確定値Validatorを拒否させる。案B/A/Cの採否までは期待結果をblockedとする。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-028 IME cancel後の入力理由](rq-candidates.md#candidate-028) | [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01) | composition開始前に理由を持つ状態を作り、Update後Cancelする。state/reason復元の採否に合わせてoracleを確定する。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-029 IME commit準備失敗](rq-candidates.md#candidate-029) | [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01) | IME Commit候補copy/退避に失敗を注入し、候補保持場所と再送可能性をCore/Adapter別に確認する。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-030 外部更新と編集中の値](rq-candidates.md#candidate-030) | [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-BIND-01](../../30_release_test/requirement-test-matrix.md#case-tc-bind-01) | preedit中に外部確定値を変更し、旧sessionの遅延Commitで新値が上書きされないことを確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-031 Command実行受付と結果](rq-candidates.md#candidate-031) | [TC-CMD-01](../../30_release_test/requirement-test-matrix.md#case-tc-cmd-01) | 受付前のrecord確保失敗と、受付後のstart失敗を分け、operation有無・execute結果・終端結果を観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-032 取消・close・handle破棄の差](rq-candidates.md#candidate-032) | [TC-CMD-01](../../30_release_test/requirement-test-matrix.md#case-tc-cmd-01) | 同ID再cancel、終端後cancel、古いID、close後完了、handle releaseのみを分けて実行する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-033 Detached subtreeの利用](rq-candidates.md#candidate-033) | [TC-TREE-01](../../30_release_test/requirement-test-matrix.md#case-tc-tree-01)、[TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01) | subtreeをdetachし、独立参照からProperty更新/Binding継続と描画/Input非対象を確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-034 Tree変更の拒否結果](rq-candidates.md#candidate-034) | [TC-TREE-01](../../30_release_test/requirement-test-matrix.md#case-tc-tree-01) | 複数親/循環/別Runtimeを拒否し、再add失敗時のDetached構造・値・登録が維持されることを確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-035 StackPanel公開設定と既定](rq-candidates.md#candidate-035) | [TC-LAYOUT-01](../../30_release_test/requirement-test-matrix.md#case-tc-layout-01) | 明示した各direction/Alignを確認し、未承認の省略時既定を受入条件へ追加しない。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-036 有効性とFocusの共通結果](rq-candidates.md#candidate-036) | [TC-FOCUS-01](../../30_release_test/requirement-test-matrix.md#case-tc-focus-01)、[TC-ACC-07](../../30_release_test/requirement-test-matrix.md#case-tc-acc-07) | 親disabledで子own値を保持しつつ操作/Focusを拒否し、detachで代替Focusが暗黙選択されないことを確認する。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-037 初期機能の除外とButton未接続](rq-candidates.md#candidate-037) | [TC-INPUT-01](../../30_release_test/requirement-test-matrix.md#case-tc-input-01)、[TC-CMD-01](../../30_release_test/requirement-test-matrix.md#case-tc-cmd-01) | key割当の将来候補を必須caseにしない。CommandなしButtonの結果は採否後に確認する。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-038 Focus通知と入力route](rq-candidates.md#candidate-038) | [TC-FOCUS-01](../../30_release_test/requirement-test-matrix.md#case-tc-focus-01)、[TC-INPUT-01](../../30_release_test/requirement-test-matrix.md#case-tc-input-01) | Focus移動時のstate/Lost/Gained順と、targetなし入力後に別targetへ誤配送しないことを確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-039 Platform資源喪失と終了](rq-candidates.md#candidate-039) | [TC-PLAT-01](../../30_release_test/requirement-test-matrix.md#case-tc-plat-01) | surface/device failureとOS close requestを注入し、Core値保持とRuntime状態を別観測する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-040 Accessibility identityと操作境界](rq-candidates.md#candidate-040) | [TC-ACC-PLAT-01](../../30_release_test/requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-ACC-05](../../30_release_test/requirement-test-matrix.md#case-tc-acc-05) | AT queryだけのFocus不変更、stale node action拒否、通常Control actionとの結果一致を確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-041 Accessibility nameの空欄](rq-candidates.md#candidate-041) | [TC-ACC-02](../../30_release_test/requirement-test-matrix.md#case-tc-acc-02) | 表示文字列/nameを別値にし、空nameの公開結果を確認する。自動fallback非提供の要求承認を先行させる。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-042 C++の例外無効buildとhandler](rq-candidates.md#candidate-042) | [TC-WRAP-CPP-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01) | 例外有効/無効buildで同じfailureを扱い、handler設定・handler例外・callback固有結果を分けて確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-043 C#の明示破棄と競合](rq-candidates.md#candidate-043) | [TC-WRAP-CS-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-cs-01) | native call中Disposeとcallback再入Disposeを競合させ、lease中利用と最終解放一回性を確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-044 Rust safe APIとpanic限界](rq-candidates.md#candidate-044) | [TC-WRAP-RS-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-rs-01) | borrowの期限/Send制約をcompile-fail型検査で、捕捉panicを実行検査で分ける。abortは別processで保証外境界を確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-045 Python明示終了とowner回収](rq-candidates.md#candidate-045) | [TC-WRAP-PY-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01) | 明示close、with例外終了、owner回収でtokenの後始末を確認し、finalizerの時刻を要求oracleにしない。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-046 Python通常終了・例外と保証外](rq-candidates.md#candidate-046) | [TC-WRAP-PY-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-LIFE-01](../../30_release_test/requirement-test-matrix.md#case-tc-life-01) | main例外/SystemExit/host loop終了/worker例外を独立processで起こし、管理対象UIが残らないかとcleanup証拠を確認する。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-047 GIL非依存とsupport区分](rq-candidates.md#candidate-047) | [TC-THREAD-01](../../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-WRAP-PY-01](../../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01) | GILの暗黙直列性を使わない競合条件でclose/root変更を試験し、UI専用状態へworkerが直接書けないことを確認する。 | 既存要求の意味維持。case改訂内容は別承認 |
| [SEP-RQ-048 診断・解析の能力と限界](rq-candidates.md#candidate-048) | [TC-TRACE-01](../../30_release_test/requirement-test-matrix.md#case-tc-trace-01) | 静的解決例と動的未解決例、同入力再解析、更新競合時のstale diagnosticを確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-049 検証状態と正式support](rq-candidates.md#candidate-049) | [TC-TARGET-ARMV7-01](../../30_release_test/requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-A64-01](../../30_release_test/requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-BOARD-01](../../30_release_test/requirement-test-matrix.md#case-tc-target-board-01) | ARMv7/AArch64 cross-build結果と実機profile結果を別集計し、他profileへhardware合格が転記されないことを確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-050 artifact選択・identity・取得](rq-candidates.md#candidate-050) | [TC-ARTIFACT-01](../../30_release_test/requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-RS-01](../../30_release_test/requirement-test-matrix.md#case-tc-pkg-rs-01) | target/profile/ABI不一致、同identity異digest、cache miss/offline/明示更新を確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-051 artifactの信頼と検証失敗](rq-candidates.md#candidate-051) | [TC-ARTIFACT-01](../../30_release_test/requirement-test-matrix.md#case-tc-artifact-01) | 改ざんpayload/不正署名/未知鍵/失効鍵/独立trust bundleを検証し、拒否時にlink/loadへ渡らないことを確認する。 | 要求採否/期待結果の選択待ち |
| [SEP-RQ-052 manifestの互換・曖昧性](rq-candidates.md#candidate-052) | [TC-ARTIFACT-01](../../30_release_test/requirement-test-matrix.md#case-tc-artifact-01) | duplicate key/unknown major/critical field/optional fieldを分けて検査する。parser内部上限はSD設計根拠でunit検証する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-053 license/noticeの継承](rq-candidates.md#candidate-053) | [TC-LICENSE-01](../../30_release_test/requirement-test-matrix.md#case-tc-license-01) | source/binaryと言語packageのinventoryをlicense/noticeの収録物へ照合する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-054 support lifecycleとEOL記録](rq-candidates.md#candidate-054) | [TC-SUPPORT-01](../../30_release_test/requirement-test-matrix.md#case-tc-support-01) | major公開日・限定保守期限・profile告知・security短縮・EOL公開記録を運用証拠で照合する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-055 Collection stable identity](rq-candidates.md#candidate-055) | [TC-COL-01](../../30_release_test/requirement-test-matrix.md#case-tc-col-01) | insert/move/remove後のitem IDと編集対象を比較し、位置の変化で別itemへ誤適用しないことを確認する。 | 次段階。専用DD/詳細case未設計をpassへ換算しない |
| [SEP-RQ-056 Selection順序と拒否](rq-candidates.md#candidate-056) | [TC-SEL-01](../../30_release_test/requirement-test-matrix.md#case-tc-sel-01) | 並べ替え後の選択集合/公開順、Singleへ複数ID設定拒否、削除後selected/currentを確認する。 | 次段階。専用DD/詳細case未設計をpassへ換算しない |
| [SEP-RQ-057 Collection batchの観測結果](rq-candidates.md#candidate-057) | [TC-COL-01](../../30_release_test/requirement-test-matrix.md#case-tc-col-01)、[TC-SEL-01](../../30_release_test/requirement-test-matrix.md#case-tc-sel-01) | batch途中に不正operationを入れ、内容・revision・Selection・通知不変と、成功時の整合snapshotを確認する。 | 次段階。専用DD/詳細case未設計をpassへ換算しない |
| [SEP-RQ-058 text coverageと編集位置](rq-candidates.md#candidate-058) | [TC-IME-01](../../30_release_test/requirement-test-matrix.md#case-tc-ime-01)、[TC-PLAT-01](../../30_release_test/requirement-test-matrix.md#case-tc-plat-01) | 対応RQの観測結果を個別assertionとして現行caseに照合する。初期版でUTF-8のLatin・日本語UI文字列を表示・編集でき、対応profileの文字・font依存条件を判断できること。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-059 性能測定と閾値](rq-candidates.md#candidate-059) | [TC-NFR-01](../../30_release_test/requirement-test-matrix.md#case-tc-nfr-01)、[TC-NFR-06](../../30_release_test/requirement-test-matrix.md#case-tc-nfr-06) | 対応RQの観測結果を個別assertionとして現行caseに照合する。対象profile、機能負荷、測定区間、revisionと各処理層の測定結果を記録できること。 | 既存要求の意味維持。case改訂内容は別承認 |
| [SEP-RQ-060 文書承認とID移行](rq-candidates.md#candidate-060) | [TC-TRACE-01](../../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-DOC-01](../../30_release_test/requirement-test-matrix.md#case-tc-doc-01) | 対応RQの観測結果を個別assertionとして現行caseに照合する。要求の変更承認、設計の変更承認、case設計、実装・実行状態を別々に追跡し、旧IDと新しい意味単位の対応を確認できること。 | 既存要求の意味維持。case改訂内容は別承認 |
| [SEP-RQ-061 通知callbackの例外後の継続](rq-candidates.md#candidate-061) | [TC-CALLBACK-01](../../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01) | 一購読callbackで捕捉例外を起こし、他購読へ継続することと種別別の異常結果を確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-062 単一操作の準備失敗](rq-candidates.md#candidate-062) | [TC-ERROR-01](../../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-PROP-01](../../30_release_test/requirement-test-matrix.md#case-tc-prop-01) | input/copy/output準備失敗を個別注入し、対象操作の状態/通知非変更と先行独立操作の保持を確認する。 | 要求案承認後。現在は未承認 |
| [SEP-RQ-063 同じBindingの重複接続](rq-candidates.md#candidate-063) | [TC-BIND-01](../../30_release_test/requirement-test-matrix.md#case-tc-bind-01) | 同じsource-target接続の再登録を拒否し、既存graph/値/通知を比較する。 | 要求案承認後。現在は未承認 |

## unit-testと実装単位の対応案

[unit-test strategy](../../32_unit_test/unit-test-strategy.md)のUT-STR-01～06は既存の未承認案であり、本案はその改訂・具体化をレビューする。実装単位IDとpathは実装構成確定時に発番し、sourceファイル1個に固定しない。

| 実装単位候補 | 局所検証・故障注入 | 対応する上位観測 |
|---|---|---|
| Handle registry / token発行 | 検証pin、retain/release線形化、発行不可時の非部分登録 | TC-CORE-OBJ-01 / TC-LIFE-01 |
| Runtime owner/finalizer | anchor循環不在、handoff失敗時token保全、Closing drain競合 | TC-LIFE-01 / TC-THREAD-01 |
| Dispatcher/bridge | lost wake、enqueue直列化点、一turn上限、shutdown優先 | TC-THREAD-01 / TC-CALLBACK-01 |
| Closure lifecycle | active検査とin-flight、自己解除、destroy一回性 | TC-CALLBACK-01 |
| Property store/notification | kind比較、copy-before-commit、generation/snapshot順序 | TC-PROP-01 / TC-OWN-01 |
| Binding topology/chain | graph prepare、generation照合、hash衝突時の同値確認、128-item計数 | TC-BIND-01と拒否専用case |
| Validator/input判定 | typed candidate、理由handle消費、禁止context、編集commit | TC-IME-01 / TC-CALLBACK-01 |
| Command operation | record確保、cancel flag、ID一回終端、start部分失敗cleanup | TC-CMD-01 |
| Logical tree/layout | 所属commit/rollback、dirty伝播、measure/arrange結果commit | TC-TREE-01 / TC-LAYOUT-01 |
| TextBox session | session照合、snapshot復元、候補退避、位置変換 | TC-IME-01 |
| Platform/Accessibility | snapshot所有、peer/provider切断、UI marshal、stale拒否 | TC-PLAT-01 / TC-ACC-PLAT-01 |
| 言語owner/root/reaper | preallocated ticket、call lease、root detach、lock外drop | 言語別TC-WRAP-* |
| Artifact selector/parser/verify | 曖昧parse拒否、署名対象byte一致、staging原子公開、cache isolation | TC-ARTIFACT-01 / TC-PKG-* |

各unit計画は `source/config path → unit → UT suite/command → 適用DD/SD → commit SHAと結果` を保持する。変更pathが対応情報にない場合は、既知の広いsuiteを実行する案を優先し、実行可能なfallbackも決められない場合は対応不足でCIを失敗/blockedにする。未マッピングを成功skipへしない。push/PR検証は実装とunit-testの同じSHAを対象とし、未push編集の検証を意味しない。

## Release/Version・測定・移管手順

Release testは各releaseの用途適合と必須gate、Version testは変更要求と影響回帰を担当する。新要求が採用されたら対象caseと回帰範囲を更新し、未解決candidateをrelease対象として採用するかも要求側で判断する。実行証跡のないcase割当をrelease合格へ数えない。

BD/DDから移す試験文は、元文書・節・ID、対応RQ、case/assertion、移管日を記録する。失敗時の状態表や所有flowは設計に保持する。代表値がRQから決まらず実施に必要な場合のみ、test計画に値・用途・選定理由・根拠・適用範囲を記す。合否閾値や保証は代表値から作らない。

監査では現行187 RQのcase割当、孤立case、重複assertion、未実装/未実行、後続stage、profile別証拠を別集計する。旧195 IDのうちRQ-08の8件は廃止済みであり、現行case割当へ復活させない。実suite inventoryがまだ無いため「無駄な実testゼロ」や実装coverage100%とは判定しない。旧DD-23を再作成しない。


## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
|---|---|---|---|
| 2026-10-17 | SEP-03 / 要求testとunit-test対応 | — | 一括作成指示に基づくレビュー案。採用・正本適用は未承認 |

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
