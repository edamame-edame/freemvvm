# 適用前記録 / 07-property-notification.md

- 記録日: 2026-10-17
- 区分: 2026-10-17改訂前の履歴。現行仕様として使用しない
- 元文書: [07-property-notification.md](../../13_detail_design/07-property-notification.md)

---

# DD-07 Propertyの保存と通知

- 状態: DD-PROP-01～10の基礎判断は承認済み（2026-10-08）。DD-PROP-08のX01 Runtime owner・cleanup改訂 v0.3はレビュー待ち（2026-10-13、DD-LIFE-03/04承認後の整合案）
- 作成日: 2026-10-07
- 対象: Core管理Propertyの初期公開値型、生成・読み書き、等価判定、変更通知、購読
- 上位判断: ADR-0001、RQ-05-002/006、A5/A6/A8/A9/A10/A11/A12/A17/A18、P1～P4、V1～V8、BD-01/02/03
- 先行詳細設計: DD-01～06（承認済み）
- 後続: DD-08 Binding、DD-09 Validation、DD-10 Async Command、DD-18～21 wrappers。要求testのcase/evidence契約は [Requirements Test Manifest](../../30_release_test/requirements-test-manifest.md)
- 作業計画: [DD-07 作業計画](../dd07-property-notification.md)

本書のDD-PROP-01～10は2026-10-08に承認済みである。承認済みの上位意味論を変更しない。C宣言は署名と境界の設計資料であり、配布用headerまたは実装済みAPIではない。未決境界は後続設計の担当として記録する。

## 1. 範囲と責務

| 領域 | DD-07の責務 | 別設計の責務 |
|---|---|---|
| 値型 | 五種類のkind ID、Property生成と型別read/write表現 | カスタム値・一般null許容・OBJECT・Collectionの上位判断（BD-01） |
| setter/getter | 型一致、等価判定、即時確定、失敗時不変、出力 | Property Validatorの全登録・差替え・検証意味論（DD-09） |
| notification | 処理区間、coalescing、順序、購読callback、保留 | Binding propagationとgraph/cycle診断（DD-08） |
| lifetime | Subscription取消し、Property先行破棄、shutdownとの境界 | Core object/Control treeの所有関係（DD-11以降） |
| wrapper | 4言語で同じobservable behaviorを得る契約 | 具体的な言語型、例外・Result変換、closure管理（DD-18～21） |
| 値変換 | 今回は型別Propertyへの明示的な値渡し | 将来のライブラリ内蔵変換機能とparse/format規則 |

## 2. 承認済み上位契約

- P1: Property生成、read/write、subscribe/cancelはUI thread専用。workerは値を`runtime_post`し、UI thread上でsetterを呼ぶ。
- P2: setterは検証に成功すると値を同期確定する。同値なら成功するが変更通知を作らない。
- P3: 通知は最外側のUI callback/API処理が戻った後に配信する。一区間に同じPropertyを複数回変更した場合は一件へまとめ、最初にqueueされた順でPropertyを通知する。通知callbackが起こした変更は次の配信回へ回す。同期flush APIは設けない。
- P4: subscribe時に初期通知を送らない。解除後に新規callbackを開始せず、shutdown時の保留通知を配信しない。
- BD-02: 明示UI transactionと自動rollbackは設けない。先に成功した別Propertyのsetterを後続失敗で戻さない。
- BD-03: 有限の作業枠を設け、通知を捨てずに完了した配信単位の境界でUI loopへ譲る。上限到達だけでnon-convergenceと断定しない。
- BD-01: 初期公開Property kindはBOOL/I64/UTF8/F64/DECIMAL。F64とDECIMALの等価性・値域を維持し、カスタム値・一般null・OBJECTを初期公開kindへ加えない。
- DD-01～06: handle、出力初期化、failure atomicity、callback解除・destroy、共通status/domain、shutdown優先dispatchは先行設計に従う。

## 3. 承認済み判断

| ID | 判断 | 出典・範囲 | 状態 |
|---|---|---|---|
| DD-PROP-01 | Property kind IDをBOOL=1、I64=2、UTF8=3、OBJECT=4（予約・生成不可）、F64=5、DECIMAL=6とする。追加kindは新IDを割当て、既存IDを再利用しない。 | A5、BD-01、DD-ABI-04/05。 | 承認済み |
| DD-PROP-02 | 既存A9の初期値付き型別create、A5の`property_kind`、型別getter/setterを維持し、F64/DECIMAL用の互換追加symbolを設ける。生成成功は所有Property handleを返し、失敗時は0。生成自体で通知を発生させない。 | A5/A9、P1/P2、DD-VAL-01/04/06/07。 | 承認済み |
| DD-PROP-03 | F64はC ABIの`double`、DECIMALは`uint64_t`のcanonical bit layoutで受け渡す。DECIMAL bit 63はsign、bits 58–62はunsigned scale、bits 0–57はunsigned coefficientとする。F64/DECIMAL Validator callbackも`double`/`uint64_t`を受け取る型別signatureとする。 | A5/A17/A18、BD-01、DD-VAL-01。登録と評価規則はDD-09。 | 承認済み |
| DD-PROP-04 | 値比較はBOOL/I64の値、UTF8のbyte列、F64のBD-01比較、DECIMALのBD-01数値比較を使う。同値setterはvalidatorがある場合はV1に従い検証するが、既存値と格納表現を置き換えず、Property notificationを生成しない。 | P2、BD-01、V1、A6/A8、DD-VAL-02/07。 | 承認済み |
| DD-PROP-05 | setterは入力検証・UTF8 copy・必要な値確保がすべて成功した後に値をcommitする。型不一致、wrong thread、invalid UTF8、validation rejection、allocation failureではProperty値・notification queueを変えない。required outputはDD-VAL-06どおり初期化し、`ui_status`を一次結果、optional `out_error`を失敗詳細とする。 | A8、P1/P2、V1～V4、DD-VAL-02/04/05/06/07、DD-ABI-05～07。 | 承認済み |
| DD-PROP-06 | getterは現在の確定値を返す。UTF8はDD-VAL-03のcaller buffer規則を使う。F64/DECIMAL getterはstored value representationを返す。同一値set後も既存representationを保持する。 | A6/A8、P2、BD-01、DD-VAL-03。 | 承認済み |
| DD-PROP-07 | notificationは最外側のUI処理終了後、通常UI event/postより先に処理を始める。Property単位のdelivery itemは、その時点のsubscriber順序snapshotへ登録順でcallbackする。処理中の変更は次のgenerationへenqueueし、同じPropertyをgeneration内で一件へcoalesceする。 | P3/P4、BD-02、DD-CB-01～09。 | 承認済み |
| DD-PROP-08 | subscription登録はUI threadで開始・cancelする。dispatch itemは開始時にactive状態を確認し、cancelではactiveを先に落として以後のcallback開始を防ぐ。実行中callbackが戻った後、UI thread safe pointでclosureを一度だけdestroyする。Property teardownとshutdownはpending deliveryを止めて同じcleanup経路へ送る。CoreのRuntime registration registryはactive subscriptionをstrong ownerとして保持し、Property indexはweakとする。handle releaseの外部効果はRQ-05-002/006に従う。 | RQ-05-002/006～008、P4、A10～A12、承認済みDD-LIFE-03/04/06、DD-CB-02～07。 | 基礎判断承認済み（2026-10-08）。X01 owner/cleanup改訂 v0.3はレビュー待ち |
| DD-PROP-09 | notification sliceは完了したProperty delivery itemを最大128件処理する。itemは一Propertyのsubscriber snapshot全員のcallback完了までを指し、callback途中で中断しない。128件後は通常UI event/postへyieldし、未配信itemを順序を保って保持する。shutdown control taskはこの残りより優先する。 | P3、BD-03、DD-DSP-03/04/08、DD-ABI-08/09。 | 承認済み |
| DD-PROP-10 | notification itemに内部因果連鎖識別を持たせる境界を設ける。Property層は同一Propertyのcoalescingとqueue順を保証する。Binding起点の連鎖停止・non-convergence判定・diagnostic recoveryの因果範囲はDD-08で定め、work budget到達だけを停止根拠にしない。 | BD-03、DD-03、DD-08への責務分割。 | 承認済み |

## 4. 初期公開Property kindとABI表現

| kind ID | kind | create/read/write ABI値 | 等価判定 |
|---:|---|---|---|
| 1 | BOOL | `ui_bool`（0/1） | 0/1の一致。その他の値は拒否 |
| 2 | I64 | `int64_t` | 符号付き64-bit値の一致 |
| 3 | UTF8 | `(const uint8_t *, uint64_t byte_length)`、getterは`uint8_t *` caller buffer | 有効UTF-8のbyte列一致。NULを含め自動正規化しない |
| 4 | OBJECT | 予約値。Property生成不可 | 対象外 |
| 5 | F64 | C ABI `double`（IEEE 754 binary64） | `+0`/`-0`は等価、すべてのNaNは等価、無限大は同符号のみ等価 |
| 6 | DECIMAL | `uint64_t`のcanonical encoding | 数値比較。末尾ゼロ・zeroのsign/scaleを同一性に含めない |

DECIMAL encoding: `bit63=sign`（0 positive, 1 negative）、`bits58..62=scale (0..31)`、`bits0..57=coefficient (0..2^58-1)`。値は`(-1)^sign × coefficient / 10^scale`。全64-bit patternがBD-01の範囲内である。例として`1.23`は`(2 << 58) | 123`。公開ABIは数値`uint64_t`であり、構造体packingやhost byte order依存のbyte arrayではない。wire/file serializationは本判断の対象外。

同値再設定では、BD-01が明示するF64 stored bit representationを保つ。DECIMALでは同値の再設定で現stored representationを維持し、`1.00`設定で格納済み`1.0`をscale 2へ書き換えない。表示桁数・入力原文はPropertyの数値状態と分け、View/Input設計で扱う。

## 5. 設計用C宣言

以下のC宣言は承認済み設計のsignatureを示すが、配布headerまたは実装済みAPIではない。ABI symbol major prefixはDD-ABI-01に従う。

```c
enum {
  UI_VALUE_BOOL = 1,
  UI_VALUE_I64 = 2,
  UI_VALUE_UTF8 = 3,
  UI_VALUE_OBJECT = 4, /* reserved; create is not exposed */
  UI_VALUE_F64 = 5,
  UI_VALUE_DECIMAL = 6
};

ui_status ui_abi_1_property_kind(ui_handle property, uint32_t *out_kind);

ui_status ui_abi_1_property_create_bool(ui_handle runtime, ui_bool initial,
                                         ui_handle *out_property,
                                         ui_handle *out_error);
ui_status ui_abi_1_property_create_i64(ui_handle runtime, int64_t initial,
                                        ui_handle *out_property,
                                        ui_handle *out_error);
ui_status ui_abi_1_property_create_utf8(ui_handle runtime,
                                         const uint8_t *bytes,
                                         uint64_t byte_length,
                                         ui_handle *out_property,
                                         ui_handle *out_error);
ui_status ui_abi_1_property_create_f64(ui_handle runtime, double initial,
                                        ui_handle *out_property,
                                        ui_handle *out_error);
ui_status ui_abi_1_property_create_decimal(ui_handle runtime,
                                            uint64_t initial_bits,
                                            ui_handle *out_property,
                                            ui_handle *out_error);

ui_status ui_abi_1_property_get_bool(ui_handle property, ui_bool *out_value);
ui_status ui_abi_1_property_set_bool(ui_handle property, ui_bool value,
                                      ui_handle *out_error);
ui_status ui_abi_1_property_get_i64(ui_handle property, int64_t *out_value);
ui_status ui_abi_1_property_set_i64(ui_handle property, int64_t value,
                                     ui_handle *out_error);
ui_status ui_abi_1_property_get_utf8(ui_handle property, uint8_t *buffer,
                                      uint64_t capacity,
                                      uint64_t *out_byte_length);
ui_status ui_abi_1_property_set_utf8(ui_handle property,
                                      const uint8_t *bytes,
                                      uint64_t byte_length,
                                      ui_handle *out_error);
ui_status ui_abi_1_property_get_f64(ui_handle property, double *out_value);
ui_status ui_abi_1_property_set_f64(ui_handle property, double value,
                                     ui_handle *out_error);
ui_status ui_abi_1_property_get_decimal(ui_handle property,
                                         uint64_t *out_bits);
ui_status ui_abi_1_property_set_decimal(ui_handle property, uint64_t bits,
                                          ui_handle *out_error);

ui_status ui_abi_1_property_subscribe(ui_handle property,
                                       ui_callback callback,
                                       void *userdata,
                                       ui_destroy destroy,
                                       ui_handle *out_subscription,
                                       ui_handle *out_error);
ui_status ui_abi_1_subscription_cancel(ui_handle subscription,
                                        ui_handle *out_error);

typedef ui_status (*ui_validate_f64)(void *userdata, double candidate,
                                     ui_handle *out_owned_error);
typedef ui_status (*ui_validate_decimal)(void *userdata,
                                         uint64_t candidate_bits,
                                         ui_handle *out_owned_error);
ui_status ui_abi_1_property_validator_register_f64(
    ui_handle property, ui_validate_f64 callback, void *userdata,
    ui_destroy destroy, ui_handle *out_registration, ui_handle *out_error);
ui_status ui_abi_1_property_validator_register_decimal(
    ui_handle property, ui_validate_decimal callback, void *userdata,
    ui_destroy destroy, ui_handle *out_registration, ui_handle *out_error);
ui_status ui_abi_1_property_validator_cancel(ui_handle registration,
                                               ui_handle *out_error);
```

F64/DECIMAL validator callbackの型別signatureはDD-PROP-03で承認済みである。登録・評価・解除の意味論と完全な関数面はDD-09で扱う。

## 6. setter/getterと失敗時

- 全Property APIは所属RuntimeのUI thread専用。worker setterは`UI_WRONG_THREAD`で値もnotificationも変更せず、workerは`runtime_post`を利用する。
- getterのrequired outputは呼出し前に既定値へ初期化する。UTF8 getterはDD-VAL-03どおり、`buffer == NULL && capacity == 0`で必要byte長を返し、容量不足時はbufferを変更しない。
- 型が異なるgetter/setterは`UI_WRONG_TYPE`、無効handle・wrong Runtime・closed RuntimeはDD-01/02/06のstatusを使う。boolが0/1以外なら`UI_INVALID_ARGUMENT`。UTF8は不正sequenceを拒否する。
- F64はBD-01に従いNaNと±infinityを受理する。アプリ固有finite制約はValidatorの責務。DECIMALの全`uint64_t` patternはvalid encodingであり、範囲外は存在しない。
- setterはcandidateを検査し、必要なUTF8 copy等を完了し、Validatorを通過してからcommitする。失敗時にProperty値、notification queue、subscription状態を部分変更しない。既に同一処理区間で成功した別setterは巻き戻さない。
- successful changed setterは値を同期確定するが、notification callback完了を待たない。notificationを生成しない同値setterも`UI_OK`を返す。Validatorの同値candidate検証はV1に従う。

## 7. Notification queueと購読

1. notification処理区間はUI callback、`runtime_post` callback、または最外側UI API callの開始から復帰までとする。入れ子のAPI処理が戻っても、その最外側処理が戻るまでは配信を始めない。
2. setter成功で値が変わったPropertyを、その処理区間のpending generationへ初回変更順に一度だけ追加する。同じPropertyの再変更は同じgeneration内でcoalesceする。古い値・中間値の履歴は持たず、callbackは配信時点でgetterから現在値を得る。
3. generationの各delivery item開始時にそのPropertyの購読者集合をsnapshotする。購読者は登録順で実行する。配信途中のcancelは当該subscriberの以後の新規開始を抑止し、追加subscribeは現在のsnapshotに追加せず次のgeneration以降に対象とする。
4. delivery itemは、一Propertyのsnapshot購読者全員のcallbackを順に完了した単位とする。callbackがsetterを呼んで起こした変更はcurrent generationへ再帰せず、次generationに追加する。
5. queueが空ならdispatch sliceを終える。通常は最外側処理の復帰後、次の通常UI event/postより前に通知処理を行う。保留notificationと同時にshutdown control taskがある場合はcontrol taskを先に実行する。
6. 一つのsubscriber callbackの言語例外はwrapper境界で捕捉し、diagnostic経路へ記録して次のsubscriberを継続する。変更済みPropertyをrollbackせず、他subscriberの配信も取り消さない（P4/DD-CB-07）。

## 8. 有限作業枠と保留

DD-PROP-09で承認された初版規則では、連続notification sliceあたり最大128個のcomplete delivery itemを実行する。128件目のsubscriber snapshotが完了したら、その処理区間を中断せずUI loopへyieldする。残りitemとgeneration順は維持し、未配信の変更を捨てない。保留通知の次のsliceは通常UI event/postより後に実行されてもよい。shutdown優先control taskはpending notificationより優先する。

一つのdelivery itemはcallbackが戻るまで割れない。単一の通知callback・Validator自体が長時間戻らない場合のpreempt/timeoutは提供しない。128という上限を変更する場合は、後続の設計判断で互換性と公平性を確認する。長い有限処理はUI loopを譲りつつ後続sliceで完了させる。

通知のwork budget到達は循環診断や停止理由ではない。non-convergent Binding chainの確定、影響範囲の切り分け、診断取得・再開・破棄APIはDD-08のBinding graph設計で決める。Property queueは保留とgeneration順をDD-08へ渡せる内部因果識別を持つが、その具体runtime representationは本書でABI化しない。

## 9. 終了と所有権

- subscribe時は初期通知を生成しない。明示cancel後は新しいcallbackを開始しない。cancel対象callbackが実行中の場合、callback終了後にclosure destroyをUI thread上で一度だけ行う。
- ABIの`subscription_cancel`と`handle_release`は別操作としてCoreへ渡す。handle release後もRuntime registration registryのstrong ownerによりactive subscriptionは存続する（外部契約: [RQ-05-002](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)）。
- owner Property teardownではweak subscription indexを走査し、activeを落としてpending deliveryを除く。in-flight callbackの復帰後にregistry ownerとclosureを解放する。残った公開Subscription tokenはInactive shellとしてreleaseできる。
- Runtime shutdownはactive registration registryをUI threadでdrainし、未開始notificationを実行せず、各subscriptionを無効化する。in-flight callback終了後にclosureを一度だけdestroyし、最後にregistry ownerを外す。
- destroy callbackはUI操作を呼ばず、Runtime shutdownの既承認破棄順に従う。

## 10. 単体確認ケース

1. 4言語でBOOL/I64/UTF8/F64/DECIMALを初期値付きで生成し、kind queryと型別getter/setterが同じ値意味論になる。
2. `property_kind`でOBJECT予約値を観測しても、OBJECT Property生成は提供されない。未知kindはwrapperが未対応として保持する。
3. BOOL無効値、UTF8不正sequence/null pointer不整合、wrong kind/thread/Runtime/handle、closed Runtimeを拒否し、値と通知queueが変わらない。
4. F64 `+0`/`-0`、複数NaN、同符号infinityの再設定は等価扱いし、異なる値は通知を発生させる。等価なNaN再設定でも元bit patternが保持される。
5. DECIMALの`1.0`/`1.00`、異なるscale/signのzeroを等価扱いし、係数・scaleの各bit境界と最大値を確認する。UTF8からの暗黙変換や値域外丸めは行わない。
6. UTF8容量照会・容量不足・再試行・埋め込みNULを確認する。getterの失敗出力はDD-VAL-03/06に従う。
7. 一処理区間で同一Propertyを複数回設定すると通知は一件、初回変更順と登録順でcallbackを呼び、callbackからの変更は次generationになる。
8. callback中の自己cancel、他subscription cancel、新規subscribe、callback例外、Property先行破棄、shutdownでcallback開始・destroy回数・実行threadを確認する。
9. 128件を超える有限notificationではdelivery item途中で割らず、完了item順を保持してyield/resumeし、通常UI workへ進める。budget到達だけで変更を破棄・cycle判定しない。
10. workerから直接操作すると`UI_WRONG_THREAD`、`runtime_post`内での操作は成功する。shutdown後は保留通知を開始せず、先行設計に従ってclosureを破棄する。

## 11. 未決境界と承認記録

- F64/DECIMAL Validator登録・評価・解除の意味論（DD-09）。
- 利用者意向（2026-10-08）: 将来、文字列↔整数・DECIMAL等の基礎的な相互変換をライブラリ内蔵機能として提供する。変換API、文字列文法、culture/locale、丸め・範囲外・不正入力の結果は後続設計で決める。
- Bindingを含む因果識別の伝達、non-convergence検出・diagnostic/recovery API（DD-08）。
- 4 wrapper固有表現・例外変換はDD-18～21で扱う。NaN payloadやtargetごとのbinary64 verificationは要求test case・実test計画で具体化する。

利用者は2026-10-08にDD-07を承認し、DD-PROP-01～10を内容承認済みとして記録した。未決境界とDD-08以降のBinding/Validation/Command意味論、wrapper固有API、C宣言の配布header化は各フェーズで確認する。 X01レビュー案は2026-10-12に承認され、DD-PROP-08の内部owner・cleanup改訂案を作成した。

## X01改訂の承認履歴

| 日付 | 対象 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-10-12 | DD-PROP-08 Runtime owner補足案 | 利用者 | X01改訂方針を承認。具体的なDD改訂本文はレビュー待ち |

## 改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 0.1 | 2026-10-08 | DD-PROP-01～10を作成・承認 | Codex | 2026-10-08承認記録 |
| 0.2 | 2026-10-12 | Subscription lifetimeの内部Runtime owner・Property weak index・cleanup flowを補足 | Codex | X01改訂案。利用者レビュー待ち |
| 0.3 | 2026-10-13 | DD-LIFE-03/04承認を反映し、subscription cleanup順とowner参照を照合 | Codex | DD-PROP-08 X01改訂案。利用者レビュー待ち |
| 0.4 | 2026-10-14 | DD-23再分類に伴い、検証担当への参照をRequirements Test Manifestとtest計画へ変更。Property設計判断は変更しない | Codex | 文書ロール整合の編集。DD-PROP判断の追加承認なし |
