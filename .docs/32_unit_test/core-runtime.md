# 単体試験：handle・値・ABI・Runtime

- 文書版: 0.1
- 状態: レビュー案・未承認。全項目planned / 未実装 / not-run
- 更新日: 2026-10-17

候補単位はHandleRegistry、ValueStorage/FloatCodec/ErrorSnapshot、ABI入口、Dispatcher/終了処理。公開結果はRQ、内部pin・lock・queue境界はDDをoracleとする。

上位根拠: [単体試験方針](unit-test-strategy.md)、[実装unit台帳契約](implementation-test-inventory.md)、[共通fixture・実行記録](README.md)。

設計根拠: [runtime-objects/objects-values.md](../13_detail_design/runtime-objects/objects-values.md) 1.1「handle・値・error / 数値変換 / クラス設計の補足」、[runtime-objects/abi.md](../13_detail_design/runtime-objects/abi.md) 1.1「検証順序 / output初期化」、[runtime-objects/runtime.md](../13_detail_design/runtime-objects/runtime.md) 1.1「終了要求 / クラス設計の補足」。

## handleと値

二つのRuntimeと同kind/異kindのobject、同objectの独立ownerを用意する。token発行器とallocatorには決定的なfailpointを設け、実在pointerを捏造して試験しない。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-handle-001"></a>UT-HANDLE-001 | 一objectのownerを二つ取得し、片方をrelease、同tokenを再release、他方からreadする | 最初のreleaseだけが参照を消費。重複releaseは失敗、他ownerのreadは成功 | [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-011](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-011)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01) |
| <a id="ut-handle-002"></a>UT-HANDLE-002 | 無効token・異kind・異Runtimeをそれぞれ単独に渡す | 失敗分類を区別し、値・登録・通知を変更しない。複数不正の優先順はABI本文に従う | [RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01) |
| <a id="ut-handle-003"></a>UT-HANDLE-003 | token新規発行不能とrelease受付失敗を個別注入し、失敗後に再試行する | 新tokenを既存対象へ誤解決しない。failed releaseは旧token所有を保持し再試行可能 | [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-WRAP-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-RS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-rs-01)、[TC-WRAP-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01) |
| <a id="ut-value-001"></a>UT-VALUE-001 | 空・日本語・埋め込みNUL・不正UTF8をtyped入力し、成功後に元bufferを書き換える | 成功値は指定byte列のcopy、不正UTF8は拒否。正規化・置換・NUL終端切捨てをしない | [RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003)、[RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020)、[RQ-05-004](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-004)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01) |
| <a id="ut-value-002"></a>UT-VALUE-002 | UTF8出力を容量0・必要量-1・必要量で取得し、不足bufferをsentinelで初期化する | 必要byte数を返し、不足時はbufferを部分変更しない。可変値の照会後変更と固定snapshotの読取りを区別 | [RQ-04-021](../11_requirements/RQ-04_abi-boundary.md#rq-04-021)<br/>[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01) |
| <a id="ut-value-003"></a>UT-VALUE-003 | BOOL/I64/UTF8/F64/DECIMALの同値・異値・異kind setterと準備失敗を試す | typed保存と同値判定はDDどおり。同値成功は不要通知なし。失敗した操作だけが値・通知を変更しない | [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)、[RQ-04-026](../11_requirements/RQ-04_abi-boundary.md#rq-04-026)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01) |
| <a id="ut-numeric-001"></a>UT-NUMERIC-001 | F32完全表現値、隣接F32の中点を二つの偶奇方向、signed zero、infinity、NaN、有限範囲超過で変換する | EXACTは非完全値を拒否、NEAREST_EVENは偶数側。zero符号とinfinityの意味を保持、NaN payload一致は要求しない | [RQ-04-029](../11_requirements/RQ-04_abi-boundary.md#rq-04-029)<br/>[TC-NUM-01](../30_release_test/requirement-test-matrix.md#case-tc-num-01) |
| <a id="ut-abi-001"></a>UT-ABI-001 | required outが有効な各入口で失敗させ、optional error有無とerror生成OOMを組み合わせる | 定義済みout初期値と元statusを保持。任意の不正addressが安全に読書きできる保証を追加しない | [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-error-001"></a>UT-ERROR-001 | error/cause snapshotを保持後に別エラーを発生。未知status/domain/codeを渡す | 保持済みsnapshotの数値・message・causeは不変。未知数値を失わずmessageを機械判定に使わない | [RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)、[RQ-04-027](../11_requirements/RQ-04_abi-boundary.md#rq-04-027)<br/>[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
## dispatchと終了

fake hostはturn進行・wake受付・host停止を手動制御する。workerはbarrierでqueue登録と終了受付の前後へ固定し、sleepの長さで順序を推測しない。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-001"></a>UT-RUNTIME-001 | worker二つから順序を制御してpostし、UI callback内でもpostする | UI threadで一度ずつ、確定したqueue順に実行。現在callbackが戻るまで再帰invokeしない | [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01) |
| <a id="ut-runtime-002"></a>UT-RUNTIME-002 | 空queueへの最初のpost、drain末尾との競合、wake pending中追加を各barrierで再現 | 受付成功した作業をlost wakeで残さず、重複wakeで二重invokeしない | [RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-runtime-003"></a>UT-RUNTIME-003 | post受付前のOOM・Closedと受付成功後のshutdownを分けて発生 | 受付失敗はcallerがclosure所有。成功後はinvokeまたはcancel経路でUI上のdestroy一回 | [RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)<br/>[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-runtime-004"></a>UT-RUNTIME-004 | callback中に終了を要求し、未開始postと通知を残す | 現在callbackが先に完了し、その後通常作業より終了を優先。未開始callbackを新規invokeしない | [RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)、[RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01) |
| <a id="ut-runtime-005"></a>UT-RUNTIME-005 | worker同期終了の完了通知とhost停止を別々に発生。停止から同hostを復旧して再試行 | 未完了・完了・host実行不可と理由を区別し、host停止だけをClosed/資源解放完了にしない | [RQ-06-007](../11_requirements/RQ-06_threading-dispatch.md#rq-06-007)、[RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01) |
| <a id="ut-runtime-006"></a>UT-RUNTIME-006 | Runtime終了後にUI操作、残owner release、immutable error読取りを行う | UI利用は終了。残参照解放と不変エラー読取りは許可されたthread契約で継続できる | [RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)、[RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-VERT-08](../30_release_test/requirement-test-matrix.md#case-tc-vert-08)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1全文 | — | レビュー案・未承認。作成指示を試験設計の承認や試験合格とは扱わない |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
