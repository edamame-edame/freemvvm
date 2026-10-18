# 単体試験：handle・値・ABI・Runtime

- 文書版: 0.4
- 状態: 0.4全文承認済み（利用者、2026-10-17）。製品test実装中
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
| <a id="ut-abi-003"></a>UT-ABI-003 | F32/F64変換APIを正しいmodeとnull required outputで呼び、failure前に出力が初期化されることを確認する | 成功時に変換結果を返す。無効mode/null output/不正確または範囲外の変換は拒否し、required outputとoptional error handleを0にする | [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-029](../11_requirements/RQ-04_abi-boundary.md#rq-04-029)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-NUM-01](../30_release_test/requirement-test-matrix.md#case-tc-num-01) |
| <a id="ut-abi-001"></a>UT-ABI-001 | required outが有効な各入口で失敗させ、optional error有無とerror生成OOMを組み合わせる | 定義済みout初期値と元statusを保持。任意の不正addressが安全に読書きできる保証を追加しない | [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-abi-002"></a>UT-ABI-002 | `get_version`を3つの有効なout pointerで呼び、次に各pointerを個別にnullにして呼ぶ | 成功時は`UI_OK`と`1.0.0`。nullがある呼出しは`UI_INVALID_ARGUMENT`で、非nullの全out値は0。任意の不正addressの安全性は保証しない | [RQ-04-009](../11_requirements/RQ-04_abi-boundary.md#rq-04-009)、[RQ-04-028](../11_requirements/RQ-04_abi-boundary.md#rq-04-028)、[RQ-04-030](../11_requirements/RQ-04_abi-boundary.md#rq-04-030)<br/>[TC-ABI-COMP-02](../30_release_test/requirement-test-matrix.md#case-tc-abi-comp-02)、[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01) |
| <a id="ut-error-001"></a>UT-ERROR-001 | error/cause snapshotを保持後に別エラーを発生。未知status/domain/codeを渡す | 保持済みsnapshotの数値・message・causeは不変。未知数値を失わずmessageを機械判定に使わない | [RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)、[RQ-04-027](../11_requirements/RQ-04_abi-boundary.md#rq-04-027)<br/>[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |

### Token issuer — implementation slice 1

以下は承認済みDD v1.1「handle登録と検証」の非zero・非再利用・枯渇時失敗を、最初の実装slice向けに具体化した試験追補案。成功した最大値の扱い、Mutex poison時の内部失敗扱いは本試験で検査するための実装選択であり、公開ABI status/error契約を追加しない。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-handle-alloc-001"></a>UT-HANDLE-ALLOC-001 | 新しいissuerから連続してtokenを要求する | 異なる非zero tokenを単調増加で返す | [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01) |
| <a id="ut-handle-alloc-002"></a>UT-HANDLE-ALLOC-002 | 次tokenを`u64::MAX`として初期化し、二度以上要求する | 最大値は一度だけ返し、その後は常に発行不能。周回・再利用しない | [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01) |
| <a id="ut-handle-alloc-003"></a>UT-HANDLE-ALLOC-003 | 複数threadから同一issuerへ同時にtokenを要求する | 返されたtokenはすべて一意な非zero値 | [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01) |
| <a id="ut-handle-alloc-004"></a>UT-HANDLE-ALLOC-004 | 同期mutexをpoison状態にしてtokenを要求する | 新tokenを返さず失敗する。既発行tokenへ戻らない | [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01) |

### HandleEntry — implementation slice 3

以下は承認済みDD v1.1のHandleEntry（kind、optional Runtime identity、一つの公開owner参照）のうち、entry-localな照合と参照消費を試験する。型とRuntime identityの具体表現、registry tableや実objectの保持方式はこの追補で決定しない。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-handle-entry-001"></a>UT-HANDLE-ENTRY-001 | kindとRuntime identityが設定されたentryに一致/不一致の期待値を照合し、RuntimeなしentryにもRuntime照合する | 一致だけtrue。不一致とRuntimeなしはfalse。どの照合でもentryのlive状態とowner参照を変更しない | [RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01) |
| <a id="ut-handle-entry-002"></a>UT-HANDLE-ENTRY-002 | 同じ対象を所有する二entryの一方だけ参照をconsumeし、重複consumeする | 一entryの参照は一度だけ返る。他entryの参照は有効で同じ対象を示す | [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01) |

### Runtime HandleRegistry — implementation slice 4

以下は承認済みDD v1.2のRuntime-owned object registry、thread-safe control block、ValidationPin、cleanup reservationを試験する。fixtureはUI object state、Runtime cleanup queue、ticket reservationを手動制御し、thread順序はbarrierで固定する。Runtime非依存ErrorSnapshotのowner方式は本追補の対象外。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-handle-reg-001"></a>UT-HANDLE-REG-001 | Runtime identityとgeneration付きobject identityを持つobjectを登録し、tokenでentryを検証する。token/table確保失敗も個別注入する | 登録成功だけが完全なentryを公開する。失敗時は部分entryとowner増分を残さない。kind/Runtimeの照合はentry-local規則どおり | [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)、[RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)、[RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01) |
| <a id="ut-handle-pin-001"></a>UT-HANDLE-PIN-001 | UI threadでValidationPinを取得し、最後の公開tokenをreleaseした後にpinを保持・解放する | ticketは受付済みでもpin中はobject stateを破棄しない。最後のpin後、UI safe pointでstateを一度だけ破棄する | [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-handle-release-001"></a>UT-HANDLE-RELEASE-001 | worker threadから最後のtokenをreleaseし、queue受理後にUI queueを手動drainする | release成功はtoken消費とcleanup受付を示す。worker上でUI stateを触らず、drain前は未破棄、drain後はUI thread上で破棄済み | [RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010)、[RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-handle-release-002"></a>UT-HANDLE-RELEASE-002 | 最終releaseのCleanupReservation取得を失敗させ、queueを復旧して同じtokenを再releaseする | 失敗したreleaseではtokenと公開参照がliveのままで、再試行できる。再試行成功時だけtokenを消費する | [RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01) |
| <a id="ut-handle-release-003"></a>UT-HANDLE-RELEASE-003 | barrierで同tokenのretain/releaseを競合させ、両方のlinearization順を再現する | retain先行なら新tokenが独立してlive。release先行ならretainは失敗しtoken resurrectionなし。重複releaseは高々一度だけ成功し、別tokenを変えない | [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-011](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-011)、[RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)<br/>[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-handle-shutdown-001"></a>UT-HANDLE-SHUTDOWN-001 | cleanup ticket予約後にRuntimeをClosingへ進め、全ticket drain前後で状態を観測する。shutdown後に残るclosed shellもreleaseする | Runtimeはticket処理前にClosedにならない。UI state cleanupはUI threadで一度だけ。Closed後のshell releaseはUI stateへ触れず完了する | [RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)、[RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006)、[RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
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

### Runtime UI thread identity context — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-thread-context-001"></a>UT-RUNTIME-THREAD-CONTEXT-001 | `UiThreadContext`を現在threadで生成し、同threadとworker threadから所属を照会する | 生成threadではtrue、worker threadではfalse。照会はthread identity以外の状態を変更しない | [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Runtime UI thread context propagation — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-thread-context-002"></a>UT-RUNTIME-THREAD-CONTEXT-002 | callerで`UiThreadContext`を取得し、workerでそのcontextを受け取ったDispatcherを構築。callerとworker双方からdispatchを試す | callerのdispatchは成功し、workerのdispatchは`WrongThread`。作成threadではなく注入contextの所属が使われる | [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### RuntimeCore bridge-free creation and bridge registration — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-owner-001"></a>UT-RUNTIME-OWNER-001 | RuntimeCoreをbridgeなしで生成し、同threadからqueryする。次にUI threadでbridgeを登録し、重複登録、worker登録、shutdown開始後の登録も試す | bridgeなし生成とthread queryは成功。一度目だけ登録成功。重複・worker・Closing後は失敗し、既存bridgeは置換されず、拒否されたbridgeは呼出し側が保持する | [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### RuntimeCore identity issuance — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-identity-001"></a>UT-RUNTIME-IDENTITY-001 | 複数threadからRuntimeCoreを同時生成し、各identityを取得する | 全生成が成功しidentityは非zeroかつ相互に異なる。共有token issuerの排他下で割当てられる | [RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)、[RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Runtime-bound CoreObjectControl creation — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-object-control-001"></a>UT-RUNTIME-OBJECT-CONTROL-001 | RuntimeCoreからobject identityを指定してcontrolを作り、共有HandleRegistryへ登録。自身と別Runtimeを指定して検証し、最後の参照をworkerからreleaseした後にRuntime queueをdrainする | 登録controlのRuntime identityとcleanup queueは生成元Runtimeに結び付く。自身のRuntime検証だけ成功し、他Runtime検証はWrongRuntimeで拒否される。worker release後の最終cleanupは生成元RuntimeのUI thread queueで完了する | [RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)、[RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010)、[RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Runtime handle registration and borrowed resolution — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-handle-reg-001"></a>UT-RUNTIME-HANDLE-REG-001 | 異なるRuntimeCoreをtableへ登録し、得られたtokenを同threadとworkerから解決。未登録tokenとzero tokenも解決し、table lockをpoisonさせたinsertも試す | tokenは非zeroかつ別Runtime間で異なる。解決結果は該当RuntimeCoreと同一ownerを指す。未登録/zero tokenはInvalidTokenで拒否され、tableを変更しない。worker解決も安全だがUI操作権を与えない。poison時は失敗し、callerのRuntime ownerを維持する | [RQ-04-004](../11_requirements/RQ-04_abi-boundary.md#rq-04-004)、[RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1全文 | — | レビュー案・未承認。作成指示を試験設計の承認や試験合格とは扱わない |
| 2026-10-17 | 0.1全文 | 利用者 | 「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」により既存試験案を承認し環境整備を開始。製品test実装・合格を意味しない |
| 2026-10-17 | 0.2 token issuer追補 | — | 承認済みDD v1.1から試験oracleを具体化したレビュー案。承認前の追補を公開契約へ拡張しない |
| 2026-10-17 | 0.2全文 | 利用者 | token issuer追補を含む本版全文を承認。テスト設計の承認であり製品test合格を意味しない |
| 2026-10-17 | 0.3 HandleEntry追補 | — | 承認済みDD v1.1からentry-localなoracleを具体化したレビュー案。具体owner型やRegistry policyを追加しない |
| 2026-10-17 | 0.3全文 | 利用者 | 「ok」によりHandleEntry追補を含む本版全文を承認。テスト設計の承認であり製品test合格を意味しない |
| 2026-10-17 | 0.4 Runtime HandleRegistry追補 | — | 承認済みDD v1.2のRuntime-owner/pin/finalizer境界を具体化したレビュー案。実装・test合格を意味しない |
| 2026-10-17 | 0.4全文 | 利用者 | 「ok」によりRuntime HandleRegistry追補を含む本版全文を承認。テスト設計の承認であり製品test合格を意味しない |
| 2026-10-18 | Runtime UI thread identity追補 | 利用者 | 「ok」により、Runtime作成thread identity contextのfocused unit-test設計を追加。RQ-06-001全体の検証完了や公開Runtime APIの承認を意味しない |
| 2026-10-18 | Runtime UI thread context propagation追補 | 利用者 | 「ok」により、共有context注入に対するfocused unit-test設計を追加。Runtime facadeや公開ABIの承認を意味しない |
| 2026-10-18 | RuntimeCore bridge registration追補 | 利用者 | 「OK」により、bridgeなし生成とRuntimeCore内の一度限りのbridge登録に対するfocused unit-test設計を追加。公開C ABIやRuntime handle実装の承認を意味しない |
| 2026-10-18 | RuntimeCore identity issuance追補 | Codex | RQ-04-018/019と承認済みRuntime identity設計から、共有token issuerを使う一意性のfocused test案を追加。Runtime identityの公開やC ABIの承認を意味しない |
| 2026-10-18 | Runtime-bound CoreObjectControl追補 | 利用者 | 「OK」により、Runtime identityとcleanup queueの組合せを生成元Runtimeへ固定するfocused test案を追加。object storeや公開handleの承認を意味しない |
| 2026-10-18 | Runtime handle registration追補 | 利用者 | 「OK」により、process tokenを使うRuntime handleの登録・borrowed lookupをfocused test案として追加。public C ABI entry pointやlast-token releaseの承認を意味しない |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
| 0.2 | 2026-10-17 | HandleRegistry全体に先行するtoken issuerの最小sliceと4件の局所oracleを追補 | Codex | 本版全文承認済み |
| 0.3 | 2026-10-17 | HandleEntryのkind/Runtime照合と一公開参照のconsumeを試験追補 | Codex | 利用者が本版全文を承認 |
| 0.4 | 2026-10-17 | Runtime HandleRegistryの登録、pin、final release、reservation失敗、競合、shutdown drainを追補 | Codex | 利用者が本版全文を承認 |
