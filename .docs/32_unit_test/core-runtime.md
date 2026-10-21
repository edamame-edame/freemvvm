# 単体試験：handle・値・ABI・Runtime

- 文書版: 1.5
- 状態: Runtime post/dispatch turn、Property create/kind、Property getters、OneWay propagation candidate/commit、およびTwoWay initial-sync oracle追補は利用者承認済み範囲。製品test実装・実行状態は別管理
- 更新日: 2026-10-21

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
| <a id="ut-object-store-001"></a>UT-OBJECT-STORE-001 | Runtime UI thread上でinsert/resolve/destroyし、slot再利用後に旧identityをresolve。workerからも各操作を呼ぶ | 現行generationだけが解決され、destroy後と再利用後の旧identityは失敗。off-thread操作は状態を変更しない。generation最大値でwrapせずslotをretire | [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)、[RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-object-store-002"></a>UT-OBJECT-STORE-002 | UI thread上で複数stateを格納し、clear後に各identityをresolve。drop counterで破棄回数を確認し、clearを再実行。workerからもclearを呼ぶ | 成功したclearは全live stateを一度だけ破棄し、全identityを無効化する。再実行は状態を変えない。worker clearはWrongThreadとなり、drop回数・resolve可能性を変えない | [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-value-001"></a>UT-VALUE-001 | 空・日本語・埋め込みNUL・不正UTF8をtyped入力し、成功後に元bufferを書き換える | 成功値は指定byte列のcopy、不正UTF8は拒否。正規化・置換・NUL終端切捨てをしない | [RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003)、[RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020)、[RQ-05-004](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-004)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01) |
| <a id="ut-value-002"></a>UT-VALUE-002 | UTF8出力を容量0・必要量-1・必要量で取得し、不足bufferをsentinelで初期化する | 必要byte数を返し、不足時はbufferを部分変更しない。可変値の照会後変更と固定snapshotの読取りを区別 | [RQ-04-021](../11_requirements/RQ-04_abi-boundary.md#rq-04-021)<br/>[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01) |
| <a id="ut-value-003"></a>UT-VALUE-003 | BOOL/I64/UTF8/F64/DECIMALの同値・異値・異kind setterと準備失敗を試す | typed保存と同値判定はDDどおり。同値成功は不要通知なし。失敗した操作だけが値・通知を変更しない | [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)、[RQ-04-026](../11_requirements/RQ-04_abi-boundary.md#rq-04-026)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01) |
| <a id="ut-value-004"></a>UT-VALUE-004 | 内部typed値のBOOL/I64/UTF8/F64について同kindの同値・異値、異kind、F64特殊値を比較する | 同kindだけを比較し、UTF8はbyte列、F64は承認済み同値規則を使う。DECIMAL・Property commit・通知はこの単体の対象外 | [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018)、[RQ-02-020](../11_requirements/RQ-02_core-features.md#rq-02-020)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |
| <a id="ut-value-005"></a>UT-VALUE-005 | Decimalのscale・係数境界、同値/異値、正負zero、格納表現の保持、TypedValue異kind比較を試す | scale 0–31・係数0–2^58−1だけを受理。末尾zero差と正負zeroを数値同値と判定し、保持中のsign/scale/係数は書き換えない | [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |
| <a id="ut-value-006"></a>UT-VALUE-006 | 検証済み候補として全kindの同値/差分/異kindをValueStorageへ適用し、結果と保持値を確認する | 同値ならUnchangedで既存表現を保持。差分はChangedとしてcommit。異kindは失敗し保存値を変更しない。validator/通知はこの単体の対象外 | [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018)、[RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019)、[RQ-02-020](../11_requirements/RQ-02_core-features.md#rq-02-020)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |
| <a id="ut-value-007"></a>UT-VALUE-007 | DECIMAL canonical `uint64_t`のsign/scale/係数境界と全scaleの往復変換を試す | bit63がsign、bits58–62が0–31のscale、bits0–57が係数。各64-bit値を同じ数値へ往復でき、byte order変換を介在させない | [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01) |
| <a id="ut-numeric-001"></a>UT-NUMERIC-001 | F32完全表現値、隣接F32の中点を二つの偶奇方向、signed zero、infinity、NaN、有限範囲超過で変換する | EXACTは非完全値を拒否、NEAREST_EVENは偶数側。zero符号とinfinityの意味を保持、NaN payload一致は要求しない | [RQ-04-029](../11_requirements/RQ-04_abi-boundary.md#rq-04-029)<br/>[TC-NUM-01](../30_release_test/requirement-test-matrix.md#case-tc-num-01) |
| <a id="ut-abi-003"></a>UT-ABI-003 | F32/F64変換APIを正しいmodeとnull required outputで呼び、failure前に出力が初期化されることを確認する | 成功時に変換結果を返す。無効mode/null output/不正確または範囲外の変換は拒否し、required outputとoptional error handleを0にする | [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-029](../11_requirements/RQ-04_abi-boundary.md#rq-04-029)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-NUM-01](../30_release_test/requirement-test-matrix.md#case-tc-num-01) |
| <a id="ut-abi-001"></a>UT-ABI-001 | required outが有効な各入口で失敗させ、optional error有無とerror生成OOMを組み合わせる | 定義済みout初期値と元statusを保持。任意の不正addressが安全に読書きできる保証を追加しない | [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-abi-002"></a>UT-ABI-002 | `get_version`を3つの有効なout pointerで呼び、次に各pointerを個別にnullにして呼ぶ | 成功時は`UI_OK`と`1.0.0`。nullがある呼出しは`UI_INVALID_ARGUMENT`で、非nullの全out値は0。任意の不正addressの安全性は保証しない | [RQ-04-009](../11_requirements/RQ-04_abi-boundary.md#rq-04-009)、[RQ-04-028](../11_requirements/RQ-04_abi-boundary.md#rq-04-028)、[RQ-04-030](../11_requirements/RQ-04_abi-boundary.md#rq-04-030)<br/>[TC-ABI-COMP-02](../30_release_test/requirement-test-matrix.md#case-tc-abi-comp-02)、[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01) |
| <a id="ut-abi-004"></a>UT-ABI-004 | `runtime_create`を有効なout pointerとoptional error pointerで呼び、required runtime outputをnullにして呼ぶ。Core creation panicも注入する | 成功時は`UI_OK`、非zero opaque handle、`out_error = 0`。null required outputは`UI_INVALID_ARGUMENT`。panicは`UI_INTERNAL_ERROR`へ変換される。任意の不正addressの安全性は保証しない | [RQ-04-004](../11_requirements/RQ-04_abi-boundary.md#rq-04-004)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-015](../11_requirements/RQ-04_abi-boundary.md#rq-04-015)、[RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01) |
| <a id="ut-abi-005"></a>UT-ABI-005 | `runtime_create`で得たhandleを作成threadとworkerから`runtime_is_ui_thread`へ渡す。zero/unknown handle、null required output、query panicも個別に試す | 作成threadは`UI_OK/1`、workerは`UI_OK/0`。zero/unknown handleは`UI_INVALID_HANDLE`で有効なscalar出力を0にする。null outputは`UI_INVALID_ARGUMENT`。panicは`UI_INTERNAL_ERROR`へ変換される | [RQ-04-004](../11_requirements/RQ-04_abi-boundary.md#rq-04-004)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-015](../11_requirements/RQ-04_abi-boundary.md#rq-04-015)、[RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01) |
| <a id="ut-runtime-handle-ret-001"></a>UT-RUNTIME-HANDLE-RET-001 | ローカルRuntime handle registryにRuntimeを登録し、同一tokenをretain。元tokenと新tokenを解決し、unknown tokenもretainする | 新tokenは元と異なる非zero tokenで同じRuntimeCoreを指す。元tokenは有効なまま。unknown tokenはInvalidTokenで拒否されregistryは変化しない | [RQ-04-005](../11_requirements/RQ-04_abi-boundary.md#rq-04-005)、[RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)、[RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01) |
| <a id="ut-abi-006"></a>UT-ABI-006 | C ABI `handle_retain`でRuntime handleを複製し、元と新token双方でthread query。zero/unknown token、null output、OOM status、panicも個別に試す | 成功時は異なる非zero tokenが同じRuntimeを参照し、元tokenも有効。失敗時は有効な`out_owned`を0に初期化。invalid tokenは`UI_INVALID_HANDLE`、null outputは`UI_INVALID_ARGUMENT`、確保失敗は`UI_OUT_OF_MEMORY`、panicは`UI_INTERNAL_ERROR` | [RQ-04-005](../11_requirements/RQ-04_abi-boundary.md#rq-04-005)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-015](../11_requirements/RQ-04_abi-boundary.md#rq-04-015)、[RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)、[RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01) |
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

### RuntimeCore UI object store shutdown cleanup — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-ui-store-001"></a>UT-RUNTIME-UI-STORE-001 | RuntimeCoreのstoreへdropを観測できるUI stateを複数登録し、各identityをresolve。workerからresolveを拒否されることを確認後、Runtime UI threadからshutdown drainを実行する | workerアクセスはWrongThreadで状態を変えない。shutdown drainは全stateをUI thread上で一度ずつ破棄し、identityを無効化してからqueueをClosedにする。再度drainしても追加破棄しない | [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)、[RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Runtime object registration and cleanup — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-object-reg-001"></a>UT-RUNTIME-OBJECT-REG-001 | UI thread上でstate factoryとHandleRegistryを使ってobjectを登録し、tokenを検証してcontrol identityからstateをresolve。workerから登録を試しfactoryが呼ばれないことを確認。workerから最終releaseし、UI threadでshutdown drain。別fixtureでtoken発行失敗を注入し、shutdown後の登録も試す | 成功時はtoken・Runtime identity・control identity・store stateが一つのobjectを指す。worker releaseはacceptedとなり、UI drainでstateを一度だけ破棄してからcontrol cleanupを完了する。worker/Closed登録はfactoryを呼ばず拒否する。登録失敗はstateをUI threadでrollbackし、live tokenも残さない | [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)、[RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Core Property state — implementation slice

Candidates are already validated values in this slice. Validator execution, dirty tracking, Binding propagation, and notifications are excluded.

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-001"></a>UT-CORE-PROPERTY-001 | RuntimeとHandleRegistryへ初期値付きPropertyを登録し、tokenからUI thread上で読み出す。異なる同kind候補、同値候補、異kind候補を順に適用する。invalid token・wrong kind・別Runtime・worker thread・Closed Runtimeでも操作する | 初期値を型付きで読める。差分のみChangedとしてcommitし、同値は既存表現を維持してUnchanged、kind mismatchは値を変えず失敗。handle/Runtime不一致、worker read/update、Closedで状態へアクセスせず失敗。token validation pinをstate access中維持し、登録済みRuntime cleanupでstateを破棄する | [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018)、[RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01) |

### Core Property change queue — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-notify-001"></a>UT-CORE-PROPERTY-NOTIFY-001 | 一つのRuntimeへProperty A/Bを登録し、Aを変更、Bを変更、Aを別値へ変更してからpending generationを取得する。次にAを変更して次generationを取得する。別fixtureで同値・異kind・invalid handleの各更新後にpending queueを取得する | 最初のgenerationは[A, B]のidentity順で一件ずつ含み、Aの最終値を読む。同じidentityの複数変更は一件にcoalesceする。generation取得後のA変更は次generationへ分離される。同値・失敗更新はqueueへ追加しない | [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01), [TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Core Binding OneWay graph — implementation slice

This slice covers topology only. Each Property starts as a singleton synchronization group; TwoWay group merging and Property values are separate units.

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-binding-graph-001"></a>UT-CORE-BINDING-GRAPH-001 | 空のgraphへA→B、B→Cをprepare/commitする。自己接続A→A、既存A→B重複、別入力元C→B、循環C→Aを個別に試し、各拒否後に合法な別edgeを追加して既存topologyの保持を確認する | 片方向chainを受け入れる。self/cycle/duplicate/second-inputはそれぞれ識別可能なCore errorとして拒否される。拒否された候補はedge数・受理済み辺・後続の合法接続へ影響しない。OneTime、TwoWay merge、Property値や通知は対象外 | [RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)、[RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01) |
| <a id="ut-core-binding-graph-002"></a>UT-CORE-BINDING-GRAPH-002 | 既存OneWay graphでTwoWay mergeをprepare/commitし、property identityが同じgroupへ解決されることを確認する。別fixtureでmergeにより自己edge・DAG cycle・同一targetへの複数入力が生じる候補を準備し、拒否後に受理済みgroup/edgeと後続の合法mergeを検査する。graph変更後のstale proposalもcommitする | 影響のないmergeは同一同期groupを作る。merge後に自己edge・cycle・複数入力元が生じる場合は識別可能なCore errorで拒否し、merge由来の重複edgeは複数入力元より優先する。拒否/stale proposalは既存group・edgeを変更しない | [RQ-07-003](../11_requirements/RQ-07_binding.md#rq-07-003)、[RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)、[RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01) |

### Runtime Subscription owner registry — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-subscription-003"></a>UT-CORE-PROPERTY-SUBSCRIPTION-003 | Runtime UI threadでidentityにSubscriptionを登録し、caller側Arcをdropしてweak参照を確認。Runtime経由でinvokeし、explicit cancelしてweak参照・callback・destroyを観測する。duplicate identity登録とworker登録も試す | Runtime ownerによりcaller Arc drop後もSubscriptionが生存し、Runtime pinでdeliveryできる。cancelはownerを外してclosureをcancelしdestroy一回、以後pin不可。duplicate・worker拒否は既存登録を変更せず、callerのSubscriptionを保持する | [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Property Subscription weak index — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-subscription-004"></a>UT-CORE-PROPERTY-SUBSCRIPTION-004 | PropertyStateへactiveなSubscription A/Bを追加し、Aをcancel後caller ownerをdropする。snapshotしてBとのpointer identityを確認する。Bをcancelした後に再度snapshotする | PropertyStateは強ownerを作らず、Aはcaller/Runtime owner drop後に解放可能。snapshotはlive active Bだけを一時pinし、cancel後のBと期限切れAを含まない。snapshot完了後のcallback実行はこのtest対象外 | [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01) |

### Runtime Property Subscription index integration — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-subscription-005"></a>UT-CORE-PROPERTY-SUBSCRIPTION-005 | 同一Propertyに異なるSubscription A/Bを順に登録しcaller Arcをdropする。Property indexのsnapshotを得て順序とweak pinを確認する。Aだけcancelし再snapshot、次にBをcancelする。別fixtureでは同一Aの重複登録とworker登録を試す | Runtimeは同一Propertyの複数Subscriptionを登録順に強保持する。Property indexは弱参照だがsnapshot中は両者をpinする。AのcancelはAだけをowner/indexから除き、Bと登録順を保つ。重複・worker拒否はindexと既存ownerを変えず、失敗候補のcaller ownershipを維持する。callback実行は対象外 | [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Property delivery generation snapshots — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-notify-002"></a>UT-CORE-PROPERTY-NOTIFY-002 | 複数Propertyの値を変更順を入れ替えて更新し、Property Aにはsubscriberを2件登録する。delivery generationをprepareしてProperty identity順と各subscriber順を観測する。prepare後にAを再更新して次generationもprepareする。別fixtureではpending列へ有効identityの後にstale identityを入れprepareを失敗させる | itemは変更Propertyの初回変更順、subscriberは登録順を保ち、prepare中はcallbackを呼ばない。成功generationから返るArc pinはitem保持中にSubscriptionを生存させる。prepare後の更新は次generationに分離される。Property解決失敗時はgeneration全体が元の順序でqueueに戻り、部分snapshotを返さない | [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Property delivery item callback invocation — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-notify-003"></a>UT-CORE-PROPERTY-NOTIFY-003 | Property Aにsubscriber A1を登録してdelivery itemをprepareする。A1 callbackからPropertyを再更新しsubscriber A2を追加する。次generationをprepareし、A1 callbackから同item内で後続のA2をcancelしてからitem処理を続ける | callbackはitem snapshot順に一度ずつ、Runtime/Property/queue lockなしで実行される。A1のsetter変更は現在itemへ再入せず次generationにqueueされる。A2は最初のsnapshotに入らず、次generationのsnapshotに入る。次generationでA1がA2をcancelした後はA2 callbackを開始しない。cancel後のdestroyは一度だけ | [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Property delivery bounded slice — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-notify-004"></a>UT-CORE-PROPERTY-NOTIFY-004 | 129個のPropertyを変更順に更新し、各Propertyへ一つのactive subscriberを登録する。workerからsliceを試した後、UI threadでslice processorを2回呼ぶ | worker呼出しはWrongThreadで拒否しcallbackを開始しない。UI threadの1回目は先頭128 itemだけをsubscriber snapshot完了まで配信しremaining workを示す。2回目は最後の1 itemを配信しwork remainingをfalseにする。callback順はProperty変更順のまま、重複・欠落・slice途中で分割されたitemはない | [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Core Property Subscription callback closure — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-subscription-002"></a>UT-CORE-PROPERTY-SUBSCRIPTION-002 | callback/destroy probeをuserdataにしたSubscriptionを作り、通常invoke、cancel、再invokeを行う。別fixtureではcallbackから同じSubscriptionをcancelし、callback復帰後のdestroy回数を観測する | active中invokeはcallbackを一回実行し、cancel前にdestroyしない。cancel後のinvokeは拒否する。自己cancelではcallback復帰前にdestroyせず、復帰後に一回だけdestroyする。外部同期化を伴わずcallbackを再入可能に呼べる | [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)<br/>[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01) |

### Core Property Subscription lifecycle state — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-subscription-001"></a>UT-CORE-PROPERTY-SUBSCRIPTION-001 | fresh lifecycleでdelivery開始と終了を行う。次にdelivery開始後にcancelし、cancel後の開始、in-flight終了、再cancelを順に行う。別fresh lifecycleはdeliveryなしでcancelを繰り返す | active中だけdelivery開始を受け付ける。cancel後の開始は拒否する。実行中deliveryがあるcancelはdestroy-readyを返さず、最後のdelivery終了でのみ一度readyになる。in-flightなしのcancelは一度だけreadyになり、再cancel・余分な終了は二重破棄を示さない | [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)<br/>[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01) |

### Core Property kind query — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-kind-001"></a>UT-CORE-PROPERTY-KIND-001 | 5種の初期値でPropertyを登録し、それぞれのtokenからUI thread上でkindを照会する。invalid token・wrong kind・別Runtime・worker thread・shutdown済みobjectも照会する | BOOL/I64/UTF8/F64/DECIMALを各対応kindとして返す。OBJECTは生成できない。handle/kind/Runtime/thread/lifecycle不整合は失敗し、値stateを変更しない。validation pinを照会中維持する | [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01) |

### Runtime handle registration and borrowed resolution — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-handle-reg-001"></a>UT-RUNTIME-HANDLE-REG-001 | 異なるRuntimeCoreをtableへ登録し、得られたtokenを同threadとworkerから解決。未登録tokenとzero tokenも解決し、table lockをpoisonさせたinsertも試す | tokenは非zeroかつ別Runtime間で異なる。解決結果は該当RuntimeCoreと同一ownerを指す。未登録/zero tokenはInvalidTokenで拒否され、tableを変更しない。worker解決も安全だがUI操作権を与えない。poison時は失敗し、callerのRuntime ownerを維持する | [RQ-04-004](../11_requirements/RQ-04_abi-boundary.md#rq-04-004)、[RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### C ABI Runtime UI-loop bridge registration — implementation slice

The C ABI bridge transfers one `(wake, userdata, destroy)` tuple to the Runtime only after successful UI-thread registration. The wake callback may run from any thread and only schedules a future host-loop turn. Dispatcher reports normal work and shutdown control work with the corresponding approved priority.

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-abi-007"></a>UT-ABI-007 | Runtimeを作成し、UI threadからbridgeを登録。通常dispatchとshutdown優先wakeを起こす。次に重複登録、worker登録、無効Runtime、null callbackを個別に試す | 成功時だけwake/userdata/destroyの所有がRuntimeへ移る。通常workは`UI_WAKE_NORMAL`、shutdown controlは`UI_WAKE_CONTROL`を一度通知する。登録失敗は元ownerを保ち、bridge登録状態を変えない。callbackはthread-safe userdataを使いunwindしない契約とする（panic注入試験はしない） | [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-abi-shutdown-001"></a>UT-ABI-SHUTDOWN-001 | C ABIでASYNC/SYNC shutdownを要求し、host event scopeを閉じてcleanup。requestを任意threadからquery・releaseし、無効mode/handleとnull required outputも試す | callback境界ではUI_PENDINGとowned receiptを返し、outer scope endでのみCompleted。worker SYNCはcleanup後にUI_OK、release済みreceiptは以後無効だが要求は取消されない。失敗時out handleは0 | [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)、[RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006)、[RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01) |
| <a id="ut-abi-host-stop-001"></a>UT-ABI-HOST-STOP-001 | 受付前のhost停止、受付後の停止、同一UI host再開後の同request retryを個別に順序制御する | 受付前停止はUI_UNSUPPORTED/out=0。受付後停止はUI_PENDINGのままqueryでHostUnavailableとreasonを返しClosed扱いにしない。resume後の明示retryは同じrequestをPendingへ戻し、再度outer boundaryでCompleted | [RQ-06-007](../11_requirements/RQ-06_threading-dispatch.md#rq-06-007)、[RQ-06-012](../11_requirements/RQ-06_threading-dispatch.md#rq-06-012)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01) |

### C ABI Runtime post and host dispatch turn — implementation slice

The host invokes `runtime_dispatch_turn` on its UI thread after the bridge wake callback has returned. One call executes at most one queued callback; a nested host event scope is closed before return, so shutdown requested by the callback is committed at the correct outer boundary.

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-runtime-dispatch-001"></a>UT-RUNTIME-DISPATCH-001 | dispatcherへ複数workをpostし、空queue、連続turn、dispatch中の追加post、reentrant turn、pending shutdownを順に操作する | 一turn最大1 callback、FIFO、非再帰を保つ。pending shutdown中は通常workを実行せず、外側scope closeでshutdownを先行する。残workはwakeを取りこぼさず次turnへ進む | [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01) |
| <a id="ut-abi-008"></a>UT-ABI-008 | C ABIからcallback/userdata/destroyをpostし、UI/worker両threadの受付、dispatch turn、queue cancellation、invalid handle/null callback/destroy、bridge未登録、wrong threadを試す | 成功時のみCoreが所有し、callbackはUI threadで高々一回、destroyは実行またはcancel後にUI threadで一回。拒否時はcaller ownershipを維持し、callback/destroyを呼ばない。UI_WAKE_NORMAL受信後のdispatch turnでFIFOに一件進む。失敗時statusを規則どおり返す | [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01) |
| <a id="ut-abi-009"></a>UT-ABI-009 | UI threadでBOOL/I64/UTF8/F64/DECIMALを各createし、`property_kind`のkind値を照会してPropertyとRuntimeを個別にrelease。workerからcreate/kind queryも試す。無効Runtime/property、BOOL値2、UTF8 null+positive length、不正UTF8、null required outも個別に試す | 成功時はkind 1/2/3/5/6と非zero所有Property handle、`out_error=0`。UTF8のembedded NULを含む入力を受理する。invalid入力・handle・wrong threadは仕様statusで拒否し、valid outputは0、Property/registry/Runtime ownerに部分状態を残さない。Property release後だけproperty tokenがinvalidになり、Runtime ownerは独立して有効 | [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-018](../11_requirements/RQ-04_abi-boundary.md#rq-04-018)、[RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-abi-010"></a>UT-ABI-010 | 各kindの初期Propertyから一致するtyped getterを呼び、次に異なるkind用getter・無効handle・worker thread・null required outputを試す。UTF8は`(NULL,0)`照会、embedded NULを含む十分なbuffer、短いbuffer、null+positive capacityを個別に試す | BOOL/I64/F64/DECIMALは初期値のC表現をそのまま返し、出力は失敗前に0。型違いは`UI_WRONG_TYPE`、無効tokenは`UI_INVALID_HANDLE`、workerは`UI_WRONG_THREAD`、null required outputは`UI_INVALID_ARGUMENT`。UTF8照会は必要byte数、成功copyは終端NULなしの完全なbyte列、短bufferは`UI_BUFFER_TOO_SMALL`と必要長のみ更新しbuffer全体を保持、null+positive capacityは`UI_INVALID_ARGUMENT`。UTF8 size-queryとcopyの間のsnapshot一致は要求しない | [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)、[RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020)、[RQ-04-021](../11_requirements/RQ-04_abi-boundary.md#rq-04-021)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01) |
| <a id="ut-abi-011"></a>UT-ABI-011 | BOOL/I64/UTF8/F64/DECIMAL各Propertyをcreateし、異なるcandidateと同値candidateをsetterへ渡してgetterで確認する。kind違い・無効handle・worker thread・BOOL=2・UTF8 null+positive length・不正UTF8を試す。F64は異なるNaN payloadと符号付きzero、DECIMALは`1.0`と`1.00`を比較する | 変更値はsetter直後のgetterで観測される。同値候補は格納済みF64 bit pattern/DECIMAL canonical bitsを保持する。kind違いは`UI_WRONG_TYPE`、無効tokenは`UI_INVALID_HANDLE`、workerは`UI_WRONG_THREAD`、BOOL不正/null positive UTF8/invalid UTF8は`UI_INVALID_ARGUMENT`相当の定義statusで拒否し、値を変えない。UTF8はembedded NULを含む正確なbyte列を保持する | [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018)、[RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019)、[RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-019](../11_requirements/RQ-04_abi-boundary.md#rq-04-019)、[RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020)、[RQ-04-025](../11_requirements/RQ-04_abi-boundary.md#rq-04-025)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-CORE-OBJ-01](../30_release_test/requirement-test-matrix.md#case-tc-core-obj-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### C ABI Property setter host scope — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-abi-012"></a>UT-ABI-012 | RuntimeにUI-loop bridgeとProperty subscriberを登録する。typed setter相当の更新をouter host scope外で実行し、別fixtureではouter scope内で変更を保留した後、setter scope内でkind不一致を起こす | 成功したouter setterはscope closeまでに通知を配信する。setterが既存scope内で呼ばれた場合、nested closeでは配信せずouter closeで一度配信する。kind不一致は値を変更せずWrongType相当を返し、nested scopeを閉じてouter boundaryを保つ | [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Core Property notification host boundary — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-property-notify-005"></a>UT-CORE-PROPERTY-NOTIFY-005 | RuntimeへUI-loop bridgeを登録し、Propertyを変更する通常postを二件FIFO登録する。最初のpost callbackでPropertyを変更し、一度dispatch turnを実行する。別fixtureでは129個の通知itemをouter host scope closeまで保留し、さらに通知callbackからshutdownを要求するfixtureも実行する | 通常post callbackが戻った後、次のpostより前にnotification sliceが開始する。128 complete item後は残り順序を保持してNormal wakeを一度予約し、次turnで残りを配信する。outer boundaryで既にshutdown pendingなら通知を開始せずshutdown cleanupをcommitする。notification callback中にshutdown要求が出た場合は現在item全subscriberを完了してから残itemより先にshutdownする。nested scope closeでは通知配信を開始しない | [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-core-property-notify-006"></a>UT-CORE-PROPERTY-NOTIFY-006 | bridge付きRuntimeでPropertyとSubscriptionを登録し、outer host scope中にPropertyを変更して通知を保留する。同scope内でAsync shutdownを要求してからscopeを閉じる。callback/destroyの実行threadと完了receiptも確認する | shutdown controlがpendingなら保留notification callbackを開始せず、Runtime shutdown cleanupがpending generation・prepared delivery・active Subscription ownerを破棄する。closure destroyはUI threadで一度だけ実行され、completionはcleanup後にCompletedとなる。shutdown後に通知sliceを再開できない | [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01) |
| <a id="ut-core-property-notify-007"></a>UT-CORE-PROPERTY-NOTIFY-007 | Propertyに複数Subscriptionを登録し、pending changeとprepared deliveryを作る。最後のProperty tokenをreleaseし、RuntimeがRunningのままouter host safe pointを通す。さらに通知callback中に同Propertyをreleaseするfixtureを使い、同じitemの後続subscriber開始も確認する | finalizerはowner bucketを取り除き、pending change/prepared itemを除去してからProperty stateを破棄する。active SubscriptionはUI threadでcancelされ、closure destroyは各一回。pendingまたは後続callbackは開始しない。Property破棄後も別Propertyを登録・参照でき、RuntimeはRunningを保つ | [RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |
| <a id="ut-abi-013"></a>UT-ABI-013 | C ABIからProperty subscriptionを登録し、初期値・同値/変更setter後のcallback、Subscription tokenのretain/release、明示cancelと繰返しcancel、callback内self-cancelを試す。null required out/callback/destroy、invalid Property/Subscription、worker threadでのregister/cancelも試す | 登録成功時だけclosure所有権をCoreへ移し、初期通知は行わない。Property変更通知はcallback引数なしでUI threadに届く。handle releaseだけでは通知を解除せず、cancelは次のcallback開始を止め、in-flight callback復帰後にdestroyを一度実行する。有効なcancelの繰返しは成功し、handleは別releaseまで有効。失敗時はout=0・destroyなしでcaller所有を保ち、wrong thread/invalid handleの状態は変化しない | [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)<br/>[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01)、[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01) |

### Binding初期値コピー候補の準備 — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-binding-init-copy-001"></a>UT-CORE-BINDING-INIT-COPY-001 | target ValueStorageへ異kind/同値/差分のsource値を候補準備し、成功候補を後から適用する。UTF-8 sourceを候補準備後に破棄する | 異kindではtargetを変更しない。同値は候補なしで既存representationを保持。差分候補の準備中はtargetを変更せず、明示適用でのみ更新。UTF-8候補はsourceから独立した所有bytesを保持 | [RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |

このoracleはtyped candidateの準備・適用前不変性のみを検証する。target validator、Binding graph/handle、notification queueとの原子的commitは含まない。

### Runtime OneWay Binding初期同期 — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-runtime-binding-init-001"></a>UT-CORE-RUNTIME-BINDING-INIT-001 | 同一Runtime内のI64 source/targetでOneWay初期同期を作り、変更値・同値・kind mismatch・graph topology拒否を試す。別Runtime handleとworker threadからも呼ぶ | 値変更時はtarget値とgraph edgeが一緒にcommitされ、target通知は一件。source/target同値ではedgeだけをcommitし通知なし。kind/topology/Runtime/thread拒否ではtarget値・既存graph・notification queueを変更しない | [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)、[RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-BIND-INIT-REJECT-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-init-reject-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |

このoracleはCore内部OneWayの初期値・graph・notification commitのみを対象とする。application validator、Binding owner/handle、後続propagation、TwoWay/OneTimeは別単位。

### Runtime TwoWay Binding初期同期とgroup commit — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-runtime-twoway-init-001"></a>UT-CORE-RUNTIME-TWOWAY-INIT-001 | 同一Runtime内の同kind source/targetでTwoWay初期同期を行い、差分値・同値・kind mismatch・TwoWay group重複・既存OneWay topologyと循環するmergeを試す。通知queueに既存targetがあるケースも確認する | 差分時はsource値・target値・target通知一件・group mergeがまとめてcommitされる。同値はgroup mergeのみcommitし通知しない。kind/topology拒否では値・既存group/edge・通知queueを変更しない。既存queueにtarget通知がある場合は重複しない | [RQ-07-003](../11_requirements/RQ-07_binding.md#rq-07-003)、[RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)、[RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-BIND-INIT-REJECT-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-init-reject-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |

このoracleはCore内部TwoWay初期値・同期group・notification reservationのatomic commitを検証する。group値伝播、direct setter順序、validator/Binding error、public owner/handle/cancel、C ABIは別単位。

### OneWay伝播対象edgeと評価順 — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-binding-propagation-order-001"></a>UT-CORE-BINDING-PROPAGATION-ORDER-001 | OneWay DAGにchainとbranch、無関係なedgeを作り、dirty source集合から伝播snapshotを準備する。TwoWay merge済みgroupのmember identityからも準備する。空集合も試す | 到達可能edgeだけを一度ずつ返し、各上流edgeを下流edgeより先に並べる。branch同士は安定した順序を保つ。TwoWay group memberをsourceとして正規groupのedgeへ到達する。空入力は空snapshot。値・graph topologyを変更せず、snapshot生成失敗時もgraphを維持 | [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |

このoracleは入力dirty identityから到達するOneWay edge snapshotの対象範囲と決定的な上流順序のみを検証する。Property値伝播、subscriber通知順、TwoWay値同期、Binding lifecycleは含まない。

### Runtime OneWay伝播候補の準備 — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-runtime-binding-prop-001"></a>UT-CORE-RUNTIME-BINDING-PROP-001 | Runtime内に同kindのOneWay chain/branchを初期同期で作り、sourceを直接変更する。dirty identityからcandidate準備を行う。中間targetもdirtyな場合と、関連しないedgeを含む場合を別fixtureで確認する | 候補は上流順に作られ、chainでは上流target候補を次edgeのsourceとして使う。dirtyなtargetもsourceから到達するedgeの候補を受ける。無関係targetは含まれない。準備前後で全Property値、graph、notification queueは不変。kind/copy/graph preparation error時も部分candidateを公開しない | [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |

このoracleはRuntime内OneWay伝播値candidateのowned preparationと部分公開防止のみを検証する。target validator、値commit、notification登録、subscriber実行、TwoWay/OneTimeは含まない。

### Runtime OneWay伝播の値・通知commit — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-runtime-binding-commit-001"></a>UT-CORE-RUNTIME-BINDING-COMMIT-001 | 初期同期済みOneWay chain/branchでsourceと中間targetを同じgenerationに直接変更し、そのdirty identity群をRuntime propagation commitへ渡す。別fixtureは伝播先が既に通知queueにある場合と、全targetが同値の場合を試す | 上流値がchain全体とbranchへcommitされる。直接変更済みtargetは通知queue上で重複せず初回順を保ち、新たに変更したtargetは次generationへ追加。全候補とqueue slot予約が済む前は値・通知を変更しない。graphとsource値を保持し、queue commit後にtarget getterは新値を返す | [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |

このoracleはRuntime内OneWay target値commitと次generation notification enqueueのみを検証する。application validator、遅延subscriber callback、TwoWay/OneTime、public Binding lifecycleは含まない。

### Runtime通知slice内のOneWay伝播順 — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-runtime-binding-delivery-001"></a>UT-CORE-RUNTIME-BINDING-DELIVERY-001 | source→middle→targetとsource→branch OneWayを作り、source/middle subscriberを登録する。sourceとmiddleを同じgenerationで直接変更し、target/branch subscriberも含めRuntime notification sliceを実行する | current-generation source/middle callback開始前に伝播値がcommit済みで、source callbackはmiddleの最終値、middle callbackはtargetの最終値を読める。source/middle通知はcurrent generation、未通知のtarget/branch通知は後続generationで順に一回ずつ届く。middleはcurrent itemで通知済みなので次generationに重複登録しない。callback再帰なし。slice item上限・shutdown確認を維持する | [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-005](../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-008](../11_requirements/RQ-07_binding.md#rq-07-008)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-PROP-01](../30_release_test/requirement-test-matrix.md#case-tc-prop-01) |

このoracleはOneWay propagation→subscriber deliveryのRuntime順序と次generationの非再帰配送を検証する。application validator、Binding error、TwoWay/OneTime、public ABIは含まない。

### Binding初期値・OneWay graph接続の準備 — implementation slice

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-core-binding-init-prepare-001"></a>UT-CORE-BINDING-INIT-PREPARE-001 | 既存graphへ合法OneWay候補を準備し、source値からtarget用候補を作る。別fixtureでself/cycle/duplicate/second-inputとkind mismatchを発生させる | 成功準備はowned proposalを返すがgraph edge/generationとtarget値をcommit前に変更しない。topologyまたはkind rejectionは既存edge/generationとtarget値を維持し、候補を公開しない | [RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-007](../11_requirements/RQ-07_binding.md#rq-07-007)、[RQ-07-009](../11_requirements/RQ-07_binding.md#rq-07-009)、[RQ-07-010](../11_requirements/RQ-07_binding.md#rq-07-010)、[RQ-07-015](../11_requirements/RQ-07_binding.md#rq-07-015)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-BIND-INIT-REJECT-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-init-reject-01) |

このoracleはOneWay proposalの準備・論理状態不変性のみを検証する。application validator、candidate commit、通知queue、Runtime owner/handleは含まない。

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
| 2026-10-18 | C ABI Runtime handle retain追補 | 利用者 | 「OK」により、既存Runtime ownerを維持した別token発行とinvalid source拒否をfocused test案として追加。Runtime releaseやshutdown動作の承認を意味しない |
| 2026-10-18 | C ABI Runtime UI thread query追補 | 利用者 | 「OK」により、creator/worker threadからの照会とinvalid handle statusをfocused testとして追加。承認済みRQ/DDの範囲内であり、Runtime APIの追加設計や製品test合格を意味しない |
| 2026-10-18 | C ABI Runtime UI-loop bridge追補 | — | DD v1.1のwake priorityとone-time bridge registrationをUT-ABI-007として具体化したレビュー案。公開契約の追加やtest合格を意味しない |
| 2026-10-19 | C ABI Runtime post/dispatch turn追補 | 利用者 | 「ok」により、選択済み`runtime_dispatch_turn`を含むUNIT-ABI-008の実装・動作確認を開始。DD v1.2の契約に沿うfocused test oracleを確定し、本unitの試験実施を承認。test合格は実行結果で別途記録 |
| 2026-10-19 | C ABI Property create/kind追補 | Codex | 承認済みRQ/DDからUNIT-ABI-009のfocused test oracleを登録 |
| 2026-10-19 | UT-ABI-009全文 | 利用者 | 「ok」により、Property create/kindのoracleとUNIT-ABI-009の実装・動作確認を承認。製品test合格は実行結果で別管理 |
| 2026-10-19 | C ABI Property getter追補 | Codex | 承認済みRQ/DDからUNIT-ABI-010のfocused test oracleを登録 |
| 2026-10-19 | UT-ABI-010全文 | 利用者 | 「ok」により、Property getter oracleとUNIT-ABI-010の実装・動作確認を承認。製品test合格は実行結果で別管理 |
| 2026-10-19 | C ABI Property setter追補 | Codex | 承認済みRQ/DDからUNIT-ABI-011のfocused setter oracleを登録 |
| 2026-10-19 | UT-ABI-011全文 | 利用者 | 「ok」により、Property setter oracleとUNIT-ABI-011の実装・動作確認を承認。通知購読・Validatorは別単位 |
| 2026-10-19 | Core Property change queue追補 | Codex | 承認済みRQ/DDからUNIT-CORE-040のfocused queue oracleを登録 |
| 2026-10-19 | UT-CORE-PROPERTY-NOTIFY-001全文 | 利用者 | 「ok」により、Property change queue oracleとUNIT-CORE-040の実装・動作確認を承認。subscriber配信は別単位 |
| 2026-10-19 | Core Property Subscription lifecycle追補 | Codex | 承認済みRQ/DDからUNIT-CORE-041のfocused lifecycle oracleを登録。callback実行・ABI登録・試験合格は別管理 |
| 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-001全文 | 利用者 | 「ok」により、lifecycle state oracleとUNIT-CORE-041の実装・動作確認を開始。callback実行・ABI登録・destroy callback実行は別単位 |
| 2026-10-19 | Core Property Subscription callback closure追補 | Codex | 承認済みRQ/DDからUNIT-CORE-042のfocused closure oracleを登録。Runtime integration・試験合格とは別管理 |
| 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-002全文 | 利用者 | 「ok」により、callback closure oracleとUNIT-CORE-042の実装・動作確認を開始。Runtime登録・公開ABIは別単位 |
| 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-002実行結果 | Codex | focused test 2件、workspace test 102件、開発ゲート成功。Runtime登録・公開ABIの未実装範囲は継続 |
| 2026-10-19 | Runtime Subscription owner registry追補 | Codex | 承認済みRQ/DDからUNIT-CORE-043のfocused owner oracleを登録。ABI token・Property index・shutdown drain・試験合格は別管理 |
| 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-003全文 | 利用者 | 「ok」により、Runtime owner registry oracleとUNIT-CORE-043の実装・動作確認を開始。公開handle integrationは別単位 |
| 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-003実行結果 | Codex | focused test 2件、workspace test 104件、開発ゲート成功。public handle・Property index・shutdown drainは未実装 |
| 2026-10-19 | Property Subscription weak index追補 | Codex | 承認済みRQ/DDからUNIT-CORE-044のfocused weak-index oracleを登録。delivery scheduling・試験合格は別管理 |
| 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-004全文 | 利用者 | 「ok」により、Property weak-index oracleとUNIT-CORE-044の実装・動作確認を開始。callback配信は別単位 |
| 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-004実行結果 | Codex | focused test 1件、workspace test 105件、開発ゲート成功。通知callback配信・public subscribe/cancel ABI・shutdown drainは未実装 |
| 2026-10-19 | Runtime Property Subscription index integration追補 | Codex | 承認済みRQ/DDからUNIT-CORE-045の複数購読・ordered weak-index oracleを登録。通知実行・公開ABI・shutdown drainは別管理 |
| 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-005全文 | 利用者 | 「ok」により、同一Propertyの複数購読、登録順snapshot、個別cancelを含むUNIT-CORE-045の実装・動作確認を開始。callback deliveryは別単位 |
| 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-005実行結果 | Codex | focused test 1件、関連owner test 2件、workspace test 106件、開発ゲート成功。通知dispatch・公開ABI・shutdown drainは未実装 |
| 2026-10-20 | Property delivery generation snapshot追補 | Codex | 承認済みRQ/DDからUNIT-CORE-046のordered delivery-item準備とfailure-atomic generation oracleを登録。callback実行・safe-point schedulingは別管理 |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-002全文 | 利用者 | 「ok」により、delivery snapshot生成、世代順序、失敗時generation復元を含むUNIT-CORE-046の実装・動作確認を開始。callback実行は別単位 |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-002実行結果 | Codex | focused test 2件、workspace test 108件、開発ゲート成功。callback dispatch・safe-point scheduling・Binding propagationは未実装 |
| 2026-10-20 | Property delivery item invocation追補 | Codex | 承認済みRQ/DDからUNIT-CORE-047の逐次callback・cancel再確認・次世代enqueue oracleを登録。safe-point schedulingは別管理 |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-003全文 | 利用者 | 「ok」により、callback内Property更新・追加登録・後続cancelを含むUNIT-CORE-047の実装・動作確認を開始。safe-point schedulingは別単位 |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-003実行結果 | Codex | focused test 1件、workspace test 109件、開発ゲート成功。outer safe-point scheduling・Binding propagation・shutdown interruptionは未実装 |
| 2026-10-20 | Property delivery bounded slice追補 | Codex | 承認済みDDの128 complete-item上限と保留順序をUNIT-CORE-048のfocused oracleとして登録。host wake schedulingは別管理 |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-004全文 | 利用者 | 「ok」により、129 itemを128件と1件のsliceへ分割するUNIT-CORE-048の実装・動作確認を開始。host safe-point接続は別単位 |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-004実行結果 | Codex | focused関連test 4件、workspace test 110件、開発ゲート成功。host wake/safe-point接続・Binding propagation・shutdown interruptionは未実装 |
| 2026-10-20 | Property notification host-boundary追補 | Codex | 承認済みRQ/DDからouter-boundary、slice wake、shutdown優先のUT-CORE-PROPERTY-NOTIFY-005案を追加。UNIT-CORE-049実装・動作確認の開始指示待ち |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-005全文 | 利用者 | 「ok」によりhost-boundary notification oracleとUNIT-CORE-049実装・動作確認を開始。試験合格は実行結果で別管理 |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-005実行結果 | Codex | focused 8件、workspace 114件、開発ゲート成功。外側host boundaryのslice接続とwake制御を確認。公開購読ABI・Binding・診断の完了を意味しない |
| 2026-10-20 | UT-ABI-012全文 | 利用者 | 「ok」により、C ABI Property setter host scopeのfocused oracleとUNIT-ABI-012の実装・動作確認を開始。試験合格は実行結果で別管理 |
| 2026-10-20 | UT-ABI-012実行結果 | Codex | focused 1件、workspace 115件、開発ゲート成功。outer/nested setter boundaryと失敗時scope closeを確認。公開購読ABI・Binding propagation全体の完了を意味しない |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-006追補 | Codex | 承認済みRQ/DDからshutdown優先時の保留通知破棄、active Subscription cancel、UI-thread destroyのoracleを登録。UNIT-CORE-050の実装・試験開始範囲 |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-006全文 | 利用者 | 「ok」により、shutdown時の保留通知破棄、active Subscription解除、UI-thread destroyのoracleとUNIT-CORE-050の実装・動作確認を開始 |
| 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-006実行結果 | Codex | focused 1件、workspace 115件、開発ゲート成功。shutdown precedence、pending queue cleanup、UI-thread destroy一回、完了receiptを確認。public subscribe ABIとProperty-first teardownは別単位 |
| 2026-10-21 | UT-CORE-PROPERTY-NOTIFY-007全文 | Codex | 承認済みRQ/DDからProperty最終release時のsafe-point cleanup、pending delivery除去、Subscription cancel、UI-thread destroy、Runtime継続のoracleを追補。UNIT-CORE-051の実装・動作確認開始範囲 |
| 2026-10-21 | UT-CORE-PROPERTY-NOTIFY-007実行結果 | Codex | focused 2件、workspace 117件、開発ゲート成功。worker release wake、prepared work除去、callback内release後の後続subscriber抑止、UI-thread destroy一回、Runtime継続を確認 |
| 2026-10-21 | UT-ABI-013全文 | Codex | 承認済みRQ/DDから公開Property subscribe/cancel、handle retain/releaseとclosure所有移転のoracleを追補。UNIT-ABI-013の実装・動作確認開始範囲 |
| 2026-10-21 | UT-ABI-013実行結果 | Codex | focused 2件、workspace 119件、開発ゲート成功。token retain/releaseとRuntime ownerの独立、明示・反復cancel、self-cancel、失敗時のcaller所有維持、wrong-thread拒否を確認 |
| 2026-10-21 | UT-CORE-BINDING-GRAPH-001 oracle | Codex | 承認済みRQ/DDからOneWay graphのself/cycle/duplicate/second-input拒否とfailure atomicityを追補 |
| 2026-10-21 | UT-CORE-BINDING-GRAPH-001全文 | 利用者 | 「ok」により、Core OneWay graph topologyのfocused oracleとUNIT-CORE-052の実装・動作確認を開始。TwoWay mergeとProperty伝播は別単位 |
| 2026-10-21 | UT-CORE-BINDING-GRAPH-001実行結果 | Codex | focused 2件、workspace 121件、開発ゲート成功。合法chain、self/cycle/duplicate/second-input拒否、旧generation proposal拒否を確認 |
| 2026-10-21 | UT-CORE-BINDING-GRAPH-002 oracle | Codex | 承認済みRQ/DDからTwoWay group merge後のself/cycle/duplicate/second-input制約とstale proposal拒否を追補 |
| 2026-10-21 | UT-CORE-BINDING-GRAPH-002全文 | 利用者 | 「ok」により、TwoWay synchronization-group mergeのfocused oracleとUNIT-CORE-053の実装・動作確認を開始。Property値同期とBinding propagationは別単位 |
| 2026-10-21 | UT-CORE-BINDING-GRAPH-002実行結果 | Codex | focused 5件、workspace 124件、開発ゲート成功。valid merge、self/cycle/duplicate/second-input rejection、stale proposal拒否を確認 |
| 2026-10-21 | UT-CORE-BINDING-INIT-COPY-001 oracle | Codex | 承認済みRQ/DDからtyped initial-copy candidateとtarget非変更のoracleを追補。Runtime graph/queueとのcommitは含まない |
| 2026-10-21 | UT-CORE-BINDING-INIT-COPY-001全文 | 利用者 | 「ok」により、typed candidate準備のfocused実装・動作確認を開始。target validatorとRuntime graph/queue integrationは別単位 |
| 2026-10-21 | UT-CORE-BINDING-INIT-COPY-001実行結果 | Codex | focused 4件、workspace 128件、開発ゲート成功。kind rejection、equivalent no-op、遅延commit、UTF-8独立所有を確認 |
| 2026-10-21 | UT-CORE-BINDING-INIT-PREPARE-001 oracle | Codex | 承認済みRQ/DDからtopology・typed initial candidateの組合せと失敗時不変のoracleを追補。Runtime integrationは別単位 |
| 2026-10-21 | UT-CORE-BINDING-INIT-PREPARE-001全文 | 利用者 | 「ok」によりOneWay prepare proposalのfocused実装・動作確認を開始。application validationとRuntime commitは別単位 |
| 2026-10-21 | UT-CORE-BINDING-INIT-PREPARE-001実行結果 | Codex | focused 3件、workspace 131件、開発ゲート成功。合法proposal、topology/kind rejection時のgraph・target不変、UTF-8候補所有を確認 |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-INIT-001 oracle | Codex | 承認済みRQ/DDからRuntimeのOneWay初期同期とvalue/graph/queue atomicity oracleを追補。public Binding contractは別範囲 |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-INIT-001全文 | 利用者 | 「ok」によりCore内部OneWay初期同期のfocused実装・動作確認を開始。application validatorとpublic lifecycleは別単位 |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-INIT-001実行結果 | Codex | focused 3件、workspace 134件、開発ゲート成功。changed/equivalent初期同期、target通知、kind/topology/Runtime/thread rejection時の不変性を確認 |
| 2026-10-21 | UT-CORE-BINDING-PROPAGATION-ORDER-001 oracle | Codex | 承認済みRQ/DDからdirty Property起点の到達可能OneWay edgeと上流順序を追補。値commit・Runtime統合は含まない |
| 2026-10-21 | UT-CORE-BINDING-PROPAGATION-ORDER-001全文 | 利用者 | 「ok」により、OneWay伝播順序snapshotのfocused実装・動作確認を開始。Property値伝播は別単位 |
| 2026-10-21 | UT-CORE-BINDING-PROPAGATION-ORDER-001実行結果 | Codex | focused 2件、workspace 136件、開発ゲート成功。branch/chain順序、到達範囲、TwoWay group解決、空入力を確認 |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-PROP-001実行結果 | Codex | focused 1件、workspace 137件、開発ゲート成功。chain/branch candidate順序、直接変更targetへの上流優先、値・dirty queue不変を確認 |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-COMMIT-001 oracle | Codex | 承認済みRQ/DDからOneWay target commit、notification reservation、次generation coalescingのoracleを追補。application validatorとsubscriber deliveryは含まない |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-COMMIT-001全文 | 利用者 | 「ok」により、Runtime OneWay target値・通知queueのatomic commit focused実装・動作確認を開始。validatorとsubscriber deliveryは別単位 |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-DELIVERY-001 oracle | Codex | 承認済みRQ/DDからBinding propagation前置、subscriberからの更新値read、次generation通知順序のoracleを追補 |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-DELIVERY-001全文 | 利用者 | 「ok」により、OneWay propagationとRuntime subscriber deliveryの接続をfocused実装・動作確認する |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-DELIVERY-001実行結果 | Codex | focused delivery 1件とgeneration snapshot 2件、workspace 139件、開発ゲート成功。current-generation targetの重複通知抑止と次generation target/branch順を確認 |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-COMMIT-001実行結果 | Codex | focused 1件、workspace 138件、開発ゲート成功。chain/branch値commit、次generation通知の順序・重複抑止、同値no-opを確認 |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-PROP-001 oracle | Codex | 承認済みRQ/DDからOneWay chain/branchのRuntime candidate準備と非変更性oracleを追補。validator・commit・通知は含まない |
| 2026-10-21 | UT-CORE-RUNTIME-BINDING-PROP-001全文 | 利用者 | 「ok」によりRuntime OneWay伝播候補準備のfocused実装・動作確認を開始。値commitとRuntime delivery接続は別単位 |
| 2026-10-21 | UT-CORE-RUNTIME-TWOWAY-INIT-001 oracle | Codex | 承認済みRQ/DDからTwoWay初期同期、group merge、notification予約のatomicity oracleを追補。group伝播とpublic Binding lifecycleは含まない |
| 2026-10-21 | UT-CORE-RUNTIME-TWOWAY-INIT-001開始 | 利用者 | 継続指示「ok」によりfocused Runtime実装・動作確認を開始 |
| 2026-10-21 | UT-CORE-RUNTIME-TWOWAY-INIT-001実行結果 | Codex | focused 2件、workspace 141件、開発ゲート成功。変更/同値の初期同期、group登録、kind/group topology rejection時の値・通知保持を確認 |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
| 0.2 | 2026-10-17 | HandleRegistry全体に先行するtoken issuerの最小sliceと4件の局所oracleを追補 | Codex | 本版全文承認済み |
| 0.3 | 2026-10-17 | HandleEntryのkind/Runtime照合と一公開参照のconsumeを試験追補 | Codex | 利用者が本版全文を承認 |
| 0.4 | 2026-10-17 | Runtime HandleRegistryの登録、pin、final release、reservation失敗、競合、shutdown drainを追補 | Codex | 利用者が本版全文を承認 |
| 0.5 | 2026-10-18 | C ABI `runtime_is_ui_thread`のthread別結果、invalid handle、output初期化、panic変換を追補 | Codex | 利用者が本版全文を承認 |
| 0.6 | 2026-10-18 | Runtime handle retainのregistry/C ABI動作、失敗時初期化、panic変換を追補 | Codex | 利用者が本版全文を承認 |
| 0.7 | 2026-10-18 | DD v1.1のUI-loop bridge登録とwake priorityをUT-ABI-007として具体化 | Codex | Runtime bridge追補はレビュー案。既存承認範囲と分離 |
| 0.8 | 2026-10-19 | DD v1.2のRuntime post受入、所有、dispatch turn実行・shutdown境界の試験を追補 | Codex | 利用者の「ok」によりfocused test oracleを承認。製品test合格は実行結果で別管理 |
| 0.9 | 2026-10-19 | C ABI Property create/kindの初期5 kind、入力検証、owner lifetime、output初期化をUT-ABI-009として追補 | Codex | 利用者がoracleと実装・動作確認を承認。製品test合格は実行結果で別管理 |
| 1.0 | 2026-10-19 | C ABI Property gettersのtyped value、kind違い、UI thread、UTF8 buffer照会とno-partial-writeをUT-ABI-010として追補 | Codex | 利用者がoracleと実装・動作確認を承認。製品test合格は実行結果で別管理 |
| 1.1 | 2026-10-19 | UT-ABI-010にnull required outputのoracleを明記 | Codex | 利用者承認済み範囲の明確化。test合格は実行結果で別管理 |
| 1.2 | 2026-10-19 | C ABI Property settersのtyped commit、equivalence、kind/input/thread failure atomicityをUT-ABI-011として追補 | Codex | 利用者承認済み範囲のoracle。通知とValidatorは別単位 |
| 1.3 | 2026-10-19 | Core Property change generation coalescingをUT-CORE-PROPERTY-NOTIFY-001として追補 | Codex | 利用者承認済みの内部queue oracle。end-to-end通知を意味しない |
| 1.4 | 2026-10-19 | Subscription active/in-flight/cancel lifecycle stateをUT-CORE-PROPERTY-SUBSCRIPTION-001として追補 | Codex | approved RQ/DDからのfocused internal oracle。callback executionとpublic ABIを含まない |
| 1.5 | 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-001のoracle承認と実装開始を記録 | Codex | 利用者の「ok」に基づく。試験合格は実行結果で別管理 |
| 1.6 | 2026-10-19 | Callback/userdata/destroy closureと自己cancel後の後始末をUT-CORE-PROPERTY-SUBSCRIPTION-002として追補 | Codex | 利用者の「ok」に基づくfocused実装範囲。Runtime integrationと公開ABIを含まない |
| 1.7 | 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-002の実行結果を追記 | Codex | focused/workspace testと開発ゲートの結果。要件全体の完了を意味しない |
| 1.8 | 2026-10-19 | Runtime active Subscription owner retention/pin/cancelをUT-CORE-PROPERTY-SUBSCRIPTION-003として追補 | Codex | 利用者の「ok」に基づくfocused Runtime内部registry。public handleとshutdown drainは含まない |
| 1.9 | 2026-10-19 | UT-CORE-PROPERTY-SUBSCRIPTION-003の実行結果を追記 | Codex | focused/workspace testと開発ゲートの結果。購読API全体の完了を意味しない |
| 2.0 | 2026-10-19 | Property weak Subscription indexとdelivery pin snapshotをUT-CORE-PROPERTY-SUBSCRIPTION-004として追補 | Codex | 利用者の「ok」に基づく。通知callback実行・safe pointを含まない |
| 2.1 | 2026-10-19 | UNIT-CORE-044のfocused/workspace testと開発ゲート実行結果を追記 | Codex | 実行結果のみ。通知購読全体の完了を意味しない |
| 2.2 | 2026-10-19 | RuntimeとProperty weak indexの接続、複数購読、登録順snapshot、個別cancelをUT-CORE-PROPERTY-SUBSCRIPTION-005として追補 | Codex | 利用者の継続指示によるfocused実装単位。callback deliveryと公開ABIを含まない |
| 2.3 | 2026-10-19 | UNIT-CORE-045のfocused/workspace testと開発ゲート実行結果を追記 | Codex | 実行結果のみ。通知購読全体の完了を意味しない |
| 2.4 | 2026-10-20 | Property delivery generationの順序・snapshot pin・失敗時復元をUT-CORE-PROPERTY-NOTIFY-002として追補 | Codex | 利用者の継続指示によるfocused実装単位。callback実行やdispatch schedulingを含まない |
| 2.5 | 2026-10-20 | UNIT-CORE-046のfocused/workspace testと開発ゲート実行結果を追記 | Codex | 実行結果のみ。notification delivery全体の完了を意味しない |
| 2.6 | 2026-10-20 | Property delivery itemの逐次callbackとcallback内変更/cancelをUT-CORE-PROPERTY-NOTIFY-003として追補 | Codex | 利用者の継続指示によるfocused実装単位。safe-point schedulingを含まない |
| 2.7 | 2026-10-20 | UNIT-CORE-047のfocused/workspace testと開発ゲート実行結果を追記 | Codex | 実行結果のみ。notification delivery全体の完了を意味しない |
| 2.8 | 2026-10-20 | 128 complete-item上限と未処理順保持をUT-CORE-PROPERTY-NOTIFY-004として追補 | Codex | 利用者の継続指示によるfocused実装単位。host safe-point schedulingを含まない |
| 2.9 | 2026-10-20 | UNIT-CORE-048のfocused/workspace testと開発ゲート実行結果を追記 | Codex | 実行結果のみ。notification delivery全体の完了を意味しない |
| 3.0 | 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-005のhost-boundary、yield wake、shutdown優先oracleを追補 | Codex | 利用者の「ok」に基づきUNIT-CORE-049のfocused実装・動作確認を開始。test合格は実行結果で別管理 |
| 3.1 | 2026-10-20 | UNIT-CORE-049のhost-boundary、wake再開、shutdown優先のfocused/workspace実行結果を追記 | Codex | 文書状態の記録のみ。通知機能全体の完了ではない |
| 3.2 | 2026-10-20 | UT-ABI-012のsetter outer/nested scopeおよび失敗時close oracleを追補 | Codex | 利用者の「ok」に基づきUNIT-ABI-012の実装・動作確認を開始 |
| 3.3 | 2026-10-20 | UNIT-ABI-012のfocused/workspace testと開発ゲート実行結果を追記 | Codex | 文書状態の記録のみ。通知機能全体の完了ではない |
| 3.4 | 2026-10-20 | UT-CORE-PROPERTY-NOTIFY-006のshutdown cleanup oracleを追補 | Codex | RQ/DD承認済み範囲のRuntime内部cleanup。public subscribe ABIとProperty-first teardownは含まない |
| 3.5 | 2026-10-20 | UNIT-CORE-050のfocused/workspace testと開発ゲート実行結果を追記 | Codex | 文書状態の記録のみ。Subscription全機能の完了を意味しない |
| 3.6 | 2026-10-21 | UT-CORE-PROPERTY-NOTIFY-007のProperty-first teardownとsafe-point cleanup oracleを追補 | Codex | 利用者の継続指示によるfocused実装単位。公開購読ABIとBinding teardownは含まない |
| 3.7 | 2026-10-21 | UNIT-CORE-051のProperty-first teardown focused/workspace testと開発ゲート結果を記録 | Codex | test oracleと当該実装revisionの検証状態。公開購読ABIとBinding teardownは別単位 |
| 3.8 | 2026-10-21 | UT-ABI-013の公開Property subscribe/cancel ABI oracleを追補 | Codex | 利用者の継続指示によるfocused実装単位。wrapperとBinding subscriptionは含まない |
| 3.9 | 2026-10-21 | UNIT-ABI-013のfocused/workspace testと開発ゲート結果を記録 | Codex | oracleと当該実装revisionの検証状態。wrapperとBinding subscriptionは別単位 |
| 4.0 | 2026-10-21 | UT-CORE-BINDING-GRAPH-001のOneWay topology oracleを追補 | Codex | 利用者の「ok」によるfocused実装単位。Binding propagation/ABIを含まない |
| 4.1 | 2026-10-21 | UNIT-CORE-052のfocused/workspace testと開発ゲート結果を記録 | Codex | oracleと当該実装revisionの検証状態。TwoWay mergeとProperty伝播は別単位 |
| 4.2 | 2026-10-21 | UT-CORE-BINDING-GRAPH-002のTwoWay merge topology oracleを追補 | Codex | 利用者の「ok」によるfocused実装単位。値同期と伝播は含まない |
| 4.3 | 2026-10-21 | UNIT-CORE-053のfocused/workspace testと開発ゲート結果を記録 | Codex | oracleと当該実装revisionの検証状態。Property値同期と伝播は別単位 |
| 4.4 | 2026-10-21 | UT-CORE-BINDING-INIT-COPY-001のtyped copy candidate oracleとUNIT-CORE-054実行結果を追補 | Codex | 利用者の「ok」に基づく実装単位。target validationおよびRuntime graph/queueとのatomic commitは別単位 |
| 4.5 | 2026-10-21 | UT-CORE-BINDING-INIT-PREPARE-001のgraph/value proposal準備oracleを追補 | Codex | 利用者の「ok」に基づくfocused実装単位。candidate commitとRuntime ownershipは別単位 |
| 4.6 | 2026-10-21 | UNIT-CORE-055のfocused/workspace testと開発ゲート結果を記録 | Codex | prepare段階の実装・検証状態。Runtime commit、Binding ownership、通知queueは別単位 |
| 4.7 | 2026-10-21 | UT-CORE-RUNTIME-BINDING-INIT-001のRuntime value/graph/queue atomicity oracleを追補 | Codex | 利用者の「ok」によるfocused Runtime実装範囲。public Binding lifecycleは含まない |
| 4.8 | 2026-10-21 | UNIT-CORE-056のfocused/workspace testと開発ゲート結果を記録 | Codex | 内部OneWay initial syncの実装・検証状態。後続propagationとpublic Binding lifecycleは別単位 |
| 4.9 | 2026-10-21 | UT-CORE-BINDING-PROPAGATION-ORDER-001の到達可能edge・上流順序oracleを追補 | Codex | 利用者の継続指示によるfocused実装単位。値伝播・Runtime integrationを含まない |
| 5.0 | 2026-10-21 | UNIT-CORE-057のfocused/workspace testと開発ゲート結果を記録 | Codex | graph propagation snapshotの実装・検証状態。Property値伝播・Runtime integrationは別単位 |
| 5.1 | 2026-10-21 | UT-CORE-RUNTIME-BINDING-PROP-001のordered candidate・非変更oracleを追補 | Codex | 利用者の継続指示によるfocused実装範囲。validator・commit・notificationを含まない |
| 5.2 | 2026-10-21 | UNIT-CORE-058のfocused/workspace testと開発ゲート結果を記録 | Codex | OneWay Runtime candidate preparationの実装・検証状態。validator・commit・delivery統合は別単位 |
| 5.3 | 2026-10-21 | UT-CORE-RUNTIME-BINDING-COMMIT-001のtarget commit・次generation通知oracleを追補 | Codex | 利用者の継続指示によるfocused実装範囲。subscriber deliveryとpublic Binding lifecycleは含まない |
| 5.4 | 2026-10-21 | UNIT-CORE-059のfocused/workspace testと開発ゲート結果を記録 | Codex | OneWay Runtime value/notification commitの実装・検証状態。validatorとsubscriber delivery統合は別単位 |
| 5.5 | 2026-10-21 | UT-CORE-RUNTIME-BINDING-DELIVERY-001の通知slice順序oracleを追補 | Codex | 利用者の継続指示によるRuntime integration範囲。validatorとpublic Binding lifecycleは含まない |
| 5.6 | 2026-10-21 | UNIT-CORE-060のfocused/workspace testと開発ゲート結果を記録 | Codex | OneWay propagation-before-subscriber Runtime integrationの検証状態。validatorとTwoWay/OneTimeは別単位 |
| 5.7 | 2026-10-21 | UT-CORE-RUNTIME-TWOWAY-INIT-001の初期同期・group merge atomicity oracleを追補 | Codex | 承認済みRQ/DDからのRuntime TwoWay initial-sync範囲。group伝播とpublic Binding lifecycleを含まない |
| 5.8 | 2026-10-21 | UNIT-CORE-061のfocused/workspace testと開発ゲート結果を記録 | Codex | TwoWay Runtime initial-sync/group commitの実装・検証状態。group伝播とpublic Binding lifecycleは別単位 |
