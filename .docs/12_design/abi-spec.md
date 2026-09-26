# MVVM UIライブラリ C ABI仕様案（Slice 1基盤・Slice 2 MVVM）

- 状態: A1～A32承認済み（2026-09-26）。Controlの表示・論理ツリーとIME詳細は後続設計
- 作成日: 2026-09-26
- 対象: C++ / C# / Rust / Python 3共通のネイティブ境界
- 上位決定: `adr-0001-rust-core.md`
- 意味論: `runtime-contract.md`

## 1. 目的と適用範囲

Rust CoreをC++、C#、Rust、Python 3から呼ぶ共通境界の初期形を規定する。A1～A32の設計判断は承認済みである。第7節にProperty値の受け渡し、第9節にProperty生成・通知購読、第11節にBinding ABI、第13節にProperty Validator ABI、第15節に非同期Command ABI、第17節にView入力Validator ABI、第20節にTextBox生成・値接続を示す。ここに示すC宣言は実装前の仕様であり、配布用ヘッダではない。Controlの描画と論理ツリーは別仕様とする。

## 2. 型とシンボル

- Cの呼出規約で公開する。公開構造体にRust/C++の実装型を含めない。
- 外部公開関数名には`ui_abi_1_`接頭辞を付け、major 1のシンボルとして固定する。
- `ui_handle`は`uint64_t`の不透明トークン。0は無効。handleをpointerへ変換しない。
- `ui_status`は`uint32_t`、`ui_bool`は`uint8_t`で0/1、長さと容量は`uint64_t`。ABIをまたぐ`size_t`やC `bool`は使わない。
- Cで公開するpointerはbyte列、caller buffer、callback、userdata、out引数に限る。pointerの幅と呼出規約は各targetのC ABIに従う。
- callbackとdestroyの関数ポインタには同じC calling conventionを使用する。Windowsを含むtarget別の指定をヘッダとwrapper生成設定で一致させる。

```c
/* 設計レビュー用の抜粋。配置・export macro・ヘッダガードは実装時に追加する。 */
#include <stdint.h>

typedef uint64_t ui_handle;
typedef uint32_t ui_status;
typedef uint8_t ui_bool;

enum {
  UI_OK = 0,
  UI_INVALID_ARGUMENT = 1,
  UI_INVALID_HANDLE = 2,
  UI_WRONG_TYPE = 3,
  UI_WRONG_RUNTIME = 4,
  UI_WRONG_THREAD = 5,
  UI_CLOSED = 6,
  UI_BUFFER_TOO_SMALL = 7,
  UI_OUT_OF_MEMORY = 8,
  UI_INTERNAL_ERROR = 9,
  UI_UNSUPPORTED = 10,
  UI_VALIDATION_FAILED = 11, /* Property setter用の追加候補 */
  UI_COMMAND_EXECUTION_FAILED = 12, /* Command失敗snapshot用の追加候補 */
  UI_INPUT_FEEDBACK = 13 /* View入力理由のsnapshot用の追加候補 */
};

/* callbackのreturnはABI外へ例外を投げない。 */
typedef void (*ui_callback)(void *userdata);
typedef void (*ui_destroy)(void *userdata);

ui_status ui_abi_1_get_version(uint32_t *out_major, uint32_t *out_minor);
ui_status ui_abi_1_has_feature(uint32_t feature_id, ui_bool *out_supported);

ui_status ui_abi_1_runtime_create(ui_handle *out_runtime, ui_handle *out_error);
ui_status ui_abi_1_runtime_shutdown(ui_handle runtime, ui_handle *out_error);
ui_status ui_abi_1_runtime_is_ui_thread(ui_handle runtime, ui_bool *out_yes);
ui_status ui_abi_1_runtime_post(ui_handle runtime, ui_callback callback,
                                 void *userdata, ui_destroy destroy,
                                 ui_handle *out_error);

ui_status ui_abi_1_handle_retain(ui_handle borrowed, ui_handle *out_owned);
ui_status ui_abi_1_handle_release(ui_handle owned);

ui_status ui_abi_1_error_status(ui_handle error, ui_status *out_status);
ui_status ui_abi_1_error_code(ui_handle error, uint32_t *out_domain,
                               uint32_t *out_code);
ui_status ui_abi_1_error_message_utf8(ui_handle error, uint8_t *buffer,
                                       uint64_t capacity, uint64_t *out_length);
```

### 2.1 出力とerror

- `out_error`は任意。nullなら詳細は捨てる。非nullなら各呼出しの開始時に0へ設定し、失敗して詳細を作れた場合に所有handleを返す。成功時は0のまま。
- その他の必須out pointerがnullなら`UI_INVALID_ARGUMENT`。失敗時にout handleは0、out scalarは0。`UI_BUFFER_TOO_SMALL`では`out_length`だけ必要byte数を返す。
- `ui_abi_1_get_version`はロード直後に利用でき、major/minorを返す。`has_feature`は未知IDをfalseとして返す。
- error handleも共通のretain/releaseで管理する。`error_message_utf8`はNULを追加せず、必要byte数を`out_length`に返す。`buffer == NULL && capacity == 0`の容量照会を許す（`UI_OK`、書き込みなし）。

### 2.2 所有権を伴う関数

| 関数 | 入力handle | 成功時の出力 | 失敗時 |
|---|---|---|---|
| `runtime_create` | なし | Runtime所有参照1個 | `out_runtime = 0` |
| `runtime_shutdown` | 借用 | なし | 詳細があればerror所有参照 |
| `runtime_post` | Runtime借用 | closureの所有権をCoreに移す | closureの所有権は呼び出し側 |
| `handle_retain` | 借用 | 同一objectへの新しいトークンの所有参照1個 | `out_owned = 0` |
| `handle_release` | 所有参照を1個消費 | なし | 無効handleの場合は消費しない |
| `error_message_utf8` | Error借用 | caller bufferにコピー | buffer不足なら必要長を返す |

`post`のcallbackはUI threadで一回実行されるか、shutdown時に未実行のまま破棄される。成功した`post`に対応するdestroyはUI thread上で必ず一回呼ぶ。`callback == NULL`または`destroy == NULL`は無効。`userdata == NULL`は許す。

## 3. ABI versionとfeature query（提案）

- 初版はmajor 1、minor 0とする。ここで示すmajor/minorは正式リリース番号ではなく設計上の初期値。
- major 1の既存シンボルについて、関数シグネチャ、呼出規約、列挙済み定数の意味、所有権、スレッド規則、失敗時の基本契約を変更しない。
- 新機能は新しい名前のシンボルまたはfeature IDで追加し、minorを上げる。既存クライアントが動作する互換変更に限る。
- 契約を破る変更は`ui_abi_2_...`の別シンボル集合として導入し、major 1の利用者とは区別する。major 1をいつまで同梱するかは配布方針で決める。
- wrapperは初期化時にversionと必要featureを検査する。未知featureはfalseとし、wrapperが明示的な未対応エラーへ変換する。
- enum/statusの未知値をwrapperは汎用エラーとして扱い、未定義動作にしない。

## 4. 境界で防ぐもの

- Rust panicがC境界を越えることを防ぐ。公開関数の失敗はstatusへ変換し、panic時は原則`UI_INTERNAL_ERROR`とする。panic後にRuntime状態が健全でない場合は停止扱いとし、継続可否を実装時に検証する。
- C++ exception、C# exception、Python exceptionはwrapper内で止める。C# delegate、Python callableをCoreに直接渡さず、wrapperがuserdataの生存期間を保持する。
- wrapperのfinalizer任せでUI thread上の確定的解除を保証しない。Subscription解除・Runtime shutdownを明示APIとし、finalizerは最後の参照の掃除に限定する。
- コピーが必要なUTF-8入力をCoreが保持する際には、呼出しが戻る前にコピーする。

## 5. ABI検証の最小集合

- CヘッダをCコンパイラでコンパイルし、4言語wrapperからversion、feature、create/retain/release/shutdownを呼ぶ。
- 同じobjectを指す異なる二つの所有トークンを独立にreleaseでき、片方の二重releaseが他方へ影響しないことを確認する。
- 解放済み、種類違い、別Runtimeのhandleを拒否できることを確認する。
- UTF-8メッセージの容量照会とbuffer不足、callbackの登録成功/失敗時の所有権を確認する。
- バイナリ互換性の検証は同じmajorの旧ヘッダでビルドしたwrapperを新しいnative artifactに接続して行う。

## 6. 承認された設計判断（2026-09-26）

| ID | 承認済みの判断 | 判断への影響 |
|---|---|---|
| A1 | `ui_handle`を64-bitトークンとし、共通retain/releaseを使う | 型別ポインタAPIやhandle表の実装方式を左右する |
| A2 | errorは返り値statusに加え、任意のout_error handleで返す | wrapperの例外・Result変換方式を左右する |
| A3 | majorを関数シンボル名へ含め、互換追加をminorで扱う | 長期のバイナリ互換維持方式を左右する |
| A4 | shutdownはUI threadで完了させ、未実行postのdestroyまで保証する | イベントループと終了順序を左右する |

## 7. Property値のC ABI（承認済み）

本節は`mvvm-semantics.md`のP1～P4、V1～V4と`language-bindings.md`のL1～L4に従う。Property handleの生成・定義とBindingの作成APIは対象外とし、既に取得したProperty handleの値を読み書きする境界だけを定める。

### 7.1 型とAPIの形

- Propertyの値の種類は作成時に固定する。初期に公開する値型は`BOOL`、`I64`、`UTF8`とし、`OBJECT`は所有の循環防止規則が確定するまで予約型として扱う。getter/setterを種類ごとの別シンボルにし、公開C構造体のtagged unionは使わない。`property_kind`で種類を問い合わせ、未知の種類はwrapperが未対応として扱う。
- `BOOL`は`ui_bool`（0または1のみ受理）、`I64`は`int64_t`、`UTF8`はbyte列と`uint64_t`長、`OBJECT`は`ui_handle`で表す。C++/C#/Rust/Pythonの数値型から`I64`へ変換するwrapperは、範囲外の値を切り詰めず拒否する。
- `F64`のNaN・符号付きゼロの等価性、その他の整数幅、nullableな値型、配列・カスタム値は別の値型設計で定める。今回の4種類へ暗黙変換しない。公開後に種類を増やす場合は新しいkindとシンボルを互換追加する。

```c
/* 追加シンボルのレビュー用抜粋。ui_handle / ui_status / ui_boolは第2節。 */
enum {
  UI_VALUE_BOOL = 1, UI_VALUE_I64 = 2,
  UI_VALUE_UTF8 = 3, UI_VALUE_OBJECT = 4 /* 予約値 */
};

ui_status ui_abi_1_property_kind(ui_handle property, uint32_t *out_kind);
ui_status ui_abi_1_property_get_bool(ui_handle property, ui_bool *out_value);
ui_status ui_abi_1_property_set_bool(ui_handle property, ui_bool value,
                                      ui_handle *out_error);
ui_status ui_abi_1_property_get_i64(ui_handle property, int64_t *out_value);
ui_status ui_abi_1_property_set_i64(ui_handle property, int64_t value,
                                     ui_handle *out_error);
ui_status ui_abi_1_property_get_utf8(ui_handle property, uint8_t *buffer,
                                      uint64_t capacity, uint64_t *out_length);
ui_status ui_abi_1_property_set_utf8(ui_handle property, const uint8_t *bytes,
                                      uint64_t byte_length, ui_handle *out_error);
ui_status ui_abi_1_property_get_object(ui_handle property, ui_handle *out_owned);
ui_status ui_abi_1_property_set_object(ui_handle property, ui_handle borrowed,
                                        ui_handle *out_error);
```

ここでの関数名・kindの数値・`UI_VALIDATION_FAILED`の値はレビュー案である。OBJECTの関数形は将来公開する際の設計案で、初期実装の公開シンボルには含めない。getterには利用者へ返す値を、setterには検証失敗時の詳細を渡す。Propertyの作成APIやどのControlがどのkindを使うかは別途決める。

### 7.2 UTF-8とbuffer

- setterの入力は呼出し中だけ借用し、成功して値を保持する場合はCoreが関数から戻るまでにコピーする。`bytes == NULL && byte_length == 0`は空文字列を表す。長さが正でpointerがnullなら`UI_INVALID_ARGUMENT`。不正UTF-8は拒否し、値と通知を変えない。埋め込みNULはProperty値として許す。特定Controlが受け入れない文字はそのControlの検証で扱う。
- getterは第2節のerror文字列と同じcaller buffer方式。`buffer == NULL && capacity == 0`は必要byte数を`out_length`へ返す容量照会として成功する。容量不足なら`UI_BUFFER_TOO_SMALL`で現在の必要長を返し、bufferを変更しない。十分ならbyte列をコピーして実際の長さを返し、終端NULは付けない。
- 容量照会と次のコピーは別呼出しなので、途中で値が変われば必要長も変わり得る。wrapperは`UI_BUFFER_TOO_SMALL`を受けたら長さを取り直して再試行する。二呼出しの間にイベントループを回さないことを推奨するが、同一値のsnapshotを保証するAPIは別途検討する。

### 7.3 object handleと所有権

- `set_object`の入力は呼出し中だけ借用する。同じRuntimeに属する有効なCore objectだけを受け付ける。公開トークン番号の一致ではなくCore objectの同一性で変更判定する。値を保持する場合の内部参照はCoreが取得し、呼出し側の所有トークンは消費しない。
- `get_object`は非null値に対して新しい所有トークンを返し、呼出し側が`release`する。nullableとして宣言されたOBJECT Propertyに限り、入力のhandle 0と成功時の出力handle 0をnull値とする。非nullableなPropertyへの0は`UI_INVALID_ARGUMENT`。null値は無効なトークンとは区別して扱う。
- objectを強参照でPropertyに保持すると内部参照の循環が生じ得る。Runtime契約の「循環する内部参照を作らない」に従い、循環を防ぐ所有規則と対象object種別が確定するまではOBJECT Propertyの作成を公開しない。上の関数形とトークンの受渡し規則は将来の有効化に備えた提案である。

### 7.4 エラーとスレッド

- これらの関数はUI thread専用。workerから呼ぶと`UI_WRONG_THREAD`で値・通知は変わらない。workerは値をコピーして`runtime_post`へ渡し、UI threadでsetterを呼ぶ。
- 種類の違うgetter/setterには`UI_WRONG_TYPE`、不正な文字列や`ui_bool`値には`UI_INVALID_ARGUMENT`、検証規則に拒否された値には`UI_VALIDATION_FAILED`を返す。これらの失敗でProperty値と値の通知は変えない。検証失敗の`out_error`は既存のerror handle規則に従う。UI停止、所属Runtime不一致、無効handleも第2節のstatusを使う。
- getterの必須出力は失敗時に0へ初期化する。OBJECT getterでは0が有効なnull値にもなり得るため、呼出し側は必ず`ui_status`で成功を判定する。`property_kind`の未知値や追加のstatusはwrapperが汎用未対応/失敗として扱い、未定義動作にしない。
- setterの成功は値の即時確定を表す。Property通知、Binding先への伝播、Binding検証エラーは`mvvm-semantics.md`の遅延配信規則に従い、setterの戻り時点で完了を保証しない。

### 7.5 検証する振る舞い

1. 4言語から`BOOL`、`I64`、`UTF8`を読み書きし、値の一致、UTF-8の容量照会・再試行・埋め込みNULを確認する。
2. 型不一致、範囲外の整数、不正UTF-8、無効なbool、workerからの呼出しを安全に拒否し、値と通知を変えない。
3. 同一Core objectの異なる所有トークンをOBJECT値として設定しても、同一性に基づいて不要な通知を出さない。OBJECTの公開テストは循環防止規則が確定してから行う。
4. 検証失敗とAPI引数エラーを区別し、失敗時の`out_error`と出力値の初期化を4言語で同じ意味で扱う。

## 8. 承認された設計判断（Property値、2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| A5 | 初期公開値はBOOL / I64 / UTF8を種類別関数で読み書きする。OBJECTは型と所有権の案を記載するが、循環防止規則まで公開しない | wrapper実装と値型の拡張 |
| A6 | UTF-8は借用入力とcaller buffer出力にし、容量不足時は再試行できる | コピーと文字列の所有権 |
| A7 | object入力は借用、getter出力は新しい所有トークンとし、nullable OBJECTの0だけをnullにする | handle寿命とnull表現 |
| A8 | 値の種類違い・入力不正・Validation失敗・スレッド違反を区別し、setter成功と通知完了を分ける | 4言語のエラー処理と更新タイミング |

## 9. Property生成と通知購読のC ABI（承認済み）

この節は、第7節の初期公開値型BOOL / I64 / UTF8だけを生成対象とする。Binding、検証規則の登録、OBJECT Propertyの生成は別設計とする。関数名・引数の形はレビュー用であり、正式ヘッダではない。

```c
ui_status ui_abi_1_property_create_bool(ui_handle runtime, ui_bool initial,
                                         ui_handle *out_property,
                                         ui_handle *out_error);
ui_status ui_abi_1_property_create_i64(ui_handle runtime, int64_t initial,
                                        ui_handle *out_property,
                                        ui_handle *out_error);
ui_status ui_abi_1_property_create_utf8(ui_handle runtime, const uint8_t *bytes,
                                         uint64_t byte_length,
                                         ui_handle *out_property,
                                         ui_handle *out_error);

ui_status ui_abi_1_property_subscribe(ui_handle property, ui_callback callback,
                                       void *userdata, ui_destroy destroy,
                                       ui_handle *out_subscription,
                                       ui_handle *out_error);
ui_status ui_abi_1_subscription_cancel(ui_handle subscription,
                                        ui_handle *out_error);
```

### 9.1 生成と値

- `runtime`は呼出し中だけ借用する。成功時の`out_property`は所有トークン一個で、呼出し側が`release`する。失敗時は0。生成時の値は第7節と同じbool値・整数・UTF-8規則で検証し、UTF-8を保持する場合は戻るまでにコピーする。
- 生成はUI thread専用。初期値を確定してから成功を返す。生成だけでは変更通知を発生させない。検証規則の登録・初期化時の検証・PropertyをControlへ割り当てる方法は後続設計で定める。
- Property handleを`release`しても、Bindingなどが内部参照を持つ場合は生存する。最後の参照がなくなったときはRuntime契約に従いUI threadで破棄する。

### 9.2 購読と解除

- `property_subscribe`はUI threadでのみ呼べる。callbackとdestroyはどちらも必須、userdataのnullは許す。成功時にCoreがclosureを引き受け、所有トークンの`out_subscription`を返す。失敗時は`out_subscription = 0`とし、closureは呼出し側の所有のままで`destroy`しない。
- callbackは引数を取らない。初期値を自動通知せず、変更が配信された時点でProperty getterから現在値を読む。登録順、通知のまとめ方、callback中の自己解除は`mvvm-semantics.md`と`runtime-contract.md`に従う。
- `subscription_cancel`はUI thread上で購読を無効化する。有効なSubscription handleなら既に解除済みでも成功する。解除成功後は新しいcallbackを開始しない。現在実行中のcallbackから自身を解除した場合は、その終了後にclosureを一度だけdestroyする。Subscription handleそのものは解除後も有効で、呼出し側が別途`release`する。
- 公開Subscription handleを`release`するだけでは購読を解除しない。Runtimeが有効な購読を終了時まで管理する。Property本体が先に破棄された場合はその購読を自動解除し、保留通知を捨て、closureをUI threadでdestroyする。残存するSubscription handleは解除済みの状態として解放できる。
- Runtimeは有効なSubscriptionを管理し、SubscriptionからPropertyへの参照とPropertyの購読リストは弱参照にして内部の強参照循環を作らない。BindingがPropertyを保持する関係は別設計とする。shutdown時は購読を自動解除し、保留通知を実行せずclosureを破棄する。

### 9.3 スレッド、失敗、検証

- 生成、購読、解除はUI thread専用であり、違反時には`UI_WRONG_THREAD`で状態を変えない。停止済みRuntimeなら`UI_CLOSED`。無効・型違い・所属違いのhandleは第2節のstatusで拒否する。
- `out_property`、`out_subscription`は必須で、失敗時に0へ初期化する。`out_error`は任意で第2節の規則に従う。生成・登録は原子的に成功または失敗し、失敗時にPropertyや購読、通知を残さない。
- 4言語から初期値付きPropertyを作り、getter・setterと通知を呼び出す。等しい値の再設定と同じUI処理内の連続更新ではP2・P3どおり通知数を確認する。購読失敗、自己解除、Property先行破棄、shutdownの各経路でdestroyがちょうど一回であることを確認する。

## 10. 承認された設計判断（Property生成・通知、2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| A9 | BOOL / I64 / UTF8を初期値付きの型別関数で生成し、所有Property handleを返す | 初期値の確定と4言語wrapper |
| A10 | callback引数なしの通知購読は初期通知を送らず、getterで現在値を読む | 通知の意味論とABIの単純さ |
| A11 | 解除はUI threadで明示し、handleの`release`だけでは解除しない | 購読の寿命とwrapperの後始末 |
| A12 | Property先行破棄とshutdownで購読を自動解除し、closureを一度だけdestroyする | 所有権と参照循環の防止 |

## 11. BindingのC ABI（承認済み）

同じRuntimeに属し、種類が一致するCore管理Propertyの接続を対象とする。型変換、他言語ViewModelへの直接参照、OBJECT Property、Bindingの検証規則の登録は対象外である。エラー状態の読み取りは、将来登録されるProperty Validatorによる伝播失敗も扱える形として定める。

```c
/* モード値と関数名はレビュー用。ui_handle / ui_statusなどは第2節。 */
enum { UI_BIND_ONCE = 1, UI_BIND_ONE_WAY = 2, UI_BIND_TWO_WAY = 3 };
enum { UI_BIND_DIRECTION_SOURCE_TO_TARGET = 1,
       UI_BIND_DIRECTION_TARGET_TO_SOURCE = 2 };

ui_status ui_abi_1_binding_create(ui_handle source_property,
                                   ui_handle target_property,
                                   uint32_t mode, ui_handle *out_binding,
                                   ui_handle *out_error);
ui_status ui_abi_1_binding_cancel(ui_handle binding, ui_handle *out_error);
ui_status ui_abi_1_binding_error_snapshot(ui_handle binding,
                                           ui_handle *out_owned_error,
                                           ui_handle *out_owned_rejected_property,
                                           uint32_t *out_direction);
ui_status ui_abi_1_binding_subscribe_error(ui_handle binding,
                                            ui_callback callback,
                                            void *userdata, ui_destroy destroy,
                                            ui_handle *out_subscription,
                                            ui_handle *out_error);
```

### 11.1 作成と解除

- `source_property`と`target_property`は呼出し中だけ借用する。`mode`は上記の三値だけ受け付ける。UI thread上で種類・所属Runtime・Bindingグラフの制約を検証し、`mvvm-semantics.md`のB1・D4に従って初期同期と接続を原子的に行う。
- `UI_BIND_ONCE`は初期コピーが成功しても接続を保持せず、成功時の`out_binding`を0にする。`UI_BIND_ONE_WAY`と`UI_BIND_TWO_WAY`は成功時に所有Binding handleを一個返す。失敗時は全モードで`out_binding = 0`とし、グラフ・値・通知を変えない。戻り値のstatusでOneTime成功時の0と失敗時の0を区別する。
- `binding_cancel`は有効なBinding handleなら既に解除済みでも成功する。解除後は保留していた伝播を含め以後の伝播を止め、値を巻き戻さない。handleは解除後も有効で、呼出し側が別途`release`する。`release`だけでは解除しない。
- Runtimeは有効なBindingを明示解除またはshutdownまで管理する。Bindingは接続したPropertyを内部的に保持し、Property側からBindingへの参照は弱参照として強参照循環を防ぐ。最後の公開Property handleが解放されても接続中はPropertyを保持する。shutdownでは接続と保留中の伝播を破棄し、残存するBinding handleは停止済みとして解放できる。

### 11.2 Bindingエラーの読み取りと通知

- `binding_error_snapshot`は、その時点の一件のBindingエラー状態をUI thread上で取得する。エラーなしなら`UI_OK`で三つの出力を0にする。エラーありなら不変のerror objectと拒否したPropertyへの**新しい所有トークン**を返し、伝播方向を返す。呼出し側は両方のhandleを`release`する。出力三つは必須で、失敗時はすべて0へ初期化する。
- error objectは既存の`error_status`、`error_code`、`error_message_utf8`で読む。Binding内部の現在のエラーが消えたり別のエラーへ変わったりしても、取得済みのerror handleは内容が変わらない。現在の状態を再取得するには`binding_error_snapshot`を呼び直す。
- `binding_subscribe_error`はBindingのエラー状態の変化を通知するSubscriptionを返す。初期通知は行わず、callbackは引数を持たない。callback中にsnapshotを取得する。状態が同じ配信区間で複数回変わった場合はProperty通知と同じくまとめる。通知とProperty伝播の順序は`mvvm-semantics.md`に従う。
- Bindingを解除すると、そのBindingのエラー状態とエラー購読は終了する。既に実行中のcallbackの後始末、closureの`destroy`、Subscription handleの`release`は第9節とRuntime契約に従う。shutdown時も未実行のエラー通知を捨て、closureを一度だけ破棄する。

### 11.3 スレッドと失敗

- 全関数はUI thread専用とする。異なるRuntimeのPropertyなら`UI_WRONG_RUNTIME`、型違いなら`UI_WRONG_TYPE`、不正modeやグラフの循環・入力元の重複なら`UI_INVALID_ARGUMENT`を返す。グラフ制約の詳細は任意の`out_error`のdomain/codeで区別できるようにする。
- 初期同期先が値を拒否した場合は`UI_VALIDATION_FAILED`とerror handleを返し、Bindingを作らない。後続の遅延伝播で検証に失敗した場合は、元のsetterの成功を覆さず、Bindingエラー状態へ記録して通知する。
- `binding_subscribe_error`のcallbackとdestroyは必須、userdataのnullは許す。登録失敗時はclosureの所有権をCoreへ移さず、`destroy`を呼ばない。成功時はCoreが引き受け、解除またはshutdownで一度だけ破棄する。

### 11.4 検証する振る舞い

1. OneTime作成成功はBinding handleを返さず、初期同期の値だけが残る。OneWay/TwoWay作成成功は解除可能なhandleを返す。
2. グラフ循環、種類違い、別Runtime、初期Validation失敗では値・グラフ・通知を変えない。OneWay分岐は成功する。
3. 継続中のBindingで検証失敗が起きた場合、入力元は維持され、snapshotで拒否Property・方向・errorを取得できる。成功した再伝播ではエラーが消えて通知が届く。
4. Binding解除、自己解除、Property公開handleの先行release、shutdownで伝播とcallbackの寿命が契約どおり終わり、closureが一度だけ破棄される。

## 12. 承認された設計判断（Binding ABI、2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| A13 | OneTimeは初期コピーのみでhandle 0、OneWay/TwoWayは解除可能な所有Binding handleを返す | 作成結果と初期同期 |
| A14 | Binding handleの`release`では解除せず、明示解除またはshutdownまでRuntimeが保持する | 画面終了時のlifetime |
| A15 | エラーは不変error handle・拒否Property handle・方向の一組で取得し、変化をSubscriptionで通知する | Validation表示と4言語wrapper |
| A16 | 作成は原子的に失敗し、後続伝播のValidation失敗はsetterへ遡及させずBindingエラーへ記録する | エラーの帰属とグラフの整合性 |

## 13. Property ValidatorのC ABI（承認済み）

本節はCore管理のBOOL / I64 / UTF8 Propertyに付ける同期Validatorだけを対象とする。View入力Validatorの`Accept / Incomplete / Reject`、非同期検証、複数Propertyをまたぐ検証、validatorの差し替えと値の再検証は別設計とする。

```c
/* 戻り値: UI_OK=受理、UI_VALIDATION_FAILED=拒否、UI_INTERNAL_ERROR=検証処理の異常。 */
typedef ui_status (*ui_validate_bool)(void *userdata, ui_bool candidate,
                                       ui_handle *out_owned_error);
typedef ui_status (*ui_validate_i64)(void *userdata, int64_t candidate,
                                      ui_handle *out_owned_error);
typedef ui_status (*ui_validate_utf8)(void *userdata, const uint8_t *bytes,
                                       uint64_t byte_length,
                                       ui_handle *out_owned_error);

ui_status ui_abi_1_validation_error_create(uint32_t domain, uint32_t code,
                                            const uint8_t *message,
                                            uint64_t message_byte_length,
                                            ui_handle *out_owned_error);
ui_status ui_abi_1_property_validator_register_bool(ui_handle property,
                                                      ui_validate_bool callback,
                                                      void *userdata,
                                                      ui_destroy destroy,
                                                      ui_handle *out_registration,
                                                      ui_handle *out_error);
ui_status ui_abi_1_property_validator_register_i64(ui_handle property,
                                                     ui_validate_i64 callback,
                                                     void *userdata,
                                                     ui_destroy destroy,
                                                     ui_handle *out_registration,
                                                     ui_handle *out_error);
ui_status ui_abi_1_property_validator_register_utf8(ui_handle property,
                                                      ui_validate_utf8 callback,
                                                      void *userdata,
                                                      ui_destroy destroy,
                                                      ui_handle *out_registration,
                                                      ui_handle *out_error);
ui_status ui_abi_1_property_validator_cancel(ui_handle registration,
                                              ui_handle *out_error);
```

### 13.1 登録と値

- Propertyごとに同期Validatorを最大一つ登録できる。型の一致とUI threadを検証し、既に登録中なら`UI_INVALID_ARGUMENT`として失敗し既存の登録を保つ。成功時は所有registration handleを返し、Coreがcallback closureを保持する。失敗時はhandle 0、closureの所有権は呼出し側に残り`destroy`しない。
- 登録時には現在値を検証しない。以後のsetterとBinding伝播が候補値の検証を受ける。候補が現値と等しい場合も検証し、値の通知は増やさず既存のBindingエラーを消せる。別のValidatorへの差し替えと既存値の再検証は後続の設計事項である。
- Validatorを明示解除すると、以後のsetterは検証を受けない。解除は有効なregistration handleに対し冪等で、handleは別途`release`する。`release`だけでは解除しない。Property先行破棄またはshutdownでも登録を無効化し、closureをUI thread上で一度だけ`destroy`する。解除だけでは既存のBindingエラーを自動消去せず、次の伝播やBinding解除時に更新する。

### 13.2 callbackの結果とerror handle

- CoreはUI thread上で候補値を渡してcallbackを同期実行する。UTF8入力pointerはcallback中だけ借用可能で、長さはbyte数。Core側が必要ならcallback前に候補値を安定した領域へ保持する。callbackは候補値を保持する場合その場でコピーする。
- callbackの`out_owned_error`は必須の出力先で、Coreが呼出し前に0へ設定する。`UI_OK`なら0のまま受理し、値を確定する。`UI_VALIDATION_FAILED`なら値を変えず、`out_owned_error`の所有error handleをCoreへ移す。エラーobjectを作れない場合に限り0を許し、statusだけで拒否を識別する。
- 検証理由を付ける場合は、callback内で`validation_error_create`を呼ぶ。この関数は入力UTF8を呼出し中だけ借用し、Coreが戻る前にコピーして`UI_VALIDATION_FAILED` status、domain、code、messageを持つ不変error handleを一個返す。関数自体の失敗ではhandle 0を返す。言語wrapperは渡す文字列のメモリをこの呼出しの間だけ保持すればよい。
- callbackが`UI_INTERNAL_ERROR`を返した場合は検証処理の異常としてsetterまたはBinding伝播を失敗させる。wrapperは言語例外を捕捉してこの値に変換し、診断経路へ記録する。callbackが未知statusや不適切なerror handleを返した場合も`UI_INTERNAL_ERROR`とし、値を確定しない。Coreが受け取った有効な所有error handleは結果にかかわらず一回だけ消費または解放する。ABI境界へ例外を伝播させない。
- `validation_error_create`はUI threadから呼べる補助関数とし、これ自体はPropertyを変更しない。domain/codeの値体系と多言語の表示文言は後続で定める。任意のstatusを設定できる汎用error生成APIはこの段階では公開しない。

### 13.3 再入と後始末

- Validator callback中は同じRuntimeのUI状態を変更するAPI（Property setter、Binding作成・解除、購読登録・解除、shutdownなど）を許さない。呼び出された場合はその操作を状態変更前に拒否する。callbackは候補値の計算、言語側の読取り専用状態の参照、`validation_error_create`に限定する。検証失敗と異常は区別する。
- callback自身から登録を解除することも許さない。外側のsetterまたは伝播が戻った後にUI threadで解除できる。解除時またはshutdown時に実行中のcallbackがあれば、その終了後に`destroy`を一度だけ呼ぶ。`destroy`からUI操作をしないというRuntime契約を適用する。
- validator callbackに実行時間の上限を設けるか、診断とタイムアウトをどう扱うかはRuntime診断設計で定める。同期callback中に時間のかかる作業を行わない。

### 13.4 検証する振る舞い

1. 4言語でBOOL / I64 / UTF8のvalidatorを登録し、受理・拒否・言語例外を区別する。UTF8の借用期間と埋め込みNULを確認する。
2. 拒否時には値の通知を出さず、直接setterへは`UI_VALIDATION_FAILED`を返す。Binding伝播時には第11節のsnapshotからdomain/code/messageを取得する。
3. 登録失敗・明示解除・Property先行破棄・shutdownでclosureがちょうど一回`destroy`される。callback中のUI変更は拒否される。

## 14. 承認された設計判断（Validator ABI、2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| A17 | BOOL / I64 / UTF8別の同期callbackをPropertyに一つ登録し、登録時の再検証はしない | 4言語のcallback変換と初期値 |
| A18 | callbackはstatusと所有error handleを返し、error生成時にメッセージをCoreへコピーする | 検証理由の寿命と表示 |
| A19 | 明示解除とProperty破棄・shutdownでclosureを一度だけ破棄し、handleの`release`だけでは解除しない | wrapperの参照寿命 |
| A20 | 検証中の同一Runtime UI状態変更を禁止し、例外などの異常を通常の入力拒否と区別する | 再入と失敗時の原子性 |

## 15. 非同期CommandのC ABI（承認済み）

本節は`mvvm-semantics.md`のC1～C4に従い、workerの生成そのものはアプリまたは言語wrapperへ委ねる。実行開始、状態・実行可否、協調的キャンセル、完了報告の境界を定める。進捗の専用APIや結果値の型は後続設計とし、進捗表示はworkerから`runtime_post`して既存のPropertyへ設定できる。

```c
/* 値と関数名はレビュー用。状態値はCommandの読取専用I64 Propertyからも読める。 */
enum { UI_COMMAND_STATE_IDLE = 0, UI_COMMAND_STATE_RUNNING = 1,
       UI_COMMAND_STATE_CANCELLING = 2, UI_COMMAND_STATE_SUCCEEDED = 3,
       UI_COMMAND_STATE_FAILED = 4, UI_COMMAND_STATE_CANCELLED = 5 };

typedef ui_status (*ui_command_start)(void *userdata,
                                      ui_handle borrowed_command,
                                      uint64_t operation_id,
                                      ui_handle borrowed_cancel_token);

ui_status ui_abi_1_command_create_async(ui_handle runtime,
                                         ui_handle eligibility_bool_or_zero,
                                         ui_command_start start, void *userdata,
                                         ui_destroy destroy,
                                         ui_handle *out_command,
                                         ui_handle *out_error);
ui_status ui_abi_1_command_execute(ui_handle command,
                                    uint64_t *out_operation_id,
                                    ui_handle *out_error);
ui_status ui_abi_1_command_cancel(ui_handle command, uint64_t operation_id,
                                   ui_handle *out_error);
ui_status ui_abi_1_command_close(ui_handle command, ui_handle *out_error);
ui_status ui_abi_1_command_state_property(ui_handle command,
                                           ui_handle *out_owned_property);
ui_status ui_abi_1_command_can_execute_property(ui_handle command,
                                                 ui_handle *out_owned_property);
ui_status ui_abi_1_command_failure_snapshot(ui_handle command,
                                             ui_handle *out_owned_error);

ui_status ui_abi_1_command_cancel_token_requested(ui_handle token,
                                                   ui_bool *out_yes);
ui_status ui_abi_1_command_complete_success(ui_handle command,
                                             uint64_t operation_id);
ui_status ui_abi_1_command_complete_failure_utf8(ui_handle command,
                                                  uint64_t operation_id,
                                                  uint32_t domain, uint32_t code,
                                                  const uint8_t *message,
                                                  uint64_t message_byte_length);
ui_status ui_abi_1_command_complete_cancelled(ui_handle command,
                                               uint64_t operation_id);
```

### 15.1 起動と状態

- `command_create_async`はUI threadで呼ぶ。`eligibility_bool_or_zero`が0ならアプリ側の実行条件はtrue、有効なBOOL Propertyならその現在値を条件とする。異なるRuntimeや種類違いは拒否する。Coreは条件Propertyを保持し、その変化を監視して実行可否を更新する。成功時は所有Command handleを返し、Coreが`start`・userdata・destroyのclosureを引き受ける。失敗時はhandle 0でclosureの所有権は呼出し側に残る。
- `command_execute`はUI threadで呼び、実行可否がfalseなら新しいoperationを作らず失敗を返し、出力IDを0にする。成功時はRuntime内で再利用しない非ゼロのoperation IDを発行し、`RUNNING`にしてから`start` callbackをUI thread上で一回呼ぶ。callbackはworkerを開始して速やかに`UI_OK`を返す。起動に失敗したら`UI_INTERNAL_ERROR`を返し、Coreがそのoperationを`FAILED`へ確定する。callback中の言語例外はwrapperが捕捉して`UI_INTERNAL_ERROR`へ変換し、ABI越しに伝播させない。`command_execute`自体はoperation生成に成功した場合、起動失敗でも`UI_OK`とIDを返し、失敗状態を状態Propertyから観測させる。
- `start`へ渡すCommandとcancel tokenのhandleはcallbackの間だけ借用できる。workerが使う場合はcallback中に`handle_retain`で別の所有トークンを取得し、worker終了時に`release`する。Command本体をUI thread外から操作せず、tokenの照会だけは任意スレッドでできる。`start` callback内で完了報告や同じCommandへの再実行・closeは行わない。
- 状態と実行可否はそれぞれCore管理の読取専用I64 / BOOL Propertyとして公開し、UI Bindingと通常のProperty通知を利用する。返されたProperty handleは所有トークンであり、呼出し側が解放する。公開setterでこれらを変更する操作は`UI_UNSUPPORTED`とする。既定では`RUNNING`と`CANCELLING`中に実行可否がfalseになる。終端状態の後も状態は読め、次の実行で新しいIDに移る。
- 状態は`start` callbackの呼出し前に`RUNNING`へ変わるため、同じUIイベントcallback内で`command_execute`を再び呼んでも、また別のControlから同じCommandを起動しても、workerが進行中なら2回目は拒否される。終端後は既定で実行可否がtrueへ戻り、次の実行を許す。状態値は`IDLE`へ自動で戻さず、直前の終端状態を次の実行まで保持する。

### 15.2 キャンセルと終了

- `command_cancel`はUI threadでoperation IDを指定する。現在のoperationが`RUNNING`なら`CANCELLING`にしてtokenの要求フラグを立てる。既に`CANCELLING`なら成功して状態を変えない。終端状態の同じIDへの要求も成功する。古いID・未知IDは状態を変えず失敗する。キャンセル要求はworkerの終了を待たない。
- workerは`command_cancel_token_requested`を任意スレッドから呼べる。tokenはRuntime停止後も保持中なら照会でき、停止時に要求フラグはtrueになる。tokenを照会する以外のUI操作はworkerに許さない。キャンセルを確認して中断したworkerは`CANCELLED`を報告し、正常完了なら要求後でも`SUCCEEDED`を報告する。
- `command_close`はUI threadで新規実行を止め、進行中ならキャンセルを要求する。有効なCommand handleへの重複closeは成功する。close後のCommand固有の状態取得と完了報告は`UI_CLOSED`となり、UI向け完了通知を開始せず、worker終了をUI threadで待たない。既に取得した状態Property handleはそれぞれの所有参照が解放されるまで残るが、close後は値の更新を止める。公開Command handleの`release`だけではcloseしない。Runtimeのshutdownも全Commandをclose相当に扱い、start closureを最後の`start` callback終了後に一度だけ`destroy`する。workerがuserdataを使う場合はその言語側参照を別に保持する。

### 15.3 完了報告と失敗情報

- 完了報告の三関数はUI thread専用で、workerは`runtime_post`した短いcallbackから呼ぶ。現在のoperation IDに対して一度だけ終端状態へ遷移する。既に終端・close済み・過去のIDの遅延報告は状態を変えず、古いIDや二重報告は失敗として返す。Runtime停止後は`UI_CLOSED`を返す。
- `complete_failure_utf8`はdomain/codeとUTF-8メッセージを呼出し中だけ借用し、Coreが戻る前にコピーする。業務上の失敗を`FAILED`状態と`UI_COMMAND_EXECUTION_FAILED` statusを持つ不変のerror objectに保存し、`command_failure_snapshot`が所有error handleを返す。失敗がなければhandle 0。次の実行開始で前回の失敗情報を消す。呼出しが引数不正や割当失敗ならoperationの終端を確定せず、呼出し側が有効な失敗報告を再試行できる。`start` callback自体の起動失敗にはCoreが一般的な失敗情報を記録する。
- `complete_success`、`complete_cancelled`は失敗情報を持たない。実際のworker結果に応じて報告し、キャンセル要求があっただけで`CANCELLED`へ強制しない。取得済みのerror handleは後でCommand状態が変わっても不変のまま保持できる。
- `runtime_post`が停止のため受け付けられない場合、workerは投稿closureを自分で破棄し、保持するCommand/token handleを解放する。shutdown後のUIで終端状態が観測できることは保証しない。

### 15.4 検証する振る舞い

1. 4言語でCommandを起動し、UI callbackが速やかに戻り、実行中の二重起動を拒否する。実行条件と状態の変化がcan-execute Propertyへ反映される。
   同じUIイベントcallback内の連続2回呼出しと、別Controlからの実行中の呼出しも新しいoperationを作らない。終端後の次の呼出しは新しいIDで実行できる。
2. workerはtokenでキャンセル要求を確認し、UIへの進捗・完了は`post`経由だけで反映する。キャンセル要求後の正常終了は`SUCCEEDED`になる。
3. 古いoperation IDと二重報告は現在状態を変更しない。失敗のsnapshotは不変で、次の実行では現在の失敗情報が消える。
4. closeとshutdownはworkerをUI threadで待たず、closureを一回だけ破棄する。worker側の所有handleは終了時に解放できる。

## 16. 承認された設計判断（非同期Command ABI、2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| A21 | Coreは状態と実行可否を読取専用Propertyで公開し、開始callbackはUI thread上でworkerを起動して戻る | Button BindingとUI応答性 |
| A22 | 実行ごとにIDとスレッド安全なキャンセルtokenを作り、協調的にキャンセルする | workerの寿命と古い通知 |
| A23 | workerの完了は`post`経由でUI threadへ報告し、ID一致時に一度だけ終端状態を確定する | 遅延報告・失敗情報 |
| A24 | closeとshutdownは新規実行を止めてキャンセルを要求するが、workerの終了をUI threadで待たない | 画面終了とリソース管理 |

## 17. View入力ValidatorのC ABI（承認済み）

`mvvm-semantics.md`のV5～V8に従い、最初はTextBoxに対するユーザー起点のテキスト編集だけを扱う。TextBox生成の最小境界は第20節で定め、layout・focus・IME状態遷移の全体はControl/Input仕様で定める。本節の関数名とTextBox handleの公開はその仕様と合わせて確定する。一覧など他のControlの候補選択は後続設計とする。

```c
/* 編集後の候補全文と編集後の選択範囲。位置はUTF-8 byte offset。 */
enum { UI_TEXT_INPUT_ACCEPT = 1, UI_TEXT_INPUT_INCOMPLETE = 2,
       UI_TEXT_INPUT_REJECT = 3 };
enum { UI_TEXT_ORIGIN_KEYBOARD = 1, UI_TEXT_ORIGIN_PASTE = 2,
       UI_TEXT_ORIGIN_DROP = 3, UI_TEXT_ORIGIN_ACCESSIBILITY = 4,
       UI_TEXT_ORIGIN_IME_COMMIT = 5 };

typedef ui_status (*ui_text_input_validate)(void *userdata,
                                             const uint8_t *candidate_utf8,
                                             uint64_t candidate_byte_length,
                                             uint64_t selection_start_byte,
                                             uint64_t selection_end_byte,
                                             uint32_t origin,
                                             uint32_t *out_decision,
                                             ui_handle *out_owned_reason);

ui_status ui_abi_1_text_input_reason_create(uint32_t domain, uint32_t code,
                                             const uint8_t *message,
                                             uint64_t message_byte_length,
                                             ui_handle *out_owned_reason);
ui_status ui_abi_1_text_box_input_validator_register(ui_handle text_box,
                                                      ui_text_input_validate callback,
                                                      void *userdata,
                                                      ui_destroy destroy,
                                                      ui_handle *out_registration,
                                                      ui_handle *out_error);
ui_status ui_abi_1_text_box_input_validator_cancel(ui_handle registration,
                                                    ui_handle *out_error);
ui_status ui_abi_1_text_box_input_state_property(ui_handle text_box,
                                                  ui_handle *out_owned_property);
ui_status ui_abi_1_text_box_input_reason_snapshot(ui_handle text_box,
                                                   ui_handle *out_owned_error);
ui_status ui_abi_1_text_box_editing_utf8(ui_handle text_box, uint8_t *buffer,
                                         uint64_t capacity, uint64_t *out_length);
ui_status ui_abi_1_text_box_input_subscribe(ui_handle text_box,
                                             ui_callback callback,
                                             void *userdata, ui_destroy destroy,
                                             ui_handle *out_subscription,
                                             ui_handle *out_error);
```

### 17.1 候補と判定

- Coreはユーザー起点の編集操作から候補**全文**と編集後の選択範囲を作ってから、UI thread上でcallbackを一回呼ぶ。キーボード・貼り付け・ドロップ・アクセシビリティ操作は同じ候補規則を通る。候補は有効なUTF-8で、pointerはcallback中だけ借用可能。選択位置はbyte長以下かつUnicode文字の境界にある。caretのgrapheme単位の移動・Undo/Redoの詳細はControl/Input仕様へ送る。
- callbackの戻り値`UI_OK`は判定処理に成功したことを表し、判定そのものは`out_decision`の三値で返す。Coreは両出力を呼出し前に0へ初期化する。不正な判定値やcallbackの異常は`UI_INTERNAL_ERROR`として編集を確定しない。wrapperは言語例外をABI越しに伝播させない。
- `ACCEPT`はTextBoxの確定Property値と画面上の編集値を候補で更新し、通常のTwoWay Bindingへ流す。モデル側のProperty Validatorが後で拒否した場合はV2・V3に従い、View側の入力を保持する。`INCOMPLETE`は画面上の編集値と選択範囲だけを更新し、確定Property値とViewModel値を変えない。`REJECT`は通常、編集値・確定Property値・caret・選択範囲を編集前のまま保つ。
- IME変換中の未確定文字列はcallbackに渡さず、確定操作の候補だけを`UI_TEXT_ORIGIN_IME_COMMIT`として渡す。その候補が`REJECT`なら文字列を黙って捨てず、`INCOMPLETE`相当の編集値として画面に保持し、理由を表示できる状態にする。ViewModelへは流さない。ControlがプログラムからPropertyを設定する場合はこのView入力callbackを通さず、ViewModel側の検証は通常どおり適用する。

### 17.2 理由・状態の読取り

- `INCOMPLETE`と`REJECT`では`out_owned_reason`に不変の理由handleを任意で返せる。callback内で`text_input_reason_create`を呼び、domain/codeとUTF-8メッセージをCoreへコピーして`UI_INPUT_FEEDBACK` statusのerror objectを生成する。入力途中は入力エラーとは限らないため、Property検証用の`UI_VALIDATION_FAILED`とは区別する。`ACCEPT`では0にする。有効な所有error handleはCoreが結果に応じ消費または解放し、種類違いはcallback異常とする。理由の生成に失敗した場合は0を許すが、IME確定候補の`REJECT`では少なくとも入力を保持し、診断上の理由がないことを識別できるようにする。
- `text_box_input_state_property`は読取専用I64 Propertyの所有トークンを返し、現在の`ACCEPT / INCOMPLETE / REJECT`を通知で観測できるようにする。通常の`REJECT`は編集値を変えずに直近の拒否状態・理由だけを更新する。IME確定の`REJECT`は画面の編集値を保持するが、状態値は`REJECT`のままとする。入力元はcallbackに通知され、直近の入力元を後から照会するAPIはControl仕様で検討する。`ACCEPT`では直近の入力理由を消す。
- `text_box_input_reason_snapshot`は現在の理由の所有error handleまたは0を返す。取得後に状態が変わってもerror objectは不変である。`text_box_editing_utf8`は画面上の編集中の文字列をcaller bufferへコピーし、第7節と同じ容量照会・不足時の再試行・NUL非終端の規則を使う。確定Property値は既存のProperty getterから読む。
- 状態値が同じまま理由や編集中の文字列だけ変わる場合がある。`text_box_input_subscribe`はこれらのいずれかが変わったとき、引数なしcallbackでまとめて通知する。初期通知は行わず、購読者は上記のgetterとsnapshotで現在値を読む。Subscriptionの明示解除、closure所有権、自己解除、TextBox破棄時の後始末は第9節に従う。

### 17.3 登録・終了・再入

- TextBoxごとに一つ登録でき、成功時に所有registration handleを返す。callbackとdestroyは必須で、userdataのnullは許す。登録失敗時はclosureをCoreが引き受けず、呼出し側が後始末をする。既に登録中なら新しい登録を拒否する。Validatorがない場合は、有効なUTF-8のユーザー編集を`ACCEPT`する。
- 明示解除はUI threadで冪等に行い、公開registration handleの`release`だけでは解除しない。TextBox破棄またはRuntime shutdownでは自動解除し、最後のcallbackが戻った後にclosureをUI threadで一度だけ`destroy`する。登録・解除・callbackはUI thread専用とする。
- callback中は同じRuntimeのUI状態変更を許さない。候補の判定と`text_input_reason_create`は許す。callbackが自身を解除することはできず、戻った後に解除する。callbackの処理が長いとUIを占有するため、短時間で終える。
- 編集値・選択範囲の保存、状態Propertyの更新、理由の保持は一つの編集操作としてまとめて確定する。例外、不正な戻り値、確保失敗では、IME確定候補を失わないための退避が必要な場合を除き、編集前の状態を保つ。IMEでの退避失敗時の診断と復旧はInput仕様で扱う。

### 17.4 検証する振る舞い

1. 4言語で候補全文と選択範囲を受け、`ACCEPT`だけが確定PropertyとBindingへ伝播し、`INCOMPLETE`では編集値のみ、通常の`REJECT`では値・caretとも旧状態を保つ。
2. IME変換中に候補を破棄せず、確定時の`REJECT`は理由と候補をViewに保持する。プログラムからのsetterはView入力Validatorを経由しない。
3. 登録失敗・明示解除・TextBox破棄・shutdownでclosureが一度だけ破棄され、異常なcallbackはABI境界を越えない。

## 18. 承認された設計判断（View入力Validator ABI、2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| A25 | TextBoxの編集候補全文・選択範囲・入力元をcallbackへ渡し、三値で判定する | 4言語の入力規則 |
| A26 | `INCOMPLETE`は編集値にだけ保持し、IME確定候補の`REJECT`も入力を失わない形で保持する | TwoWayと多言語入力 |
| A27 | 編集値、入力状態、理由を確定Propertyとは別に読み取り、変化を購読できるようにする | エラー表示と途中入力の観測 |
| A28 | 登録は明示解除し、TextBox破棄・shutdownで自動解除してclosureを一度だけ破棄する | callback所有権とUI thread |

## 19. 今後に送る項目

IME・Undo/Redoの詳細、Commandの進捗専用APIと結果値、値変換・浮動小数点・objectの循環防止、Collectionの配列形式、Platform event loopのポンプ方法、ネイティブ配布・CIのtarget matrixは後続の設計で定義する。

## 20. TextBox生成と値Propertyの接続（承認済み）

第17節のTextBox handleを生成し、確定値を第7・9・11節のUTF8 Propertyに接続する最小境界を定める。論理ツリーへの取り付け、表示・layout、focus、IME・Undo/Redoの詳細は対象外。以下の関数名と引数はレビュー用であり、正式ヘッダではない。

```c
ui_status ui_abi_1_text_box_create(ui_handle runtime,
                                    const uint8_t *initial_utf8,
                                    uint64_t initial_byte_length,
                                    ui_handle *out_text_box,
                                    ui_handle *out_error);
ui_status ui_abi_1_text_box_value_property(ui_handle text_box,
                                            ui_handle *out_owned_property);
```

### 20.1 生成と確定値

- `text_box_create`はUI thread上で、有効なRuntimeとUTF-8初期値を検証し、未接続のTextBoxとその確定値用UTF8 Propertyを一体で作る。空文字列はnull pointerと長さ0で指定でき、入力bytesは呼出しが戻る前にコピーする。長さが正なのにpointerがnull、不正UTF-8なら`UI_INVALID_ARGUMENT`。成功時は所有TextBox handle一個、失敗時は0を返し、Control・Property・通知を残さない。`out_error`は任意とする。
- 初期の確定値と編集値は同一で、入力状態は`ACCEPT`、入力理由はない。生成だけではPropertyの値通知や入力通知を発生させない。第17節のView入力Validatorは生成時の初期値には適用しない。
- `text_box_value_property`はTextBoxが所有する**同じ**UTF8 Propertyへの新しい所有トークンを毎回返す。`out_owned_property`は必須で失敗時0。これは確定値を表し、第17節の`text_box_editing_utf8`が返す編集中の値とは異なり得る。一般のUTF8 Property getter/setterとProperty通知を使い、通常のBindingのsourceまたはtargetにできる。別Runtime・種類違い・Bindingグラフの制約は第11節どおり。

### 20.2 表示値とBinding更新

- ユーザー編集が第17節で`ACCEPT`になると、確定値Propertyが更新され、既存のBinding規則で伝播する。`INCOMPLETE`と入力の`REJECT`では確定値PropertyへのsetterとBinding伝播を行わない。ViewModel側のProperty Validatorで伝播が拒否されたときは入力表示とTextBoxの確定値を保ち、Bindingエラーとして報告する。
- 他の経路からTextBoxの確定値Propertyに有効な値が確定したら、編集中の値もその値に置き換え、入力状態を`ACCEPT`、入力理由を0にする。直前の編集値が`INCOMPLETE`やIME確定の`REJECT`として残っていても同様。View入力Validatorは呼ばない。Property setterで値が拒否された場合は編集値・入力状態・理由を変えない。外部更新の上書きは`mvvm-semantics.md`の14.2に従う。
- プログラムから同じ確定値を設定したときに編集値が確定値と異なっていても、新しい値の確定は起きていないため編集値を上書きしない。入力途中の破棄を意図する場合の明示的なリセット操作と、View側のProperty Validatorを登録した場合の入力確定失敗は、Input/Control詳細設計で定める。

### 20.3 所有と終了

- TextBoxは確定値Propertyと入力状態を内部参照で保持する。公開TextBox handleの`release`はその参照を一つ減らすだけで、論理ツリーなどの内部所有があれば破棄しない。最後のTextBox参照がなくなった時点でUI thread上で破棄し、第17節のValidatorと入力購読を自動解除する。別途保持されたProperty handleやBindingによるProperty参照は有効なまま残り得るが、それらからTextBoxを強参照しない。破棄後の値変更は描画や入力状態の更新を発生させない。
- 画面閉鎖時のツリーからの取り外し、画面が作ったBindingの解除、アプリが保持したProperty handleの解放は後続のUI Tree/lifetime仕様で定める。Runtime shutdownは既存の停止契約に従う。

### 20.4 確認する振る舞い

1. 4言語でTextBoxを作成し、異なる所有トークンから同じ確定値Propertyを取得できる。初期値・編集値・入力状態が一致し、生成で通知は出ない。
2. TwoWayでViewModelと接続し、`ACCEPT`だけが伝播する。モデル側の検証失敗ではTextBoxの値と表示を保ち、後から届く別の有効値では編集値・理由が更新される。
3. 未接続TextBoxをreleaseして破棄した後も、別途保持したPropertyは有効で、Validatorと入力購読のclosureは一度だけ破棄される。

## 21. 承認された設計判断（TextBox生成・値接続、2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| A29 | RuntimeとUTF-8初期値から未接続TextBoxを原子的に作る | 初期表示とControl生成ABI |
| A30 | 確定値を同一のCore管理UTF8 Propertyとして公開し、通常のBindingにつなぐ | ViewModel接続と4言語wrapper |
| A31 | 外部から異なる値が確定した場合は編集中の値も上書きし、入力状態・理由を初期化する | 入力途中とTwoWay更新の競合 |
| A32 | TextBox破棄時は入力用closureを解除し、別途保持されたPropertyにはTextBoxを保持させない | UI寿命と参照循環 |
